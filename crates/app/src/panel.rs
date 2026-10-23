use std::sync::atomic::{AtomicBool, Ordering};

use gpui::{
    AppContext, LinuxWindowBackground, PlatformDisplay, WindowBounds, WindowHandle, WindowKind,
    WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px,
    session_lock::SessionLockOptions,
};

use crate::lockscreen::LockScreen;
use crate::settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMode {
    Capsule,
    LockScreen,
    Settings,
}

static PANEL_WINDOW_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static WATCHDOG_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct CapsulePanel;

impl CapsulePanel {
    pub fn set_panel_window_id(id: u64) {
        PANEL_WINDOW_ID.store(id, Ordering::SeqCst);
    }
    pub fn window_options(
        module_manager: &crate::capsule::module::CapsuleModuleManager,
        cx: &gpui::App,
    ) -> WindowOptions {
        let size = crate::capsule::Capsule::maximum_window_size(
            module_manager,
            cx.displays().first().map(|display| display.bounds().size),
            cx,
        );

        let display_bounds = cx.displays().first().map(|d| d.bounds());
        let exclusive_zone = if cx.has_global::<services::AppState>() {
            let config = cx.global::<services::AppState>().config.get();
            config.ui.exclusive_zone()
        } else {
            25.0 + 8.0
        };

        let is_kwin = (cx.has_global::<services::AppState>()
            && cx.global::<services::AppState>().compositor.is_kwin())
            || std::env::var("KWIN_RUNNING").is_ok()
            || std::env::var("KDE_FULL_SESSION").is_ok()
            || std::env::var("XDG_CURRENT_DESKTOP")
                .map(|d| d.to_ascii_lowercase().contains("kde"))
                .unwrap_or(false);
        let mut options = WindowOptions {
            titlebar: None,
            window_bounds: display_bounds.map(WindowBounds::Windowed),
            app_id: Some("capsule-panel".to_string()),
            linux_window_background: LinuxWindowBackground::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: if is_kwin {
                    "dock".to_string()
                } else {
                    "capsule-panel".to_string()
                },
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
                margin: Some((px(0.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(px(exclusive_zone)),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        };

        options.inactive_frame_interval = None;
        options.window_bounds = Some(WindowBounds::Windowed(gpui::Bounds {
            origin: gpui::point(px(0.0), px(0.0)),
            size,
        }));

        if let WindowKind::LayerShell(layer) = &mut options.kind {
            layer.anchor = match cx
                .global::<services::AppState>()
                .config
                .get()
                .ui
                .capsule_location
            {
                services::CapsuleLocation::Top => Anchor::TOP,
                services::CapsuleLocation::Bottom => Anchor::BOTTOM,
            };
        }
        options
    }

    pub fn open(cx: &mut gpui::App) -> Option<WindowHandle<crate::capsule::Capsule>> {
        let module_manager = crate::capsule::module::CapsuleModuleManager::new(cx);
        let options = Self::window_options(&module_manager, cx);
        let window = match cx.open_window(options, |window, cx| {
            let entity = cx.new(|cx| crate::capsule::Capsule::with_module_manager(module_manager, window, cx));
            let entity_weak = entity.downgrade();
            window.on_window_should_close(cx, move |_window, cx| {
                if let Some(capsule) = entity_weak.upgrade() {
                    let _ = capsule.update(cx, |capsule, cx| {
                        capsule.handle_module_event(
                            &crate::capsule::module::CapsuleModuleEvent::Close,
                            cx,
                        );
                    });
                }
                false
            });
            entity
        }) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Failed to open new Capsule window: {error}");
                return None;
            }
        };

        let window_id = window.window_id();
        PANEL_WINDOW_ID.store(window_id.as_u64(), Ordering::SeqCst);

        if !WATCHDOG_INITIALIZED.swap(true, Ordering::SeqCst) {
            cx.on_window_closed(move |cx, closed_id| {
                if closed_id.as_u64() == PANEL_WINDOW_ID.load(Ordering::SeqCst) {
                    services::log_warn!(
                        "PANEL",
                        "Capsule panel window closed unexpectedly by compositor; respawning..."
                    );
                    cx.defer(move |cx| {
                        Self::open(cx);
                    });
                }
            })
            .detach();
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
            linux_window_background: LinuxWindowBackground::Opaque,
            kind: WindowKind::SessionLock(SessionLockOptions {}),
            ..Default::default()
        }
    }

    pub fn fallback_window_options(display: &dyn PlatformDisplay) -> WindowOptions {
        WindowOptions {
            display_id: Some(display.id()),
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(display.bounds())),
            app_id: Some("capsule-lockscreen".to_string()),
            linux_window_background: LinuxWindowBackground::Opaque,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-lockscreen".to_string(),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
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
            return Vec::new();
        }

        let is_kwin = cx.has_global::<services::AppState>()
            && cx.global::<services::AppState>().compositor.is_kwin();

        let fallback_blur = cx
            .has_global::<services::AppState>()
            .then(|| {
                cx.global::<services::AppState>()
                    .wallpaper
                    .get_blurred_wallpaper()
            })
            .flatten();

        let displays = cx.displays();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let blur_paths: Vec<Option<std::path::PathBuf>> = if is_kwin {
            // KWin does not implement wlr-screencopy, so grim will always fail.
            // Use cached wallpaper blur directly.
            vec![fallback_blur.clone(); displays.len()]
        } else {
            std::thread::scope(|s| {
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
            })
        };

        let mut handles = Vec::new();

        for (index, display) in displays.iter().enumerate() {
            let is_primary = index == 0;
            let blur_path = blur_paths
                .get(index)
                .cloned()
                .flatten()
                .or_else(|| fallback_blur.clone());

            let open_fn = |options: WindowOptions, cx: &mut gpui::App| {
                let blur = blur_path.clone();
                cx.open_window(options, |window, cx| {
                    window.on_window_should_close(cx, |_, _| false);
                    let view = cx.new(|cx| LockScreen::new(cx, is_primary, blur));
                    if is_primary {
                        let focus = view.read(cx).focus_handle.clone();
                        window.focus(&focus, cx);
                    }
                    view
                })
            };

            let window = if is_kwin {
                open_fn(Self::fallback_window_options(&**display), cx).ok()
            } else {
                match open_fn(Self::window_options(&**display), cx) {
                    Ok(w) => Some(w),
                    Err(err) => {
                        eprintln!(
                            "SessionLock unsupported or failed ({err}); falling back to LayerShell Overlay"
                        );
                        open_fn(Self::fallback_window_options(&**display), cx).ok()
                    }
                }
            };

            if let Some(w) = window {
                handles.push(w);
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

        Self::surface_options(display_id)
    }

    fn surface_options(display_id: Option<gpui::DisplayId>) -> WindowOptions {
        WindowOptions {
            display_id,
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::default())),
            app_id: Some("capsule-settings".to_string()),
            inactive_frame_interval: None,
            linux_window_background: LinuxWindowBackground::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-settings".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(px(-1.0)),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn open(cx: &mut gpui::App) -> Option<WindowHandle<Settings>> {
        if IS_SETTINGS_OPEN
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return None;
        }

        cx.defer(|cx| {
            for window in cx.windows() {
                if let Some(capsule_window) = window.downcast::<crate::capsule::Capsule>() {
                    let _ = capsule_window.update(cx, |capsule, _, cx| {
                        capsule.handle_module_event(
                            &crate::capsule::module::CapsuleModuleEvent::Close,
                            cx,
                        );
                    });
                }
            }
        });

        let options = Self::window_options(cx);

        match cx.open_window(options, |window, cx| cx.new(|cx| Settings::new(window, cx))) {
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
            if let Some(settings_window) = window.downcast::<Settings>() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_surface_size_is_assigned_by_compositor() {
        for display_id in [
            None,
            Some(gpui::DisplayId::new(1)),
            Some(gpui::DisplayId::new(2)),
        ] {
            let options = SettingsPanel::surface_options(display_id);
            assert_eq!(options.display_id, display_id);
            assert_eq!(
                options.window_bounds.unwrap().get_bounds().size,
                gpui::Size::default()
            );
            let WindowKind::LayerShell(layer) = options.kind else {
                panic!("Settings must use layer shell");
            };
            assert_eq!(
                layer.anchor,
                Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT
            );
            assert_eq!(layer.exclusive_zone, Some(px(-1.0)));
        }
    }

    #[test]
    fn settings_panel_open_state_tracks_correctly() {
        SettingsPanel::mark_closed();
        assert!(!SettingsPanel::is_open());
        IS_SETTINGS_OPEN.store(true, Ordering::SeqCst);
        assert!(SettingsPanel::is_open());
        SettingsPanel::mark_closed();
        assert!(!SettingsPanel::is_open());
    }
}
