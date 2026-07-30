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

    let (power_icon, power_label) = match (battery_percentage, battery_charging) {
        (Some(pct), true) => ("battery-charging.svg", format!("{pct}%")),
        (Some(pct), false) if pct <= 20 => ("battery-low.svg", format!("{pct}%")),
        (Some(pct), false) if pct <= 60 => ("battery-medium.svg", format!("{pct}%")),
        (Some(pct), false) => ("battery-full.svg", format!("{pct}%")),
        (None, _) => ("plug.svg", "AC Desktop".to_string()),
    };

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
                        .id("header-power-btn")
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1_5()
                        .px_2()
                        .py_1()
                        .rounded_full()
                        .bg(theme.surface())
                        .hover(|s| s.bg(theme.surface().opacity(0.8)))
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            cx.emit(DashboardEvent::PowerClicked);
                        }))
                        .text_color(theme.foreground())
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .child(
                            svg()
                                .path(power_icon)
                                .size(px(13.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .text_color(theme.foreground())
                                .child(power_label),
                        ),
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
