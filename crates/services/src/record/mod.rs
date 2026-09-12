use arc_swap::ArcSwap;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{Mutex, broadcast};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordStatus {
    Recording,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordBackend {
    GpuScreenRecorder,
    WlScreenRec,
}

impl RecordBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordBackend::GpuScreenRecorder => "gpu-screen-recorder",
            RecordBackend::WlScreenRec => "wl-screenrec",
        }
    }

    pub fn is_available(&self) -> bool {
        match self {
            RecordBackend::GpuScreenRecorder => std::process::Command::new("which")
                .arg("gpu-screen-recorder")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false),
            RecordBackend::WlScreenRec => std::process::Command::new("which")
                .arg("wl-screenrec")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false),
        }
    }

    pub fn detect_preferred() -> Option<Self> {
        if RecordBackend::GpuScreenRecorder.is_available() {
            Some(RecordBackend::GpuScreenRecorder)
        } else if RecordBackend::WlScreenRec.is_available() {
            Some(RecordBackend::WlScreenRec)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordTarget {
    Screen,
    Monitor(String),
    Region {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    },
    Portal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordAudio {
    None,
    Desktop,
    Microphone,
    Both,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordOptions {
    pub backend: RecordBackend,
    pub target: RecordTarget,
    pub audio: RecordAudio,
    pub fps: u32,
    pub resolution: String,
    pub quality: String,
    pub container: String,
    pub output_path: Option<PathBuf>,
    pub include_cursor: bool,
}

impl Default for RecordOptions {
    fn default() -> Self {
        Self {
            backend: RecordBackend::GpuScreenRecorder,
            target: RecordTarget::Screen,
            audio: RecordAudio::Desktop,
            fps: 60,
            resolution: "native".to_string(),
            quality: "very_high".to_string(),
            container: "mp4".to_string(),
            output_path: None,
            include_cursor: true,
        }
    }
}

impl RecordOptions {
    pub fn from_config(config: &crate::config::RecordConfig) -> Self {
        let audio = match config.audio.as_str() {
            "mic" | "microphone" => RecordAudio::Microphone,
            "both" => RecordAudio::Both,
            "none" | "off" => RecordAudio::None,
            _ => RecordAudio::Desktop,
        };

        let target = if config.output == "screen" || config.output.is_empty() {
            RecordTarget::Screen
        } else {
            RecordTarget::Monitor(config.output.clone())
        };

        Self {
            backend: RecordBackend::detect_preferred().unwrap_or(RecordBackend::GpuScreenRecorder),
            target,
            audio,
            fps: config.fps,
            resolution: config.resolution.clone(),
            quality: config.quality.clone(),
            container: config.container.clone(),
            output_path: None,
            include_cursor: config.include_cursor,
        }
    }
}

struct ActiveSession {
    backend: RecordBackend,
    output_path: PathBuf,
    ipc_socket_path: Option<PathBuf>,
    child_pid: u32,
    accumulated_active_duration: Duration,
    current_segment_start: Option<Instant>,
}

impl ActiveSession {
    fn current_duration(&self) -> Duration {
        let mut total = self.accumulated_active_duration;
        if let Some(start) = self.current_segment_start {
            total += start.elapsed();
        }
        total
    }
}

#[derive(Clone)]
pub struct RecordService {
    status: Arc<ArcSwap<RecordStatus>>,
    active_session: Arc<Mutex<Option<ActiveSession>>>,
    status_tx: broadcast::Sender<RecordStatus>,
}

impl Default for RecordService {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordService {
    pub fn new() -> Self {
        let (status_tx, _) = broadcast::channel(16);
        Self {
            status: Arc::new(ArcSwap::from_pointee(RecordStatus::Stopped)),
            active_session: Arc::new(Mutex::new(None)),
            status_tx,
        }
    }

    pub fn get_status(&self) -> RecordStatus {
        **self.status.load()
    }

    pub fn is_recording(&self) -> bool {
        self.get_status() == RecordStatus::Recording
    }

    pub fn is_paused(&self) -> bool {
        self.get_status() == RecordStatus::Paused
    }

    pub fn subscribe_status(&self) -> broadcast::Receiver<RecordStatus> {
        self.status_tx.subscribe()
    }

    pub fn mark_stopped(&self) {
        self.status.store(Arc::new(RecordStatus::Stopped));
        let _ = self.status_tx.send(RecordStatus::Stopped);
    }

    pub fn get_duration_secs(&self) -> u64 {
        if let Ok(guard) = self.active_session.try_lock()
            && let Some(ref session) = *guard
        {
            return session.current_duration().as_secs();
        }
        0
    }

    pub async fn get_duration(&self) -> Duration {
        let guard = self.active_session.lock().await;
        if let Some(ref session) = *guard {
            session.current_duration()
        } else {
            Duration::ZERO
        }
    }

    pub async fn get_output_path(&self) -> Option<PathBuf> {
        let guard = self.active_session.lock().await;
        guard.as_ref().map(|s| s.output_path.clone())
    }

    pub async fn get_active_backend(&self) -> Option<RecordBackend> {
        let guard = self.active_session.lock().await;
        guard.as_ref().map(|s| s.backend)
    }

    pub async fn start(&self, options: RecordOptions) -> Result<PathBuf, String> {
        let mut guard = self.active_session.lock().await;
        if guard.is_some() {
            return Err("Ya hay una grabación en curso".to_string());
        }

        let output_path = if let Some(ref p) = options.output_path {
            p.clone()
        } else {
            Self::generate_default_output_path(&options.container)
        };

        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let session = match options.backend {
            RecordBackend::GpuScreenRecorder => {
                Self::spawn_gpu_screen_recorder(&options, &output_path).await?
            }
            RecordBackend::WlScreenRec => Self::spawn_wl_screenrec(&options, &output_path).await?,
        };

        let child_pid = session.child_pid;
        *guard = Some(session);
        self.status.store(Arc::new(RecordStatus::Recording));
        let _ = self.status_tx.send(RecordStatus::Recording);

        let active_clone = self.active_session.clone();
        let status_clone = self.status.clone();
        let tx_clone = self.status_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let is_alive = unsafe { libc::kill(child_pid as libc::pid_t, 0) == 0 };
                if !is_alive {
                    let mut lock = active_clone.lock().await;
                    if let Some(session) = lock.take()
                        && let Some(ref socket) = session.ipc_socket_path
                    {
                        let _ = std::fs::remove_file(socket);
                    }
                    status_clone.store(Arc::new(RecordStatus::Stopped));
                    let _ = tx_clone.send(RecordStatus::Stopped);
                    break;
                }
            }
        });

        Ok(output_path)
    }

    async fn spawn_gpu_screen_recorder(
        options: &RecordOptions,
        output_path: &Path,
    ) -> Result<ActiveSession, String> {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        let socket_path = PathBuf::from(format!(
            "{}/capsule_gsr_{}_{}.sock",
            runtime_dir,
            std::process::id(),
            Local::now().timestamp_millis()
        ));
        let _ = std::fs::remove_file(&socket_path);

        let mut cmd = tokio::process::Command::new("gpu-screen-recorder");

        let target_str = match &options.target {
            RecordTarget::Screen => "screen".to_string(),
            RecordTarget::Monitor(m) => m.clone(),
            RecordTarget::Portal => "portal".to_string(),
            RecordTarget::Region { .. } => "region".to_string(),
        };
        cmd.arg("-w").arg(&target_str);

        if let RecordTarget::Region {
            x,
            y,
            width,
            height,
        } = &options.target
        {
            cmd.arg("-region").arg(format!("{width}x{height}+{x}+{y}"));
        }

        cmd.arg("-f").arg(options.fps.to_string());
        cmd.arg("-c").arg(&options.container);
        cmd.arg("-cursor")
            .arg(if options.include_cursor { "yes" } else { "no" });
        cmd.arg("-q").arg(&options.quality);

        if options.resolution != "native" && options.resolution.contains('x') {
            cmd.arg("-s").arg(&options.resolution);
        }

        match &options.audio {
            RecordAudio::None => {}
            RecordAudio::Desktop => {
                cmd.arg("-a").arg("default_output");
            }
            RecordAudio::Microphone => {
                cmd.arg("-a").arg("default_input");
            }
            RecordAudio::Both => {
                cmd.arg("-a").arg("default_output|default_input");
            }
            RecordAudio::Custom(src) => {
                cmd.arg("-a").arg(src);
            }
        }

        cmd.arg("-o").arg(output_path);
        cmd.arg("-ipc").arg(&socket_path);

        let log_path = PathBuf::from(format!("{}/capsule_record.log", runtime_dir));
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| format!("Fallo al crear archivo de log: {e}"))?;
        let err_file = log_file
            .try_clone()
            .map_err(|e| format!("Fallo al duplicar archivo de log: {e}"))?;

        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::from(log_file));
        cmd.stderr(Stdio::from(err_file));

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Fallo al iniciar gpu-screen-recorder: {e}"))?;

        let child_pid = child
            .id()
            .ok_or_else(|| "No se pudo obtener el PID del proceso de grabación".to_string())?;

        tokio::time::sleep(Duration::from_millis(200)).await;

        if let Ok(Some(status)) = child.try_wait() {
            let err_msg = std::fs::read_to_string(&log_path).unwrap_or_default();
            return Err(format!(
                "gpu-screen-recorder terminó inesperadamente ({status}): {err_msg}"
            ));
        }

        Ok(ActiveSession {
            backend: RecordBackend::GpuScreenRecorder,
            output_path: output_path.to_path_buf(),
            ipc_socket_path: Some(socket_path),
            child_pid,
            accumulated_active_duration: Duration::ZERO,
            current_segment_start: Some(Instant::now()),
        })
    }

    async fn spawn_wl_screenrec(
        options: &RecordOptions,
        output_path: &Path,
    ) -> Result<ActiveSession, String> {
        let mut cmd = tokio::process::Command::new("wl-screenrec");
        cmd.arg("-f").arg(output_path);

        match &options.target {
            RecordTarget::Screen => {}
            RecordTarget::Monitor(m) => {
                cmd.arg("-o").arg(m);
            }
            RecordTarget::Region {
                x,
                y,
                width,
                height,
            } => {
                cmd.arg("-g").arg(format!("{x},{y} {width}x{height}"));
            }
            RecordTarget::Portal => {}
        }

        match &options.audio {
            RecordAudio::None => {}
            _ => {
                cmd.arg("--audio");
            }
        }

        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        let log_path = PathBuf::from(format!("{}/capsule_wl_record.log", runtime_dir));
        let log_file = std::fs::File::create(&log_path)
            .map_err(|e| format!("Fallo al crear archivo de log: {e}"))?;
        let err_file = log_file
            .try_clone()
            .map_err(|e| format!("Fallo al duplicar archivo de log: {e}"))?;

        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::from(log_file));
        cmd.stderr(Stdio::from(err_file));

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Fallo al iniciar wl-screenrec: {e}"))?;

        let child_pid = child
            .id()
            .ok_or_else(|| "No se pudo obtener el PID del proceso de grabación".to_string())?;

        tokio::time::sleep(Duration::from_millis(200)).await;

        if let Ok(Some(status)) = child.try_wait() {
            let err_msg = std::fs::read_to_string(&log_path).unwrap_or_default();
            return Err(format!(
                "wl-screenrec terminó prematuramente ({status}): {err_msg}"
            ));
        }

        Ok(ActiveSession {
            backend: RecordBackend::WlScreenRec,
            output_path: output_path.to_path_buf(),
            ipc_socket_path: None,
            child_pid,
            accumulated_active_duration: Duration::ZERO,
            current_segment_start: Some(Instant::now()),
        })
    }

    pub async fn stop(&self) -> Result<PathBuf, String> {
        let mut guard = self.active_session.lock().await;
        let session = guard
            .take()
            .ok_or_else(|| "No hay ninguna grabación en curso".to_string())?;

        self.status.store(Arc::new(RecordStatus::Stopped));
        let _ = self.status_tx.send(RecordStatus::Stopped);

        let mut stopped_via_ipc = false;
        if let Some(ref socket_path) = session.ipc_socket_path
            && let Ok(mut stream) = UnixStream::connect(socket_path).await
        {
            let msg = "{\"id\":1,\"name\":\"stop\"}\n";
            if stream.write_all(msg.as_bytes()).await.is_ok() {
                let mut reader = BufReader::new(stream);
                let mut response = String::new();
                if reader.read_line(&mut response).await.is_ok() {
                    stopped_via_ipc = true;
                }
            }
        }

        if !stopped_via_ipc {
            unsafe {
                libc::kill(session.child_pid as libc::pid_t, libc::SIGINT);
            }
        }

        for _ in 0..60 {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let is_alive = unsafe { libc::kill(session.child_pid as libc::pid_t, 0) == 0 };
            if !is_alive {
                break;
            }
        }

        if let Some(ref socket) = session.ipc_socket_path {
            let _ = std::fs::remove_file(socket);
        }

        Ok(session.output_path)
    }

    pub async fn pause(&self) -> Result<(), String> {
        let mut guard = self.active_session.lock().await;
        let session = guard
            .as_mut()
            .ok_or_else(|| "No hay grabación activa".to_string())?;

        if session.current_segment_start.is_none() {
            return Ok(());
        }

        if let Some(ref socket_path) = session.ipc_socket_path {
            if let Ok(mut stream) = UnixStream::connect(socket_path).await {
                let msg = "{\"id\":2,\"name\":\"set-paused\",\"data\":true}\n";
                let _ = stream.write_all(msg.as_bytes()).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
            }
        } else {
            unsafe {
                libc::kill(session.child_pid as libc::pid_t, libc::SIGUSR2);
            }
        }

        if let Some(start) = session.current_segment_start.take() {
            session.accumulated_active_duration += start.elapsed();
        }
        self.status.store(Arc::new(RecordStatus::Paused));
        let _ = self.status_tx.send(RecordStatus::Paused);
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), String> {
        let mut guard = self.active_session.lock().await;
        let session = guard
            .as_mut()
            .ok_or_else(|| "No hay grabación activa".to_string())?;

        if session.current_segment_start.is_some() {
            return Ok(());
        }

        if let Some(ref socket_path) = session.ipc_socket_path {
            if let Ok(mut stream) = UnixStream::connect(socket_path).await {
                let msg = "{\"id\":3,\"name\":\"set-paused\",\"data\":false}\n";
                let _ = stream.write_all(msg.as_bytes()).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
            }
        } else {
            unsafe {
                libc::kill(session.child_pid as libc::pid_t, libc::SIGUSR2);
            }
        }

        session.current_segment_start = Some(Instant::now());
        self.status.store(Arc::new(RecordStatus::Recording));
        let _ = self.status_tx.send(RecordStatus::Recording);
        Ok(())
    }

    pub async fn toggle_pause(&self) -> Result<(), String> {
        match self.get_status() {
            RecordStatus::Recording => self.pause().await,
            RecordStatus::Paused => self.resume().await,
            RecordStatus::Stopped => Err("No hay grabación activa".to_string()),
        }
    }

    pub fn list_monitors() -> Vec<String> {
        if let Ok(output) = std::process::Command::new("gpu-screen-recorder")
            .arg("--list-monitors")
            .output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let monitors: Vec<String> = text
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.split('|').next().unwrap_or(trimmed).to_string())
                    }
                })
                .collect();
            if !monitors.is_empty() {
                return monitors;
            }
        }

        if let Ok(output) = std::process::Command::new("grim").arg("-l").output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let list: Vec<String> = text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            if !list.is_empty() {
                return list;
            }
        }

        vec!["screen".to_string()]
    }

    pub fn list_audio_devices() -> Vec<String> {
        if let Ok(output) = std::process::Command::new("gpu-screen-recorder")
            .arg("--list-audio-devices")
            .output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            return text
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.split('|').next().unwrap_or(trimmed).to_string())
                    }
                })
                .collect();
        }
        vec!["default_output".to_string(), "default_input".to_string()]
    }

    fn generate_default_output_path(container: &str) -> PathBuf {
        let base_dir = dirs::video_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Videos")
        });
        let filename = format!(
            "Capsule_Record_{}.{}",
            Local::now().format("%Y-%m-%d_%H-%M-%S"),
            container
        );
        base_dir.join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RecordConfig;

    #[test]
    fn test_record_options_default() {
        let opts = RecordOptions::default();
        assert_eq!(opts.backend, RecordBackend::GpuScreenRecorder);
        assert_eq!(opts.target, RecordTarget::Screen);
        assert_eq!(opts.fps, 60);
        assert_eq!(opts.container, "mp4");
        assert_eq!(opts.quality, "very_high");
        assert!(opts.include_cursor);
    }

    #[test]
    fn test_record_options_from_config() {
        let config = RecordConfig {
            output: "screen".to_string(),
            fps: 30,
            resolution: "1920x1080".to_string(),
            quality: "high".to_string(),
            container: "mkv".to_string(),
            audio: "mic".to_string(),
            include_cursor: false,
        };

        let opts = RecordOptions::from_config(&config);
        assert_eq!(opts.target, RecordTarget::Screen);
        assert_eq!(opts.fps, 30);
        assert_eq!(opts.resolution, "1920x1080");
        assert_eq!(opts.quality, "high");
        assert_eq!(opts.container, "mkv");
        assert_eq!(opts.audio, RecordAudio::Microphone);
        assert!(!opts.include_cursor);
    }

    #[test]
    fn test_backend_as_str() {
        assert_eq!(
            RecordBackend::GpuScreenRecorder.as_str(),
            "gpu-screen-recorder"
        );
        assert_eq!(RecordBackend::WlScreenRec.as_str(), "wl-screenrec");
    }

    #[test]
    fn test_record_status() {
        let service = RecordService::new();
        assert_eq!(service.get_status(), RecordStatus::Stopped);
        assert!(!service.is_recording());
        assert!(!service.is_paused());
    }

    #[test]
    fn test_generate_default_output_path() {
        let path = RecordService::generate_default_output_path("mp4");
        assert!(path.to_string_lossy().ends_with(".mp4"));
        assert!(path.to_string_lossy().contains("Capsule_Record_"));
    }

    #[tokio::test]
    async fn test_live_recording_lifecycle() {
        if !RecordBackend::GpuScreenRecorder.is_available() {
            return;
        }

        let temp_dir = std::env::temp_dir();
        let output_file = temp_dir.join("test_capsule_live_record.mp4");
        let _ = std::fs::remove_file(&output_file);

        let service = RecordService::new();
        let options = RecordOptions {
            backend: RecordBackend::GpuScreenRecorder,
            target: RecordTarget::Screen,
            audio: RecordAudio::None,
            fps: 30,
            resolution: "native".to_string(),
            quality: "very_high".to_string(),
            container: "mp4".to_string(),
            output_path: Some(output_file.clone()),
            include_cursor: false,
        };

        let start_res = service.start(options).await;
        if start_res.is_err() {
            return;
        }

        assert!(service.is_recording());
        tokio::time::sleep(Duration::from_millis(1500)).await;
        assert!(service.is_recording());

        let stop_res = service.stop().await;
        assert!(stop_res.is_ok());
        assert_eq!(service.get_status(), RecordStatus::Stopped);

        assert!(output_file.exists());
        let meta = std::fs::metadata(&output_file);
        assert!(meta.map(|m| m.len() > 0).unwrap_or(false));
        let _ = std::fs::remove_file(&output_file);
    }

    #[test]
    fn test_active_session_duration_math() {
        let mut session = ActiveSession {
            backend: RecordBackend::GpuScreenRecorder,
            output_path: PathBuf::from("/tmp/dummy.mp4"),
            ipc_socket_path: None,
            child_pid: 1234,
            accumulated_active_duration: Duration::from_secs(5),
            current_segment_start: None,
        };

        assert_eq!(session.current_duration().as_secs(), 5);

        session.current_segment_start = Some(Instant::now());
        std::thread::sleep(Duration::from_millis(50));
        assert!(session.current_duration() >= Duration::from_millis(5050));

        let start = session
            .current_segment_start
            .take()
            .unwrap_or_else(Instant::now);
        session.accumulated_active_duration += start.elapsed();
        assert!(session.current_duration().as_secs() >= 5);
        assert!(session.current_segment_start.is_none());
    }

    #[test]
    fn test_record_service_duration_stopped() {
        let service = RecordService::new();
        assert_eq!(service.get_duration_secs(), 0);
    }
}
