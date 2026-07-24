use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::idle_hover::{IdleHoverEvent, IdleHoverModule};

pub fn render_header(
    battery_percentage: Option<i32>,
    battery_charging: bool,
    greeting_str: &str,
    greeting_icon: &str,
    date_str: &str,
    time_str: &str,
    theme: &Theme,
    cx: &mut Context<IdleHoverModule>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            svg()
                                .path("dashboard.svg")
                                .size(px(15.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(theme.foreground())
                                .child("Dashboard"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_1_5()
                                .text_color(theme.foreground_muted())
                                .text_size(px(11.0))
                                .child(
                                    svg()
                                        .path(if battery_charging {
                                            "battery-charging.svg"
                                        } else {
                                            "battery.svg"
                                        })
                                        .size(px(15.0))
                                        .text_color(theme.foreground_muted()),
                                )
                                .child(format!("{}%", battery_percentage.unwrap_or(100))),
                        )
                        .child(
                            div()
                                .id("header-theme-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(24.0))
                                .h(px(24.0))
                                .rounded_full()
                                .bg(theme.surface())
                                .cursor_pointer()
                                .on_click(cx.listener(|_this, _, _, cx| {
                                    cx.emit(IdleHoverEvent::SelectThemeRequested);
                                }))
                                .child(
                                    svg()
                                        .path("palette.svg")
                                        .size(px(13.0))
                                        .text_color(theme.accent()),
                                ),
                        )
                        .child(
                            div()
                                .id("header-close-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(24.0))
                                .h(px(24.0))
                                .rounded_full()
                                .bg(theme.surface())
                                .cursor_pointer()
                                .on_click(cx.listener(|_this, _, _, cx| {
                                    cx.emit(IdleHoverEvent::CloseRequested);
                                }))
                                .child(
                                    svg()
                                        .path("close.svg")
                                        .size(px(12.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1_5()
                        .child(
                            svg()
                                .path(greeting_icon)
                                .size(px(15.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(theme.accent())
                                .child(greeting_str.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child(date_str.to_string()),
                )
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(22.0))
                        .text_color(theme.foreground())
                        .child(time_str.to_string()),
                ),
        )
}
