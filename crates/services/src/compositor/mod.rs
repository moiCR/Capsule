pub mod hyprland;
pub mod kwin;
pub mod mangowm;
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
    fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        vec![]
    }
    fn switch_workspace(&self, _id: i64) {}
    fn request_layer_focus(&self) {}
    fn logout(&self) -> futures::future::BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(crate::SystemService::logout())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompositorBackend {
    KWin,
    Hyprland,
    Niri,
    Mango,
}

impl CompositorBackend {
    fn detect(desktop: &str, kwin: bool, hyprland: bool, niri: bool, mango: bool) -> Self {
        for component in desktop.split(':') {
            match component.trim().to_ascii_lowercase().as_str() {
                "kde" | "plasma" | "kinetic" => return Self::KWin,
                "hyprland" => return Self::Hyprland,
                "niri" => return Self::Niri,
                "mango" | "mangowm" => return Self::Mango,
                _ => {}
            }
        }
        if kwin {
            Self::KWin
        } else if niri {
            Self::Niri
        } else if hyprland {
            Self::Hyprland
        } else if mango {
            Self::Mango
        } else {
            Self::KWin
        }
    }
}

#[derive(Clone)]
pub struct CompositorService {
    refresh_rate: Arc<ArcSwap<f64>>,
    current_workspace: Arc<ArcSwap<WorkspaceInfo>>,
    workspace_tx: broadcast::Sender<WorkspaceInfo>,
    compositor: Arc<dyn Compositor>,
    backend: CompositorBackend,
    kwin: Option<kwin::KWin>,
    mango: Option<mangowm::Mango>,
}

impl CompositorService {
    pub async fn logout(&self) -> anyhow::Result<()> {
        self.compositor.logout().await
    }

    pub fn is_kwin(&self) -> bool {
        self.backend == CompositorBackend::KWin
    }

    pub fn new() -> Self {
        let refresh_rate = Arc::new(ArcSwap::from_pointee(60.0));
        let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        let backend = CompositorBackend::detect(
            &desktop,
            std::env::var_os("KWIN_WAYLAND").is_some()
                || std::env::var_os("KDE_FULL_SESSION").is_some(),
            std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some(),
            std::env::var_os(niri_ipc::socket::SOCKET_PATH_ENV).is_some(),
            std::env::var_os(mango_ipc::SOCKET_PATH_ENV).is_some(),
        );
        let kwin = (backend == CompositorBackend::KWin).then(kwin::KWin::new);
        let mango = (backend == CompositorBackend::Mango).then(mangowm::Mango::new);
        let compositor: Arc<dyn Compositor> = if let Some(backend) = &kwin {
            Arc::new(backend.clone())
        } else if let Some(backend) = &mango {
            Arc::new(backend.clone())
        } else if backend == CompositorBackend::Hyprland {
            Arc::new(hyprland::Hyprland::new())
        } else {
            Arc::new(niri::Niri::new())
        };
        let initial_ws = compositor.get_workspace().unwrap_or_default();
        let current_workspace = Arc::new(ArcSwap::from_pointee(initial_ws));
        let (workspace_tx, _) = broadcast::channel(32);

        let service = Self {
            refresh_rate,
            current_workspace,
            workspace_tx,
            compositor,
            backend,
            kwin,
            mango,
        };

        let service_clone = service.clone();
        if let Some(backend) = service.mango.clone() {
            crate::spawn_tokio(mangowm::run_output_listener(backend));
        }
        crate::spawn_tokio(async move {
            service_clone.run_polling_loop().await;
        });

        let service_workspace = service.clone();
        crate::spawn_tokio(async move {
            service_workspace.run_workspace_events_loop().await;
        });

        service
    }

    pub async fn focused_application(&self) -> Option<String> {
        if let Some(mango) = &self.mango {
            return mango.focused_application().await;
        }
        if let Some(kwin) = &self.kwin {
            return kwin.focused_application().await;
        }
        let mut command = match self.backend {
            CompositorBackend::Hyprland => {
                let mut command = tokio::process::Command::new("hyprctl");
                command.args(["-j", "activewindow"]);
                command
            }
            CompositorBackend::Niri => {
                let mut command = tokio::process::Command::new("niri");
                command.args(["msg", "-j", "focused-window"]);
                command
            }
            _ => return None,
        };
        command.kill_on_drop(true);
        let output = tokio::time::timeout(Duration::from_secs(1), command.output())
            .await
            .ok()?
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
        value
            .get("class")
            .or_else(|| value.get("app_id"))?
            .as_str()
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
    }

    pub fn get_refresh_rate(&self) -> f64 {
        **self.refresh_rate.load()
    }

    pub fn get_frame_duration(&self) -> Duration {
        let mut rate = self.get_refresh_rate();
        if rate > 1000.0 {
            rate /= 1000.0;
        }
        let rate = rate.clamp(30.0, 360.0);
        let micros = (1_000_000.0 / rate).round() as u64;
        Duration::from_micros(micros).max(Duration::from_millis(2))
    }

    pub fn get_frame_duration_ms(&self) -> u64 {
        let mut rate = self.get_refresh_rate();
        if rate > 1000.0 {
            rate /= 1000.0;
        }
        let rate = rate.clamp(30.0, 360.0);
        (1000.0 / rate).round().max(1.0) as u64
    }

    pub fn get_workspace(&self) -> WorkspaceInfo {
        (**self.current_workspace.load()).clone()
    }

    pub fn on_change_workspace(&self) -> broadcast::Receiver<WorkspaceInfo> {
        self.workspace_tx.subscribe()
    }

    pub fn request_layer_focus(&self) {
        self.compositor.request_layer_focus();
    }

    pub fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        self.compositor.get_workspaces()
    }

    pub fn switch_workspace(&self, id: i64) {
        self.compositor.switch_workspace(id);
    }

    async fn run_workspace_events_loop(&self) {
        if let Some(backend) = &self.mango {
            mangowm::run_events_listener(
                backend.clone(),
                self.current_workspace.clone(),
                self.workspace_tx.clone(),
            )
            .await;
            return;
        }
        if let Some(backend) = &self.kwin {
            kwin::run_events_listener(
                backend.clone(),
                self.current_workspace.clone(),
                self.workspace_tx.clone(),
            )
            .await;
            return;
        }
        let compositor = self.compositor.clone();
        let initial = crate::spawn_blocking(move || compositor.get_workspace())
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

        if self.backend == CompositorBackend::Hyprland {
            hyprland::run_events_listener(
                self.current_workspace.clone(),
                self.workspace_tx.clone(),
            )
            .await;
        } else if self.backend == CompositorBackend::Niri {
            niri::run_events_listener(self.current_workspace.clone(), self.workspace_tx.clone())
                .await;
        }
    }

    async fn run_polling_loop(&self) {
        let use_hyprland = self.backend == CompositorBackend::Hyprland;
        let use_niri = self.backend == CompositorBackend::Niri;

        loop {
            if let Some(backend) = &self.mango {
                backend.refresh().await;
            }
            let mango = self.mango.clone();
            let kwin = self.kwin.clone();
            let mut rate = crate::spawn_blocking(move || {
                std::panic::catch_unwind(|| {
                    if let Some(backend) = mango {
                        backend.get_refresh_rate()
                    } else if let Some(backend) = kwin {
                        backend.get_refresh_rate()
                    } else if use_hyprland {
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

            if rate > 1000.0 {
                rate /= 1000.0;
            }
            let rate = rate.clamp(30.0, 360.0);

            let prev = **self.refresh_rate.load();
            if (prev - rate).abs() > 0.1 {
                crate::log_info!(
                    "COMPOSITOR",
                    "Detected active monitor refresh rate: {rate:.2} Hz (frame duration: {:.2} ms)",
                    1000.0 / rate
                );
            }
            self.refresh_rate.store(Arc::new(rate));

            let compositor = self.compositor.clone();
            if let Some(active) = crate::spawn_blocking(move || compositor.get_workspace())
                .await
                .ok()
                .flatten()
            {
                let prev_ws = self.current_workspace.load();
                if **prev_ws != active {
                    self.current_workspace.store(Arc::new(active.clone()));
                    let _ = self.workspace_tx.send(active);
                }
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

    #[test]
    fn backend_selection_prefers_current_desktop_over_inherited_sockets() {
        for (desktop, expected) in [
            ("niri", CompositorBackend::Niri),
            ("Niri", CompositorBackend::Niri),
            ("Wayland:niri", CompositorBackend::Niri),
            ("Hyprland", CompositorBackend::Hyprland),
            ("mango", CompositorBackend::Mango),
            ("KDE", CompositorBackend::KWin),
            ("Plasma", CompositorBackend::KWin),
            ("KDE:plasma", CompositorBackend::KWin),
        ] {
            assert_eq!(
                CompositorBackend::detect(desktop, true, true, true, true),
                expected,
                "{desktop}",
            );
        }
    }

    #[test]
    fn backend_selection_falls_back_to_session_sockets() {
        assert_eq!(
            CompositorBackend::detect("", false, false, true, true),
            CompositorBackend::Niri,
        );
        assert_eq!(
            CompositorBackend::detect("", false, true, false, false),
            CompositorBackend::Hyprland,
        );
        assert_eq!(
            CompositorBackend::detect("", false, false, false, true),
            CompositorBackend::Mango,
        );
        assert_eq!(
            CompositorBackend::detect("", false, false, false, false),
            CompositorBackend::KWin,
        );
    }

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
        assert!(dur.as_micros() >= 2000);

        let ms = service.get_frame_duration_ms();
        assert!(ms >= 1);

        // Test defensive normalization with mHz (144000 mHz = 144 Hz)
        service.refresh_rate.store(Arc::new(144000.0));
        let dur_144 = service.get_frame_duration();
        assert!(
            dur_144.as_micros() >= 6000 && dur_144.as_micros() <= 8000,
            "144000 mHz should produce ~6944 us, got: {:?}",
            dur_144
        );
        let ms_144 = service.get_frame_duration_ms();
        assert_eq!(ms_144, 7);

        // Test safety floor with extreme values
        service.refresh_rate.store(Arc::new(1_000_000.0));
        let dur_floor = service.get_frame_duration();
        assert!(dur_floor.as_micros() >= 2000);
    }
}
