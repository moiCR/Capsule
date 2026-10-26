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
static WATCHDOG_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static IS_SWITCHING_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static IPC_INITIALIZED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub struct ShellState {
    pub mode: services::ShellMode,
    pub window: Option<gpui::AnyWindowHandle>,
    pub capsule: Option<gpui::Entity<crate::island::Capsule>>,
}

impl gpui::Global for ShellState {}

pub struct CapsulePanel;
pub type IslandPanel = CapsulePanel;

impl CapsulePanel {
    pub fn set_panel_window_id(id: u64) {
        PANEL_WINDOW_ID.store(id, Ordering::SeqCst);
    }

    pub fn bar_window_options(cx: &gpui::App) -> WindowOptions {
        let display_bounds = cx.displays().first().map(|d| d.bounds());
        let width = display_bounds.map(|b| b.size.width).unwrap_or(px(1920.0));
        let screen_height = display_bounds.map(|b| b.size.height).unwrap_or(px(1080.0));
        let bar_height = px(crate::bar::style::BAR_HEIGHT);

        let is_kwin = (cx.has_global::<services::AppState>()
            && cx.global::<services::AppState>().compositor.is_kwin())
            || std::env::var("KWIN_RUNNING").is_ok()
            || std::env::var("KDE_FULL_SESSION").is_ok()
            || std::env::var("XDG_CURRENT_DESKTOP")
                .map(|d| d.to_ascii_lowercase().contains("kde"))
                .unwrap_or(false);

        let location = if cx.has_global::<services::AppState>() {
            match cx
                .global::<services::AppState>()
                .config
                .get()
                .ui
                .capsule_location
            {
                services::CapsuleLocation::Top => Anchor::TOP,
                services::CapsuleLocation::Bottom => Anchor::BOTTOM,
            }
        } else {
            Anchor::TOP
        };

        let mut options = WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds {
                origin: gpui::point(px(0.0), px(0.0)),
                size: gpui::size(width, screen_height),
            })),
            app_id: Some("capsule-bar".to_string()),
            linux_window_background: LinuxWindowBackground::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: if is_kwin {
                    "dock".to_string()
                } else {
                    "capsule-panel".to_string()
                },
                layer: Layer::Top,
                anchor: location | Anchor::LEFT | Anchor::RIGHT,
                margin: Some((px(0.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(bar_height),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        };

        options.inactive_frame_interval = None;
        options
    }

    pub fn window_options(
        module_manager: &crate::island::module::CapsuleModuleManager,
        cx: &gpui::App,
    ) -> WindowOptions {
        let size = crate::island::Capsule::maximum_window_size(
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

    pub fn open_island(
        cx: &mut gpui::App,
    ) -> Option<(
        WindowHandle<crate::island::Capsule>,
        gpui::Entity<crate::island::Capsule>,
    )> {
        let module_manager = crate::island::module::CapsuleModuleManager::new(cx);
        let options = Self::window_options(&module_manager, cx);
        let mut entity_handle = None;
        let window = match cx.open_window(options, |window, cx| {
            let entity = cx
                .new(|cx| crate::island::Capsule::with_module_manager(module_manager, window, cx));
            entity_handle = Some(entity.clone());
            let entity_weak = entity.downgrade();
            window.on_window_should_close(cx, move |_window, cx| {
                if let Some(capsule) = entity_weak.upgrade() {
                    let _ = capsule.update(cx, |capsule, cx| {
                        capsule.handle_module_event(
                            &crate::island::module::CapsuleModuleEvent::Close,
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
        let entity = entity_handle?;
        Some((window, entity))
    }

    pub fn open_bar(cx: &mut gpui::App) -> Option<WindowHandle<crate::bar::Bar>> {
        let options = Self::bar_window_options(cx);
        let location = if cx.has_global::<services::AppState>() {
            match cx
                .global::<services::AppState>()
                .config
                .get()
                .ui
                .capsule_location
            {
                services::CapsuleLocation::Top => crate::bar::BarLocation::Top,
                services::CapsuleLocation::Bottom => crate::bar::BarLocation::Bottom,
            }
        } else {
            crate::bar::BarLocation::Top
        };

        let window = match cx.open_window(options, |window, cx| {
            window.on_window_should_close(cx, |_, _| false);
            let handle = window.window_handle();
            cx.new(|cx| crate::bar::Bar::new(location, handle, cx))
        }) {
            Ok(window) => window,
            Err(error) => {
                eprintln!("Failed to open new Bar window: {error}");
                return None;
            }
        };

        let window_id = window.window_id();
        PANEL_WINDOW_ID.store(window_id.as_u64(), Ordering::SeqCst);
        Some(window)
    }

    pub fn sync_shell_mode(cx: &mut gpui::App) {
        if !cx.has_global::<services::AppState>() || !cx.has_global::<ShellState>() {
            return;
        }
        let desired_mode = cx.global::<services::AppState>().config.get().ui.shell_mode;
        let current_mode = cx.global::<ShellState>().mode;
        if desired_mode == current_mode {
            return;
        }

        for window in cx.windows() {
            if let Some(bar_window) = window.downcast::<crate::bar::Bar>() {
                let _ = bar_window.update(cx, |bar, window, cx| {
                    bar.close_launcher(window, cx);
                    bar.close_polkit(window, cx);
                });
            }
        }

        IS_SWITCHING_MODE.store(true, Ordering::SeqCst);
        let previous_window = cx.global::<ShellState>().window;

        let (new_handle, new_capsule) = match desired_mode {
            services::ShellMode::Island => {
                if let Some((w, e)) = Self::open_island(cx) {
                    (Some(w.into()), Some(e))
                } else {
                    (None, None)
                }
            }
            services::ShellMode::Bar => (Self::open_bar(cx).map(|h| h.into()), None),
        };

        if let Some(prev) = previous_window {
            let _ = prev.update(cx, |_, window, _| window.remove_window());
        }

        cx.global_mut::<ShellState>().mode = desired_mode;
        cx.global_mut::<ShellState>().window = new_handle;
        cx.global_mut::<ShellState>().capsule = new_capsule;
        IS_SWITCHING_MODE.store(false, Ordering::SeqCst);
    }

    pub fn handle_ipc_command(command: services::IpcCommand, cx: &mut gpui::App) {
        match command {
            services::IpcCommand::Quit => {
                cx.quit();
            }
            services::IpcCommand::Lock => {
                crate::panel::LockScreenPanel::open_all(cx);
            }
            services::IpcCommand::Terminal
            | services::IpcCommand::Browser
            | services::IpcCommand::Editor
            | services::IpcCommand::FileManager => {
                if cx.has_global::<services::AppState>() {
                    crate::island::ipc_handler::launch_apps::launch(
                        command,
                        cx.global::<services::AppState>(),
                    );
                }
            }
            services::IpcCommand::ToggleSettings => {
                crate::panel::SettingsPanel::toggle(cx);
            }
            services::IpcCommand::ShowSettings => {
                crate::panel::SettingsPanel::open(cx);
            }
            services::IpcCommand::SetWallpaper(path) => {
                if cx.has_global::<services::AppState>() {
                    let state = cx.global::<services::AppState>().clone();
                    let _ = state.config.update(|cfg| {
                        cfg.ui.wallpaper_path = Some(path.clone());
                    });
                    state.wallpaper.set_wallpaper(path);
                }
            }
            services::IpcCommand::Ping => {}
            services::IpcCommand::Hide => {
                if crate::panel::SettingsPanel::is_open() {
                    crate::panel::SettingsPanel::close(cx);
                }
                for window in cx.windows() {
                    if let Some(bar_window) = window.downcast::<crate::bar::Bar>() {
                        let _ = bar_window.update(cx, |bar, window, cx| {
                            bar.handle_ipc_command(services::IpcCommand::Hide, window, cx);
                        });
                    }
                }
                if let Some(capsule) = cx
                    .has_global::<ShellState>()
                    .then(|| cx.global::<ShellState>().capsule.clone())
                    .flatten()
                {
                    let _ = capsule.update(cx, |capsule, cx| {
                        capsule.handle_module_event(
                            &crate::island::module::CapsuleModuleEvent::Close,
                            cx,
                        );
                        capsule.sync_window(cx);
                    });
                }
            }
            command @ (services::IpcCommand::ToggleLauncher
            | services::IpcCommand::ShowLauncher
            | services::IpcCommand::ToggleClipboard
            | services::IpcCommand::ShowClipboard
            | services::IpcCommand::ToggleEmoji
            | services::IpcCommand::ShowEmoji
            | services::IpcCommand::ToggleNotification
            | services::IpcCommand::ShowNotification
            | services::IpcCommand::ToggleDashboard
            | services::IpcCommand::ShowDashboard
            | services::IpcCommand::ToggleRecord
            | services::IpcCommand::ShowRecord
            | services::IpcCommand::ToggleShelf
            | services::IpcCommand::ShowShelf) => {
                let mode = if cx.has_global::<services::AppState>() {
                    cx.global::<services::AppState>().config.get().ui.shell_mode
                } else {
                    services::ShellMode::Island
                };
                match mode {
                    services::ShellMode::Island => {
                        if crate::panel::SettingsPanel::is_open() {
                            crate::panel::SettingsPanel::close(cx);
                        }
                        if let Some(capsule) = cx
                            .has_global::<ShellState>()
                            .then(|| cx.global::<ShellState>().capsule.clone())
                            .flatten()
                        {
                            let _ = capsule.update(cx, |capsule, cx| {
                                capsule.handle_ipc_command(command, cx);
                                capsule.sync_window(cx);
                            });
                        }
                    }
                    services::ShellMode::Bar => {
                        for window in cx.windows() {
                            if let Some(bar_window) = window.downcast::<crate::bar::Bar>() {
                                let cmd = command.clone();
                                let _ = bar_window.update(cx, move |bar, window, cx| {
                                    bar.handle_ipc_command(cmd, window, cx);
                                });
                            }
                        }
                    }
                }
            }
            island_command => {
                if crate::panel::SettingsPanel::is_open() {
                    crate::panel::SettingsPanel::close(cx);
                }
                if let Some(capsule) = cx
                    .has_global::<ShellState>()
                    .then(|| cx.global::<ShellState>().capsule.clone())
                    .flatten()
                {
                    let _ = capsule.update(cx, |capsule, cx| {
                        capsule.handle_ipc_command(island_command, cx);
                        capsule.sync_window(cx);
                    });
                }
            }
        }
    }

    pub fn open(cx: &mut gpui::App) -> Option<gpui::AnyWindowHandle> {
        let mode = if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>().config.get().ui.shell_mode
        } else {
            services::ShellMode::Island
        };

        let (handle, capsule) = match mode {
            services::ShellMode::Island => {
                if let Some((w, e)) = Self::open_island(cx) {
                    (Some(w.into()), Some(e))
                } else {
                    (None, None)
                }
            }
            services::ShellMode::Bar => (Self::open_bar(cx).map(|h| h.into()), None),
        };

        cx.set_global(ShellState {
            mode,
            window: handle,
            capsule,
        });

        if !IPC_INITIALIZED.swap(true, Ordering::SeqCst) {
            cx.spawn(async move |cx| {
                loop {
                    services::wait_for_ipc_command().await;
                    let _ = cx.update(|cx| {
                        while let Some(command) = services::pop_ipc_command() {
                            Self::handle_ipc_command(command, cx);
                        }
                    });
                }
            })
            .detach();
        }

        if !WATCHDOG_INITIALIZED.swap(true, Ordering::SeqCst) {
            cx.on_window_closed(move |cx, closed_id| {
                if IS_SWITCHING_MODE.load(Ordering::SeqCst) {
                    return;
                }
                if closed_id.as_u64() == PANEL_WINDOW_ID.load(Ordering::SeqCst) {
                    services::log_warn!(
                        "PANEL",
                        "Panel window closed unexpectedly by compositor; respawning..."
                    );
                    cx.defer(move |cx| {
                        Self::open(cx);
                    });
                }
            })
            .detach();

            if cx.has_global::<services::AppState>() {
                let mut config_changes = cx.global::<services::AppState>().config.subscribe();
                cx.spawn(async move |cx| {
                    loop {
                        match config_changes.recv().await {
                            Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                        let _ = cx.update(|cx| {
                            Self::sync_shell_mode(cx);
                        });
                    }
                })
                .detach();
            }
        }

        handle
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
                if let Some(capsule_window) = window.downcast::<crate::island::Capsule>() {
                    let _ = capsule_window.update(cx, |capsule, _, cx| {
                        capsule.handle_module_event(
                            &crate::island::module::CapsuleModuleEvent::Close,
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
