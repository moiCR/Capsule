use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Default)]
pub struct SystemStatus {
    pub volume: u32,
    pub is_muted: bool,
    pub brightness: u32,
}

#[derive(Clone, Default)]
pub struct SystemService {
    status: Arc<ArcSwap<SystemStatus>>,
}

impl SystemService {
    pub fn new() -> Self {
        let service = Self {
            status: Arc::new(ArcSwap::from_pointee(SystemStatus::default())),
        };

        let service_clone = service.clone();
        tokio::spawn(async move {
            let _ = service_clone.refresh().await;
            service_clone.listen_pactl_events().await;
        });

        service
    }

    pub fn get_status(&self) -> SystemStatus {
        (**self.status.load()).clone()
    }

    pub async fn refresh(&self) -> Result<()> {
        let vol_muted = Self::fetch_audio_status().await.unwrap_or((50, false));
        let brightness = Self::fetch_brightness().await.unwrap_or(100);

        let new_status = SystemStatus {
            volume: vol_muted.0,
            is_muted: vol_muted.1,
            brightness,
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
                    if line.contains("sink") {
                        let _ = self.refresh().await;
                    }
                }
            }
        }
    }

    pub async fn set_volume(&self, percent: u32) -> Result<()> {
        let percent = percent.min(100);
        let percent_str = format!("{percent}%");

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

        self.refresh().await
    }

    pub async fn toggle_mute(&self) -> Result<()> {
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
                if let Some(end) = rest.find('%') {
                    let num_str = rest[..end].trim();
                    if let Ok(val) = num_str.parse::<u32>() {
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
                if let Some(end) = rest.find("%)") {
                    let num_str = &rest[..end];
                    if let Ok(val) = num_str.parse::<u32>() {
                        return Ok(val);
                    }
                }
            }
        }

        Ok(100)
    }
}
