pub mod hyprland;
pub mod kinetic;
pub mod niri;

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: i64,
    pub num: i32,
    pub name: String,
    pub is_special: bool,
    pub special_name: Option<String>,
}

impl Default for WorkspaceInfo {
    fn default() -> Self {
        Self {
            id: 1,
            num: 1,
            name: "1".to_string(),
            is_special: false,
            special_name: None,
        }
    }
}

pub trait Compositor: Send + Sync {
    fn get_refresh_rate(&self) -> f64;
    fn get_workspace(&self) -> Option<WorkspaceInfo>;
}

#[derive(Clone)]
pub struct CompositorService {
    refresh_rate: Arc<ArcSwap<f64>>,
    current_workspace: Arc<ArcSwap<WorkspaceInfo>>,
    workspace_tx: broadcast::Sender<WorkspaceInfo>,
}

impl CompositorService {
    pub fn new() -> Self {
        let refresh_rate = Arc::new(ArcSwap::from_pointee(60.0));
        let initial_ws = if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            hyprland::Hyprland::new().get_workspace()
        } else if std::env::var("NIRI_SOCKET").is_ok() {
            niri::Niri::new().get_workspace()
        } else {
            kinetic::KineticWE::new().get_workspace()
        }
        .unwrap_or_default();
        let current_workspace = Arc::new(ArcSwap::from_pointee(initial_ws));
        let (workspace_tx, _) = broadcast::channel(32);

        let service = Self {
            refresh_rate,
            current_workspace,
            workspace_tx,
        };

        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.run_polling_loop().await;
        });

        let service_workspace = service.clone();
        tokio::spawn(async move {
            service_workspace.run_workspace_events_loop().await;
        });

        service
    }

    pub fn get_refresh_rate(&self) -> f64 {
        **self.refresh_rate.load()
    }

    pub fn get_frame_duration(&self) -> Duration {
        let rate = self.get_refresh_rate().max(30.0);
        let micros = (1_000_000.0 / rate).round() as u64;
        Duration::from_micros(micros)
    }

    pub fn get_frame_duration_ms(&self) -> u64 {
        let rate = self.get_refresh_rate().max(30.0);
        (1000.0 / rate).round().max(1.0) as u64
    }

    pub fn get_workspace(&self) -> WorkspaceInfo {
        (**self.current_workspace.load()).clone()
    }

    pub fn on_change_workspace(&self) -> broadcast::Receiver<WorkspaceInfo> {
        self.workspace_tx.subscribe()
    }

    async fn run_workspace_events_loop(&self) {
        let use_hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
        let use_niri = !use_hyprland && std::env::var("NIRI_SOCKET").is_ok();

        let initial = tokio::task::spawn_blocking(move || {
            if use_hyprland {
                hyprland::Hyprland::new().get_workspace()
            } else if use_niri {
                niri::Niri::new().get_workspace()
            } else {
                kinetic::KineticWE::new().get_workspace()
            }
        })
        .await
        .ok()
        .flatten();

        if let Some(ws) = initial {
            let prev = self.current_workspace.load();
            if **prev != ws {
                self.current_workspace.store(Arc::new(ws.clone()));
                let _ = self.workspace_tx.send(ws);
            }
        }

        if use_hyprland {
            hyprland::run_events_listener(
                self.current_workspace.clone(),
                self.workspace_tx.clone(),
            )
            .await;
        } else if use_niri {
            niri::run_events_listener(self.current_workspace.clone(), self.workspace_tx.clone())
                .await;
        }
    }

    async fn run_polling_loop(&self) {
        let use_hyprland = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok();
        let use_niri = !use_hyprland && std::env::var("NIRI_SOCKET").is_ok();

        loop {
            // Call get_refresh_rate in a blocking thread so that:
            //  1. Any panic (e.g. old hyprland crate IPC bug) stays in that thread.
            //  2. Blocking I/O doesn't starve the async runtime.
            let rate = tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(|| {
                    if use_hyprland {
                        hyprland::Hyprland::new().get_refresh_rate()
                    } else if use_niri {
                        niri::Niri::new().get_refresh_rate()
                    } else {
                        60.0
                    }
                })
                .unwrap_or(60.0)
            })
            .await
            .unwrap_or(60.0);

            if rate > 0.0 {
                let prev = **self.refresh_rate.load();
                if (prev - rate).abs() > 0.1 {
                    crate::log_info!(
                        "COMPOSITOR",
                        "Detected active monitor refresh rate: {rate:.2} Hz (frame duration: {:.2} ms)",
                        1000.0 / rate
                    );
                }
                self.refresh_rate.store(Arc::new(rate));
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

impl Default for CompositorService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compositor_service_workspace_methods() {
        let service = CompositorService::new();
        let mut rx = service.on_change_workspace();

        let initial_ws = service.get_workspace();
        assert!(initial_ws.num >= 1);

        let test_ws = WorkspaceInfo {
            id: 42,
            num: 42,
            name: "42".to_string(),
            is_special: false,
            special_name: None,
        };

        let _ = service.workspace_tx.send(test_ws.clone());
        let received = rx.recv().await;
        assert_eq!(received.ok(), Some(test_ws));
    }

    #[tokio::test]
    async fn test_compositor_service_refresh_rate() {
        let service = CompositorService::new();
        let rate = service.get_refresh_rate();
        assert!(rate >= 30.0);

        let dur = service.get_frame_duration();
        assert!(dur.as_micros() > 0);

        let ms = service.get_frame_duration_ms();
        assert!(ms >= 1);
    }
}
