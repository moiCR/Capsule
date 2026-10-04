use std::sync::atomic::{AtomicBool, Ordering};

use gpui::{
    AppContext, PlatformDisplay, WindowBackgroundAppearance, WindowBounds, WindowHandle,
    WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px,
    session_lock::SessionLockOptions,
};

use services::IpcSubscriber;

use crate::capsule::Capsule;
use crate::lockscreen::LockScreen;
use crate::settings::SettingsWindow;

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
        let exclusive_zone = if cx.has_global::<services::AppState>() {
            let config = cx.global::<services::AppState>().config.get();
            config.ui.exclusive_zone()
        } else {
            25.0 + 8.0
        };

        WindowOptions {
            titlebar: None,
            window_bounds: display_bounds.map(WindowBounds::Windowed),
            app_id: Some("capsule-panel".to_string()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                margin: Some((px(0.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(px(exclusive_zone)),
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
        let window =
            match cx.open_window(options, |window, cx| cx.new(|cx| Capsule::new(window, cx))) {
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
            kind: WindowKind::SessionLock(SessionLockOptions {}),
            ..Default::default()
        }
    }

    pub fn open_all(cx: &mut gpui::App) -> Vec<WindowHandle<LockScreen>> {
        if IS_LOCKSCREEN_OPEN
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Vec::new();
        }

        let displays = cx.displays();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let blur_paths: Vec<Option<std::path::PathBuf>> = std::thread::scope(|s| {
            let mut threads = Vec::new();
            for (index, display) in displays.iter().enumerate() {
                let bounds = display.bounds();
                threads.push(s.spawn(move || {
                    let blur_file = format!("/tmp/capsule_lock_blur_{index}_{timestamp}.jpg");
                    let x: f32 = bounds.origin.x.into();
                    let y: f32 = bounds.origin.y.into();
                    let w: f32 = bounds.size.width.into();
                    let h: f32 = bounds.size.height.into();
                    let geom = format!(
                        "{},{} {}x{}",
                        x as i32,
                        y as i32,
                        w as u32,
                        h as u32
                    );
                    let cmd = format!(
                        "IM_BIN=magick; command -v magick >/dev/null 2>&1 || IM_BIN=convert; grim -g '{geom}' -t ppm - | $IM_BIN - -scale 20% -blur 0x5 -resize 500% {blur_file}"
                    );
                    let ok = std::process::Command::new("sh")
                        .args(["-c", &cmd])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false);
                    if ok && std::path::Path::new(&blur_file).exists() {
                        Some(std::path::PathBuf::from(blur_file))
                    } else {
                        None
                    }
                }));
            }
            threads
                .into_iter()
                .map(|t| t.join().unwrap_or(None))
                .collect()
        });

        let mut handles = Vec::new();

        for (index, display) in displays.iter().enumerate() {
            let is_primary = index == 0;
            let options = Self::window_options(&**display);
            let blur_path = blur_paths.get(index).cloned().flatten();

            match cx.open_window(options, |window, cx| {
                let view = cx.new(|cx| LockScreen::new(cx, is_primary, blur_path));
                if is_primary {
                    let focus = view.read(cx).focus_handle.clone();
                    window.focus(&focus, cx);
                }
                view
            }) {
                Ok(w) => handles.push(w),
                Err(err) => eprintln!("Failed to open lockscreen on display {index}: {err}"),
            }
        }

        if handles.is_empty() {
            IS_LOCKSCREEN_OPEN.store(false, Ordering::SeqCst);
        }

        handles
    }

    pub fn close_all(cx: &mut gpui::App) {
        IS_LOCKSCREEN_OPEN.store(false, Ordering::SeqCst);
        for window in cx.windows() {
            if let Some(lock_window) = window.downcast::<LockScreen>() {
                let _ = lock_window.update(cx, |_, window, _| {
                    window.remove_window();
                });
            }
        }
    }
}

static IS_SETTINGS_OPEN: AtomicBool = AtomicBool::new(false);

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn is_open() -> bool {
        IS_SETTINGS_OPEN.load(Ordering::SeqCst)
    }

    pub fn mark_closed() {
        IS_SETTINGS_OPEN.store(false, Ordering::SeqCst);
    }

    pub fn window_options(cx: &gpui::App) -> WindowOptions {
        let displays = cx.displays();
        let display = displays.first();
        let display_id = display.map(|d| d.id());
        let display_bounds = display.map(|d| d.bounds());

        WindowOptions {
            display_id,
            titlebar: None,
            window_bounds: display_bounds.map(WindowBounds::Windowed),
            app_id: Some("capsule-settings".to_string()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-settings".to_string(),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(px(-1.0)),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn open(cx: &mut gpui::App) -> Option<WindowHandle<SettingsWindow>> {
        if IS_SETTINGS_OPEN
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return None;
        }

        let options = Self::window_options(cx);

        match cx.open_window(options, |window, cx| {
            cx.new(|cx| SettingsWindow::new(window, cx))
        }) {
            Ok(w) => Some(w),
            Err(err) => {
                IS_SETTINGS_OPEN.store(false, Ordering::SeqCst);
                eprintln!("Failed to open settings window: {err}");
                None
            }
        }
    }

    pub fn close(cx: &mut gpui::App) {
        for window in cx.windows() {
            if let Some(settings_window) = window.downcast::<SettingsWindow>() {
                let _ = settings_window.update(cx, |this, _, cx| {
                    this.request_close(cx);
                });
            }
        }
    }

    pub fn toggle(cx: &mut gpui::App) {
        if Self::is_open() {
            Self::close(cx);
        } else {
            Self::open(cx);
        }
    }
}
