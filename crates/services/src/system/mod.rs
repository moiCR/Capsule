use anyhow::Result;
use arc_swap::ArcSwap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

pub mod audio;
pub mod brightness;
mod capture;
pub mod power;
mod usage;

pub use audio::{AudioSink, AudioSource};
pub use capture::CaptureStatus;
pub use usage::UsageStatus;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemStatus {
    pub volume: u32,
    pub is_muted: bool,
    pub input_volume: u32,
    pub is_input_muted: bool,
    pub brightness: u32,
    pub audio_sinks: Vec<AudioSink>,
    pub audio_sources: Vec<AudioSource>,
}

#[derive(Clone)]
pub struct SystemService {
    status: Arc<ArcSwap<SystemStatus>>,
    capture_status: Arc<ArcSwap<CaptureStatus>>,
    target_volume: Arc<AtomicU32>,
    volume_pending: Arc<AtomicBool>,
    volume_notify: Arc<tokio::sync::Notify>,
    target_input_volume: Arc<AtomicU32>,
    input_volume_pending: Arc<AtomicBool>,
    input_volume_notify: Arc<tokio::sync::Notify>,
    audio_changed: Arc<tokio::sync::Notify>,
    usage_sample: Arc<tokio::sync::Mutex<Option<(u64, u64)>>>,
    current_usage: Arc<ArcSwap<UsageStatus>>,
}

impl SystemService {
    pub fn new() -> Self {
        let _guard = crate::tokio_handle().enter();
        let service = Self {
            status: Arc::new(ArcSwap::from_pointee(SystemStatus::default())),
            capture_status: Arc::new(ArcSwap::from_pointee(CaptureStatus::default())),
            target_volume: Arc::new(AtomicU32::new(0)),
            volume_pending: Arc::new(AtomicBool::new(false)),
            volume_notify: Arc::new(tokio::sync::Notify::new()),
            target_input_volume: Arc::new(AtomicU32::new(0)),
            input_volume_pending: Arc::new(AtomicBool::new(false)),
            input_volume_notify: Arc::new(tokio::sync::Notify::new()),
            audio_changed: Arc::new(tokio::sync::Notify::new()),
            usage_sample: Arc::new(tokio::sync::Mutex::new(None)),
            current_usage: Arc::new(ArcSwap::from_pointee(UsageStatus::default())),
        };

        crate::spawn_tokio(capture::monitor(service.capture_status.clone()));

        let usage_service = service.clone();
        crate::spawn_tokio(async move {
            loop {
                let u = usage_service.sample_usage().await;
                usage_service.current_usage.store(Arc::new(u));
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        let service_clone = service.clone();
        crate::spawn_tokio(async move {
            let _ = service_clone.refresh().await;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let _ = service_clone.refresh_audio().await;
            }
        });

        let service_events = service.clone();
        crate::spawn_tokio(async move {
            audio::listen_pactl_events(move || {
                let s = service_events.clone();
                tokio::spawn(async move {
                    let _ = s.refresh().await;
                })
            })
            .await;
        });

        let service_volume = service.clone();
        crate::spawn_tokio(async move {
            service_volume.run_volume_worker().await;
        });

        let service_input = service.clone();
        crate::spawn_tokio(async move {
            service_input.run_input_volume_worker().await;
        });

        service
    }

    pub async fn reset_usage_sample(&self) {
        *self.usage_sample.lock().await = None;
    }

    pub async fn sample_usage(&self) -> UsageStatus {
        let mut previous = self.usage_sample.lock().await;
        usage::sample(&mut previous).await
    }

    pub fn get_status(&self) -> SystemStatus {
        (**self.status.load()).clone()
    }

    pub fn get_usage_status(&self) -> UsageStatus {
        **self.current_usage.load()
    }

    pub fn get_capture_status(&self) -> CaptureStatus {
        **self.capture_status.load()
    }

    pub fn audio_changed(&self) -> Arc<tokio::sync::Notify> {
        self.audio_changed.clone()
    }

    pub async fn monitor_audio_levels(sender: tokio::sync::watch::Sender<[f32; 12]>) {
        audio::monitor_audio_levels(sender).await
    }

    pub fn audio_levels(frame: &[u8; 1536]) -> [f32; 12] {
        audio::audio_levels(frame)
    }

    async fn run_volume_worker(&self) {
        loop {
            self.volume_notify.notified().await;
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;

            if self.volume_pending.swap(false, Ordering::SeqCst) {
                let target = self.target_volume.load(Ordering::SeqCst);
                audio::apply_volume(target).await;
            }
        }
    }

    async fn run_input_volume_worker(&self) {
        loop {
            self.input_volume_notify.notified().await;
            tokio::time::sleep(std::time::Duration::from_millis(30)).await;

            if self.input_volume_pending.swap(false, Ordering::SeqCst) {
                let target = self.target_input_volume.load(Ordering::SeqCst);
                audio::apply_input_volume(target).await;
            }
        }
    }

    pub fn set_volume_fast(&self, percent: u32) {
        let percent = percent.min(100);
        let mut current = (**self.status.load()).clone();
        current.volume = percent;
        if percent > 0 {
            current.is_muted = false;
        }
        self.status.store(Arc::new(current));

        self.target_volume.store(percent, Ordering::SeqCst);
        self.volume_pending.store(true, Ordering::SeqCst);
        self.volume_notify.notify_one();
        self.audio_changed.notify_waiters();
    }

    pub fn set_input_volume_fast(&self, percent: u32) {
        let percent = percent.min(100);
        let mut current = (**self.status.load()).clone();
        current.input_volume = percent;
        if percent > 0 {
            current.is_input_muted = false;
        }
        self.status.store(Arc::new(current));

        self.target_input_volume.store(percent, Ordering::SeqCst);
        self.input_volume_pending.store(true, Ordering::SeqCst);
        self.input_volume_notify.notify_one();
        self.audio_changed.notify_waiters();
    }

    pub async fn refresh_audio(&self) -> Result<()> {
        let output = audio::fetch_audio_status().await;
        let input = audio::fetch_input_audio_status().await;
        let mut current = (**self.status.load()).clone();
        let (volume, is_muted) = output.unwrap_or((current.volume, current.is_muted));
        let (input_volume, is_input_muted) =
            input.unwrap_or((current.input_volume, current.is_input_muted));
        if current.volume != volume
            || current.is_muted != is_muted
            || current.input_volume != input_volume
            || current.is_input_muted != is_input_muted
        {
            current.volume = volume;
            current.is_muted = is_muted;
            current.input_volume = input_volume;
            current.is_input_muted = is_input_muted;
            self.status.store(Arc::new(current));
            self.audio_changed.notify_waiters();
        }
        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        let output = audio::fetch_audio_status().await;
        let input = audio::fetch_input_audio_status().await;
        let brightness = brightness::fetch_brightness().await.unwrap_or(100);
        let audio_sinks = audio::fetch_audio_sinks().await;
        let audio_sources = audio::fetch_audio_sources().await;

        let current = self.status.load();
        let vol_muted = output.unwrap_or((current.volume, current.is_muted));
        let input_vol_muted = input.unwrap_or((current.input_volume, current.is_input_muted));
        let new_status = SystemStatus {
            volume: vol_muted.0,
            is_muted: vol_muted.1,
            input_volume: input_vol_muted.0,
            is_input_muted: input_vol_muted.1,
            brightness,
            audio_sinks,
            audio_sources,
        };

        self.status.store(Arc::new(new_status));
        self.audio_changed.notify_waiters();

        Ok(())
    }

    pub async fn set_volume(&self, percent: u32) -> Result<()> {
        self.set_volume_fast(percent);
        Ok(())
    }

    pub async fn set_input_volume(&self, percent: u32) -> Result<()> {
        self.set_input_volume_fast(percent);
        Ok(())
    }

    pub async fn set_default_sink(&self, sink_name: &str) -> Result<()> {
        audio::set_default_sink(sink_name).await?;
        self.refresh().await
    }

    pub async fn set_default_source(&self, source_name: &str) -> Result<()> {
        audio::set_default_source(source_name).await?;
        self.refresh().await
    }

    pub async fn toggle_mute(&self) -> Result<()> {
        let is_muted = {
            let mut current = (**self.status.load()).clone();
            current.is_muted = !current.is_muted;
            let res = current.is_muted;
            self.status.store(Arc::new(current));
            res
        };

        audio::apply_toggle_mute().await;

        let _ = is_muted;
        self.refresh().await
    }

    pub async fn toggle_input_mute(&self) -> Result<()> {
        let is_muted = {
            let mut current = (**self.status.load()).clone();
            current.is_input_muted = !current.is_input_muted;
            let res = current.is_input_muted;
            self.status.store(Arc::new(current));
            res
        };

        audio::apply_toggle_input_mute().await;

        let _ = is_muted;
        self.refresh().await
    }

    pub async fn set_brightness(&self, percent: u32) -> Result<()> {
        brightness::set_brightness(percent).await?;
        self.refresh().await
    }

    pub async fn lock() -> Result<()> {
        power::lock().await
    }

    pub async fn suspend() -> Result<()> {
        power::suspend().await
    }

    pub async fn reboot() -> Result<()> {
        power::reboot().await
    }

    pub async fn logout() -> Result<()> {
        power::logout().await
    }

    pub async fn poweroff() -> Result<()> {
        power::poweroff().await
    }

    #[cfg(test)]
    pub(crate) async fn fetch_audio_status() -> Result<(u32, bool)> {
        audio::fetch_audio_status().await
    }

    #[cfg(test)]
    pub(crate) async fn fetch_input_audio_status() -> Result<(u32, bool)> {
        audio::fetch_input_audio_status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn audio_visualizer_tracks_pcm_energy() {
        assert_eq!(SystemService::audio_levels(&[0; 1536]), [0.0; 12]);
        let mut frame = [0_u8; 1536];
        for sample in frame[..128].chunks_exact_mut(2) {
            sample.copy_from_slice(&8192_i16.to_le_bytes());
        }
        let levels = SystemService::audio_levels(&frame);
        assert_eq!(levels[0], 1.0);
        assert_eq!(&levels[1..], &[0.0; 11]);
        for sample in frame.chunks_exact_mut(2) {
            sample.copy_from_slice(&i16::MIN.to_le_bytes());
        }
        assert_eq!(SystemService::audio_levels(&frame), [1.0; 12]);
    }

    #[test]
    fn audio_refresh_recovers_and_preserves_last_volume() -> Result<()> {
        const FIXTURE_ENV: &str = "CAPSULE_AUDIO_TEST_DIR";
        if let Some(directory) = std::env::var_os(FIXTURE_ENV) {
            let directory = std::path::PathBuf::from(directory);
            return tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(async {
                    let service = SystemService {
                        status: Arc::new(ArcSwap::from_pointee(SystemStatus::default())),
                        capture_status: Arc::new(ArcSwap::from_pointee(CaptureStatus::default())),
                        target_volume: Arc::new(AtomicU32::new(0)),
                        volume_pending: Arc::new(AtomicBool::new(false)),
                        volume_notify: Arc::new(tokio::sync::Notify::new()),
                        target_input_volume: Arc::new(AtomicU32::new(0)),
                        input_volume_pending: Arc::new(AtomicBool::new(false)),
                        input_volume_notify: Arc::new(tokio::sync::Notify::new()),
                        audio_changed: Arc::new(tokio::sync::Notify::new()),
                        usage_sample: Arc::new(tokio::sync::Mutex::new(None)),
                        current_usage: Arc::new(ArcSwap::from_pointee(UsageStatus::default())),
                    };
                    assert!(SystemService::fetch_audio_status().await.is_err());
                    assert!(SystemService::fetch_input_audio_status().await.is_err());
                    service.refresh().await?;
                    assert_eq!(service.get_status().volume, 0);

                    let sink = directory.join("@DEFAULT_AUDIO_SINK@");
                    let source = directory.join("@DEFAULT_AUDIO_SOURCE@");
                    std::fs::write(&sink, "Volume: 0.83 [MUTED]\n")?;
                    service.refresh_audio().await?;
                    assert_eq!(service.get_status().volume, 83);
                    assert!(service.get_status().is_muted);

                    std::fs::write(&source, "Volume: 0.27 [MUTED]\n")?;
                    service.refresh().await?;
                    assert_eq!(service.get_status().input_volume, 27);
                    assert!(service.get_status().is_input_muted);

                    std::fs::write(directory.join("fail"), "")?;
                    std::fs::write(&sink, "Volume: 0.50\n")?;
                    std::fs::write(&source, "Volume: 0.50\n")?;
                    service.refresh_audio().await?;
                    service.refresh().await?;
                    assert_eq!(service.get_status().volume, 83);
                    assert!(service.get_status().is_muted);
                    assert_eq!(service.get_status().input_volume, 27);
                    assert!(service.get_status().is_input_muted);

                    std::fs::remove_file(directory.join("fail"))?;
                    std::fs::write(&sink, "Volume: 0.64\n")?;
                    service.refresh_audio().await?;
                    assert_eq!(service.get_status().volume, 64);
                    assert!(!service.get_status().is_muted);
                    Ok(())
                });
        }

        let directory = std::env::temp_dir().join(format!(
            "capsule-audio-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&directory)?;
        for (name, script) in [
            (
                "wpctl",
                "#!/bin/sh\n/bin/cat \"$CAPSULE_AUDIO_TEST_DIR/$2\" || exit 1\n[ ! -f \"$CAPSULE_AUDIO_TEST_DIR/fail\" ]\n",
            ),
            ("pactl", "#!/bin/sh\nexit 1\n"),
            ("brightnessctl", "#!/bin/sh\nexit 1\n"),
        ] {
            let path = directory.join(name);
            std::fs::write(&path, script)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
        }
        let result = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "system::tests::audio_refresh_recovers_and_preserves_last_volume",
                "--nocapture",
            ])
            .env(FIXTURE_ENV, &directory)
            .env("PATH", &directory)
            .output();
        std::fs::remove_dir_all(&directory)?;
        let output = result?;
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
