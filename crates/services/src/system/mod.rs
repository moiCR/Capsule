use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioSink {
    pub name: String,
    pub description: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    pub volume: u32,
    pub is_muted: bool,
    pub brightness: u32,
    pub audio_sinks: Vec<AudioSink>,
}

#[derive(Clone, Default)]
pub struct SystemService {
    status: Arc<ArcSwap<SystemStatus>>,
    target_volume: Arc<AtomicU32>,
    volume_pending: Arc<AtomicBool>,
}

impl SystemService {
    pub fn new() -> Self {
        let service = Self {
            status: Arc::new(ArcSwap::from_pointee(SystemStatus::default())),
            target_volume: Arc::new(AtomicU32::new(50)),
            volume_pending: Arc::new(AtomicBool::new(false)),
        };

        let service_clone = service.clone();
        tokio::spawn(async move {
            let _ = service_clone.refresh().await;
            service_clone.listen_pactl_events().await;
        });

        let service_volume = service.clone();
        tokio::spawn(async move {
            service_volume.run_volume_worker().await;
        });

        service
    }

    pub fn get_status(&self) -> SystemStatus {
        (**self.status.load()).clone()
    }

    async fn run_volume_worker(&self) {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            if self.volume_pending.swap(false, Ordering::SeqCst) {
                let target = self.target_volume.load(Ordering::SeqCst);
                let percent_str = format!("{target}%");

                if Command::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &percent_str])
                    .status()
                    .await
                    .is_err()
                {
                    let _ = Command::new("pactl")
                        .args(["set-sink-volume", "@DEFAULT_SINK@", &percent_str])
                        .status()
                        .await;
                }
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
    }

    pub async fn refresh(&self) -> Result<()> {
        let vol_muted = Self::fetch_audio_status().await.unwrap_or((50, false));
        let brightness = Self::fetch_brightness().await.unwrap_or(100);
        let audio_sinks = Self::fetch_audio_sinks().await;

        let new_status = SystemStatus {
            volume: vol_muted.0,
            is_muted: vol_muted.1,
            brightness,
            audio_sinks,
        };

        self.status.store(Arc::new(new_status));

        Ok(())
    }

    async fn listen_pactl_events(&self) {
        if let Ok(mut child) = Command::new("pactl")
            .arg("subscribe")
            .stdout(Stdio::piped())
            .spawn()
        {
            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if line.contains("sink") || line.contains("server") {
                        let _ = self.refresh().await;
                    }
                }
            }
        }
    }

    pub async fn set_volume(&self, percent: u32) -> Result<()> {
        self.set_volume_fast(percent);
        Ok(())
    }

    pub async fn set_default_sink(&self, sink_name: &str) -> Result<()> {
        if Command::new("pactl")
            .args(["set-default-sink", sink_name])
            .status()
            .await
            .is_err()
        {
            let _ = Command::new("wpctl")
                .args(["set-default", sink_name])
                .status()
                .await;
        }
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

        if Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
            .status()
            .await
            .is_err()
        {
            let _ = Command::new("pactl")
                .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                .status()
                .await;
        }

        let _ = is_muted;
        self.refresh().await
    }

    pub async fn set_brightness(&self, percent: u32) -> Result<()> {
        let percent = percent.min(100).max(5);
        let percent_str = format!("{percent}%");

        let _ = Command::new("brightnessctl")
            .args(["set", &percent_str])
            .status()
            .await;

        self.refresh().await
    }

    pub async fn lock() -> Result<()> {
        if Command::new("hyprlock").status().await.is_err()
            && Command::new("swaylock").status().await.is_err()
        {
            Command::new("loginctl")
                .arg("lock-session")
                .status()
                .await
                .context("Failed to lock session")?;
        }
        Ok(())
    }

    pub async fn suspend() -> Result<()> {
        Command::new("systemctl")
            .arg("suspend")
            .status()
            .await
            .context("Failed to suspend system")?;
        Ok(())
    }

    pub async fn reboot() -> Result<()> {
        Command::new("systemctl")
            .arg("reboot")
            .status()
            .await
            .context("Failed to reboot system")?;
        Ok(())
    }

    pub async fn poweroff() -> Result<()> {
        Command::new("systemctl")
            .arg("poweroff")
            .status()
            .await
            .context("Failed to power off system")?;
        Ok(())
    }

    async fn fetch_audio_status() -> Result<(u32, bool)> {
        if let Ok(output) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .await
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let is_muted = text.contains("[MUTED]");
            if let Some(vol_str) = text.split_whitespace().nth(1) {
                if let Ok(val) = vol_str.parse::<f32>() {
                    return Ok(((val * 100.0).round() as u32, is_muted));
                }
            }
        }

        if let Ok(output) = Command::new("pactl")
            .args(["get-sink-volume", "@DEFAULT_SINK@"])
            .output()
            .await
        {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(pos) = text.find('/') {
                let rest = &text[pos + 1..];
                if let Some(pos2) = rest.find('%') {
                    if let Ok(val) = rest[..pos2].trim().parse::<u32>() {
                        return Ok((val, false));
                    }
                }
            }
        }

        Ok((50, false))
    }

    async fn fetch_brightness() -> Result<u32> {
        if let Ok(output) = Command::new("brightnessctl").arg("info").output().await {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(pos) = text.find('(') {
                let rest = &text[pos + 1..];
                if let Some(pos2) = rest.find('%') {
                    if let Ok(val) = rest[..pos2].parse::<u32>() {
                        return Ok(val);
                    }
                }
            }
        }
        Ok(100)
    }

    async fn fetch_audio_sinks() -> Vec<AudioSink> {
        let default_sink = Command::new("pactl")
            .arg("get-default-sink")
            .output()
            .await
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_default();

        let mut sinks = Vec::new();
        if let Ok(output) = Command::new("pactl")
            .arg("list")
            .arg("sinks")
            .output()
            .await
        {
            let text = String::from_utf8_lossy(&output.stdout);
            let mut current_name = String::new();
            let mut current_desc = String::new();

            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("Name: ") {
                    if !current_name.is_empty() {
                        let is_def = current_name == default_sink;
                        sinks.push(AudioSink {
                            name: current_name.clone(),
                            description: if current_desc.is_empty() {
                                current_name.clone()
                            } else {
                                current_desc.clone()
                            },
                            is_default: is_def,
                        });
                        current_desc.clear();
                    }
                    current_name = trimmed.trim_start_matches("Name: ").trim().to_string();
                } else if trimmed.starts_with("Description: ") {
                    current_desc = trimmed
                        .trim_start_matches("Description: ")
                        .trim()
                        .to_string();
                }
            }

            if !current_name.is_empty() {
                let is_def = current_name == default_sink;
                sinks.push(AudioSink {
                    name: current_name,
                    description: if current_desc.is_empty() {
                        "Audio Output".to_string()
                    } else {
                        current_desc
                    },
                    is_default: is_def,
                });
            }
        }
        sinks
    }
}
