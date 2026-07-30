use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

#[allow(clippy::too_many_arguments)]
pub fn render_header(
    battery_percentage: Option<i32>,
    battery_charging: bool,
    _greeting_str: &str,
    _greeting_icon: &str,
    date_str: &str,
    time_str: &str,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let date_time_text = format!("{date_str} • {time_str}");

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            div()
                .id("header-calendar-btn")
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .overflow_hidden()
                .cursor_pointer()
                .px_1p5()
                .py_1()
                .rounded_md()
                .hover(|s| s.bg(theme.surface().opacity(0.5)))
                .on_click(cx.listener(|_this, _, _, cx| {
                    cx.emit(DashboardEvent::CalendarClicked);
                }))
                .child(
                    svg()
                        .path("calendar-days.svg")
                        .size(px(14.0))
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.0))
                        .text_color(theme.foreground())
                        .truncate()
                        .child(date_time_text),
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
                            cx.emit(DashboardEvent::SelectThemeRequested);
                        }))
                        .child(
                            svg()
                                .path("palette_2.svg")
                                .size(px(13.0))
                                .text_color(theme.accent()),
                        ),
                )
                .child(
                    div()
                        .id("header-wallpaper-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded_full()
                        .bg(theme.surface())
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            cx.emit(DashboardEvent::WallpaperRequested);
                        }))
                        .child(
                            svg()
                                .path("wallpaper.svg")
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
                            cx.emit(DashboardEvent::CloseRequested);
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(12.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
}
