use super::super::DashboardAction as Action;
use super::empty;
use super::list;
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::widgets::style;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

pub(super) fn audio(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = list("dashboard-audio-list");
    if module.snapshot.audio.audio_sinks.is_empty() {
        content = content.child(empty(module.text("dashboard_new.no_outputs", cx), theme));
    }
    for sink in &module.snapshot.audio.audio_sinks {
        let desc_lower = sink.description.to_lowercase();
        let name_lower = sink.name.to_lowercase();
        let icon = if desc_lower.contains("headphone")
            || desc_lower.contains("auricular")
            || desc_lower.contains("headset")
            || name_lower.contains("headphone")
        {
            "headphones.svg"
        } else if desc_lower.contains("hdmi")
            || desc_lower.contains("displayport")
            || desc_lower.contains("monitor")
            || desc_lower.contains("tv")
        {
            "monitor.svg"
        } else {
            "volume-2.svg"
        };
        let hover = style::hover(theme);
        let sink_name = sink.name.clone();
        let is_selected = sink.is_default;
        content = content.child(
            div()
                .id(format!("dashboard-sink-{}", sink.name))
                .w_full()
                .min_h(px(52.0))
                .p(px(10.0))
                .rounded(px(style::INNER_RADIUS))
                .flex()
                .items_center()
                .gap(px(12.0))
                .bg(if is_selected {
                    style::accent_card(theme)
                } else {
                    style::surface(theme)
                })
                .border_1()
                .border_color(if is_selected {
                    theme.accent().opacity(0.4)
                } else {
                    style::border(theme)
                })
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.dispatch(Action::Sink(sink_name.clone()), cx);
                }))
                .child(
                    div()
                        .size(px(32.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(style::background(theme))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(svg().path(icon).size(px(16.0)).text_color(if is_selected {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        })),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_ellipsis()
                                .child(sink.description.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.foreground_muted())
                                .text_ellipsis()
                                .child(sink.name.clone()),
                        ),
                )
                .child(style::radio(is_selected, 18.0, theme)),
        );
    }
    content.into_any_element()
}
