use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ColorExt, Context, EventEmitter, FocusHandle,
    FontWeight, IntoElement, KeyDownEvent, Pixels, Render, Size, Task, Window, div, ease_in_out,
    px, svg,
};
use services::{AppState, RecordOptions, RecordStatus};
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub struct RecordModule {
    pub status: RecordStatus,
    pub seconds: u64,
    pub transition: Option<RecordTransition>,
    pub transition_epoch: usize,
    pub transition_finish: Option<Task<()>>,
    focus: FocusHandle,
    _status_task: Option<Task<()>>,
    _clock_task: Task<()>,
}

pub type RecordPanelState = RecordModule;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecordPresentation {
    pub status: RecordStatus,
    pub seconds: u64,
}

#[derive(Clone, Copy)]
pub struct RecordTransition {
    pub outgoing: RecordPresentation,
    pub epoch: usize,
    pub duration: Duration,
}

impl RecordModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (status, seconds) = if let Some(state) = cx.try_global::<AppState>() {
            (state.record.get_status(), state.record.get_duration_secs())
        } else {
            (RecordStatus::Stopped, 0)
        };

        let status_task = if cx.has_global::<AppState>() {
            let mut rx = cx.global::<AppState>().record.subscribe_status();
            Some(cx.spawn(async move |this, cx| {
                while let Ok(new_status) = rx.recv().await {
                    if this
                        .update(cx, |module, cx| {
                            module.handle_status_change(new_status, cx);
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        let clock_task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this
                    .update(cx, |module, cx| {
                        if let Some(state) = cx.try_global::<AppState>() {
                            module.seconds = state.record.get_duration_secs();
                            module.status = state.record.get_status();
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        Self {
            status,
            seconds,
            transition: None,
            transition_epoch: 0,
            transition_finish: None,
            focus: cx.focus_handle(),
            _status_task: status_task,
            _clock_task: clock_task,
        }
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            self.status = state.record.get_status();
            self.seconds = state.record.get_duration_secs();
            self.transition = None;
        }
        cx.notify();
    }

    pub fn handle_status_change(&mut self, new_status: RecordStatus, cx: &mut Context<Self>) {
        if self.status == new_status {
            return;
        }
        let previous = RecordPresentation {
            status: self.status,
            seconds: self.seconds,
        };
        self.status = new_status;

        if (previous.status == RecordStatus::Stopped) != (new_status == RecordStatus::Stopped) {
            let duration = Duration::from_millis(
                cx.try_global::<AppState>()
                    .map(|s| s.config.get().ui.animation_duration_ms as u64)
                    .unwrap_or(220),
            );
            if duration.is_zero() {
                self.transition = None;
                return;
            }
            self.transition_epoch = self.transition_epoch.wrapping_add(1);
            let epoch = self.transition_epoch;
            self.transition = Some(RecordTransition {
                outgoing: previous,
                epoch,
                duration,
            });
            self.transition_finish = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(duration).await;
                let _ = this.update(cx, |module, cx| {
                    if module.transition.as_ref().is_some_and(|t| t.epoch == epoch) {
                        module.transition = None;
                        cx.notify();
                    }
                });
            }));
        }
    }

    fn controls(
        &self,
        presentation: RecordPresentation,
        interactive: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.global::<Theme>().clone();
        let language = cx.try_global::<AppState>().map(|s| s.language.clone());
        let hover = style::hover(&theme);

        if presentation.status == RecordStatus::Stopped {
            let label = language
                .as_ref()
                .map(|l| l.get("record.record_screen"))
                .unwrap_or_else(|| "Grabar pantalla".to_string());

            div()
                .w_full()
                .h(px(36.0))
                .child(
                    div()
                        .id("record-panel-start")
                        .w_full()
                        .h_full()
                        .px(px(14.0))
                        .rounded_full()
                        .bg(style::surface(&theme))
                        .border_1()
                        .border_color(style::border(&theme).opacity(0.18))
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(9.0))
                        .when(interactive, |s| {
                            s.cursor_pointer()
                                .hover(move |s| s.bg(hover))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        this.handle_status_change(RecordStatus::Recording, cx);
                                        if let Some(state) = cx.try_global::<AppState>() {
                                            let record = state.record.clone();
                                            services::spawn_tokio(async move {
                                                let mut opts = RecordOptions::default();
                                                if let Some(backend) =
                                                services::record::RecordBackend::detect_preferred()
                                            {
                                                opts.backend = backend;
                                            }
                                                let _ = record.start(opts).await;
                                            });
                                        }
                                        cx.notify();
                                    }),
                                )
                        })
                        .child(
                            div()
                                .size(px(10.0))
                                .flex_shrink_0()
                                .rounded_full()
                                .bg(theme.red()),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(style::text(&theme))
                                .child(label),
                        ),
                )
                .into_any_element()
        } else {
            let paused = presentation.status == RecordStatus::Paused;
            let stop_hover = theme.red().opacity(0.24);
            let status_label = if paused {
                language
                    .as_ref()
                    .map(|l| l.get("record.paused"))
                    .unwrap_or_else(|| "Pausa".to_string())
            } else {
                language
                    .as_ref()
                    .map(|l| l.get("record.recording"))
                    .unwrap_or_else(|| "Grabando".to_string())
            };

            div()
                .w_full()
                .h(px(36.0))
                .px(px(6.0))
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .size(px(10.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(if paused {
                            theme.foreground_muted()
                        } else {
                            theme.red()
                        }),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(style::text(&theme))
                        .flex_shrink_0()
                        .child(format_duration(presentation.seconds)),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(status_label),
                )
                .child(
                    div()
                        .id("record-panel-pause")
                        .size(px(30.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(style::surface(&theme))
                        .border_1()
                        .border_color(style::border(&theme).opacity(0.18))
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(interactive, |s| {
                            s.cursor_pointer()
                                .hover(move |s| s.bg(hover))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(move |this, _, _window, cx| {
                                        let next_status = if paused {
                                            RecordStatus::Recording
                                        } else {
                                            RecordStatus::Paused
                                        };
                                        this.handle_status_change(next_status, cx);
                                        if let Some(state) = cx.try_global::<AppState>() {
                                            let record = state.record.clone();
                                            services::spawn_tokio(async move {
                                                let _ = record.toggle_pause().await;
                                            });
                                        }
                                        cx.notify();
                                    }),
                                )
                        })
                        .child(
                            svg()
                                .path(if paused { "play.svg" } else { "pause.svg" })
                                .size(px(13.0))
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .id("record-panel-stop")
                        .size(px(30.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(theme.red().opacity(0.14))
                        .border_1()
                        .border_color(theme.red().opacity(0.3))
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(interactive, |s| {
                            s.cursor_pointer()
                                .hover(move |s| s.bg(stop_hover))
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        this.handle_status_change(RecordStatus::Stopped, cx);
                                        if let Some(state) = cx.try_global::<AppState>() {
                                            let record = state.record.clone();
                                            services::spawn_tokio(async move {
                                                let _ = record.stop().await;
                                            });
                                        }
                                        cx.emit(SharedPanelEvent::Close);
                                        cx.notify();
                                    }),
                                )
                        })
                        .child(
                            svg()
                                .path("square.svg")
                                .size(px(12.0))
                                .text_color(theme.red()),
                        ),
                )
                .into_any_element()
        }
    }
}

pub fn format_duration(seconds: u64) -> String {
    if seconds >= 3600 {
        format!(
            "{:02}:{:02}:{:02}",
            seconds / 3600,
            seconds % 3600 / 60,
            seconds % 60
        )
    } else {
        format!("{:02}:{:02}", seconds / 60, seconds % 60)
    }
}

impl EventEmitter<SharedPanelEvent> for RecordModule {}

impl SharedPanelItem for RecordModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        Size {
            width: px(280.0),
            height: px(68.0),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

impl Render for RecordModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let presentation = RecordPresentation {
            status: self.status,
            seconds: self.seconds,
        };

        let bar = if let Some(transition) = self.transition {
            let animation = Animation::new(transition.duration).with_easing(ease_in_out);
            let outgoing = self.controls(transition.outgoing, false, cx);
            let incoming = self.controls(presentation, true, cx);
            div()
                .relative()
                .w_full()
                .h(px(36.0))
                .flex_shrink_0()
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .with_animation(
                            ("record-outgoing", transition.epoch),
                            animation.clone(),
                            |element, progress| {
                                element.opacity(1.0 - progress).blur(px(progress * 5.0))
                            },
                        )
                        .child(outgoing),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .with_animation(
                            ("record-incoming", transition.epoch),
                            animation,
                            |element, progress| {
                                element.opacity(progress).blur(px((1.0 - progress) * 5.0))
                            },
                        )
                        .child(incoming),
                )
                .into_any_element()
        } else {
            self.controls(presentation, true, cx)
        };

        div()
            .id("shared-panel-record")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SharedPanelEvent::Close);
                }
            }))
            .size_full()
            .px(px(12.0))
            .flex()
            .items_center()
            .justify_center()
            .font_family(theme.font_family())
            .child(bar)
    }
}
