use crate::compositor::{Compositor, WorkspaceInfo};
use arc_swap::ArcSwap;
use hyprland::data::{Monitors, Workspace};
use hyprland::shared::{HyprData, HyprDataActive, WorkspaceType};
use std::panic::catch_unwind;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

fn ensure_socket_link() {
    let Ok(sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") else {
        return;
    };
    if sig.is_empty() {
        return;
    }

    let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") else {
        return;
    };

    let target = std::path::PathBuf::from(format!("{runtime_dir}/hypr/{sig}"));
    if !target.exists() {
        return;
    }

    let link_dir = std::path::PathBuf::from("/tmp/hypr");
    let link_path = link_dir.join(&sig);

    if link_path.is_symlink() && !link_path.exists() {
        let _ = std::fs::remove_file(&link_path);
    }

    if !link_path.exists() {
        let _ = std::fs::create_dir_all(&link_dir);
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&target, &link_path);
    }
}

fn is_command_socket_available() -> bool {
    ensure_socket_link();
    let Ok(sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") else {
        return false;
    };
    if sig.is_empty() {
        return false;
    }
    std::path::Path::new(&format!("/tmp/hypr/{sig}/.socket.sock")).exists()
}

#[derive(Default)]
pub struct Hyprland;

impl Hyprland {
    pub fn new() -> Self {
        ensure_socket_link();
        Self
    }

    pub fn from_hypr_workspace(active: &Workspace) -> WorkspaceInfo {
        let is_special = active.name.starts_with("special:") || active.id < 0;
        let special_name = if is_special {
            Some(
                active
                    .name
                    .strip_prefix("special:")
                    .unwrap_or(&active.name)
                    .to_string(),
            )
        } else {
            None
        };

        let num = active.name.parse::<i32>().unwrap_or(active.id.max(1));

        WorkspaceInfo {
            id: active.id as i64,
            num,
            name: active.name.clone(),
            is_special,
            special_name,
        }
    }

    pub fn from_workspace_type(ws_type: &WorkspaceType, fallback_base_num: i32) -> WorkspaceInfo {
        match ws_type {
            WorkspaceType::Regular(name) => {
                let num = name.parse::<i32>().unwrap_or(1);
                WorkspaceInfo {
                    id: num as i64,
                    num,
                    name: name.clone(),
                    is_special: false,
                    special_name: None,
                }
            }
            WorkspaceType::Special(opt_name) => {
                let special_name = opt_name.clone().unwrap_or_else(|| "special".to_string());
                let name = format!("special:{special_name}");
                WorkspaceInfo {
                    id: -99,
                    num: fallback_base_num,
                    name,
                    is_special: true,
                    special_name: Some(special_name),
                }
            }
        }
    }
}

impl Compositor for Hyprland {
    fn get_refresh_rate(&self) -> f64 {
        if !is_command_socket_available() {
            return 60.0;
        }

        let res = catch_unwind(|| {
            if let Ok(monitors) = Monitors::get() {
                let mut fallback_rate = 0.0;
                for monitor in monitors {
                    if monitor.focused && monitor.refresh_rate > 0.0 {
                        return monitor.refresh_rate as f64;
                    }
                    if fallback_rate <= 0.0 && monitor.refresh_rate > 0.0 {
                        fallback_rate = monitor.refresh_rate as f64;
                    }
                }
                if fallback_rate > 0.0 {
                    return fallback_rate;
                }
            }
            60.0
        });
        res.unwrap_or(60.0)
    }

    fn get_workspace(&self) -> Option<WorkspaceInfo> {
        if !is_command_socket_available() {
            return None;
        }

        let res = catch_unwind(|| {
            let active = Workspace::get_active().ok()?;
            Some(Self::from_hypr_workspace(&active))
        });
        res.ok().flatten()
    }
}

pub async fn run_events_listener(
    current_workspace: Arc<ArcSwap<WorkspaceInfo>>,
    tx: broadcast::Sender<WorkspaceInfo>,
) {
    let signature = match std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        Ok(s) if !s.is_empty() => s,
        _ => return,
    };

    let socket_path = match std::env::var("XDG_RUNTIME_DIR") {
        Ok(runtime_dir) => format!("{runtime_dir}/hypr/{signature}/.socket2.sock"),
        Err(_) => format!("/tmp/hypr/{signature}/.socket2.sock"),
    };

    loop {
        let Ok(stream) = tokio::net::UnixStream::connect(&socket_path).await else {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        };

        use tokio::io::AsyncBufReadExt;
        let mut reader = tokio::io::BufReader::new(stream).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            let is_workspace_event = line.starts_with("workspace>>")
                || line.starts_with("workspacev2>>")
                || line.starts_with("focusedmon>>")
                || line.starts_with("focusedmonv2>>")
                || line.starts_with("activespecial>>")
                || line.starts_with("createworkspace>>")
                || line.starts_with("createworkspacev2>>")
                || line.starts_with("destroyworkspace>>")
                || line.starts_with("destroyworkspacev2>>")
                || line.starts_with("renameworkspace>>");

            if is_workspace_event {
                let maybe_ws = tokio::task::spawn_blocking(|| Hyprland::new().get_workspace())
                    .await
                    .ok()
                    .flatten();

                if let Some(ws) = maybe_ws {
                    let prev = current_workspace.load();
                    if **prev != ws {
                        current_workspace.store(Arc::new(ws.clone()));
                        let _ = tx.send(ws);
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_workspace_type_regular() {
        let ws_type = WorkspaceType::Regular("5".to_string());
        let ws = Hyprland::from_workspace_type(&ws_type, 1);
        assert_eq!(
            ws,
            WorkspaceInfo {
                id: 5,
                num: 5,
                name: "5".to_string(),
                is_special: false,
                special_name: None,
            }
        );
    }

    #[test]
    fn test_from_workspace_type_special() {
        let ws_type = WorkspaceType::Special(Some("scratchpad".to_string()));
        let ws = Hyprland::from_workspace_type(&ws_type, 3);
        assert_eq!(
            ws,
            WorkspaceInfo {
                id: -99,
                num: 3,
                name: "special:scratchpad".to_string(),
                is_special: true,
                special_name: Some("scratchpad".to_string()),
            }
        );
    }

    #[test]
    fn test_from_hypr_workspace_regular() {
        let hypr_ws = Workspace {
            id: 2,
            name: "2".to_string(),
            monitor: "DP-1".to_string(),
            windows: 2,
            fullscreen: false,
            last_window: hyprland::shared::Address::new("0x0"),
            last_window_title: String::new(),
        };

        let ws = Hyprland::from_hypr_workspace(&hypr_ws);
        assert_eq!(
            ws,
            WorkspaceInfo {
                id: 2,
                num: 2,
                name: "2".to_string(),
                is_special: false,
                special_name: None,
            }
        );
    }

    #[test]
    fn test_real_hyprland_ipc() {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let ws = Hyprland::new().get_workspace();
            assert!(ws.is_some());
            let rate = Hyprland::new().get_refresh_rate();
            assert!(rate > 0.0);
        }
    }
}
