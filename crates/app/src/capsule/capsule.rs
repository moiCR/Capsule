use gpui::{Bounds, Context, Entity, Render, Size, Task, Window, div, point, prelude::*, px};
use services::{AppState, NotificationStore};
use std::time::{Duration, Instant};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::satellites::PanelManager;
use super::{CapsuleMode, MARGIN_TOP, apple_island_ease};

use super::modules::clipboard::{ClipboardEvent, ClipboardModule};
use super::modules::create_theme::CreateThemeModule;
use super::modules::dashboard::DashboardModule;
use super::modules::emoji::{EmojiEvent, EmojiModule};
use super::modules::idle::IdleModule;
use super::modules::launcher::LauncherModule;
use super::modules::notification::NotificationModule;
use super::modules::polkit::PolkitModule;
use super::modules::select_theme::SelectThemeModule;
use super::modules::volume::VolumeModule;
use super::modules::wallpaper::{WallpaperEvent, WallpaperModule};

pub struct Capsule {
    mode: CapsuleMode,
    idle_view: Entity<IdleModule>,
    dashboard_view: Entity<DashboardModule>,
    notification_view: Entity<NotificationModule>,
    launcher_view: Entity<LauncherModule>,
    volume_view: Entity<VolumeModule>,
    polkit_view: Entity<PolkitModule>,
    select_theme_view: Entity<SelectThemeModule>,
    create_theme_view: Entity<CreateThemeModule>,
    wallpaper_view: Entity<WallpaperModule>,
    clipboard_view: Entity<ClipboardModule>,
    emoji_view: Entity<EmojiModule>,
    panel_manager: PanelManager,
    current_width: f32,
    current_height: f32,
    current_radius: f32,
    target_width: f32,
    target_height: f32,
    target_radius: f32,
    anim_progress: f32,
    anim_start_time: Option<Instant>,
    anim_start_w: f32,
    anim_start_h: f32,
    anim_start_r: f32,
    anim_start_progress: f32,
    animating: bool,
    anim_task: Option<Task<()>>,
    last_activity_time: Instant,
    inactivity_generation: u64,
    last_vol_status: Option<(u32, bool)>,
    volume_timer_gen: u64,
    dimension_tracker: DimensionTracker,
    last_rendered_mode: Option<CapsuleMode>,
    is_mode_transition: bool,
}

impl Capsule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let idle_view = cx.new(IdleModule::new);
        let (initial_w, initial_h) = idle_view.read(cx).desired_dimensions();
        let r = CapsuleMode::Default.radius();

        cx.observe(&idle_view, |capsule, idle_view, cx| {
            if capsule.mode == CapsuleMode::Default {
                let (desired_w, desired_h) = idle_view.read(cx).desired_dimensions();
                capsule.update_target_dimensions(desired_w, desired_h, cx);
            }
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &idle_view,
            |capsule, _, event: &super::modules::idle::IdleEvent, cx| match event {
                super::modules::idle::IdleEvent::ExpandRequested => {
                    if capsule.mode == CapsuleMode::Default {
                        capsule.start_transition_internal(CapsuleMode::Dashboard, None, cx);
                    }
                }
            },
        )
        .detach();

        let dashboard_view = cx.new(DashboardModule::new);
        cx.observe(&dashboard_view, |_, _, cx| {
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &dashboard_view,
            |capsule, _, event: &super::modules::dashboard::DashboardEvent, cx| match event {
                super::modules::dashboard::DashboardEvent::CloseRequested => {
                    if capsule.mode == CapsuleMode::Dashboard {
                        capsule.panel_manager.close_all();
                        capsule.sync_panel_indices(cx);
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
                    cx.notify();
                }
                super::modules::dashboard::DashboardEvent::WallpaperRequested => {
                    capsule.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.reload_items(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Wallpaper, None, cx);
                }
                super::modules::dashboard::DashboardEvent::PowerClicked => {
                    let max_h = capsule.current_height;
                    capsule
                        .panel_manager
                        .toggle(super::satellites::PanelKind::Power, 130.0, max_h);
                    cx.notify();
                }
                super::modules::dashboard::DashboardEvent::LanguageClicked => {
                    let max_h = capsule.current_height;
                    capsule.panel_manager.toggle(
                        super::satellites::PanelKind::Language,
                        110.0,
                        max_h,
                    );
                    cx.notify();
                }
            },
        )
        .detach();

        let notification_view = cx.new(NotificationModule::new);
        cx.observe(&notification_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        let launcher_view = cx.new(LauncherModule::new);
        cx.observe(&launcher_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &launcher_view,
            |capsule, _, event: &super::modules::launcher::LauncherEvent, cx| match event {
                super::modules::launcher::LauncherEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let polkit_view = cx.new(PolkitModule::new);
        cx.observe(&polkit_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &polkit_view,
            |capsule, _, event: &super::modules::polkit::PolkitEvent, cx| match event {
                super::modules::polkit::PolkitEvent::Authenticated
                | super::modules::polkit::PolkitEvent::Cancelled => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let select_theme_view = cx.new(SelectThemeModule::new);
        cx.observe(&select_theme_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &select_theme_view,
            |capsule, _, event: &super::modules::select_theme::SelectThemeEvent, cx| match event {
                super::modules::select_theme::SelectThemeEvent::CreateThemeRequested => {
                    capsule.start_transition_internal(CapsuleMode::CreateTheme, None, cx);
                }
                super::modules::select_theme::SelectThemeEvent::ThemeSelected => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let create_theme_view = cx.new(CreateThemeModule::new);
        cx.observe(&create_theme_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &create_theme_view,
            |capsule, _, event: &super::modules::create_theme::CreateThemeEvent, cx| match event {
                super::modules::create_theme::CreateThemeEvent::ThemeCreated => {
                    let _ = capsule.select_theme_view.update(cx, |st, cx| {
                        st.refresh_themes(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::SelectTheme, None, cx);
                }
                super::modules::create_theme::CreateThemeEvent::Cancelled => {
                    capsule.start_transition_internal(CapsuleMode::SelectTheme, None, cx);
                }
            },
        )
        .detach();

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if this
                    .update(cx, |capsule: &mut Self, cx| {
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
                                let _ = capsule.polkit_view.update(cx, |p, cx| {
                                    p.set_request(req, responder, cx);
                                });
                                capsule.start_transition_internal(CapsuleMode::Polkit, None, cx);
                            }
                        }

                        let _ = capsule.polkit_view.update(cx, |p, cx| {
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
                            }
                        }

                        if cx.has_global::<ui::language::language_manager::LanguageManager>() {
                            let lang_updated = cx
                                .global_mut::<ui::language::language_manager::LanguageManager>()
                                .check_and_reload();
                            if lang_updated {
                                let new_lang = cx
                                    .global::<ui::language::language_manager::LanguageManager>()
                                    .current_language
                                    .clone();
                                cx.set_global(new_lang);
                                services::log_info!(
                                    "LANG",
                                    "Reloaded current_language.toml!"
                                );
                            }
                        }

                        let (m_w, m_h) = match capsule.mode {
                            CapsuleMode::Default => (0.0, 0.0),
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

                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        services::start_polkit_agent();

        let volume_view = cx.new(VolumeModule::new);
        cx.observe(&volume_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        let wallpaper_view = cx.new(WallpaperModule::new);
        cx.subscribe(
            &wallpaper_view,
            |capsule, _, event: &WallpaperEvent, cx| match event {
                WallpaperEvent::CloseRequested => {
                    capsule.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.clear_cache(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
                WallpaperEvent::WallpaperSelected(path) => {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().wallpaper.set_wallpaper(path);
                    }
                    capsule.wallpaper_view.update(cx, |wallpaper, cx| {
                        wallpaper.clear_cache(cx);
                    });
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;

                let state = this
                    .update(cx, |_, cx| {
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

                                capsule.volume_view.update(cx, |vol_mod, cx| {
                                    vol_mod.update_status(vol, muted, cx);
                                });

                                if capsule.mode == CapsuleMode::Default
                                    || capsule.mode == CapsuleMode::Volume
                                {
                                    capsule.start_transition_internal(
                                        CapsuleMode::Volume,
                                        None,
                                        cx,
                                    );
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
                    if latest_id != last_seen_notif_id {
                        last_seen_notif_id = latest_id;
                        if let Some(item) = latest {
                            let _ = capsule.notification_view.update(cx, |notif, cx| {
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

        let clipboard_view = cx.new(ClipboardModule::new);
        cx.observe(&clipboard_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &clipboard_view,
            |capsule, _, event: &ClipboardEvent, cx| match event {
                ClipboardEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        let emoji_view = cx.new(EmojiModule::new);
        cx.observe(&emoji_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        cx.subscribe(
            &emoji_view,
            |capsule, _, event: &EmojiEvent, cx| match event {
                EmojiEvent::Close => {
                    capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                }
            },
        )
        .detach();

        Self {
            mode: CapsuleMode::Default,
            idle_view,
            dashboard_view,
            notification_view,
            launcher_view,
            volume_view,
            polkit_view,
            select_theme_view,
            create_theme_view,
            wallpaper_view,
            clipboard_view,
            emoji_view,
            panel_manager: PanelManager::new(),
            current_width: initial_w,
            current_height: initial_h,
            current_radius: r,
            target_width: initial_w,
            target_height: initial_h,
            target_radius: r,
            anim_progress: 0.0,
            anim_start_time: None,
            anim_start_w: initial_w,
            anim_start_h: initial_h,
            anim_start_r: r,
            anim_start_progress: 0.0,
            animating: false,
            anim_task: None,
            last_activity_time: Instant::now(),
            inactivity_generation: 0,
            last_vol_status: None,
            volume_timer_gen: 0,
            dimension_tracker: DimensionTracker::new(),
            last_rendered_mode: None,
            is_mode_transition: false,
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
        let _ = self.dashboard_view.update(cx, |module, _cx| {
            module.open_panel_indices = open_indices;
        });
    }

    pub fn update_target_dimensions(
        &mut self,
        target_w: f32,
        target_h: f32,
        cx: &mut Context<Self>,
    ) {
        if self.mode == CapsuleMode::Default {
            self.target_width = target_w;
            self.target_height = target_h;
            if !self.animating {
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
                        let finished = capsule.tick_animation();
                        cx.notify();
                        finished
                    })
                    .unwrap_or(true);

                if done {
                    this.update(cx, |capsule, cx| {
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

    fn start_transition_internal(
        &mut self,
        mode: CapsuleMode,
        window_opt: Option<&mut Window>,
        cx: &mut Context<Self>,
    ) {
        if self.mode == mode && !self.animating {
            return;
        }

        let old_mode = self.mode;
        self.reset_inactivity_timer();
        self.mode = mode;
        services::log_info!("UI", "Transitioning to mode: {:?}", mode);

        if old_mode == CapsuleMode::Wallpaper && mode != CapsuleMode::Wallpaper {
            let _ = self.wallpaper_view.update(cx, |wallpaper, cx| {
                wallpaper.clear_cache(cx);
            });
        }

        if mode == CapsuleMode::Launcher {
            let _ = self.launcher_view.update(cx, |launcher, cx| {
                launcher.reset_search(cx);
            });
        } else if mode == CapsuleMode::SelectTheme {
            let _ = self.select_theme_view.update(cx, |st, cx| {
                st.refresh_themes(cx);
            });
        } else if mode == CapsuleMode::CreateTheme {
            let _ = self.create_theme_view.update(cx, |ct, cx| {
                ct.reset_form(cx);
            });
        }

        if mode != CapsuleMode::Default
            && mode != CapsuleMode::Dashboard
            && mode != CapsuleMode::Launcher
            && mode != CapsuleMode::Polkit
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

        let (mut target_w, mut target_h) = mode.dimensions();
        if mode == CapsuleMode::Default {
            let (w, h) = self.idle_view.read(cx).desired_dimensions();
            target_w = w;
            let _ = h;
        } else {
            let (w, h) = self.dimension_tracker.dimensions(0.0, 0.0);
            if w > 0.0 {
                target_w = w;
            }
            if h > 0.0 {
                target_h = h;
            }
        }
        let target_r = mode.radius();

        self.target_width = target_w;
        self.target_height = target_h;
        self.target_radius = target_r;

        self.anim_start_w = self.current_width;
        self.anim_start_h = self.current_height;
        self.anim_start_r = self.current_radius;
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
                        let finished = capsule.tick_animation();
                        cx.notify();
                        finished
                    })
                    .unwrap_or(true);

                if done {
                    this.update(cx, |capsule, cx| {
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
            services::IpcCommand::ToggleCreateTheme => {
                let target = if self.mode == CapsuleMode::CreateTheme {
                    CapsuleMode::Default
                } else {
                    CapsuleMode::CreateTheme
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
            _ => {}
        }
    }

    fn tick_animation(&mut self) -> bool {
        if !self.animating || self.anim_start_time.is_none() {
            self.animating = false;
            self.is_mode_transition = false;
            return true;
        }

        if let Some(start_time) = self.anim_start_time {
            let duration = 0.28;
            let t = (start_time.elapsed().as_secs_f32() / duration).min(1.0);
            let eased = apple_island_ease(t);

            let target_progress = if self.mode == CapsuleMode::Dashboard {
                1.0
            } else {
                0.0
            };

            self.current_width =
                self.anim_start_w + (self.target_width - self.anim_start_w) * eased;
            self.current_height =
                self.anim_start_h + (self.target_height - self.anim_start_h) * eased;
            self.current_radius =
                self.anim_start_r + (self.target_radius - self.anim_start_r) * eased;
            self.anim_progress =
                self.anim_start_progress + (target_progress - self.anim_start_progress) * eased;

            if t >= 1.0 {
                self.current_width = self.target_width;
                self.current_height = self.target_height;
                self.current_radius = self.target_radius;
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

        let win_w: f32 = window.bounds().size.width.into();
        let is_modal = self.mode == CapsuleMode::Launcher
            || self.mode == CapsuleMode::Dashboard
            || self.mode == CapsuleMode::Polkit
            || self.mode == CapsuleMode::SelectTheme
            || self.mode == CapsuleMode::CreateTheme
            || self.mode == CapsuleMode::Clipboard
            || self.mode == CapsuleMode::Emoji;

        if is_modal {
            window.set_input_region(None);
        } else {
            let pill_x = (win_w - self.current_width) / 2.0;
            let pill_y = MARGIN_TOP / 2.0;
            let pill_bounds = Bounds {
                origin: point(px(pill_x), px(pill_y)),
                size: Size::new(px(self.current_width), px(self.current_height)),
            };
            window.set_input_region(Some(&[pill_bounds]));
        }

        let mut content_container = div().relative().size_full();

        let anim_t = self
            .anim_start_time
            .map(|start| (start.elapsed().as_secs_f32() / 0.28).min(1.0))
            .unwrap_or(1.0);
        let eased = apple_island_ease(anim_t);

        if self.last_rendered_mode != Some(self.mode) {
            self.last_rendered_mode = Some(self.mode);
            if self.mode == CapsuleMode::Launcher
                || self.mode == CapsuleMode::Dashboard
                || self.mode == CapsuleMode::Polkit
                || self.mode == CapsuleMode::SelectTheme
                || self.mode == CapsuleMode::CreateTheme
                || self.mode == CapsuleMode::Wallpaper
                || self.mode == CapsuleMode::Clipboard
                || self.mode == CapsuleMode::Emoji
            {
                window.activate_window();
            }
            if self.mode == CapsuleMode::Launcher {
                let _ = self.launcher_view.update(cx, |launcher, cx| {
                    launcher.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Clipboard {
                let _ = self.clipboard_view.update(cx, |clip, cx| {
                    clip.reload_items(cx);
                    clip.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Emoji {
                let _ = self.emoji_view.update(cx, |emoji, cx| {
                    emoji.reload_items(cx);
                    emoji.focus(window, cx);
                });
            }
            if self.mode == CapsuleMode::Wallpaper {
                let _ = self.wallpaper_view.update(cx, |wallpaper, cx| {
                    wallpaper.reload_items(cx);
                });
            }
        }

        let mode_element = match self.mode {
            CapsuleMode::Launcher => Some(self.launcher_view.clone().into_any_element()),
            CapsuleMode::Polkit => Some(self.polkit_view.clone().into_any_element()),
            CapsuleMode::SelectTheme => Some(self.select_theme_view.clone().into_any_element()),
            CapsuleMode::CreateTheme => Some(self.create_theme_view.clone().into_any_element()),
            CapsuleMode::Wallpaper => Some(self.wallpaper_view.clone().into_any_element()),
            CapsuleMode::Clipboard => Some(self.clipboard_view.clone().into_any_element()),
            CapsuleMode::Emoji => Some(self.emoji_view.clone().into_any_element()),
            CapsuleMode::Volume => Some(self.volume_view.clone().into_any_element()),
            CapsuleMode::Notification => Some(self.notification_view.clone().into_any_element()),
            CapsuleMode::Dashboard => Some(self.dashboard_view.clone().into_any_element()),
            CapsuleMode::Default => None,
        };

        if let Some(el) = mode_element {
            let opacity = if self.animating && self.is_mode_transition {
                eased
            } else {
                1.0
            };
            let tracked_content = self.dimension_tracker.track(el);

            content_container = content_container.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(opacity)
                    .child(tracked_content),
            );
        } else {
            let default_opacity = (1.0 - self.anim_progress).clamp(0.0, 1.0);
            let hover_opacity = (self.anim_progress).clamp(0.0, 1.0);

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
                        .child(self.idle_view.clone().into_any_element()),
                );
            }

            if hover_opacity > 0.001 {
                let tracked_content = self
                    .dimension_tracker
                    .track(self.dashboard_view.clone().into_any_element());

                content_container = content_container.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(hover_opacity)
                        .child(tracked_content),
                );
            }
        }

        let is_dashboard = self.mode == CapsuleMode::Dashboard;
        let border_opacity = if is_dashboard {
            0.6
        } else {
            (0.12 + 0.4 * (1.0 - self.anim_progress)).clamp(0.12, 0.5)
        };
        let border_color = theme.surface().opacity(border_opacity);

        let _shadow_class = if is_dashboard {
            "shadow-2xl shadow-black/60"
        } else {
            "shadow-xl shadow-black/40"
        };

        let active_theme = cx.global::<Theme>().clone();

        let mut pill_container = div()
            .id("capsule-pill")
            .font_family(theme.font_family())
            .w(px(self.current_width))
            .h(px(self.current_height))
            .rounded(px(self.current_radius))
            .bg(active_theme.background())
            .border_1()
            .border_color(border_color)
            .shadow_lg()
            .overflow_hidden();

        if self.mode == CapsuleMode::Default {
            pill_container =
                pill_container
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start_transition_internal(CapsuleMode::Dashboard, None, cx);
                    }));
        }

        // Wrap the pill in a relative container so satellite panels can be
        // absolutely positioned outside it without affecting flex layout.
        let mut pill_wrapper = div()
            .relative()
            .w(px(self.current_width))
            .h(px(self.current_height))
            .child(pill_container.child(content_container));

        // Satellite mini-panels: compact chips that orbit the Dashboard in dynamic lanes.
        // Animated: they emerge from the pill center and slide into position.
        let has_panels =
            !self.panel_manager.left.is_empty() || !self.panel_manager.right.is_empty();
        if self.mode == CapsuleMode::Dashboard && has_panels && cx.has_global::<AppState>() {
            use super::satellites as PM;

            let dash_w = self.current_width;
            let dash_h = self.current_height;
            let sni_items = cx.global::<AppState>().sni_host.get_items();
            self.panel_manager.prune_invalid(sni_items.len());
            self.panel_manager.update_animations();

            if self.panel_manager.any_animating() {
                let frame_ms = cx.global::<AppState>().compositor.get_frame_duration_ms();
                let this = cx.entity().downgrade();
                cx.spawn(async move |_this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(frame_ms))
                        .await;
                    let _ = this.update(cx, |_, cx| cx.notify());
                })
                .detach();
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

            // Left lane
            let left_lane_x = -(PM::PANEL_W + PM::LANE_GAP);
            let left_snapshot: Vec<_> = self
                .panel_manager
                .left
                .iter()
                .map(|p| {
                    (
                        p.kind.clone(),
                        p.anim_t(),
                        p.height,
                        p.is_closing(),
                        p.tracker.clone(),
                    )
                })
                .collect();

            let mut y_stack = 0.0;
            for (kind, anim_t, panel_h, is_closing, tracker) in left_snapshot {
                let current_y = y_stack;
                if !is_closing {
                    y_stack += panel_h + PM::PANEL_GAP;
                }

                let mini_opt = match kind {
                    PM::PanelKind::Tray(sni_idx) => {
                        if let Some(item) = sni_items.get(sni_idx) {
                            Some(self.dashboard_view.update(cx, |_, cx| {
                                super::satellites::tray::render_mini_panel(
                                    item,
                                    sni_idx,
                                    anim_t,
                                    panel_h,
                                    &active_theme,
                                    cx,
                                )
                            }))
                        } else {
                            None
                        }
                    }
                    PM::PanelKind::Wifi => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::wifi::render_wifi_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Bluetooth => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::bluetooth::render_bluetooth_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Calendar => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::calendar::render_calendar_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Volume => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::volume::render_volume_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Power => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::power::render_power_widget(&active_theme, cx)
                            .into_any_element()
                    })),
                    PM::PanelKind::Language => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::language::render_language_widget(&active_theme, cx)
                            .into_any_element()
                    })),
                };

                if let Some(mini) = mini_opt {
                    let (off_x, off_y) = PM::PanelManager::animated_position(
                        dash_w,
                        dash_h,
                        left_lane_x,
                        current_y,
                        anim_t,
                        is_closing,
                    );

                    let tracked_mini = tracker.track(mini);
                    pill_wrapper = pill_wrapper.child(
                        div()
                            .absolute()
                            .left(px(off_x))
                            .top(px(off_y))
                            .child(tracked_mini),
                    );
                }
            }

            // Right lane
            let right_lane_x = dash_w + PM::LANE_GAP;
            let right_snapshot: Vec<_> = self
                .panel_manager
                .right
                .iter()
                .map(|p| {
                    (
                        p.kind.clone(),
                        p.anim_t(),
                        p.height,
                        p.is_closing(),
                        p.tracker.clone(),
                    )
                })
                .collect();

            let mut y_stack = 0.0;
            for (kind, anim_t, panel_h, is_closing, tracker) in right_snapshot {
                let current_y = y_stack;
                if !is_closing {
                    y_stack += panel_h + PM::PANEL_GAP;
                }

                let mini_opt = match kind {
                    PM::PanelKind::Tray(sni_idx) => {
                        if let Some(item) = sni_items.get(sni_idx) {
                            Some(self.dashboard_view.update(cx, |_, cx| {
                                super::satellites::tray::render_mini_panel(
                                    item,
                                    sni_idx,
                                    anim_t,
                                    panel_h,
                                    &active_theme,
                                    cx,
                                )
                            }))
                        } else {
                            None
                        }
                    }
                    PM::PanelKind::Wifi => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::wifi::render_wifi_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Bluetooth => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::bluetooth::render_bluetooth_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Calendar => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::calendar::render_calendar_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Volume => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::volume::render_volume_mini_panel(
                            anim_t,
                            panel_h,
                            &active_theme,
                            cx,
                        )
                    })),
                    PM::PanelKind::Power => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::power::render_power_widget(&active_theme, cx)
                            .into_any_element()
                    })),
                    PM::PanelKind::Language => Some(self.dashboard_view.update(cx, |_, cx| {
                        super::satellites::language::render_language_widget(&active_theme, cx)
                            .into_any_element()
                    })),
                };

                if let Some(mini) = mini_opt {
                    let (off_x, off_y) = PM::PanelManager::animated_position(
                        dash_w,
                        dash_h,
                        right_lane_x,
                        current_y,
                        anim_t,
                        is_closing,
                    );

                    let tracked_mini = tracker.track(mini);
                    pill_wrapper = pill_wrapper.child(
                        div()
                            .absolute()
                            .left(px(off_x))
                            .top(px(off_y))
                            .child(tracked_mini),
                    );
                }
            }
        }

        // flex_container always has ONE child (pill_wrapper), so justify_center
        // always positions the Dashboard at the center — never moves it.
        let flex_container = div()
            .flex()
            .flex_row()
            .items_start()
            .justify_center()
            .child(pill_wrapper);

        div()
            .size_full()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(MARGIN_TOP / 2.0))
            .child(flex_container)
    }
}
