use super::super::DashboardAction as Action;
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use chrono::{Datelike, Local, NaiveDate};
use gpui::ColorExt;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use ui::theme::Theme;
pub(super) fn calendar(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let (year, month) = module.snapshot.month;
    let Some(first) = NaiveDate::from_ymd_opt(year, month, 1) else {
        return empty(String::new(), theme);
    };
    let mut content = div().flex_1().flex().flex_col().gap(px(12.0));
    let month_name = cx
        .global::<services::AppState>()
        .language
        .get_list("datetime.months")
        .get(month.saturating_sub(1) as usize)
        .cloned()
        .unwrap_or_else(|| month.to_string());
    content = content.child(
        div()
            .flex()
            .justify_between()
            .items_center()
            .child(button(
                "dashboard-calendar-prev",
                String::new(),
                "chevron-left.svg",
                Action::Month(-1),
                false,
                theme,
                cx,
            ))
            .child(button(
                "dashboard-calendar-today",
                format!("{month_name} {year}"),
                "calendar-days.svg",
                Action::Month(0),
                false,
                theme,
                cx,
            ))
            .child(button(
                "dashboard-calendar-next",
                String::new(),
                "chevron-right.svg",
                Action::Month(1),
                false,
                theme,
                cx,
            )),
    );
    let weekdays = cx
        .global::<services::AppState>()
        .language
        .get_list("datetime.days_short");
    let mut labels = div().flex().gap(px(4.0));
    for label in weekdays {
        labels = labels.child(
            div()
                .flex_1()
                .text_center()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(label),
        );
    }
    content = content.child(labels);
    let today = Local::now().date_naive();
    let offset = first.weekday().num_days_from_monday() as i32;
    for week in 0..6 {
        let mut row = div().flex().gap(px(4.0)).h(px(38.0));
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
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .bg(if selected {
                        theme.accent().opacity(0.2)
                    } else {
                        theme.surface().opacity(0.12)
                    })
                    .text_color(if selected {
                        theme.accent()
                    } else {
                        theme.foreground()
                    })
                    .child(date.map(|d| d.day().to_string()).unwrap_or_default()),
            );
        }
        content = content.child(row);
    }
    content.into_any_element()
}
