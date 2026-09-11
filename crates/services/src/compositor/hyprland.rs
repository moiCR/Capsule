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

#[derive(serde::Deserialize)]
struct MonitorWorkspaceData {
    #[serde(default)]
    id: i64,
    #[serde(default)]
    name: String,
}

#[derive(serde::Deserialize)]
struct MonitorData {
    #[serde(default)]
    focused: bool,
    #[serde(rename = "refreshRate", default)]
    refresh_rate: f64,
    #[serde(rename = "activeWorkspace")]
    active_workspace: Option<MonitorWorkspaceData>,
    #[serde(rename = "specialWorkspace")]
    special_workspace: Option<MonitorWorkspaceData>,
}

fn query_hypr_command(cmd: &str) -> Option<String> {
    ensure_socket_link();
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    if sig.is_empty() {
        return None;
    }
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").ok();
    let socket_path = if let Some(ref r) = runtime_dir {
        let p = std::path::PathBuf::from(format!("{r}/hypr/{sig}/.socket.sock"));
        if p.exists() {
            p
        } else {
            std::path::PathBuf::from(format!("/tmp/hypr/{sig}/.socket.sock"))
        }
    } else {
        std::path::PathBuf::from(format!("/tmp/hypr/{sig}/.socket.sock"))
    };

    use std::io::{Read, Write};
    let mut stream = std::os::unix::net::UnixStream::connect(socket_path).ok()?;
    stream.write_all(cmd.as_bytes()).ok()?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp).ok()?;
    Some(resp)
}

fn query_hypr_monitors() -> Option<Vec<MonitorData>> {
    let json_str = query_hypr_command("j/monitors")?;
    serde_json::from_str(&json_str).ok()
}

impl Compositor for Hyprland {
    fn get_refresh_rate(&self) -> f64 {
        let res = catch_unwind(|| {
            if let Some(monitors) = query_hypr_monitors() {
                let mut fallback_rate = 0.0;
                for monitor in monitors {
                    if monitor.focused && monitor.refresh_rate > 0.0 {
                        return monitor.refresh_rate;
                    }
                    if fallback_rate <= 0.0 && monitor.refresh_rate > 0.0 {
                        fallback_rate = monitor.refresh_rate;
                    }
                }
                if fallback_rate > 0.0 {
                    return fallback_rate;
                }
            }

            if is_command_socket_available()
                && let Ok(monitors) = Monitors::get()
            {
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
        let res = catch_unwind(|| {
            if let Some(monitors) = query_hypr_monitors() {
                let focused_mon = monitors
                    .iter()
                    .find(|m| m.focused)
                    .or_else(|| monitors.first());
                if let Some(mon) = focused_mon {
                    let base_num = mon
                        .active_workspace
                        .as_ref()
                        .and_then(|w| {
                            w.name
                                .parse::<i32>()
                                .ok()
                                .or_else(|| if w.id > 0 { Some(w.id as i32) } else { None })
                        })
                        .unwrap_or(1);

                    if let Some(ref special) = mon.special_workspace {
                        if special.id != 0 && !special.name.is_empty() {
                            let special_name = special
                                .name
                                .strip_prefix("special:")
                                .unwrap_or(&special.name)
                                .to_string();
                            return Some(WorkspaceInfo {
                                id: special.id,
                                num: base_num,
                                name: special.name.clone(),
                                is_special: true,
                                special_name: Some(special_name),
                            });
                        }
                    }

                    if let Some(ref active) = mon.active_workspace {
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
                        let num = active
                            .name
                            .parse::<i32>()
                            .unwrap_or(active.id.max(1) as i32);
                        return Some(WorkspaceInfo {
                            id: active.id,
                            num,
                            name: active.name.clone(),
                            is_special,
                            special_name,
                        });
                    }
                }
            }

            if is_command_socket_available()
                && let Ok(active) = Workspace::get_active()
            {
                return Some(Self::from_hypr_workspace(&active));
            }

            None
        });
        res.ok().flatten()
    }

    fn request_layer_focus(&self) {}
}

pub fn parse_hyprland_event(line: &str, current: &WorkspaceInfo) -> Option<WorkspaceInfo> {
    if let Some(data) = line.strip_prefix("activespecialv2>>") {
        let mut parts = data.split(',');
        let id_str = parts.next().unwrap_or("");
        let name_str = parts.next().unwrap_or("");
        if name_str.is_empty() {
            return Some(WorkspaceInfo {
                id: current.num as i64,
                num: current.num,
                name: current.num.to_string(),
                is_special: false,
                special_name: None,
            });
        } else {
            let special_name = name_str
                .strip_prefix("special:")
                .unwrap_or(name_str)
                .to_string();
            let id = id_str.parse::<i64>().unwrap_or(-99);
            return Some(WorkspaceInfo {
                id,
                num: current.num,
                name: name_str.to_string(),
                is_special: true,
                special_name: Some(special_name),
            });
        }
    }

    if let Some(data) = line.strip_prefix("activespecial>>") {
        let mut parts = data.split(',');
        let name_str = parts.next().unwrap_or("");
        if name_str.is_empty() {
            return Some(WorkspaceInfo {
                id: current.num as i64,
                num: current.num,
                name: current.num.to_string(),
                is_special: false,
                special_name: None,
            });
        } else {
            let special_name = name_str
                .strip_prefix("special:")
                .unwrap_or(name_str)
                .to_string();
            return Some(WorkspaceInfo {
                id: -99,
                num: current.num,
                name: name_str.to_string(),
                is_special: true,
                special_name: Some(special_name),
            });
        }
    }

    if let Some(data) = line.strip_prefix("workspacev2>>") {
        let mut parts = data.split(',');
        let id_str = parts.next().unwrap_or("");
        let name_str = parts.next().unwrap_or("");
        let is_special = name_str.starts_with("special:") || id_str.starts_with('-');
        if is_special {
            let special_name = name_str
                .strip_prefix("special:")
                .unwrap_or(name_str)
                .to_string();
            let id = id_str.parse::<i64>().unwrap_or(-99);
            return Some(WorkspaceInfo {
                id,
                num: current.num,
                name: name_str.to_string(),
                is_special: true,
                special_name: Some(special_name),
            });
        } else {
            let num = name_str
                .parse::<i32>()
                .unwrap_or_else(|_| id_str.parse::<i32>().unwrap_or(1));
            let id = id_str.parse::<i64>().unwrap_or(num as i64);
            return Some(WorkspaceInfo {
                id,
                num,
                name: name_str.to_string(),
                is_special: false,
                special_name: None,
            });
        }
    }

    if let Some(name_str) = line.strip_prefix("workspace>>") {
        let is_special = name_str.starts_with("special:");
        if is_special {
            let special_name = name_str
                .strip_prefix("special:")
                .unwrap_or(name_str)
                .to_string();
            return Some(WorkspaceInfo {
                id: -99,
                num: current.num,
                name: name_str.to_string(),
                is_special: true,
                special_name: Some(special_name),
            });
        } else {
            let num = name_str.parse::<i32>().unwrap_or(1);
            return Some(WorkspaceInfo {
                id: num as i64,
                num,
                name: name_str.to_string(),
                is_special: false,
                special_name: None,
            });
        }
    }

    if let Some(data) = line.strip_prefix("focusedmon>>") {
        let mut parts = data.split(',');
        let _mon = parts.next();
        let name_str = parts.next().unwrap_or("");
        if !name_str.is_empty() {
            let is_special = name_str.starts_with("special:");
            if is_special {
                let special_name = name_str
                    .strip_prefix("special:")
                    .unwrap_or(name_str)
                    .to_string();
                return Some(WorkspaceInfo {
                    id: -99,
                    num: current.num,
                    name: name_str.to_string(),
                    is_special: true,
                    special_name: Some(special_name),
                });
            } else {
                let num = name_str.parse::<i32>().unwrap_or(1);
                return Some(WorkspaceInfo {
                    id: num as i64,
                    num,
                    name: name_str.to_string(),
                    is_special: false,
                    special_name: None,
                });
            }
        }
    }

    if let Some(data) = line.strip_prefix("focusedmonv2>>") {
        let mut parts = data.split(',');
        let _mon = parts.next();
        let id_str = parts.next().unwrap_or("");
        if let Ok(id) = id_str.parse::<i64>() {
            if id < 0 {
                return Some(WorkspaceInfo {
                    id,
                    num: current.num,
                    name: format!("special:{id}"),
                    is_special: true,
                    special_name: Some(id.to_string()),
                });
            } else {
                let num = id as i32;
                return Some(WorkspaceInfo {
                    id,
                    num,
                    name: num.to_string(),
                    is_special: false,
                    special_name: None,
                });
            }
        }
    }

    None
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
            let parsed = parse_hyprland_event(&line, &current_workspace.load());
            let ws_opt = if let Some(ws) = parsed {
                Some(ws)
            } else if line.starts_with("renameworkspace>>")
                || line.starts_with("createworkspace>>")
                || line.starts_with("createworkspacev2>>")
                || line.starts_with("destroyworkspace>>")
                || line.starts_with("destroyworkspacev2>>")
            {
                tokio::task::spawn_blocking(|| Hyprland::new().get_workspace())
                    .await
                    .ok()
                    .flatten()
            } else {
                None
            };

            if let Some(ws) = ws_opt {
                let prev = current_workspace.load();
                if **prev != ws {
                    current_workspace.store(Arc::new(ws.clone()));
                    let _ = tx.send(ws);
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

    #[test]
    fn test_parse_hyprland_event_special() {
        let base = WorkspaceInfo {
            id: 2,
            num: 2,
            name: "2".to_string(),
            is_special: false,
            special_name: None,
        };

        let ws_open = parse_hyprland_event("activespecialv2>>-99,special:scratchpad,DP-1", &base);
        assert_eq!(
            ws_open,
            Some(WorkspaceInfo {
                id: -99,
                num: 2,
                name: "special:scratchpad".to_string(),
                is_special: true,
                special_name: Some("scratchpad".to_string()),
            })
        );

        let ws_close = parse_hyprland_event("activespecialv2>>,,DP-1", &base);
        assert_eq!(
            ws_close,
            Some(WorkspaceInfo {
                id: 2,
                num: 2,
                name: "2".to_string(),
                is_special: false,
                special_name: None,
            })
        );

        let ws_open_v1 = parse_hyprland_event("activespecial>>special:music,DP-1", &base);
        assert_eq!(
            ws_open_v1,
            Some(WorkspaceInfo {
                id: -99,
                num: 2,
                name: "special:music".to_string(),
                is_special: true,
                special_name: Some("music".to_string()),
            })
        );

        let ws_close_v1 = parse_hyprland_event("activespecial>>,DP-1", &base);
        assert_eq!(
            ws_close_v1,
            Some(WorkspaceInfo {
                id: 2,
                num: 2,
                name: "2".to_string(),
                is_special: false,
                special_name: None,
            })
        );
    }

    #[test]
    fn test_parse_hyprland_event_regular() {
        let base = WorkspaceInfo::default();

        let ws_reg_v2 = parse_hyprland_event("workspacev2>>3,3", &base);
        assert_eq!(
            ws_reg_v2,
            Some(WorkspaceInfo {
                id: 3,
                num: 3,
                name: "3".to_string(),
                is_special: false,
                special_name: None,
            })
        );

        let ws_reg_v1 = parse_hyprland_event("workspace>>4", &base);
        assert_eq!(
            ws_reg_v1,
            Some(WorkspaceInfo {
                id: 4,
                num: 4,
                name: "4".to_string(),
                is_special: false,
                special_name: None,
            })
        );

        let ws_mon = parse_hyprland_event("focusedmon>>DP-1,5", &base);
        assert_eq!(
            ws_mon,
            Some(WorkspaceInfo {
                id: 5,
                num: 5,
                name: "5".to_string(),
                is_special: false,
                special_name: None,
            })
        );
    }
}
