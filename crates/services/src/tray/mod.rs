pub mod dbus_menu;
pub mod sni_host;
pub use dbus_menu::DBusMenuItem;
pub use sni_host::{SniHostService, SniItem};

use std::sync::{Arc, Mutex};

use ksni::TrayMethods; // provides .spawn()

/// Actions the tray can request from the main app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    ToggleDashboard,
    ToggleLauncher,
    SelectTheme,
    Quit,
}

/// Shared queue that the tray posts actions into, and the capsule heartbeat drains.
#[derive(Clone)]
pub struct TrayService {
    queue: Arc<Mutex<Vec<TrayAction>>>,
}

impl TrayService {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Push an action into the queue (called from the tray thread).
    pub fn push(&self, action: TrayAction) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(action);
        }
    }

    /// Pop the next action (called from the capsule heartbeat).
    pub fn pop(&self) -> Option<TrayAction> {
        self.queue.lock().ok()?.drain(..1).next()
    }

    /// Start the StatusNotifierItem tray in a background tokio task.
    pub fn start(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let tray = CapsuleTray {
                service: service.clone(),
            };
            match tray.spawn().await {
                Ok(_handle) => {
                    // Keep the handle alive forever so the tray stays registered.
                    std::future::pending::<()>().await;
                }
                Err(e) => {
                    eprintln!("[TRAY] Failed to start system tray: {e}");
                }
            }
        });
    }
}

// ── ksni implementation ─────────────────────────────────────────────────────

struct CapsuleTray {
    service: TrayService,
}

impl ksni::Tray for CapsuleTray {
    fn id(&self) -> String {
        "capsule".to_string()
    }

    fn icon_name(&self) -> String {
        // Place a PNG named "capsule" in ~/.local/share/icons/hicolor/64x64/apps/
        // for a themed icon; falls back to a generic system icon otherwise.
        "capsule".to_string()
    }

    fn title(&self) -> String {
        "Capsule".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            icon_name: String::new(),
            icon_pixmap: vec![],
            title: "Capsule".to_string(),
            description: "Dynamic Island para Linux".to_string(),
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;

        vec![
            StandardItem {
                label: "Dashboard".to_string(),
                icon_name: "preferences-system".to_string(),
                activate: Box::new(|tray: &mut CapsuleTray| {
                    tray.service.push(TrayAction::ToggleDashboard);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Lanzador".to_string(),
                icon_name: "system-search".to_string(),
                activate: Box::new(|tray: &mut CapsuleTray| {
                    tray.service.push(TrayAction::ToggleLauncher);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Temas".to_string(),
                icon_name: "preferences-desktop-theme".to_string(),
                activate: Box::new(|tray: &mut CapsuleTray| {
                    tray.service.push(TrayAction::SelectTheme);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Salir".to_string(),
                icon_name: "application-exit".to_string(),
                activate: Box::new(|tray: &mut CapsuleTray| {
                    tray.service.push(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}
