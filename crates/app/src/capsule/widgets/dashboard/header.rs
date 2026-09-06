use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use super::tray::render_tray_widget;
use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

#[allow(clippy::too_many_arguments)]
pub fn render_header(
    battery_percentage: Option<i32>,
    battery_charging: bool,
    open_panel_indices: &[usize],
    _greeting_str: &str,
    _greeting_icon: &str,
    date_str: &str,
    time_str: &str,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let date_time_text = format!("{date_str} • {time_str}");

    let battery_chip = if let Some(pct) = battery_percentage {
        let icon = if battery_charging {
            "battery-charging.svg"
        } else if pct <= 20 {
            "battery-low.svg"
        } else if pct <= 60 {
            "battery-medium.svg"
        } else {
            "battery-full.svg"
        };
        Some(
            div()
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .gap(px(3.0))
                .px_2()
                .py_1()
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .text_color(theme.foreground())
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .child(
                    svg()
                        .path(icon)
                        .size(px(13.0))
                        .flex_shrink_0()
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .text_color(theme.foreground())
                        .child(format!("{pct}%")),
                ),
        )
    } else {
        None
    };

    div()
        .id("dashboard-header")
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
                .flex_shrink_0()
                .gap_2()
                .cursor_pointer()
                .px_2p5()
                .py_1()
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.45)))
                .active(|s| s.bg(theme.surface().opacity(0.7)))
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
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.5))
                        .text_color(theme.foreground())
                        .child(date_time_text),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .flex_shrink_0()
                .gap_1p5()
                .child(render_tray_widget(open_panel_indices, theme, cx))
                .children(battery_chip),
        )
}
