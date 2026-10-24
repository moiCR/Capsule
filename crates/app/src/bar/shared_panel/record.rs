use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, AnyElement, ColorExt, Context, FontWeight, IntoElement, Task, Window,
    div, ease_in_out, px, svg,
};
use services::{AppState, RecordOptions, RecordStatus};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub struct RecordPanelState {
    pub status: RecordStatus,
    pub seconds: u64,
    pub transition: Option<RecordTransition>,
    pub transition_epoch: usize,
    pub transition_finish: Option<Task<()>>,
}

impl Default for RecordPanelState {
    fn default() -> Self {
        Self {
            status: RecordStatus::Stopped,
            seconds: 0,
            transition: None,
            transition_epoch: 0,
            transition_finish: None,
        }
    }
}

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

impl RecordPanelState {
    pub fn handle_status_change(&mut self, new_status: RecordStatus, cx: &mut Context<Bar>) {
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
                let _ = this.update(cx, |bar, cx| {
                    if bar
                        .shared_panel
                        .record_state
                        .transition
                        .as_ref()
                        .is_some_and(|t| t.epoch == epoch)
                    {
                        bar.shared_panel.record_state.transition = None;
                        cx.notify();
                    }
                });
            }));
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

fn controls(
    presentation: RecordPresentation,
    interactive: bool,
    cx: &mut Context<Bar>,
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
                                cx.listener(|bar, _, _window, cx| {
                                    bar.shared_panel
                                        .record_state
                                        .handle_status_change(RecordStatus::Recording, cx);
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
                                cx.listener(move |bar, _, _window, cx| {
                                    let next_status = if paused {
                                        RecordStatus::Recording
                                    } else {
                                        RecordStatus::Paused
                                    };
                                    bar.shared_panel
                                        .record_state
                                        .handle_status_change(next_status, cx);
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
                                cx.listener(|bar, _, window, cx| {
                                    bar.shared_panel
                                        .record_state
                                        .handle_status_change(RecordStatus::Stopped, cx);
                                    if let Some(state) = cx.try_global::<AppState>() {
                                        let record = state.record.clone();
                                        services::spawn_tokio(async move {
                                            let _ = record.stop().await;
                                        });
                                    }
                                    bar.close_shared_panel(window, cx);
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

pub fn render(state: &RecordPanelState, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let presentation = RecordPresentation {
        status: state.status,
        seconds: state.seconds,
    };

    let bar = if let Some(transition) = &state.transition {
        let animation = Animation::new(transition.duration).with_easing(ease_in_out);
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
                    .child(controls(transition.outgoing, false, cx)),
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
                    .child(controls(presentation, true, cx)),
            )
            .into_any_element()
    } else {
        controls(presentation, true, cx)
    };

    div()
        .id("shared-panel-record")
        .size_full()
        .px(px(12.0))
        .flex()
        .items_center()
        .justify_center()
        .font_family(theme.font_family())
        .child(bar)
        .into_any_element()
}
