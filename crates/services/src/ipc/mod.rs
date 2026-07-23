pub mod service;

pub use service::{IpcMessage, IpcSubscriber, pop_ipc_command, push_ipc_command};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcCommand {
    ToggleLauncher,
    ToggleDashboard,
    ToggleNotification,
    ShowLauncher,
    ShowDashboard,
    ShowNotification,
    Hide,
    Default,
    Ping,
    Quit,
}

impl FromStr for IpcCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "toggle-launcher" | "toggle launcher" | "launcher" => Ok(IpcCommand::ToggleLauncher),
            "toggle-dashboard" | "toggle dashboard" | "dashboard" => {
                Ok(IpcCommand::ToggleDashboard)
            }
            "toggle-notification" | "toggle notification" | "notification" | "notifications" => {
                Ok(IpcCommand::ToggleNotification)
            }
            "show-launcher" | "show launcher" => Ok(IpcCommand::ShowLauncher),
            "show-dashboard" | "show dashboard" => Ok(IpcCommand::ShowDashboard),
            "show-notification" | "show notification" => Ok(IpcCommand::ShowNotification),
            "hide" | "close" => Ok(IpcCommand::Hide),
            "default" => Ok(IpcCommand::Default),
            "ping" => Ok(IpcCommand::Ping),
            "quit" | "exit" => Ok(IpcCommand::Quit),
            _ => anyhow::bail!("Unknown IPC command: '{s}'"),
        }
    }
}

pub fn encode_command(command: &IpcCommand) -> String {
    match command {
        IpcCommand::ToggleLauncher => "toggle-launcher".to_string(),
        IpcCommand::ToggleDashboard => "toggle-dashboard".to_string(),
        IpcCommand::ToggleNotification => "toggle-notification".to_string(),
        IpcCommand::ShowLauncher => "show-launcher".to_string(),
        IpcCommand::ShowDashboard => "show-dashboard".to_string(),
        IpcCommand::ShowNotification => "show-notification".to_string(),
        IpcCommand::Hide => "hide".to_string(),
        IpcCommand::Default => "default".to_string(),
        IpcCommand::Ping => "ping".to_string(),
        IpcCommand::Quit => "quit".to_string(),
    }
}

pub fn decode_command(payload: &str) -> Option<IpcCommand> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<IpcCommand>().ok()
}

pub fn get_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("capsule.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
        PathBuf::from(format!("/tmp/capsule-{user}.sock"))
    }
}
