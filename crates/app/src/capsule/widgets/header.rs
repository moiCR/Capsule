use gpui::{FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

pub fn render_header(
    battery_percentage: Option<i32>,
    battery_charging: bool,
    greeting_str: &str,
    greeting_icon: &str,
    date_str: &str,
    time_str: &str,
    theme: &Theme,
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
