use crate::compositor::Compositor;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct HyprMonitor {
    pub focused: bool,
    #[serde(rename = "refreshRate")]
    pub refresh_rate: f64,
}

pub struct Hyprland;

impl Hyprland {
    pub fn new() -> Self {
        Self
    }
}

impl Compositor for Hyprland {
    fn get_refresh_rate(&self) -> f64 {
        if let Ok(output) = Command::new("hyprctl").args(["-j", "monitors"]).output() {
            if output.status.success() {
                if let Ok(monitors) = serde_json::from_slice::<Vec<HyprMonitor>>(&output.stdout) {
                    if let Some(focused) = monitors.iter().find(|m| m.focused) {
                        if focused.refresh_rate > 0.0 {
                            return focused.refresh_rate;
                        }
                    }
                }
            }
        }
        60.0
    }
}
