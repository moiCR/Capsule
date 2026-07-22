use gpui::{div, prelude::*, px, Context, Entity, Render, Size, Task, Window};
use services::NotificationStore;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use super::modules::idle::IdleModule;
use super::modules::idle_hover::IdleHoverModule;
use super::modules::notification::NotificationModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CapsuleMode {
    #[default]
    Default,
    Dashboard,
    Notification,
}

const MAX_WINDOW_WIDTH: f32 = 1200.0;

impl CapsuleMode {
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            CapsuleMode::Default => (138.0, 42.0),
            CapsuleMode::Dashboard => (348.0, 500.0),
            CapsuleMode::Notification => (348.0, 68.0),
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            CapsuleMode::Default => 21.0,
            CapsuleMode::Dashboard => 28.0,
            CapsuleMode::Notification => 22.0,
        }
    }
}

const MARGIN_TOP: f32 = 8.0;

pub struct Capsule {
    mode: CapsuleMode,
    idle_view: Entity<IdleModule>,
    idle_hover_view: Entity<IdleHoverModule>,
    notification_view: Entity<NotificationModule>,
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
    hovered: bool,
    hover_revert_generation: u64,
    anim_task: Option<Task<()>>,
    needs_window_shrink: bool,
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
        cx.observe(&notification_view, |_, _, cx| {
            cx.notify();
        })
        .detach();

        // Monitor D-Bus active notification status for mode transition
        cx.spawn(async move |this, cx| {
            loop {
                let store = NotificationStore::global();
                let latest = store.get_latest_active_notification();
                let has_notif = latest.is_some();

                let res = this.update(cx, |capsule: &mut Self, cx| {
                    if capsule.mode != CapsuleMode::Dashboard {
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
            hovered: false,
            hover_revert_generation: 0,
            anim_task: None,
            needs_window_shrink: false,
        }
    }

    fn update_target_dimensions(&mut self, desired_w: f32, desired_h: f32, cx: &mut Context<Self>) {
        if self.mode == CapsuleMode::Default {
            if (self.target_width - desired_w).abs() > 1.0 || (self.target_height - desired_h).abs() > 1.0 {
                self.target_width = desired_w;
                self.target_height = desired_h;
                self.anim_start_w = self.current_width;
                self.anim_start_h = self.current_height;
                self.anim_start_r = self.current_radius;
                self.anim_start_progress = self.anim_progress;
                self.animating = true;
                self.anim_start_time = Some(Instant::now());

                if self.anim_task.is_none() {
                    let task = cx.spawn(async move |this, cx| {
                        loop {
                            cx.background_executor().timer(Duration::from_millis(16)).await;
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
            }
        }
    }

    fn start_transition_internal(&mut self, mode: CapsuleMode, window_opt: Option<&mut Window>, cx: &mut Context<Self>) {
        if self.mode == mode && !self.animating {
            return;
        }

        self.mode = mode;

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

        if let Some(window) = window_opt {
            let required_h = target_h.max(self.current_height);
            window.resize(Size::new(px(MAX_WINDOW_WIDTH), px(required_h + MARGIN_TOP)));
        }

        self.animating = true;
        self.anim_start_time = Some(Instant::now());

        let task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_millis(16)).await;
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

    fn tick_animation(&mut self) -> bool {
        if !self.animating {
            return true;
        }

        if let Some(start_time) = self.anim_start_time {
            let duration = 0.24; // 240ms fluid transition
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

                if self.mode == CapsuleMode::Default || self.mode == CapsuleMode::Notification {
                    self.needs_window_shrink = true;
                }
                return true;
            }
        }
        false
    }
}

impl Render for Capsule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        if self.mode == CapsuleMode::Notification || self.mode == CapsuleMode::Dashboard {
            let target_h = self.mode.dimensions().1;
            if self.current_height < target_h {
                window.resize(Size::new(px(MAX_WINDOW_WIDTH), px(target_h + MARGIN_TOP)));
            }
        }

        if self.needs_window_shrink {
            let target_h = match self.mode {
                CapsuleMode::Notification => CapsuleMode::Notification.dimensions().1,
                _ => self.current_height,
            };
            window.resize(Size::new(px(MAX_WINDOW_WIDTH), px(target_h + MARGIN_TOP)));
            self.needs_window_shrink = false;
        }

        let mut content_container = div().relative().size_full();

        let anim_t = self
            .anim_start_time
            .map(|start| (start.elapsed().as_secs_f32() / 0.24).min(1.0))
            .unwrap_or(1.0);
        let eased = apple_island_ease(anim_t);

        if self.mode == CapsuleMode::Notification {
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

            if self.animating && (1.0 - eased) > 0.001 {
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
                        .opacity(1.0 - eased)
                        .child(self.idle_view.clone().into_any_element()),
                );
            }
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

            let dash_opacity = self.anim_progress.clamp(0.0, 1.0);
            if dash_opacity > 0.001 {
                content_container = content_container.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .opacity(dash_opacity)
                        .child(self.idle_hover_view.clone().into_any_element()),
                );
            }
        }

        let dash_h = CapsuleMode::Dashboard.dimensions().1;

        div()
            .flex()
            .items_start()
            .justify_center()
            .w(px(MAX_WINDOW_WIDTH))
            .h(px(dash_h + MARGIN_TOP))
            .pt(px(MARGIN_TOP / 2.0))
            .child(
                div()
                    .id("capsule-pill")
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
                    .on_hover(cx.listener(|capsule, &hovered, window, cx| {
                        if capsule.hovered != hovered {
                            capsule.hovered = hovered;
                            capsule.hover_revert_generation += 1;

                            if hovered {
                                let dash_h = CapsuleMode::Dashboard.dimensions().1;
                                window.resize(Size::new(px(MAX_WINDOW_WIDTH), px(dash_h + MARGIN_TOP)));
                                capsule.needs_window_shrink = false;
                                capsule.start_transition_internal(CapsuleMode::Dashboard, Some(window), cx);
                            } else {
                                let current_gen = capsule.hover_revert_generation;
                                cx.spawn(async move |weak, cx| {
                                    cx.background_executor().timer(Duration::from_millis(300)).await;
                                    let _ = weak.update(cx, |capsule: &mut Self, cx| {
                                        if capsule.hover_revert_generation == current_gen
                                            && !capsule.hovered
                                        {
                                            capsule.start_transition_internal(CapsuleMode::Default, None, cx);
                                        }
                                    });
                                })
                                .detach();
                            }
                        }
                    }))
                    .child(content_container),
            )
    }
}
