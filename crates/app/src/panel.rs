use gpui::{
    AppContext, PlatformDisplay, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px,
};

use services::IpcSubscriber;

use crate::capsule::Capsule;
use crate::lockscreen::LockScreen;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMode {
    Capsule,
    LockScreen,
    Settings,
}

pub struct CapsulePanel;

impl CapsulePanel {
    pub fn window_options(cx: &gpui::App) -> WindowOptions {
        let display_bounds = cx.displays().first().map(|d| d.bounds());
        let idle_h = 25.0 + 8.0;

        WindowOptions {
            titlebar: None,
            window_bounds: display_bounds.map(WindowBounds::Windowed),
            app_id: Some("capsule-panel".to_string()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                margin: Some((px(8.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(px(idle_h)),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn open(
        cx: &mut gpui::App,
        ipc_subscriber: IpcSubscriber,
    ) -> Option<WindowHandle<Capsule>> {
        let options = Self::window_options(cx);
        let window = match cx.open_window(options, |_, cx| cx.new(Capsule::new)) {
            Ok(w) => w,
            Err(err) => {
                eprintln!("Failed to open layer shell window: {err}");
                return None;
            }
        };

        if let Ok(capsule_handle) = window.entity(cx) {
            ipc_subscriber.start(cx, capsule_handle, Capsule::handle_ipc_command);
        }

        Some(window)
    }
}

use std::sync::atomic::{AtomicBool, Ordering};

static IS_LOCKSCREEN_OPEN: AtomicBool = AtomicBool::new(false);

pub struct LockScreenPanel;

impl LockScreenPanel {
    pub fn is_open() -> bool {
        IS_LOCKSCREEN_OPEN.load(Ordering::SeqCst)
    }

    pub fn mark_closed() {
        IS_LOCKSCREEN_OPEN.store(false, Ordering::SeqCst);
    }

    pub fn window_options(display: &dyn PlatformDisplay) -> WindowOptions {
        WindowOptions {
            display_id: Some(display.id()),
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(display.bounds())),
            app_id: Some("capsule-lockscreen".to_string()),
            window_background: WindowBackgroundAppearance::Opaque,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-lockscreen".to_string(),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                margin: None,
                exclusive_zone: Some(px(-1.0)),
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn open_all(cx: &mut gpui::App) -> Vec<WindowHandle<LockScreen>> {
        if IS_LOCKSCREEN_OPEN
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            eprintln!("[LockScreenPanel] Lockscreen is already open. Skipping open_all.");
            return Vec::new();
        }

        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            let _ = std::process::Command::new("hyprctl")
                .args(["eval", "hl.dsp.submap(\"lock\")"])
                .spawn();
        }

        let displays = cx.displays();
        let mut handles = Vec::new();

        for (index, display) in displays.iter().enumerate() {
            let is_primary = index == 0;
            let options = Self::window_options(&**display);

            match cx.open_window(options, |_, cx| {
                cx.new(|cx| LockScreen::new(cx, is_primary))
            }) {
                Ok(w) => handles.push(w),
                Err(err) => eprintln!("Failed to open lockscreen on display {index}: {err}"),
            }
        }

        handles
    }
}
