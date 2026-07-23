use gpui::{Bounds, Context, Entity, Render, Size, Task, Window, div, point, prelude::*, px};
use services::{AppState, NotificationStore};
use std::time::{Duration, Instant};
use ui::theme::Theme;

use super::modules::idle::IdleModule;
use super::modules::idle_hover::IdleHoverModule;
use super::modules::launcher::LauncherModule;
use super::modules::notification::NotificationModule;
use super::modules::polkit::PolkitModule;
use super::modules::volume::VolumeModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleMode {
    #[default]
    Default,
    Dashboard,
    Notification,
    Launcher,
    Volume,
    Polkit,
}

impl CapsuleMode {
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            CapsuleMode::Default => (138.0, 42.0),
            CapsuleMode::Dashboard => (348.0, 500.0),
            CapsuleMode::Notification => (348.0, 68.0),
            CapsuleMode::Launcher => (348.0, 480.0),
            CapsuleMode::Volume => (280.0, 48.0),
            CapsuleMode::Polkit => (348.0, 240.0),
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            CapsuleMode::Default => 21.0,
            CapsuleMode::Dashboard => 28.0,
            CapsuleMode::Notification => 22.0,
            CapsuleMode::Launcher => 28.0,
            CapsuleMode::Volume => 22.0,
            CapsuleMode::Polkit => 28.0,
        }
    }
}

const MARGIN_TOP: f32 = 8.0;

pub struct Capsule {
    mode: CapsuleMode,
    idle_view: Entity<IdleModule>,
    idle_hover_view: Entity<IdleHoverModule>,
    notification_view: Entity<NotificationModule>,
    launcher_view: Entity<LauncherModule>,
    volume_view: Entity<VolumeModule>,
    polkit_view: Entity<PolkitModule>,
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
    last_rendered_mode: Option<CapsuleMode>,
}

fn apple_island_ease(t: f32) -> f32 {
    if t >= 1.0 {
        return 1.0;
    }
    let p = 1.0 - t;
    1.0 - p * p * p
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

        let idle_hover_view = cx.new(IdleHoverModule::new);
        cx.observe(&idle_hover_view, |_, _, cx| {
            cx.notify();
        })
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

        // === HEARTBEAT ===
        // GPUI on Wayland only processes foreground tasks when Hyprland sends
        // frame callbacks. Hyprland stops sending them when the layer-shell
        // surface is idle. Calling cx.notify() every 100 ms requests a new
        // frame callback continuously, keeping the foreground executor pumping
        // so IPC, MPRIS, and all other cx.spawn loops never freeze during
        // periods of user inactivity.
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
                                    "Reloaded current_theme and applied to GTK/Qt apps!"
                                );
                            }
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

        // Start Polkit Agent listener loop for system authentication requests
        if let Ok(mut polkit_rx) = services::start_polkit_agent() {
            cx.spawn(async move |this, cx| {
                while let Some((req, responder)) = polkit_rx.recv().await {
                    let _ = this.update(cx, |capsule: &mut Self, cx| {
                        if capsule.mode != CapsuleMode::Dashboard {
                            let _ = capsule.polkit_view.update(cx, |p, cx| {
                                p.set_request(req, responder, cx);
                            });
                            capsule.start_transition_internal(CapsuleMode::Polkit, None, cx);
                        }
                    });
                }
            })
            .detach();
        }

        let volume_view = cx.new(VolumeModule::new);
        cx.observe(&volume_view, |capsule, _, cx| {
            capsule.reset_inactivity_timer();
            cx.notify();
        })
        .detach();

        // Monitor volume changes for dedicated Volume OSD mode transition
        let sys_service_clone = cx.global::<AppState>().system.clone();

        cx.spawn(async move |this, cx| {
            loop {
                let status = sys_service_clone.get_status();
                let current_status = (status.volume, status.is_muted);

                let res = this.update(cx, |capsule: &mut Self, cx| {
                    if let Some(last) = capsule.last_vol_status {
                        if last != current_status {
                            let (vol, muted) = current_status;
                            let _ = capsule.volume_view.update(cx, |v, cx| {
                                v.update_status(vol, muted, cx);
                            });

                            capsule.volume_timer_gen += 1;
                            let current_gen = capsule.volume_timer_gen;

                            if capsule.mode != CapsuleMode::Dashboard
                                && capsule.mode != CapsuleMode::Launcher
                                && capsule.mode != CapsuleMode::Polkit
                            {
                                capsule.start_transition_internal(CapsuleMode::Volume, None, cx);

                                cx.spawn(async move |weak, cx| {
                                    cx.background_executor()
                                        .timer(Duration::from_millis(2000))
                                        .await;

                                    let _ = weak.update(cx, |capsule: &mut Self, cx| {
                                        if capsule.mode == CapsuleMode::Volume
                                            && capsule.volume_timer_gen == current_gen
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
                        }
                    }
                    capsule.last_vol_status = Some(current_status);
                });

                if res.is_err() {
                    break;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
            }
        })
        .detach();

        // Monitor D-Bus active notification status for mode transition
        cx.spawn(async move |this, cx| {
            loop {
                let store = NotificationStore::global();
                let latest = store.get_latest_active_notification();
                let has_notif = latest.is_some();

                let res = this.update(cx, |capsule: &mut Self, cx| {
                    if capsule.mode != CapsuleMode::Dashboard
                        && capsule.mode != CapsuleMode::Launcher
                        && capsule.mode != CapsuleMode::Volume
                        && capsule.mode != CapsuleMode::Polkit
                    {
                        if has_notif && capsule.mode != CapsuleMode::Notification {
                            capsule.start_transition_internal(CapsuleMode::Notification, None, cx);
                        } else if !has_notif && capsule.mode == CapsuleMode::Notification {
                            capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                        }
                    }
                });
                if res.is_err() {
                    break;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(150))
                    .await;
            }
        })
        .detach();

        Self {
            mode: CapsuleMode::Default,
            idle_view,
            idle_hover_view,
            notification_view,
            launcher_view,
            volume_view,
            polkit_view,
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
            last_rendered_mode: None,
        }
    }

    fn update_target_dimensions(&mut self, desired_w: f32, desired_h: f32, cx: &mut Context<Self>) {
        if self.mode == CapsuleMode::Default {
            if (self.current_width - desired_w).abs() > 0.5
                || (self.current_height - desired_h).abs() > 0.5
            {
                self.target_width = desired_w;
                self.target_height = desired_h;
                if !self.animating {
                    self.current_width = desired_w;
                    self.current_height = desired_h;
                    cx.notify();
                }
            }
        }
    }

    pub fn reset_inactivity_timer(&mut self) {
        self.last_activity_time = Instant::now();
    }

    pub fn start_transition(
        &mut self,
        mode: CapsuleMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_transition_internal(mode, Some(window), cx);
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

        self.reset_inactivity_timer();
        self.mode = mode;
        services::log_info!("UI", "Transitioning to mode: {:?}", mode);

        if mode == CapsuleMode::Launcher {
            let _ = self.launcher_view.update(cx, |launcher, cx| {
                launcher.reset_search(cx);
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

        let (mut target_w, target_h) = mode.dimensions();
        if mode == CapsuleMode::Default {
            let (w, h) = self.idle_view.read(cx).desired_dimensions();
            target_w = w;
            let _ = h;
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
            services::IpcCommand::ShowLauncher => {
                self.start_transition_internal(CapsuleMode::Launcher, None, cx);
            }
            services::IpcCommand::ShowDashboard => {
                self.start_transition_internal(CapsuleMode::Dashboard, None, cx);
            }
            services::IpcCommand::ShowNotification => {
                self.start_transition_internal(CapsuleMode::Notification, None, cx);
            }
            services::IpcCommand::Hide | services::IpcCommand::Default => {
                self.start_transition_internal(CapsuleMode::Default, None, cx);
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
            return true;
        }

        if let Some(start_time) = self.anim_start_time {
            let duration = 0.24;
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
        let is_modal = self.mode == CapsuleMode::Launcher || self.mode == CapsuleMode::Dashboard;

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
            .map(|start| (start.elapsed().as_secs_f32() / 0.24).min(1.0))
            .unwrap_or(1.0);
        let eased = apple_island_ease(anim_t);

        if self.last_rendered_mode != Some(self.mode) {
            self.last_rendered_mode = Some(self.mode);
            if self.mode == CapsuleMode::Launcher || self.mode == CapsuleMode::Dashboard {
                window.activate_window();
            }
            if self.mode == CapsuleMode::Launcher {
                let _ = self.launcher_view.update(cx, |launcher, cx| {
                    launcher.focus(window, cx);
                });
            }
        }

        if self.mode == CapsuleMode::Launcher {
            let launcher_opacity = if self.animating { eased } else { 1.0 };

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
                    .opacity(launcher_opacity)
                    .child(self.launcher_view.clone().into_any_element()),
            );
        } else if self.mode == CapsuleMode::Polkit {
            let polkit_opacity = if self.animating { eased } else { 1.0 };

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
                    .opacity(polkit_opacity)
                    .child(self.polkit_view.clone().into_any_element()),
            );
        } else if self.mode == CapsuleMode::Volume {
            let volume_opacity = if self.animating { eased } else { 1.0 };

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
                    .opacity(volume_opacity)
                    .child(self.volume_view.clone().into_any_element()),
            );
        } else if self.mode == CapsuleMode::Notification {
            let notif_opacity = if self.animating { eased } else { 1.0 };

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
                    .opacity(notif_opacity)
                    .child(self.notification_view.clone().into_any_element()),
            );
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
                        .child(self.idle_view.clone().into_any_element()),
                );
            }

            if self.anim_progress > 0.001 {
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
                        .opacity(self.anim_progress)
                        .child(self.idle_hover_view.clone().into_any_element()),
                );
            }
        }

        let is_modal = self.mode == CapsuleMode::Launcher || self.mode == CapsuleMode::Dashboard;

        let mut root = div()
            .id("capsule-backdrop")
            .flex()
            .items_start()
            .justify_center()
            .size_full()
            .pt(px(MARGIN_TOP / 2.0));

        if is_modal {
            root = root.on_click(cx.listener(|capsule, _event, _window, cx| {
                capsule.start_transition_internal(CapsuleMode::Default, None, cx);
            }));
        }

        root.child(
            div()
                .id("capsule-pill")
                .on_click(|_event, _window, _cx| {})
                .flex()
                .flex_col()
                .overflow_hidden()
                .w(px(self.current_width))
                .h(px(self.current_height))
                .bg(theme.background())
                .border_1()
                .border_color(theme.background_alt())
                .rounded(px(self.current_radius))
                .shadow_lg()
                .on_click(cx.listener(|capsule, _event, window, cx| {
                    if capsule.mode == CapsuleMode::Default {
                        capsule.start_transition(CapsuleMode::Dashboard, window, cx);
                    }
                }))
                .child(content_container),
        )
    }
}
