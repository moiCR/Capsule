use crate::new_capsule::widgets::style;
use chrono::{Datelike, Local, NaiveDate};
use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::new_capsule::module::dashboard::DashboardModule;

pub const WIDTH: f32 = 300.0;
pub const HEIGHT: f32 = 300.0;

fn control(
    id: &'static str,
    icon: &'static str,
    direction: Option<i8>,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let hover = theme.surface();
    div()
        .id(id)
        .size(px(28.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| {
            if let Some(direction) = direction {
                this.dispatch(
                    crate::new_capsule::widgets::dashboard::DashboardAction::Month(direction),
                    cx,
                );
            } else {
                cx.emit(
                    crate::new_capsule::module::CapsuleModuleEvent::CloseSatelliteId(
                        crate::new_capsule::satellite::SatelliteId::Calendar,
                    ),
                );
            }
        }))
        .child(
            svg()
                .path(icon)
                .size(px(14.0))
                .text_color(theme.foreground_muted()),
        )
}

pub fn render(height: gpui::Pixels, cx: &mut Context<DashboardModule>) -> impl IntoElement {
    let theme = cx.global::<Theme>().clone();
    let state = cx.global::<services::AppState>();
    let (year, month) = state.calendar.get_view_date();
    let month_name = state
        .language
        .get_list("datetime.months")
        .get(month.saturating_sub(1) as usize)
        .cloned()
        .unwrap_or_else(|| month.to_string());
    let weekdays = state.language.get_list("datetime.days_short");
    let today = Local::now().date_naive();
    let first = NaiveDate::from_ymd_opt(year, month, 1);
    let offset = first
        .map(|date| date.weekday().num_days_from_monday() as i32)
        .unwrap_or(0);
    let mut labels = div().flex().gap(px(4.0)).h(px(20.0));
    for label in weekdays.into_iter().take(7) {
        labels = labels.child(
            div()
                .flex_1()
                .text_center()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(label),
        );
    }
    let mut grid = div().flex_1().min_h_0().flex().flex_col().gap(px(4.0));
    for week in 0..6 {
        let mut row = div().flex_1().min_h_0().flex().gap(px(4.0));
        for weekday in 0..7 {
            let day = week * 7 + weekday - offset + 1;
            let date = if day > 0 {
                NaiveDate::from_ymd_opt(year, month, day as u32)
            } else {
                None
            };
            let selected = date == Some(today);
            row = row.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .when(selected, |s| s.bg(theme.accent()))
                    .text_color(if selected {
                        style::on_accent(&theme)
                    } else {
                        theme.foreground()
                    })
                    .child(date.map(|date| date.day().to_string()).unwrap_or_default()),
            );
        }
        grid = grid.child(row);
    }
    div()
        .w_full()
        .h(height.min(px(HEIGHT - 2.0)))
        .flex()
        .flex_col()
        .p(px(16.0))
        .gap(px(12.0))
        .font_family(theme.font_family())
        .text_size(px(13.0))
        .text_color(theme.foreground())
        .overflow_hidden()
        .child(
            div()
                .h(px(28.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .id("satellite-calendar-today")
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.dispatch(
                                crate::new_capsule::widgets::dashboard::DashboardAction::Month(0),
                                cx,
                            )
                        }))
                        .child(
                            svg()
                                .path("calendar-days.svg")
                                .size(px(16.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_ellipsis()
                                .child(format!("{month_name} {year}")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(control(
                            "satellite-calendar-prev",
                            "chevron-left.svg",
                            Some(-1),
                            &theme,
                            cx,
                        ))
                        .child(control(
                            "satellite-calendar-next",
                            "chevron-right.svg",
                            Some(1),
                            &theme,
                            cx,
                        ))
                        .child(control(
                            "satellite-calendar-close",
                            "close.svg",
                            None,
                            &theme,
                            cx,
                        )),
                ),
        )
        .child(labels)
        .child(grid)
}
