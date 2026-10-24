use std::path::PathBuf;

use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, Window, div, img, px, svg};
use services::{AppState, NotificationStore};
use ui::components::Switch;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(_window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let now = Local::now();
    let app_state = cx.try_global::<AppState>();

    let (year, month) = if let Some(state) = app_state {
        state.calendar.get_view_date()
    } else {
        (now.year(), now.month())
    };

    let (
        time_str,
        weekday_full,
        month_name,
        weekdays_short,
        no_notifs_label,
        dnd_label,
        clear_label,
    ) = if let Some(state) = app_state {
        let fmt = state.config.get().ui.clock_format();
        let t_str = now.format(fmt).to_string();
        let w_full = state
            .language
            .get_list("datetime.days")
            .get(now.weekday().num_days_from_monday() as usize)
            .cloned()
            .unwrap_or_else(|| now.weekday().to_string());
        let m_name = state
            .language
            .get_list("datetime.months")
            .get(month.saturating_sub(1) as usize)
            .cloned()
            .unwrap_or_else(|| month.to_string());
        let w_short = state.language.get_list("datetime.weekdays_short");
        let no_notifs = state.language.get("dashboard.no_notifications");
        let dnd_txt = state.language.get("dashboard.dnd");
        let clear_txt = if state.language.current_target() == "es" {
            "Limpiar".to_string()
        } else {
            "Clear".to_string()
        };
        (
            t_str, w_full, m_name, w_short, no_notifs, dnd_txt, clear_txt,
        )
    } else {
        (
            now.format("%H:%M").to_string(),
            "Domingo".to_string(),
            month.to_string(),
            vec![
                "Lun".into(),
                "Mar".into(),
                "Mié".into(),
                "Jue".into(),
                "Vie".into(),
                "Sáb".into(),
                "Dom".into(),
            ],
            "Sin notificaciones".to_string(),
            "No molestar".to_string(),
            "Limpiar".to_string(),
        )
    };

    let notifications = NotificationStore::global().get_all_notifications();
    let is_dnd = NotificationStore::global().is_dnd_enabled();
    let hover_bg = style::hover(&theme);

    let notif_card_content = if notifications.is_empty() {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(14.0))
            .child(
                svg()
                    .path("bell.svg")
                    .size(px(62.0))
                    .text_color(theme.accent()),
            )
            .child(
                div()
                    .text_size(px(15.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child(no_notifs_label),
            )
            .into_any_element()
    } else {
        let mut list = div()
            .id("shared-notifications-list")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .p(px(14.0));

        for notif in notifications {
            let id = notif.id;
            let icon_el = if !notif.app_icon.is_empty()
                && (notif.app_icon.starts_with('/') || notif.app_icon.contains('.'))
            {
                img(PathBuf::from(&notif.app_icon))
                    .size(px(22.0))
                    .rounded_md()
                    .into_any_element()
            } else {
                svg()
                    .path("bell.svg")
                    .size(px(17.0))
                    .text_color(theme.accent())
                    .into_any_element()
            };

            let mut item_card = div()
                .p(px(12.0))
                .rounded_xl()
                .bg(style::background(&theme).opacity(0.65))
                .border_1()
                .border_color(style::border(&theme).opacity(0.35))
                .flex()
                .flex_col()
                .gap(px(5.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(icon_el)
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground_muted())
                                        .line_clamp(1)
                                        .child(if notif.app_name.is_empty() {
                                            "Sistema".to_string()
                                        } else {
                                            notif.app_name.clone()
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .id(format!("shared-notif-remove-{id}"))
                                .size(px(24.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .cursor_pointer()
                                .hover(move |s| s.bg(hover_bg))
                                .child(
                                    svg()
                                        .path("x.svg")
                                        .size(px(13.0))
                                        .text_color(theme.foreground_muted()),
                                )
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    NotificationStore::global().remove_notification(id);
                                    cx.notify();
                                })),
                        ),
                )
                .when(!notif.summary.is_empty(), |s| {
                    s.child(
                        div()
                            .text_size(px(13.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .line_clamp(1)
                            .child(notif.summary),
                    )
                })
                .when(!notif.body.is_empty(), |s| {
                    s.child(
                        div()
                            .text_size(px(12.5))
                            .text_color(theme.foreground_muted())
                            .line_clamp(2)
                            .child(notif.body),
                    )
                });

            if !notif.actions.is_empty() {
                let mut actions_row = div().flex().items_center().gap(px(6.0)).pt(px(4.0));
                for (act_idx, (key, label)) in notif.actions.into_iter().enumerate() {
                    let key_clone = key.clone();
                    let label_str = if label.is_empty() {
                        "Abrir".to_string()
                    } else {
                        label
                    };
                    actions_row = actions_row.child(
                        div()
                            .id(format!("shared-notif-act-{id}-{act_idx}"))
                            .h(px(26.0))
                            .px(px(12.0))
                            .rounded_full()
                            .bg(style::surface(&theme))
                            .hover(move |s| s.bg(hover_bg))
                            .cursor_pointer()
                            .flex()
                            .items_center()
                            .text_size(px(11.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(label_str)
                            .on_click(cx.listener(move |_, _, _, cx| {
                                NotificationStore::global().invoke_action(id, key_clone.clone());
                                NotificationStore::global().remove_notification(id);
                                cx.notify();
                            })),
                    );
                }
                item_card = item_card.child(actions_row);
            }

            list = list.child(item_card);
        }

        list.into_any_element()
    };

    let left_column = div()
        .flex_1()
        .h_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .rounded_2xl()
                .bg(style::surface(&theme))
                .border_1()
                .border_color(style::border(&theme).opacity(0.35))
                .overflow_hidden()
                .child(notif_card_content),
        )
        .child(
            div()
                .h(px(36.0))
                .w_full()
                .flex_shrink_0()
                .px(px(4.0))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .id("shared-notif-dnd-row")
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .cursor_pointer()
                        .on_click(cx.listener(|_, _, _, cx| {
                            NotificationStore::global().toggle_dnd();
                            cx.notify();
                        }))
                        .child(
                            Switch::new("shared-notif-dnd-switch")
                                .checked(is_dnd)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    cx.stop_propagation();
                                    NotificationStore::global().toggle_dnd();
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(dnd_label),
                        ),
                )
                .child(
                    div()
                        .id("shared-notif-clear-btn")
                        .h(px(34.0))
                        .px(px(18.0))
                        .rounded_xl()
                        .bg(style::surface(&theme))
                        .border_1()
                        .border_color(style::border(&theme).opacity(0.4))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(cx.listener(|_, _, _, cx| {
                            NotificationStore::global().clear_all_notifications();
                            cx.notify();
                        }))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(clear_label),
                        ),
                ),
        );

    let today = now.date_naive();
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    let offset = first.weekday().num_days_from_monday() as i64;

    let mut weekdays_row = div().flex().items_center().h(px(26.0)).gap(px(4.0));
    for day_label in weekdays_short.into_iter().take(7) {
        weekdays_row = weekdays_row.child(
            div()
                .flex_1()
                .text_center()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child(day_label),
        );
    }

    let mut grid = div().flex_1().flex().flex_col().gap(px(4.0));
    for week in 0..6 {
        let mut row = div().flex_1().flex().gap(px(4.0));
        for weekday in 0..7 {
            let delta = (week * 7 + weekday) as i64 - offset;
            let cell_date = first + ChronoDuration::days(delta);
            let is_current_month = cell_date.month() == month && cell_date.year() == year;
            let is_today = cell_date == today;

            let mut cell = div()
                .id(format!("calendar-cell-{week}-{weekday}"))
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .text_size(px(13.0))
                .child(cell_date.day().to_string());

            if is_today {
                cell = cell
                    .bg(theme.accent())
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.on_accent());
            } else if is_current_month {
                cell = cell
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .hover(move |s| s.bg(hover_bg))
                    .cursor_pointer();
            } else {
                cell = cell.text_color(theme.foreground_muted().opacity(0.38));
            }

            row = row.child(cell);
        }
        grid = grid.child(row);
    }

    let calendar_card = div()
        .flex_1()
        .w_full()
        .rounded_2xl()
        .bg(style::surface(&theme))
        .border_1()
        .border_color(style::border(&theme).opacity(0.35))
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .h(px(30.0))
                .child(
                    div()
                        .id("shared-calendar-header-title")
                        .flex()
                        .items_center()
                        .cursor_pointer()
                        .on_click(cx.listener(|_, _, _, cx| {
                            if let Some(state) = cx.try_global::<AppState>() {
                                state.calendar.reset_to_today();
                                cx.notify();
                            }
                        }))
                        .child(
                            div()
                                .text_size(px(16.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.accent())
                                .child(format!("{month_name} {year}")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .id("shared-calendar-prev-btn")
                                .size(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .cursor_pointer()
                                .hover(move |s| s.bg(hover_bg))
                                .on_click(cx.listener(|_, _, _, cx| {
                                    if let Some(state) = cx.try_global::<AppState>() {
                                        state.calendar.prev_month();
                                        cx.notify();
                                    }
                                }))
                                .child(
                                    svg()
                                        .path("chevron-left.svg")
                                        .size(px(14.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        )
                        .child(
                            div()
                                .id("shared-calendar-next-btn")
                                .size(px(28.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .rounded_full()
                                .cursor_pointer()
                                .hover(move |s| s.bg(hover_bg))
                                .on_click(cx.listener(|_, _, _, cx| {
                                    if let Some(state) = cx.try_global::<AppState>() {
                                        state.calendar.next_month();
                                        cx.notify();
                                    }
                                }))
                                .child(
                                    svg()
                                        .path("chevron-right.svg")
                                        .size(px(14.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                ),
        )
        .child(weekdays_row)
        .child(grid);

    let right_column = div()
        .w(px(308.0))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .w_full()
                .flex_shrink_0()
                .py(px(4.0))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(44.0))
                        .line_height(px(48.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.foreground())
                        .child(time_str),
                )
                .child(
                    div()
                        .text_size(px(20.0))
                        .line_height(px(24.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.accent())
                        .child(weekday_full),
                ),
        )
        .child(calendar_card);

    div()
        .id("shared-panel-calendar")
        .size_full()
        .p(px(18.0))
        .flex()
        .items_center()
        .gap(px(16.0))
        .font_family(theme.font_family())
        .child(left_column)
        .child(right_column)
        .into_any_element()
}
