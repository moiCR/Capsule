use chrono::{Local, Timelike};
use gpui::{Context, EventEmitter, IntoElement, Render, Task, Window, div, prelude::*, px};
use services::{
    AppState, MediaTrack, MprisService, NetworkService, NotificationStore, PowerService,
    SystemService, WorkspaceInfo,
};
use std::time::Duration;
use ui::theme::Theme;

use crate::capsule::widgets::default::{
    FlipClock, render_clock_widget, render_media_dock, render_privacy_dock, render_status_dock,
    render_workspaces_widget,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DefaultEvent {
    WorkspaceClicked(i64),
    StatusDockClicked,
}

pub struct DefaultModule {
    time_str: String,
    flip_clock: FlipClock,
    flip_anim_task: Option<Task<()>>,
    active_workspace: WorkspaceInfo,
    special_anim_progress: f32,
    special_anim_task: Option<Task<()>>,
    network: NetworkService,
    mpris: MprisService,
    power: PowerService,
    system: SystemService,
}

impl DefaultModule {
    const OUTER_PADDING: f32 = 12.0;
    const SIDE_WIDTH: f32 = 152.0;
    const ZONE_GAP: f32 = 32.0;
    const CLOCK_WIDTH: f32 = 36.0;

    pub fn new(cx: &mut Context<Self>) -> Self {
        let compositor = cx.global::<AppState>().compositor.clone();
        let network = cx.global::<AppState>().network.clone();
        let mpris = cx.global::<AppState>().mpris.clone();
        let power = cx.global::<AppState>().power.clone();
        let system = cx.global::<AppState>().system.clone();

        let initial_ws = compositor.get_workspace();
        let special_anim_progress = if initial_ws.is_special { 1.0 } else { 0.0 };

        let now = Local::now();
        let time_str = format!("{:02}:{:02}", now.hour(), now.minute());
        let flip_clock = FlipClock::new(&time_str);

        cx.spawn(async move |this, cx| {
            let mut last_track: Option<MediaTrack> = None;
            let mut last_battery = None;
            let mut last_capture = None;
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;

                let now = Local::now();
                let time_str = format!("{:02}:{:02}", now.hour(), now.minute());

                let res = this.update(cx, |this: &mut Self, cx| {
                    let mut changed = false;
                    if this.time_str != time_str {
                        this.time_str = time_str.clone();
                        let started = this.flip_clock.update_time(&time_str);
                        if started {
                            this.start_flip_clock_anim(cx);
                        }
                        changed = true;
                    }

                    let cur_track = this.mpris.get_current_track();
                    if last_track.as_ref() != Some(&cur_track) {
                        last_track = Some(cur_track);
                        changed = true;
                    }

                    let cur_battery = this.power.get_battery();
                    if cur_battery != last_battery {
                        last_battery = cur_battery;
                        changed = true;
                    }

                    let capture = this.system.get_capture_status();
                    if last_capture != Some(capture) {
                        last_capture = Some(capture);
                        changed = true;
                    }

                    if changed {
                        cx.notify();
                    }
                });

                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            time_str,
            flip_clock,
            flip_anim_task: None,
            active_workspace: initial_ws,
            special_anim_progress,
            special_anim_task: None,
            network,
            mpris,
            power,
            system,
        }
    }

    fn start_flip_clock_anim(&mut self, cx: &mut Context<Self>) {
        if self.flip_anim_task.is_some() {
            return;
        }
        let compositor = cx.global::<AppState>().compositor.clone();
        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 350.0;
            loop {
                cx.background_executor()
                    .timer(compositor.get_frame_duration())
                    .await;
                let all_done = this
                    .update(cx, |this: &mut Self, cx| {
                        let done = this.flip_clock.tick(duration_ms);
                        cx.notify();
                        done
                    })
                    .unwrap_or(true);

                if all_done {
                    let _ = this.update(cx, |this: &mut Self, _| {
                        this.flip_anim_task = None;
                    });
                    break;
                }
            }
        });
        self.flip_anim_task = Some(anim_task);
    }

    fn start_special_anim(&mut self, to_special: bool, cx: &mut Context<Self>) {
        let frame_duration = if cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.get_frame_duration()
        } else {
            Duration::from_millis(16)
        };
        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 180.0;
            let dt_ms = (frame_duration.as_secs_f64() * 1000.0).max(1.0) as f32;
            let step = dt_ms / duration_ms;
            loop {
                cx.background_executor().timer(frame_duration).await;
                let done = this
                    .update(cx, |this: &mut Self, cx| {
                        if to_special {
                            this.special_anim_progress =
                                (this.special_anim_progress + step).min(1.0);
                            let done = this.special_anim_progress >= 1.0;
                            cx.notify();
                            done
                        } else {
                            this.special_anim_progress =
                                (this.special_anim_progress - step).max(0.0);
                            let done = this.special_anim_progress <= 0.0;
                            cx.notify();
                            done
                        }
                    })
                    .unwrap_or(true);

                if done {
                    let _ = this.update(cx, |this: &mut Self, _| {
                        this.special_anim_task = None;
                    });
                    break;
                }
            }
        });
        self.special_anim_task = Some(anim_task);
    }

    #[allow(dead_code)]
    pub fn set_active_module(&mut self, _idx: usize) {}
    #[allow(dead_code)]
    pub fn set_open_panel_indices(&mut self, _indices: Vec<usize>) {}

    pub fn set_active_workspace(&mut self, ws: WorkspaceInfo, cx: &mut Context<Self>) {
        let was_special = self.active_workspace.is_special;
        let is_special = ws.is_special;
        if self.active_workspace != ws {
            self.active_workspace = ws;
            if was_special != is_special {
                self.start_special_anim(is_special, cx);
            }
            cx.notify();
        }
    }

    pub fn desired_dimensions(&self) -> (f32, f32) {
        (
            Self::OUTER_PADDING * 2.0
                + Self::SIDE_WIDTH * 2.0
                + Self::ZONE_GAP * 2.0
                + Self::CLOCK_WIDTH,
            42.0,
        )
    }
}

impl EventEmitter<DefaultEvent> for DefaultModule {}

impl Render for DefaultModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Local::now();
        let current_time_str = format!("{:02}:{:02}", now.hour(), now.minute());
        if self.time_str != current_time_str {
            self.time_str = current_time_str.clone();
            self.flip_clock = FlipClock::new(&current_time_str);
        }

        let theme = cx.global::<Theme>().clone();
        let active_ws = if cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.get_workspace()
        } else {
            self.active_workspace.clone()
        };

        if self.active_workspace != active_ws {
            let was_special = self.active_workspace.is_special;
            let is_special = active_ws.is_special;
            self.active_workspace = active_ws.clone();
            if was_special != is_special {
                self.start_special_anim(is_special, cx);
            }
        }

        let net_status = self.network.get_status();
        let is_dnd = NotificationStore::global().is_dnd_enabled();
        let notif_count = NotificationStore::global().get_all_notifications().len();
        let battery = self.power.get_battery();
        let track = self.mpris.get_current_track();
        let capture = self.system.get_capture_status();

        let left_container = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .child(render_workspaces_widget(
                &active_ws,
                self.special_anim_progress,
                &theme,
                cx,
            ))
            .children(render_media_dock(&track, &theme, cx));

        let center_container = div()
            .w(px(Self::CLOCK_WIDTH))
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .child(render_clock_widget(&self.flip_clock, &theme));

        let right_container = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap(px(6.0))
            .flex_1()
            .min_w_0()
            .children(render_privacy_dock(capture, &theme))
            .child(render_status_dock(
                &net_status,
                battery,
                is_dnd,
                notif_count,
                &theme,
                cx,
            ));

        div()
            .id("default-module")
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h_full()
            .px(px(Self::OUTER_PADDING))
            .gap(px(Self::ZONE_GAP))
            .child(left_container)
            .child(center_container)
            .child(right_container)
    }
}
