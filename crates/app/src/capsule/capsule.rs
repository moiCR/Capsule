use gpui::{
    Bounds, Context, DispatchPhase, Entity, MouseExitEvent, MouseMoveEvent, Render, Size, Task,
    Window, div, layer_shell::KeyboardInteractivity, point, prelude::*, px, svg,
};
use services::{AppState, NotificationStore};
use std::time::{Duration, Instant};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::capsule::modules::CapsuleModules;

use super::orbit::{ORB_SIZE, OrbKind, Orbit};
use super::satellites::PanelManager;
use super::widgets::{record::orb::render_record_orb, shelf::orb::render_shelf_orb};
use super::{CapsuleMode, apple_island_morph, apple_island_spring};

use super::modules::clipboard::ClipboardEvent;
use super::modules::default::DefaultEvent;
use super::modules::emoji::EmojiEvent;
use super::modules::record::RecordEvent;
use super::modules::shelf::ShelfEvent;
use super::modules::wallpaper::WallpaperEvent;

pub struct Capsule {
    mode: CapsuleMode,
    modules: CapsuleModules,
    panel_manager: PanelManager,
    current_width: f32,
    current_height: f32,
    current_radius: f32,
    current_y: f32,
    target_width: f32,
    target_height: f32,
    target_radius: f32,
    target_y: f32,
    anim_progress: f32,
    anim_start_time: Option<Instant>,
    anim_start_w: f32,
    anim_start_h: f32,
    anim_start_r: f32,
    anim_start_y: f32,
    window_height: f32,
    anim_start_progress: f32,
    animating: bool,
    anim_task: Option<Task<()>>,
    satellite_anim_task: Option<Task<()>>,
    orbit: Entity<Orbit>,
    drag_target_active: bool,
    last_drag_over: Option<Instant>,
    drag_monitor_task: Option<Task<()>>,
    last_activity_time: Instant,
    inactivity_generation: u64,
    last_vol_status: Option<(u32, bool)>,
    volume_timer_gen: u64,
    dimension_tracker: DimensionTracker,
    last_rendered_mode: Option<CapsuleMode>,
    is_mode_transition: bool,
    is_hovered: bool,
    last_known_workspace: services::WorkspaceInfo,
}

impl Capsule {
    pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let modules = CapsuleModules::new(cx);
        let orbit = cx.new(|cx| Orbit::new(window, cx));
        cx.observe(&orbit, |_, _, cx| cx.notify()).detach();
        let (initial_w, initial_h) = modules.default_view.read(cx).desired_dimensions();
        let r = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.capsule_round
        } else {
            CapsuleMode::Default.radius()
        };

        cx.observe(&modules.default_view, |capsule, default_view, cx| {
            if capsule.mode == CapsuleMode::Default && !capsule.drag_target_active {
                let (desired_w, desired_h) = default_view.read(cx).desired_dimensions();
                capsule.update_target_dimensions(desired_w, desired_h, cx);
            }
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.default_view,
            |capsule, _, event: &DefaultEvent, cx| match event {
                DefaultEvent::WorkspaceClicked(ws_id) => {
                    let ws_id = *ws_id;
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().compositor.switch_workspace(ws_id);
                    }
                }
                DefaultEvent::StatusDockClicked => {
                    capsule.start_transition_internal(CapsuleMode::Dashboard, None, cx);
                }
            },
        )
        .detach();

        if cx.has_global::<AppState>() {
            let compositor = cx.global::<AppState>().compositor.clone();
            let (ws_tx, mut ws_rx) =
                tokio::sync::mpsc::unbounded_channel::<services::WorkspaceInfo>();

            tokio::spawn(async move {
                let mut rx = compositor.on_change_workspace();
                let mut interval = tokio::time::interval(Duration::from_millis(250));
                let mut last_ws = compositor.get_workspace();
                loop {
                    tokio::select! {
                        result = rx.recv() => {
                            match result {
                                Ok(ws) => {
                                    last_ws = ws.clone();
                                    if ws_tx.send(ws).is_err() {
                                        break;
                                    }
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                    let ws = compositor.get_workspace();
                                    last_ws = ws.clone();
                                    if ws_tx.send(ws).is_err() {
                                        break;
                                    }
                                }
                                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                            }
                        }
                        _ = interval.tick() => {
                            let current_ws = compositor.get_workspace();
                            if current_ws != last_ws {
                                last_ws = current_ws.clone();
                                if ws_tx.send(current_ws).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            });

            let entity = cx.entity().downgrade();
            let mut window_context = window.to_async(cx);
            cx.foreground_executor()
                .spawn(async move {
                    while let Some(ws) = ws_rx.recv().await {
                        if entity.upgrade().is_none() {
                            break;
                        }
                        let _ = window_context.update(|window, cx| {
                            let _ = entity.update(cx, |capsule, cx| {
                                capsule.last_known_workspace = ws.clone();
                                capsule.modules.default_view.update(cx, |default_mod, cx| {
                                    default_mod.set_active_workspace(ws, cx);
                                });
                                capsule.on_workspace_changed(cx);
                            });
                            window.refresh();
                        });
                    }
                })
                .detach();

            let config_service = cx.global::<AppState>().config.clone();
            let (cfg_tx, mut cfg_rx) =
                tokio::sync::mpsc::unbounded_channel::<std::sync::Arc<services::AppConfig>>();

            let cfg_service_clone = config_service.clone();
            tokio::spawn(async move {
                let mut rx = cfg_service_clone.subscribe();
                loop {
                    match rx.recv().await {
                        Ok(cfg) => {
                            if cfg_tx.send(cfg).is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                            let cfg = cfg_service_clone.get();
                            if cfg_tx.send(cfg).is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            });

            let entity = cx.entity().downgrade();
            let mut window_context = window.to_async(cx);
            cx.foreground_executor()
                .spawn(async move {
                    while let Some(cfg) = cfg_rx.recv().await {
                        if entity.upgrade().is_none() {
                            break;
                        }
                        let _ = window_context.update(|window, cx| {
                            let _ = entity.update(cx, |capsule, cx| {
                                capsule.on_config_changed(&cfg, window, cx);
                            });
                            window.refresh();
                        });
                    }
                })
                .detach();
        }

        cx.observe(&modules.dashboard_view, |_, _, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.dashboard_view,
            |capsule, _, event: &super::modules::dashboard::DashboardEvent, cx| match event {
                super::modules::dashboard::DashboardEvent::CloseRequested => {
                    if capsule.mode == CapsuleMode::Dashboard {
                        capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                    }
                }
                super::modules::dashboard::DashboardEvent::SelectThemeRequested => {
                    capsule.start_transition_internal(CapsuleMode::SelectTheme, None, cx);
                }
                super::modules::dashboard::DashboardEvent::TrayIconClicked(idx) => {
                    let idx = *idx;
                    let max_h = CapsuleMode::Dashboard.dimensions().1;
                    let panel_h = if cx.has_global::<AppState>() {
                        if let Some(item) = cx.global::<AppState>().sni_host.get_items().get(idx) {
                            super::widgets::dashboard::tray::compute_panel_height(item)
                        } else {
                            super::satellites::DEFAULT_PANEL_H
                        }
                    } else {
                        super::satellites::DEFAULT_PANEL_H
                    };
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Tray(idx),
                        panel_h,
                        max_h,
                    );
                    capsule.sync_panel_indices(cx);
                    capsule.start_satellite_animation(cx);
                    cx.notify();
                }
                super::modules::dashboard::DashboardEvent::WifiChevronClicked => {
                    let max_h = CapsuleMode::Dashboard.dimensions().1;
                    let panel_h = if cx.has_global::<AppState>() {
                        let status = cx.global::<AppState>().network.get_status();
                        super::satellites::wifi::compute_wifi_panel_height(&status)
                    } else {
                        180.0
                    };
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Wifi,
                        panel_h,
                        max_h,
                    );
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().network.rescan_wifi();
                    }
                    capsule.start_satellite_animation(cx);
                    cx.notify();
                }
                super::modules::dashboard::DashboardEvent::BluetoothChevronClicked => {
                    let max_h = CapsuleMode::Dashboard.dimensions().1;
                    let panel_h = if cx.has_global::<AppState>() {
                        let status = cx.global::<AppState>().network.get_status();
                        super::satellites::bluetooth::compute_bluetooth_panel_height(&status)
                    } else {
                        180.0
                    };
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Bluetooth,
                        panel_h,
                        max_h,
                    );
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().network.start_bluetooth_scan();
                    }
                    capsule.start_satellite_animation(cx);
                    cx.notify();
                }

                super::modules::dashboard::DashboardEvent::CalendarClicked => {
                    let max_h = CapsuleMode::Dashboard.dimensions().1;
                    let panel_h = super::satellites::calendar::compute_calendar_panel_height();
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Calendar,
                        panel_h,
                        max_h,
                    );
                    capsule.start_satellite_animation(cx);
                    cx.notify();
                }

                super::modules::dashboard::DashboardEvent::VolumeChevronClicked => {
                    let max_h = CapsuleMode::Dashboard.dimensions().1;
                    let sink_count = if cx.has_global::<AppState>() {
                        cx.global::<AppState>()
                            .system
                            .get_status()
                            .audio_sinks
                            .len()
                    } else {
                        1
                    };
                    let panel_h =
                        super::satellites::volume::compute_volume_panel_height(sink_count);
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Volume,
                        panel_h,
                        max_h,
                    );
                    capsule.start_satellite_animation(cx);
                    cx.notify();
                }
                super::modules::dashboard::DashboardEvent::WallpaperRequested => {
                    capsule.modules.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.reload_items(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Wallpaper, None, cx);
                }

                super::modules::dashboard::DashboardEvent::SettingsRequested => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                    crate::panel::SettingsPanel::open(cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.notification_view, |capsule, view, cx| {
            if capsule.mode == CapsuleMode::Notification {
                let (width, height) = view.read(cx).desired_dimensions();
                capsule.update_target_dimensions(width, height, cx);
            }
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.observe(&modules.launcher_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.launcher_view,
            |capsule, _, event: &super::modules::launcher::LauncherEvent, cx| match event {
                super::modules::launcher::LauncherEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.polkit_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.polkit_view,
            |capsule, _, event: &super::modules::polkit::PolkitEvent, cx| match event {
                super::modules::polkit::PolkitEvent::Authenticated
                | super::modules::polkit::PolkitEvent::Cancelled => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.select_theme_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.select_theme_view,
            |capsule, _, event: &super::modules::select_theme::SelectThemeEvent, cx| match event {
                super::modules::select_theme::SelectThemeEvent::ThemeSelected => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let compositor = cx.global::<AppState>().compositor.clone();
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(compositor.get_frame_duration())
                    .await;
                if this
                    .update(cx, |capsule: &mut Self, cx| {
                        let current_ws = compositor.get_workspace();
                        if capsule.last_known_workspace != current_ws {
                            capsule.last_known_workspace = current_ws.clone();
                            capsule.modules.default_view.update(cx, |default_mod, cx| {
                                default_mod.set_active_workspace(current_ws, cx);
                            });
                            capsule.on_workspace_changed(cx);
                        }

                        while let Some(cmd) = services::pop_ipc_command() {
                            services::log_info!("IPC", "Processing queued IPC command: {:?}", cmd);
                            capsule.handle_ipc_command(cmd, cx);
                        }

                        if cx.has_global::<services::AppState>() {
                            let polkit = cx.global::<services::AppState>().polkit.clone();
                            while let Some((req, responder)) = polkit.pop_request() {
                                services::log_info!(
                                    "POLKIT",
                                    "Processing Polkit auth request: '{}'",
                                    req.action_id
                                );
                                capsule.modules.polkit_view.update(cx, |p, cx| {
                                    p.set_request(req, responder, cx);
                                });
                                capsule.start_transition_internal(CapsuleMode::Polkit, None, cx);
                            }

                            while let Some(cancelled_cookie) = polkit.pop_cancelled() {
                                capsule.modules.polkit_view.update(cx, |p, cx| {
                                    if let Some(ref req) = p.request
                                        && req.cookie == cancelled_cookie
                                    {
                                        services::log_info!(
                                            "POLKIT",
                                            "Polkit authority cancelled active request for cookie='{}'",
                                            cancelled_cookie
                                        );
                                        p.cancel(cx);
                                    }
                                });
                            }
                        }

                        capsule.modules.polkit_view.update(cx, |p, cx| {
                            p.poll_result(cx);
                        });

                        if cx.has_global::<ui::theme::theme_manager::ThemeManager>() {
                            let theme_updated = cx
                                .global_mut::<ui::theme::theme_manager::ThemeManager>()
                                .check_and_reload();
                            if theme_updated {
                                let new_theme = cx
                                    .global::<ui::theme::theme_manager::ThemeManager>()
                                    .current_theme
                                    .clone();
                                cx.set_global(new_theme);
                                services::log_info!(
                                    "THEME",
                                    "Reloaded current_theme and applied to GTK/Qt/Ghostty/Fish/Yazi apps!"
                                );
                                cx.notify();
                            }
                        }

                        if cx.has_global::<services::AppState>() {
                            let lang_updated = cx
                                .global::<services::AppState>()
                                .language
                                .check_and_reload();
                            if lang_updated {
                                services::log_info!(
                                    "LANG",
                                    "Reloaded language bundle!"
                                );
                                capsule.modules.notify_all(cx);
                                cx.notify();
                            }
                        }

                        let (m_w, m_h) = match capsule.mode {
                            CapsuleMode::Default
                            | CapsuleMode::Volume
                            | CapsuleMode::Record
                            | CapsuleMode::Shelf
                            | CapsuleMode::Notification => (0.0, 0.0),
                            _ => capsule.dimension_tracker.dimensions(0.0, 0.0),
                        };

                        let mut dimension_changed = false;

                        if m_h > 0.0 && (m_h - capsule.target_height).abs() > 0.5 {
                            capsule.target_height = m_h;
                            dimension_changed = true;
                        }

                        if m_w > 0.0 && (m_w - capsule.target_width).abs() > 0.5 {
                            capsule.target_width = m_w;
                            dimension_changed = true;
                        }

                        if dimension_changed {
                            if !capsule.animating {
                                capsule.animate_dimension_change(cx);
                            }
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        services::start_polkit_agent();

        cx.observe(&modules.volume_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.wallpaper_view,
            |capsule, _, event: &WallpaperEvent, cx| match event {
                WallpaperEvent::CloseRequested => {
                    capsule.modules.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.clear_cache(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
                WallpaperEvent::WallpaperSelected(path) => {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().wallpaper.set_wallpaper(path);
                    }
                    capsule.modules.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.clear_cache(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let notify = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().system.audio_changed())
        } else {
            None
        };

        cx.spawn(async move |this, cx| {
            loop {
                if let Some(ref n) = notify {
                    tokio::select! {
                        _ = n.notified() => {},
                        _ = cx.background_executor().timer(Duration::from_millis(50)) => {},
                    }
                } else {
                    cx.background_executor()
                        .timer(Duration::from_millis(50))
                        .await;
                }

                let state = this
                    .update(cx, |_, cx| {
                        if !cx.has_global::<AppState>() {
                            return (50, false);
                        }
                        let sys = cx.global::<AppState>().system.get_status();
                        (sys.volume, sys.is_muted)
                    })
                    .ok();

                if let Some((vol, muted)) = state {
                    let changed = this
                        .update(cx, |capsule: &mut Self, cx| {
                            if capsule.last_vol_status.is_none() {
                                capsule.last_vol_status = Some((vol, muted));
                                return false;
                            }
                            if capsule.last_vol_status != Some((vol, muted)) {
                                capsule.last_vol_status = Some((vol, muted));

                                capsule.modules.volume_view.update(cx, |vol_mod, cx| {
                                    vol_mod.update_status(vol, muted, cx);
                                });

                                if capsule.mode == CapsuleMode::Default {
                                    capsule.start_transition_internal(
                                        CapsuleMode::Volume,
                                        None,
                                        cx,
                                    );
                                }

                                if capsule.mode == CapsuleMode::Volume
                                    || capsule.mode == CapsuleMode::Default
                                {
                                    capsule.volume_timer_gen += 1;
                                    let current_gen = capsule.volume_timer_gen;

                                    cx.spawn(async move |weak, cx| {
                                        cx.background_executor()
                                            .timer(Duration::from_secs(2))
                                            .await;
                                        let _ = weak.update(cx, |capsule: &mut Self, cx| {
                                            if capsule.volume_timer_gen == current_gen
                                                && capsule.mode == CapsuleMode::Volume
                                            {
                                                capsule.start_transition_internal(
                                                    CapsuleMode::Default,
                                                    None,
                                                    cx,
                                                );
                                            }
                                        });
                                    })
                                    .detach();
                                }
                                return true;
                            }
                            false
                        })
                        .unwrap_or(false);

                    if changed {
                        let _ = this.update(cx, |_, cx| cx.notify());
                    }
                }
            }
        })
        .detach();

        let mut last_seen_notif_id: Option<u32> = None;
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;

                let latest = NotificationStore::global().get_latest_active_notification();
                let latest_id = latest.as_ref().map(|n| n.id);

                let res = this.update(cx, |capsule: &mut Self, cx| {
                    capsule
                        .modules
                        .notification_view
                        .update(cx, |notif, cx| notif.poll_reply(cx));
                    if capsule.mode == CapsuleMode::Notification
                        && capsule.modules.notification_view.read(cx).is_replying()
                    {
                        return;
                    }
                    if latest_id != last_seen_notif_id {
                        last_seen_notif_id = latest_id;
                        if let Some(item) = latest {
                            capsule.modules.notification_view.update(cx, |notif, cx| {
                                notif.set_item(Some(item), cx);
                            });
                            if capsule.mode == CapsuleMode::Default
                                || capsule.mode == CapsuleMode::Notification
                            {
                                capsule.start_transition_internal(
                                    CapsuleMode::Notification,
                                    None,
                                    cx,
                                );
                            }
                        } else if capsule.mode == CapsuleMode::Notification {
                            capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                        }
                    } else if latest_id.is_none() && capsule.mode == CapsuleMode::Notification {
                        capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                    }
                });

                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        cx.observe(&modules.clipboard_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.clipboard_view,
            |capsule, _, event: &ClipboardEvent, cx| match event {
                ClipboardEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.emoji_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.emoji_view,
            |capsule, _, event: &EmojiEvent, cx| match event {
                EmojiEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.shelf_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &modules.shelf_view,
            |capsule, _, event: &ShelfEvent, cx| match event {
                ShelfEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
                ShelfEvent::ItemCopied(item) => {
                    services::log_info!("SHELF", "Item copied: {}", item.name);
                }
            },
        )
        .detach();

        cx.subscribe(
            &modules.record_view,
            |capsule, _, event: &RecordEvent, cx| match event {
                RecordEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.observe(&modules.record_view, |capsule, record_view, cx| {
            if capsule.mode == CapsuleMode::Record {
                let desired_w = record_view.read(cx).desired_width(cx);
                capsule.update_target_dimensions(desired_w, 42.0, cx);
            }
            cx.notify();
        })
        .detach();

        let initial_margin_top = if cx.has_global::<AppState>() {
            let ui_cfg = &cx.global::<AppState>().config.get().ui;
            if ui_cfg.capsule_style == services::CapsuleStyle::Concave {
                0.0
            } else {
                ui_cfg.margin_top
            }
        } else {
            8.0
        };
        let initial_y = initial_margin_top;

        let initial_win_h = cx
            .displays()
            .first()
            .map(|d| f32::from(d.bounds().size.height))
            .unwrap_or(1080.0);

        Self {
            mode: CapsuleMode::Default,
            modules,
            panel_manager: PanelManager::new(),
            current_width: initial_w,
            current_height: initial_h,
            current_radius: r,
            current_y: initial_y,
            target_width: initial_w,
            target_height: initial_h,
            target_radius: r,
            target_y: initial_y,
            anim_progress: 0.0,
            anim_start_time: None,
            anim_start_w: initial_w,
            anim_start_h: initial_h,
            anim_start_r: r,
            anim_start_y: initial_y,
            window_height: initial_win_h,
            anim_start_progress: 0.0,
            animating: false,
            anim_task: None,
            satellite_anim_task: None,
            orbit,
            drag_target_active: false,
            last_drag_over: None,
            drag_monitor_task: None,
            last_activity_time: Instant::now(),
            inactivity_generation: 0,
            last_vol_status: None,
            volume_timer_gen: 0,
            dimension_tracker: DimensionTracker::new(),
            last_rendered_mode: None,
            is_mode_transition: false,
            is_hovered: false,
            last_known_workspace: if cx.has_global::<AppState>() {
                cx.global::<AppState>().compositor.get_workspace()
            } else {
                services::WorkspaceInfo::default()
            },
        }
    }

    /// Sync the open panel indices from PanelManager to the DashboardModule
    /// so the tray widget knows which icons are highlighted.
    fn sync_panel_indices(&mut self, cx: &mut Context<Self>) {
        let open_indices: Vec<usize> = self
            .panel_manager
            .left
            .iter()
            .chain(self.panel_manager.right.iter())
            .filter_map(|p| match p.kind {
                super::satellites::PanelKind::Tray(idx) => Some(idx),
                _ => None,
            })
            .collect();
        let is_media_open = self
            .panel_manager
            .left
            .iter()
            .chain(self.panel_manager.right.iter())
            .any(|p| matches!(p.kind, super::satellites::PanelKind::Media));
        self.modules.default_view.update(cx, |module, _cx| {
            module.set_open_panel_indices(open_indices.clone());
        });
        self.modules.dashboard_view.update(cx, |module, _cx| {
            module.open_panel_indices = open_indices;
            module.is_media_panel_open = is_media_open;
        });
    }

    pub fn update_target_dimensions(
        &mut self,
        target_w: f32,
        target_h: f32,
        cx: &mut Context<Self>,
    ) {
        if self.mode == CapsuleMode::Default {
            if self.drag_target_active {
                return;
            }
            let diff_w = (target_w - self.target_width).abs();
            if diff_w > 1.0 {
                self.target_width = target_w;
                self.target_height = target_h;
                if !self.animating {
                    self.anim_start_w = self.current_width;
                    self.anim_start_h = self.current_height;
                    self.anim_start_r = self.current_radius;
                    self.anim_start_y = self.current_y;
                    self.anim_start_progress = self.anim_progress;
                    self.anim_start_time = Some(Instant::now());
                    self.animate_dimension_change(cx);
                }
                cx.notify();
            }
        } else if matches!(self.mode, CapsuleMode::Record | CapsuleMode::Notification) {
            let diff_w = (target_w - self.target_width).abs();
            let diff_h = (target_h - self.target_height).abs();
            if diff_w > 0.5 || diff_h > 0.5 {
                self.target_width = target_w;
                self.target_height = target_h;
                self.anim_start_w = self.current_width;
                self.anim_start_h = self.current_height;
                self.anim_start_r = self.current_radius;
                self.anim_start_y = self.current_y;
                self.anim_start_progress = self.anim_progress;
                self.anim_start_time = Some(Instant::now());
                if !self.animating {
                    self.animate_dimension_change(cx);
                }
            } else if !self.animating {
                self.target_width = target_w;
                self.target_height = target_h;
                self.current_width = target_w;
                self.current_height = target_h;
            }
            cx.notify();
        }
    }

    pub fn animate_dimension_change(&mut self, cx: &mut Context<Self>) {
        if self.animating {
            return;
        }
        self.anim_start_w = self.current_width;
        self.anim_start_h = self.current_height;
        self.anim_start_r = self.current_radius;
        self.anim_start_y = self.current_y;
        self.anim_start_progress = self.anim_progress;
        self.animating = true;
        self.is_mode_transition = false;
        self.anim_start_time = Some(Instant::now());

        let compositor = cx.global::<AppState>().compositor.clone();
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(compositor.get_frame_duration())
                    .await;
                let done = this
                    .update(cx, |capsule, cx| {
                        let duration = if cx.has_global::<AppState>() {
                            cx.global::<AppState>()
                                .config
                                .get()
                                .ui
                                .animation_duration_ms as f32
                                / 1000.0
                        } else {
                            0.28
                        };
                        let finished = capsule.tick_animation(duration);
                        cx.notify();
                        finished
                    })
                    .unwrap_or(true);

                if done {
                    this.update(cx, |capsule, cx| {
                        capsule.animating = false;
                        capsule.is_mode_transition = false;
                        capsule.current_width = capsule.target_width;
                        capsule.current_height = capsule.target_height;
                        capsule.current_radius = capsule.target_radius;
                        capsule.current_y = capsule.target_y;
                        capsule.anim_task = None;
                        cx.notify();
                    })
                    .ok();
                    break;
                }
            }
        });
        self.anim_task = Some(task);
    }

    fn reset_inactivity_timer(&mut self) {
        self.last_activity_time = Instant::now();
    }

    fn start_satellite_animation(&mut self, cx: &mut Context<Self>) {
        if self.satellite_anim_task.is_some() {
            return;
        }

        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };

        let task = cx.spawn(async move |this, cx| {
            loop {
                let duration = compositor
                    .as_ref()
                    .map(|c| c.get_frame_duration())
                    .unwrap_or(Duration::from_millis(16));
                cx.background_executor().timer(duration).await;

                let done = this
                    .update(cx, |capsule, cx| {
                        capsule.panel_manager.update_animations();
                        cx.notify();
                        !capsule.panel_manager.any_animating()
                    })
                    .unwrap_or(true);

                if done {
                    this.update(cx, |capsule, cx| {
                        capsule.panel_manager.update_animations();
                        capsule.sync_panel_indices(cx);
                        capsule.satellite_anim_task = None;
                        cx.notify();
                    })
                    .ok();
                    break;
                }
            }
        });
        self.satellite_anim_task = Some(task);
    }

    fn on_workspace_changed(&mut self, cx: &mut Context<Self>) {
        cx.notify();
    }

    fn on_config_changed(
        &mut self,
        config: &services::AppConfig,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_target_y = if config.ui.capsule_style == services::CapsuleStyle::Concave {
            0.0
        } else {
            config.ui.margin_top
        };
        let new_target_r = config.ui.capsule_round;

        let y_diff = (self.target_y - new_target_y).abs();
        let r_diff = (self.target_radius - new_target_r).abs();

        if y_diff > 0.1 || r_diff > 0.1 {
            self.target_y = new_target_y;
            self.target_radius = new_target_r;

            if self.animating {
                self.anim_start_w = self.current_width;
                self.anim_start_h = self.current_height;
                self.anim_start_r = self.current_radius;
                self.anim_start_y = self.current_y;
                self.anim_start_progress = self.anim_progress;
                self.anim_start_time = Some(Instant::now());
            } else {
                self.animate_dimension_change(cx);
            }
        }

        window.set_exclusive_zone(px(config.ui.exclusive_zone()));
        cx.notify();
    }

    fn sync_orbit_visibility(&mut self, cx: &mut Context<Self>) {
        let visible = self.mode == CapsuleMode::Default && !self.drag_target_active;
        self.orbit
            .update(cx, |orbit, cx| orbit.set_visible(visible, cx));
    }

    pub(crate) fn open_orb(&mut self, kind: OrbKind, cx: &mut Context<Self>) {
        if self.mode != CapsuleMode::Default || self.drag_target_active {
            return;
        }
        let orbit = self.orbit.read(cx);
        let active = match kind {
            OrbKind::Shelf => orbit.shelf_count > 0,
            OrbKind::Recording => orbit.record_status != services::RecordStatus::Stopped,
        };
        if active {
            let mode = match kind {
                OrbKind::Shelf => CapsuleMode::Shelf,
                OrbKind::Recording => CapsuleMode::Record,
            };
            self.start_transition_internal(mode, None, cx);
        }
    }

    pub(crate) fn drop_on_shelf(
        &mut self,
        external_paths: &gpui::ExternalPaths,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.drag_target_active = false;
        self.last_drag_over = None;
        self.drag_monitor_task = None;
        cx.global::<AppState>()
            .shelf
            .add_paths(external_paths.paths());
        self.modules
            .shelf_view
            .update(cx, |shelf, cx| shelf.reload_items(cx));
        self.sync_orbit_visibility(cx);
        if self.mode == CapsuleMode::Default {
            let (width, _) = self.modules.default_view.read(cx).desired_dimensions();
            self.target_width = width;
            self.target_height = CapsuleMode::Default.dimensions().1;
            self.target_radius = cx.global::<AppState>().config.get().ui.capsule_round;
            self.anim_task = None;
            self.animating = false;
            self.animate_dimension_change(cx);
        }
        cx.notify();
    }

    pub fn on_drag_over_capsule(&mut self, cx: &mut Context<Self>) {
        self.last_drag_over = Some(Instant::now());
        if !self.drag_target_active {
            self.drag_target_active = true;
            self.sync_orbit_visibility(cx);
            if self.mode == CapsuleMode::Default {
                self.anim_start_w = self.current_width;
                self.anim_start_h = self.current_height;
                self.anim_start_r = self.current_radius;
                self.anim_start_y = self.current_y;
                self.anim_start_progress = self.anim_progress;
                self.anim_start_time = Some(Instant::now());
                self.animating = true;
                self.is_mode_transition = false;
                self.target_width = 560.0;
                self.target_height = 104.0;
                self.target_radius = if cx.has_global::<AppState>() {
                    cx.global::<AppState>().config.get().ui.capsule_round
                } else {
                    42.0
                };

                let compositor = cx.global::<AppState>().compositor.clone();
                let task = cx.spawn(async move |this, cx| {
                    loop {
                        cx.background_executor()
                            .timer(compositor.get_frame_duration())
                            .await;
                        let done = this
                            .update(cx, |capsule, cx| {
                                let duration = if cx.has_global::<AppState>() {
                                    cx.global::<AppState>()
                                        .config
                                        .get()
                                        .ui
                                        .animation_duration_ms
                                        as f32
                                        / 1000.0
                                } else {
                                    0.28
                                };
                                let finished = capsule.tick_animation(duration);
                                cx.notify();
                                finished
                            })
                            .unwrap_or(true);

                        if done {
                            this.update(cx, |capsule, cx| {
                                capsule.animating = false;
                                capsule.is_mode_transition = false;
                                capsule.current_width = capsule.target_width;
                                capsule.current_height = capsule.target_height;
                                capsule.anim_task = None;
                                cx.notify();
                            })
                            .ok();
                            break;
                        }
                    }
                });
                self.anim_task = Some(task);
            }
            self.start_drag_monitor(cx);
            cx.notify();
        }
    }

    fn start_drag_monitor(&mut self, cx: &mut Context<Self>) {
        if self.drag_monitor_task.is_some() {
            return;
        }

        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(40))
                    .await;

                let should_stop = this
                    .update(cx, |capsule, cx| {
                        if !capsule.drag_target_active {
                            capsule.drag_monitor_task = None;
                            return true;
                        }

                        if let Some(last) = capsule.last_drag_over
                            && last.elapsed().as_millis() > 140
                        {
                            capsule.drag_target_active = false;
                            capsule.sync_orbit_visibility(cx);
                            capsule.last_drag_over = None;
                            capsule.drag_monitor_task = None;

                            if capsule.mode == CapsuleMode::Default {
                                let (w, h) = {
                                    let (desired_w, _h) =
                                        capsule.modules.default_view.read(cx).desired_dimensions();
                                    (desired_w, CapsuleMode::Default.dimensions().1)
                                };
                                let r = if cx.has_global::<AppState>() {
                                    cx.global::<AppState>().config.get().ui.capsule_round
                                } else {
                                    CapsuleMode::Default.radius()
                                };
                                capsule.anim_start_w = capsule.current_width;
                                capsule.anim_start_h = capsule.current_height;
                                capsule.anim_start_r = capsule.current_radius;
                                capsule.anim_start_y = capsule.current_y;
                                capsule.anim_start_progress = capsule.anim_progress;
                                capsule.anim_start_time = Some(Instant::now());
                                capsule.animating = true;
                                capsule.is_mode_transition = false;
                                capsule.target_width = w;
                                capsule.target_height = h;
                                capsule.target_radius = r;
                            }

                            let compositor = cx.global::<AppState>().compositor.clone();
                            let task = cx.spawn(async move |this, cx| {
                                loop {
                                    cx.background_executor()
                                        .timer(compositor.get_frame_duration())
                                        .await;
                                    let done = this
                                        .update(cx, |capsule, cx| {
                                            let duration = if cx.has_global::<AppState>() {
                                                cx.global::<AppState>()
                                                    .config
                                                    .get()
                                                    .ui
                                                    .animation_duration_ms
                                                    as f32
                                                    / 1000.0
                                            } else {
                                                0.28
                                            };
                                            let finished = capsule.tick_animation(duration);
                                            cx.notify();
                                            finished
                                        })
                                        .unwrap_or(true);

                                    if done {
                                        this.update(cx, |capsule, cx| {
                                            capsule.animating = false;
                                            capsule.is_mode_transition = false;
                                            capsule.current_width = capsule.target_width;
                                            capsule.current_height = capsule.target_height;
                                            capsule.anim_task = None;
                                            cx.notify();
                                        })
                                        .ok();
                                        break;
                                    }
                                }
                            });
                            capsule.anim_task = Some(task);
                            cx.notify();
                            return true;
                        }
                        false
                    })
                    .unwrap_or(true);

                if should_stop {
                    break;
                }
            }
        });

        self.drag_monitor_task = Some(task);
    }

    pub(crate) fn start_transition_internal(
        &mut self,
        mode: CapsuleMode,
        window_opt: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if self.mode == mode && !self.animating {
            return;
        }

        let old_mode = self.mode;
        if old_mode == CapsuleMode::Notification && mode != CapsuleMode::Notification {
            self.modules
                .notification_view
                .update(cx, |view, cx| view.deactivate(cx));
        }
        if old_mode == CapsuleMode::Dashboard && mode != CapsuleMode::Dashboard {
            self.panel_manager.clear();
            self.sync_panel_indices(cx);
            self.satellite_anim_task = None;
        }
        self.reset_inactivity_timer();
        self.mode = mode;
        self.sync_orbit_visibility(cx);
        services::log_info!("UI", "Transitioning to mode: {:?}", mode);

        if mode != CapsuleMode::Default && cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.request_layer_focus();
        }

        if old_mode == CapsuleMode::Wallpaper && mode != CapsuleMode::Wallpaper {
            self.modules.wallpaper_view.update(cx, |wallpaper, cx| {
                wallpaper.clear_cache(cx);
            });
        }

        if mode == CapsuleMode::Launcher {
            self.modules.launcher_view.update(cx, |launcher, cx| {
                launcher.reset_search(cx);
            });
        } else if mode == CapsuleMode::SelectTheme {
            self.modules.select_theme_view.update(cx, |st, cx| {
                st.refresh_themes(cx);
            });
        } else if mode == CapsuleMode::Shelf {
            self.modules.shelf_view.update(cx, |shelf, cx| {
                shelf.reload_items(cx);
            });
        }

        if mode != CapsuleMode::Default
            && mode != CapsuleMode::Dashboard
            && mode != CapsuleMode::Launcher
            && mode != CapsuleMode::Polkit
            && mode != CapsuleMode::Record
            && mode != CapsuleMode::Shelf
            && mode != CapsuleMode::Notification
        {
            self.inactivity_generation += 1;
            let current_gen = self.inactivity_generation;
            cx.spawn(async move |weak, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(500))
                        .await;

                    let should_stop = weak
                        .update(cx, |capsule: &mut Self, cx| {
                            if capsule.inactivity_generation != current_gen
                                || capsule.mode == CapsuleMode::Dashboard
                                || capsule.mode == CapsuleMode::Launcher
                                || capsule.mode == CapsuleMode::Default
                                || capsule.mode == CapsuleMode::Polkit
                                || capsule.mode == CapsuleMode::Record
                                || capsule.mode == CapsuleMode::Shelf
                            {
                                return true;
                            }

                            if capsule.last_activity_time.elapsed() >= Duration::from_secs(5) {
                                capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                                return true;
                            }

                            false
                        })
                        .unwrap_or(true);

                    if should_stop {
                        break;
                    }
                }
            })
            .detach();
        }

        let (target_w, target_h) = if mode == CapsuleMode::Default {
            let (w, _h) = self.modules.default_view.read(cx).desired_dimensions();
            (w, mode.dimensions().1)
        } else if mode == CapsuleMode::Notification {
            self.modules.notification_view.read(cx).desired_dimensions()
        } else if mode == CapsuleMode::Dashboard {
            (
                self.modules.dashboard_view.read(cx).desired_width(cx),
                mode.dimensions().1,
            )
        } else if mode == CapsuleMode::Record {
            (
                self.modules.record_view.read(cx).desired_width(cx),
                mode.dimensions().1,
            )
        } else {
            mode.dimensions()
        };
        self.dimension_tracker.reset();

        if let Some(display) = cx.displays().first() {
            let dh: f32 = display.bounds().size.height.into();
            if dh > 100.0 {
                self.window_height = dh;
            }
        }
        let target_r = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.capsule_round
        } else {
            mode.radius()
        };

        let margin_top = if cx.has_global::<AppState>() {
            let ui_cfg = &cx.global::<AppState>().config.get().ui;
            if ui_cfg.capsule_style == services::CapsuleStyle::Concave {
                0.0
            } else {
                ui_cfg.margin_top
            }
        } else {
            8.0
        };
        let target_y = margin_top;

        self.target_width = target_w;
        self.target_height = target_h;
        self.target_radius = target_r;
        self.target_y = target_y;

        self.anim_start_w = self.current_width;
        self.anim_start_h = self.current_height;
        self.anim_start_r = self.current_radius;
        self.anim_start_y = self.current_y;
        self.anim_start_progress = self.anim_progress;

        let _ = window_opt;

        self.animating = true;
        self.is_mode_transition = true;
        self.anim_start_time = Some(Instant::now());

        let compositor = cx.global::<AppState>().compositor.clone();
        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(compositor.get_frame_duration())
                    .await;
                let done = this
                    .update(cx, |capsule, cx| {
                        let duration = if cx.has_global::<AppState>() {
                            cx.global::<AppState>()
                                .config
                                .get()
                                .ui
                                .animation_duration_ms as f32
                                / 1000.0
                        } else {
                            0.28
                        };
                        let finished = capsule.tick_animation(duration);
                        cx.notify();
                        finished
                    })
                    .unwrap_or(true);

                if done {
                    this.update(cx, |capsule, cx| {
                        capsule.animating = false;
                        capsule.is_mode_transition = false;
                        capsule.current_width = capsule.target_width;
                        capsule.current_height = capsule.target_height;
                        capsule.current_radius = capsule.target_radius;
                        capsule.current_y = capsule.target_y;
                        capsule.anim_task = None;
                        cx.notify();
                    })
                    .ok();
                    break;
                }
            }
        });
        self.anim_task = Some(task);

        cx.notify();
    }

    #[allow(dead_code)]
    pub fn handle_ipc_command(&mut self, cmd: services::IpcCommand, cx: &mut Context<Self>) {
        self.reset_inactivity_timer();
        match cmd {
            services::IpcCommand::ToggleLauncher => {
                let target = if self.mode == CapsuleMode::Launcher {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Launcher
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ToggleDashboard => {
                let target = if self.mode == CapsuleMode::Dashboard {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Dashboard
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ToggleNotification => {
                let target = if self.mode == CapsuleMode::Notification {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Notification
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ToggleSelectTheme => {
                let target = if self.mode == CapsuleMode::SelectTheme {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::SelectTheme
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ToggleClipboard => {
                let target = if self.mode == CapsuleMode::Clipboard {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Clipboard
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ToggleEmoji => {
                let target = if self.mode == CapsuleMode::Emoji {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Emoji
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ShowLauncher => {
                self.start_transition_internal(CapsuleMode::Launcher, None, cx);
            }
            services::IpcCommand::ShowDashboard => {
                self.start_transition_internal(CapsuleMode::Dashboard, None, cx);
            }
            services::IpcCommand::ShowNotification => {
                self.start_transition_internal(CapsuleMode::Notification, None, cx);
            }
            services::IpcCommand::ShowClipboard => {
                self.start_transition_internal(CapsuleMode::Clipboard, None, cx);
            }
            services::IpcCommand::ShowEmoji => {
                self.start_transition_internal(CapsuleMode::Emoji, None, cx);
            }
            services::IpcCommand::Hide | services::IpcCommand::Default => {
                self.start_transition_internal(CapsuleMode::Default, None, cx);
            }
            services::IpcCommand::Lock => {
                crate::panel::LockScreenPanel::open_all(cx);
            }
            services::IpcCommand::Quit => {
                cx.quit();
            }
            services::IpcCommand::Terminal => {
                if cx.has_global::<AppState>() {
                    let config = cx.global::<AppState>().config.get();
                    let term = &config.defaults.terminal;
                    let cmd = format!("{term} >/dev/null 2>&1 &");
                    let mut command = std::process::Command::new("sh");
                    if let Some(home) = dirs::home_dir() {
                        command.current_dir(home);
                    }
                    let _ = command.arg("-c").arg(cmd).spawn();
                }
            }
            services::IpcCommand::Browser => {
                if cx.has_global::<AppState>() {
                    let config = cx.global::<AppState>().config.get();
                    let browser = &config.defaults.browser;
                    let cmd = format!("{browser} >/dev/null 2>&1 &");
                    let mut command = std::process::Command::new("sh");
                    if let Some(home) = dirs::home_dir() {
                        command.current_dir(home);
                    }
                    let _ = command.arg("-c").arg(cmd).spawn();
                }
            }
            services::IpcCommand::Editor => {
                if cx.has_global::<AppState>() {
                    let config = cx.global::<AppState>().config.get();
                    let editor = &config.defaults.editor;
                    let is_terminal_app = matches!(
                        editor.split_whitespace().next().unwrap_or(""),
                        "nvim" | "vim" | "vi" | "nano" | "hx" | "helix" | "micro"
                    );
                    let cmd = if is_terminal_app {
                        let term = &config.defaults.terminal;
                        format!("{term} -e {editor} >/dev/null 2>&1 &")
                    } else {
                        format!("{editor} >/dev/null 2>&1 &")
                    };
                    let mut command = std::process::Command::new("sh");
                    if let Some(home) = dirs::home_dir() {
                        command.current_dir(home);
                    }
                    let _ = command.arg("-c").arg(cmd).spawn();
                }
            }
            services::IpcCommand::FileManager => {
                if cx.has_global::<AppState>() {
                    let config = cx.global::<AppState>().config.get();
                    let file_manager = &config.defaults.file_manager;
                    let is_terminal_app = matches!(
                        file_manager.split_whitespace().next().unwrap_or(""),
                        "yazi" | "ranger" | "lf" | "nnn" | "mc" | "vifm"
                    );
                    let cmd = if is_terminal_app {
                        let term = &config.defaults.terminal;
                        format!("{term} -e {file_manager} >/dev/null 2>&1 &")
                    } else {
                        format!("{file_manager} >/dev/null 2>&1 &")
                    };
                    let mut command = std::process::Command::new("sh");
                    if let Some(home) = dirs::home_dir() {
                        command.current_dir(home);
                    }
                    let _ = command.arg("-c").arg(cmd).spawn();
                }
            }
            services::IpcCommand::ToggleSettings => {
                crate::panel::SettingsPanel::toggle(cx);
            }
            services::IpcCommand::ShowSettings => {
                crate::panel::SettingsPanel::open(cx);
            }
            services::IpcCommand::ToggleRecord => {
                let target = if self.mode == CapsuleMode::Record {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Record
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ShowRecord => {
                self.start_transition_internal(CapsuleMode::Record, None, cx);
            }
            services::IpcCommand::ToggleShelf => {
                let target = if self.mode == CapsuleMode::Shelf {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::Shelf
                };
                self.start_transition_internal(target, None, cx);
            }
            services::IpcCommand::ShowShelf => {
                self.start_transition_internal(CapsuleMode::Shelf, None, cx);
            }
            _ => {}
        }
    }

    fn tick_animation(&mut self, duration: f32) -> bool {
        if !self.animating || self.anim_start_time.is_none() {
            self.animating = false;
            self.is_mode_transition = false;
            return true;
        }

        if let Some(start_time) = self.anim_start_time {
            let target_progress = if self.mode == CapsuleMode::Dashboard {
                1.0
            } else {
                0.0
            };

            if duration <= 0.001 {
                self.current_width = self.target_width;
                self.current_height = self.target_height;
                self.current_radius = self.target_radius;
                self.current_y = self.target_y;
                self.anim_progress = target_progress;
                self.animating = false;
                self.is_mode_transition = false;
                return true;
            }

            let t = (start_time.elapsed().as_secs_f32() / duration).min(1.0);
            let is_expanding =
                self.target_width > self.anim_start_w || self.target_height > self.anim_start_h;
            let (w_eased, h_eased) = apple_island_morph(t, is_expanding);
            let eased = apple_island_spring(t);

            self.current_width =
                (self.anim_start_w + (self.target_width - self.anim_start_w) * w_eased).max(1.0);
            self.current_height =
                (self.anim_start_h + (self.target_height - self.anim_start_h) * h_eased).max(1.0);
            self.current_radius =
                (self.anim_start_r + (self.target_radius - self.anim_start_r) * eased).max(0.0);
            self.current_y = self.anim_start_y + (self.target_y - self.anim_start_y) * eased;
            self.anim_progress = (self.anim_start_progress
                + (target_progress - self.anim_start_progress) * eased)
                .clamp(0.0, 1.0);

            if t >= 1.0 {
                self.current_width = self.target_width;
                self.current_height = self.target_height;
                self.current_radius = self.target_radius;
                self.current_y = self.target_y;
                self.anim_progress = target_progress;
                self.animating = false;
                self.is_mode_transition = false;

                return true;
            }
        }

        false
    }
}

impl Render for Capsule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let ui_config = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.clone()
        } else {
            services::UIConfig::default()
        };
        let anim_duration = (ui_config.animation_duration_ms as f32 / 1000.0).max(0.001);
        window.set_exclusive_zone(px(ui_config.exclusive_zone()));

        let win_w: f32 = window.bounds().size.width.into();
        let win_h: f32 = window.bounds().size.height.into();
        if win_h > 100.0 {
            self.window_height = win_h;
        }

        let orbit = self.orbit.read(cx);
        let shelf_count = orbit.shelf_count;
        let record_status = orbit.record_status;
        let orbs = orbit.geometry(self.current_width, self.current_height, ui_config.gap);

        let is_modal = self.mode == CapsuleMode::Launcher
            || self.mode == CapsuleMode::Dashboard
            || self.mode == CapsuleMode::Polkit
            || self.mode == CapsuleMode::SelectTheme
            || self.mode == CapsuleMode::Wallpaper
            || self.mode == CapsuleMode::Clipboard
            || self.mode == CapsuleMode::Emoji
            || (self.mode == CapsuleMode::Default && self.panel_manager.has_open())
            || self.drag_target_active;

        let needs_exclusive_focus = self.mode == CapsuleMode::Launcher
            || self.mode == CapsuleMode::Dashboard
            || self.mode == CapsuleMode::Polkit
            || self.mode == CapsuleMode::SelectTheme
            || self.mode == CapsuleMode::Wallpaper
            || self.mode == CapsuleMode::Clipboard
            || self.mode == CapsuleMode::Emoji
            || (self.mode == CapsuleMode::Notification
                && self.modules.notification_view.read(cx).is_replying());

        if needs_exclusive_focus {
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        } else {
            window.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        }

        if is_modal {
            window.set_input_region(None);
        } else {
            let pill_x = (win_w - self.current_width) / 2.0;
            let pill_y = self.current_y.max(0.0);
            let pill_bounds = Bounds {
                origin: point(px(pill_x), px(pill_y)),
                size: Size::new(px(self.current_width), px(self.current_height)),
            };
            let mut input_bounds = vec![pill_bounds];
            for orb in orbs.iter().filter(|orb| orb.interactive) {
                input_bounds.push(Bounds {
                    origin: point(px(pill_x + orb.x), px(pill_y + orb.y)),
                    size: Size::new(px(ORB_SIZE), px(ORB_SIZE)),
                });
            }
            window.set_input_region(Some(&input_bounds));
        }

        if self.mode == CapsuleMode::Notification {
            let entity = cx.entity().downgrade();
            window.on_mouse_event(move |_: &MouseExitEvent, phase, _window, cx| {
                if phase == DispatchPhase::Bubble {
                    let _ = entity.update(cx, |capsule, cx| {
                        capsule.is_hovered = false;
                        capsule.modules.notification_view.update(cx, |view, cx| {
                            view.set_expanded(false, cx);
                        });
                    });
                }
            });

            let entity = cx.entity().downgrade();
            let pill_x = (win_w - self.current_width) / 2.0;
            let pill_y = self.current_y.max(0.0);
            let pill_w = self.current_width;
            let pill_h = self.current_height;

            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Bubble {
                    let mx: f32 = event.position.x.into();
                    let my: f32 = event.position.y.into();
                    let inside = window.is_window_hovered()
                        && mx >= pill_x
                        && mx <= pill_x + pill_w
                        && my >= pill_y
                        && my <= pill_y + pill_h;

                    let _ = entity.update(cx, |capsule, cx| {
                        capsule.is_hovered = inside;
                        if capsule.modules.notification_view.read(cx).is_replying() {
                            return;
                        }
                        capsule.modules.notification_view.update(cx, |view, cx| {
                            view.set_expanded(inside, cx);
                        });
                    });
                }
            });

            if !window.is_window_hovered()
                && !self.modules.notification_view.read(cx).is_replying()
                && self.modules.notification_view.read(cx).is_expanded()
            {
                let notif_view = self.modules.notification_view.clone();
                cx.defer(move |cx| {
                    notif_view.update(cx, |view, cx| {
                        view.set_expanded(false, cx);
                    });
                });
            }
        }

        let mut content_container = div().relative().size_full();

        let anim_t = self
            .anim_start_time
            .map(|start| (start.elapsed().as_secs_f32() / anim_duration).min(1.0))
            .unwrap_or(1.0);

        if self.last_rendered_mode != Some(self.mode) {
            self.last_rendered_mode = Some(self.mode);
            if self.mode == CapsuleMode::Launcher
                || self.mode == CapsuleMode::Dashboard
                || self.mode == CapsuleMode::Polkit
                || self.mode == CapsuleMode::SelectTheme
                || self.mode == CapsuleMode::Wallpaper
                || self.mode == CapsuleMode::Clipboard
                || self.mode == CapsuleMode::Emoji
                || self.mode == CapsuleMode::Shelf
            {
                window.activate_window();
                if cx.has_global::<AppState>() {
                    cx.global::<AppState>().compositor.request_layer_focus();
                }
            }
            if self.mode == CapsuleMode::Launcher {
                self.modules.launcher_view.update(cx, |launcher, cx| {
                    launcher.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Clipboard {
                self.modules.clipboard_view.update(cx, |clip, cx| {
                    clip.reload_items(cx);
                    clip.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Emoji {
                self.modules.emoji_view.update(cx, |emoji, cx| {
                    emoji.reload_items(cx);
                    emoji.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Wallpaper {
                self.modules.wallpaper_view.update(cx, |wallpaper, cx| {
                    wallpaper.reload_items(cx);
                    wallpaper.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Shelf {
                self.modules.shelf_view.update(cx, |shelf, cx| {
                    shelf.reload_items(cx);
                    shelf.focus(window, cx);
                });
            }
        }

        let mode_element = if self.mode == CapsuleMode::Default {
            None
        } else {
            Some(self.modules.render_active_view(self.mode))
        };

        if self.drag_target_active {
            content_container = content_container.child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("cloud-upload.svg")
                            .size(px(32.0))
                            .text_color(gpui::rgb(0x64748b)),
                    ),
            );
        } else if let Some(el) = mode_element {
            let reveal_progress = if self.animating && self.is_mode_transition {
                ((anim_t - 0.2) / 0.55).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let offset_y = -8.0 * (1.0 - reveal_progress).powi(3);
            let fade_progress = (reveal_progress / 0.5).min(1.0);
            let opacity = 1.0 - (1.0 - fade_progress).powi(3);
            let wrapper = if self.mode == CapsuleMode::Volume
                || self.mode == CapsuleMode::Record
                || self.mode == CapsuleMode::Shelf
            {
                div()
                    .absolute()
                    .top(px(offset_y))
                    .left_0()
                    .size_full()
                    .opacity(opacity)
                    .child(el)
            } else if self.mode == CapsuleMode::Notification {
                div()
                    .absolute()
                    .top(px(offset_y))
                    .left_0()
                    .w(px(super::modules::notification::MAX_NOTIFICATION_WIDTH))
                    .flex()
                    .items_start()
                    .opacity(opacity)
                    .child(el)
            } else {
                let tracked_content = self.dimension_tracker.track(el);
                div()
                    .absolute()
                    .top(px(offset_y))
                    .left_0()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(opacity)
                    .child(tracked_content)
            };

            content_container = content_container.child(wrapper);
        } else {
            let default_opacity = (1.0 - self.anim_progress).clamp(0.0, 1.0);

            if default_opacity > 0.001 {
                content_container = content_container.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .overflow_hidden()
                        .opacity(default_opacity)
                        .child(self.modules.default_view.clone().into_any_element()),
                );
            }
        }

        let active_theme = cx.global::<Theme>().clone();

        let is_dashboard = self.mode == CapsuleMode::Dashboard;
        let border_opacity = if is_dashboard {
            0.6
        } else {
            (0.12 + 0.4 * (1.0 - self.anim_progress)).clamp(0.12, 0.5)
        };
        let border_color = theme.surface().opacity(border_opacity);
        let bg_color = active_theme.background();

        let params = super::container::ContainerParams {
            width: self.current_width,
            height: self.current_height,
            radius: self.current_radius,
            border_color,
            bg_color,
            font_family: theme.font_family(),
            mode: if self.drag_target_active {
                CapsuleMode::Shelf
            } else {
                self.mode
            },
        };

        let renderer: Box<dyn super::container::CapsuleContainerRenderer> =
            if self.drag_target_active {
                Box::new(super::container::NormalContainer::new())
            } else {
                match cx.global::<AppState>().config.get().ui.capsule_style {
                    services::CapsuleStyle::Normal => {
                        Box::new(super::container::NormalContainer::new())
                    }
                    services::CapsuleStyle::Concave => {
                        Box::new(super::container::ConcaveContainer::new())
                    }
                }
            };

        let pill_wrapper = renderer
            .render(content_container.into_any_element(), &params, cx)
            .id("capsule-hover")
            .on_hover(cx.listener(|capsule, hovered, window, cx| {
                let win_hovered = window.is_window_hovered();
                let actual_hovered = *hovered && win_hovered;

                if capsule.mode == CapsuleMode::Notification {
                    capsule.is_hovered = actual_hovered;
                    capsule
                        .modules
                        .notification_view
                        .update(cx, |view, cx| view.set_expanded(actual_hovered, cx));
                } else {
                    capsule.is_hovered = actual_hovered;
                }
            }))
            .drag_over::<gpui::ExternalPaths>(move |style, _paths, window, cx| {
                if let Some(Some(root)) = window.root::<Capsule>() {
                    root.update(cx, |capsule, cx| {
                        capsule.on_drag_over_capsule(cx);
                    });
                }
                style
            })
            .drag_over::<crate::capsule::widgets::shelf::shelf_card::DraggedShelfItem>(
                move |style, _item, window, cx| {
                    if let Some(Some(root)) = window.root::<Capsule>() {
                        root.update(cx, |capsule, cx| {
                            capsule.on_drag_over_capsule(cx);
                        });
                    }
                    style
                },
            )
            .on_drop(cx.listener(Self::drop_on_shelf));

        if self.mode == CapsuleMode::Dashboard || self.mode == CapsuleMode::Default {
            self.panel_manager
                .set_animation_duration(ui_config.animation_duration_ms as f32 / 1000.0);
        }
        let mut satellite_surfaces = Vec::new();
        let mut satellites_layer = div().absolute().inset_0();

        let has_panels =
            !self.panel_manager.left.is_empty() || !self.panel_manager.right.is_empty();
        if (self.mode == CapsuleMode::Dashboard || self.mode == CapsuleMode::Default)
            && has_panels
            && cx.has_global::<AppState>()
        {
            use super::satellites as PM;

            let dash_w = self.current_width;
            let dash_h = self.current_height;
            let sni_items = cx.global::<AppState>().sni_host.get_items();
            self.panel_manager.prune_invalid(sni_items.len());
            self.panel_manager.update_animations();

            if self.panel_manager.any_animating() && self.satellite_anim_task.is_none() {
                self.start_satellite_animation(cx);
            }

            for p in self.panel_manager.left.iter_mut() {
                let measured_h = p.tracker.height(0.0);
                if measured_h > 10.0 && (measured_h - p.height).abs() > 2.0 {
                    p.height = measured_h;
                }
            }
            for p in self.panel_manager.right.iter_mut() {
                let measured_h = p.tracker.height(0.0);
                if measured_h > 10.0 && (measured_h - p.height).abs() > 2.0 {
                    p.height = measured_h;
                }
            }

            for (lane, panels) in [
                (PM::Lane::Left, &self.panel_manager.left),
                (PM::Lane::Right, &self.panel_manager.right),
            ] {
                let mut y_stack = 0.0;
                for panel in panels {
                    let phase = panel.anim_t();
                    let panel_h = panel.height;
                    let panel_w = panel.tracker.width(0.0).max(PM::PANEL_MIN_W);
                    let content_size = Size::new(panel_w, panel_h);
                    let reveal = PM::surface::PanelReveal::new(
                        content_size,
                        y_stack,
                        Size::new(dash_w, dash_h),
                        self.current_radius,
                        phase,
                    );
                    let lane_x = match lane {
                        PM::Lane::Left => -(reveal.size.width + PM::LANE_GAP),
                        PM::Lane::Right => dash_w + PM::LANE_GAP,
                    };
                    let (x, y) = PM::PanelManager::animated_position(
                        lane,
                        dash_w,
                        dash_h,
                        reveal.size.width,
                        lane_x,
                        reveal.y,
                        phase,
                        panel.is_closing(),
                    );
                    y_stack += (panel_h + ui_config.gap) * panel.stack_weight();

                    let mini = match &panel.kind {
                        PM::PanelKind::Tray(index) => {
                            let Some(item) = sni_items.get(*index) else {
                                continue;
                            };
                            self.modules.dashboard_view.update(cx, |_, cx| {
                                PM::tray::render_mini_panel(
                                    item,
                                    *index,
                                    panel_h,
                                    &active_theme,
                                    cx,
                                )
                            })
                        }
                        PM::PanelKind::Wifi => {
                            self.modules.dashboard_view.update(cx, |module, cx| {
                                PM::wifi::render_wifi_mini_panel(panel_h, module, &active_theme, cx)
                            })
                        }
                        PM::PanelKind::Bluetooth => {
                            self.modules.dashboard_view.update(cx, |_, cx| {
                                PM::bluetooth::render_bluetooth_mini_panel(
                                    panel_h,
                                    &active_theme,
                                    cx,
                                )
                            })
                        }
                        PM::PanelKind::Calendar => {
                            self.modules.dashboard_view.update(cx, |_, cx| {
                                PM::calendar::render_calendar_mini_panel(panel_h, &active_theme, cx)
                            })
                        }
                        PM::PanelKind::Volume => self.modules.dashboard_view.update(cx, |_, cx| {
                            PM::volume::render_volume_mini_panel(panel_h, &active_theme, cx)
                        }),
                        PM::PanelKind::Media => {
                            self.modules.dashboard_view.update(cx, |module, cx| {
                                PM::media::render_media_mini_panel(
                                    panel_h,
                                    module,
                                    &active_theme,
                                    cx,
                                )
                            })
                        }
                    };
                    let surface = PM::surface::PanelSurface {
                        lane,
                        x,
                        y,
                        width: reveal.size.width,
                        height: reveal.size.height,
                        phase,
                        closing: panel.is_closing(),
                        radius: ui_config.satellite_round,
                    };
                    let measured_content = panel.tracker.track(mini).into_any_element();
                    let framed = surface.render(measured_content, content_size, &active_theme);
                    let panel_wrapper_id = match &panel.kind {
                        PM::PanelKind::Tray(i) => *i as u32,
                        PM::PanelKind::Wifi => 10001,
                        PM::PanelKind::Bluetooth => 10002,
                        PM::PanelKind::Calendar => 10003,
                        PM::PanelKind::Volume => 10004,
                        PM::PanelKind::Media => 10005,
                    };
                    satellites_layer = satellites_layer.child(
                        div()
                            .id(("satellite-panel-wrapper", panel_wrapper_id))
                            .absolute()
                            .left(px(x))
                            .top(px(y))
                            .on_hover(cx.listener(|capsule, hovered: &bool, _window, _cx| {
                                capsule.is_hovered = *hovered;
                            }))
                            .on_click(cx.listener(|_this, _, _, cx| cx.stop_propagation()))
                            .child(framed),
                    );
                    if phase < 0.85 {
                        satellite_surfaces.push(surface);
                    }
                }
            }
        }

        let mut content_stack = div()
            .relative()
            .w(px(self.current_width))
            .h(px(self.current_height))
            .child(satellites_layer);

        for orb in orbs {
            let content = match orb.kind {
                OrbKind::Shelf => render_shelf_orb(shelf_count, orb.interactive, &active_theme, cx)
                    .into_any_element(),
                OrbKind::Recording => {
                    render_record_orb(record_status, orb.interactive, &active_theme, cx)
                        .into_any_element()
                }
            };
            content_stack = content_stack.child(
                div()
                    .absolute()
                    .left(px(orb.x))
                    .top(px(orb.y))
                    .opacity(orb.opacity)
                    .child(content),
            );
        }

        content_stack = content_stack.child(pill_wrapper);
        if !satellite_surfaces.is_empty() {
            content_stack = content_stack.child(super::satellites::surface::connections(
                satellite_surfaces,
                self.current_width,
                self.current_height,
                self.current_radius,
                &active_theme,
            ));
        }

        let flex_container = div()
            .flex()
            .flex_row()
            .items_start()
            .justify_center()
            .child(content_stack);

        let mut root = div()
            .id("capsule-root")
            .size_full()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(self.current_y.max(0.0)))
            .child(flex_container);

        if self.mode == CapsuleMode::Default && self.panel_manager.has_open() {
            root = root.on_click(cx.listener(|this, _, _, cx| {
                if this.panel_manager.has_open() {
                    this.panel_manager.clear();
                    this.sync_panel_indices(cx);
                    this.satellite_anim_task = None;
                    cx.notify();
                }
            }));
        }

        root
    }
}
