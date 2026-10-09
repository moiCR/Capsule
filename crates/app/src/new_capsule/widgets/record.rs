use crate::new_capsule::module::record::{PendingAction, Presentation, RecordModule};
use crate::new_capsule::widgets::style;
use gpui::{
    Animation, AnimationExt, Context, FontWeight, IntoElement, div, ease_in_out, prelude::*, px,
    svg,
};
use services::{AppState, RecordStatus};
use ui::theme::Theme;

pub const WIDTH: f32 = 280.0;
pub const MAX_HEIGHT: f32 = 88.0;
pub fn height(error: bool) -> f32 {
    if error { MAX_HEIGHT } else { 48.0 }
}
pub fn duration(seconds: u64) -> String {
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
    presentation: Presentation,
    interactive: bool,
    cx: &mut Context<RecordModule>,
) -> gpui::Div {
    let theme = cx.global::<Theme>().clone();
    let language = &cx.global::<AppState>().language;
    let label = language.get(match presentation.pending {
        Some(PendingAction::Start) => "record.starting",
        Some(PendingAction::Stop) => "record.stopping",
        _ if presentation.status == RecordStatus::Paused => "record.paused",
        _ => "record.record_screen",
    });
    let hover = style::hover(&theme);
    let paused = presentation.status == RecordStatus::Paused;
    let mut bar = div()
        .w_full()
        .h(px(32.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(10.0));
    if presentation.status == RecordStatus::Stopped {
        bar = bar.child(
            div()
                .id("record-start")
                .flex_1()
                .min_w_0()
                .h_full()
                .px(px(8.0))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(9.0))
                .when(interactive && presentation.pending.is_none(), |s| {
                    s.cursor_pointer()
                        .on_click(cx.listener(|module, _, _, cx| module.primary_action(cx)))
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
                        .text_ellipsis()
                        .child(label),
                ),
        );
    } else {
        bar = bar
            .px(px(4.0))
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
                    .font_weight(FontWeight::MEDIUM)
                    .flex_shrink_0()
                    .child(duration(presentation.seconds)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(11.0))
                    .text_color(theme.foreground_muted())
                    .text_ellipsis()
                    .when(
                        paused || presentation.pending == Some(PendingAction::Stop),
                        |s| s.child(label),
                    ),
            )
            .child(
                div()
                    .id("record-pause")
                    .size(px(28.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(style::surface(&theme))
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(if presentation.pending.is_some() {
                        0.4
                    } else {
                        1.0
                    })
                    .when(interactive && presentation.pending.is_none(), |s| {
                        s.cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(|module, _, _, cx| module.primary_action(cx)))
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
                    .id("record-stop")
                    .size(px(28.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(style::surface(&theme))
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(if presentation.pending.is_some() {
                        0.4
                    } else {
                        1.0
                    })
                    .when(interactive && presentation.pending.is_none(), |s| {
                        s.cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(|module, _, _, cx| module.stop(cx)))
                    })
                    .child(
                        svg()
                            .path("square.svg")
                            .size(px(13.0))
                            .text_color(theme.red()),
                    ),
            );
    }
    bar
}

pub fn render(module: &RecordModule, cx: &mut Context<RecordModule>) -> gpui::Stateful<gpui::Div> {
    let theme = cx.global::<Theme>().clone();
    let bar = if let Some(transition) = &module.transition {
        let animation = Animation::new(transition.duration).with_easing(ease_in_out);
        div()
            .relative()
            .w_full()
            .h(px(32.0))
            .flex_shrink_0()
            .child(
                controls(transition.outgoing, false, cx)
                    .absolute()
                    .inset_0()
                    .with_animation(
                        ("record-outgoing", transition.epoch),
                        animation.clone(),
                        |element, progress| {
                            element.opacity(1.0 - progress).blur(px(progress * 5.0))
                        },
                    ),
            )
            .child(
                controls(module.presentation(), true, cx)
                    .absolute()
                    .inset_0()
                    .with_animation(
                        ("record-incoming", transition.epoch),
                        animation,
                        |element, progress| {
                            element.opacity(progress).blur(px((1.0 - progress) * 5.0))
                        },
                    ),
            )
            .into_any_element()
    } else {
        controls(module.presentation(), true, cx).into_any_element()
    };
    div()
        .id("record-module")
        .size_full()
        .p(px(8.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(theme.font_family())
        .child(bar)
        .when_some(module.error.as_ref(), |s, error| {
            s.child(
                div()
                    .w_full()
                    .h(px(32.0))
                    .flex_shrink_0()
                    .overflow_hidden()
                    .text_size(px(11.0))
                    .line_height(px(16.0))
                    .text_color(theme.red())
                    .child(error.clone()),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elapsed_time_does_not_wrap_at_one_hour() {
        assert_eq!(duration(0), "00:00");
        assert_eq!(duration(3599), "59:59");
        assert_eq!(duration(3600), "01:00:00");
        assert_eq!(duration(3661), "01:01:01");
    }
}
