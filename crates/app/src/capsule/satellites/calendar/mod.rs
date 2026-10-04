use chrono::{Datelike, Local, NaiveDate};
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, calendar::NavDirection};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;

use std::sync::atomic::{AtomicU64, Ordering};

pub fn compute_calendar_panel_height() -> f32 {
    274.0
}

static NAV_ANIM_GEN: AtomicU64 = AtomicU64::new(0);

fn spawn_nav_animation(cx: &mut Context<DashboardModule>) {
    let generation = NAV_ANIM_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let frame_ms = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .compositor
            .get_frame_duration_ms()
            .max(16)
    } else {
        16
    };
    let this = cx.entity().downgrade();
    cx.spawn(async move |_this, cx| {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(220) {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(frame_ms))
                .await;
            if NAV_ANIM_GEN.load(Ordering::Relaxed) != generation {
                break;
            }
            if this.update(cx, |_view, cx| cx.notify()).is_err() {
                break;
            }
        }
    })
    .detach();
}

pub fn render_calendar_mini_panel(
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let now = Local::now();
    let now_year = now.year();
    let now_month = now.month();
    let now_day = now.day();

    let ((view_year, view_month), nav_dir, nav_t) = if cx.has_global::<AppState>() {
        let app_state = cx.global::<AppState>();
        (
            app_state.calendar.get_view_date(),
            app_state.calendar.get_nav_anim().0,
            app_state.calendar.get_nav_anim().1,
        )
    } else {
        ((now_year, now_month), NavDirection::Right, 1.0)
    };

    let is_current_month = view_year == now_year && view_month == now_month;

    let default_days_short = vec![
        "L".to_string(),
        "M".to_string(),
        "M".to_string(),
        "J".to_string(),
        "V".to_string(),
        "S".to_string(),
        "D".to_string(),
    ];

    let (today_text, month_name, day_headers) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        let months = lang.get_list("datetime.months");
        let m_name = months
            .get((view_month as usize).saturating_sub(1))
            .cloned()
            .unwrap_or_default();
        let short_list = lang.get_list("datetime.days_short");
        let headers = if short_list.len() == 7 {
            short_list
        } else {
            default_days_short
        };
        (lang.get("datetime.today"), m_name, headers)
    } else {
        ("Hoy".to_string(), String::new(), default_days_short)
    };

    let days_in_month = match view_month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (view_year % 4 == 0 && view_year % 100 != 0) || (view_year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    };

    let start_date = NaiveDate::from_ymd_opt(view_year, view_month, 1);
    let start_weekday = start_date
        .map(|d| d.weekday().num_days_from_monday() as usize)
        .unwrap_or(0);

    let total_days = start_weekday + days_in_month;
    let mut grid = div().id("cal-grid").flex().flex_col().w_full().gap_1();

    let mut header_row = div()
        .id("cal-header-row")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_1();

    for (idx, d) in day_headers.into_iter().enumerate() {
        header_row = header_row.child(
            div()
                .id(("cal-header", idx as u32))
                .w(px(30.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground_muted())
                        .child(d),
                ),
        );
    }

    grid = grid.child(header_row);

    for row_idx in 0..6 {
        let mut row = div()
            .id(("cal-row", row_idx as u32))
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px_1();

        for col_idx in 0..7 {
            let slot_idx = row_idx * 7 + col_idx;
            if slot_idx < start_weekday {
                row = row.child(
                    div()
                        .id(("cal-empty-lead", slot_idx as u32))
                        .w(px(30.0))
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center(),
                );
            } else if slot_idx < total_days {
                let day = (slot_idx - start_weekday + 1) as u32;
                let is_today = is_current_month && day == now_day;
                row = row.child(
                    div()
                        .id(("cal-day", day))
                        .w(px(30.0))
                        .h(px(30.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(if is_today {
                            theme.accent()
                        } else {
                            gpui::hsla(0.0, 0.0, 0.0, 0.0)
                        })
                        .hover(|s| {
                            if is_today {
                                s
                            } else {
                                s.bg(theme.surface().opacity(0.45))
                            }
                        })
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(if is_today {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if is_today {
                                    theme.background()
                                } else {
                                    theme.foreground()
                                })
                                .child(day.to_string()),
                        ),
                );
            } else {
                row = row.child(
                    div()
                        .id(("cal-empty-trail", slot_idx as u32))
                        .w(px(30.0))
                        .h(px(30.0))
                        .flex()
                        .items_center()
                        .justify_center(),
                );
            }
        }

        grid = grid.child(row);
    }

    let eased_nav = 1.0 - (1.0 - nav_t) * (1.0 - nav_t);
    let grid_opacity = 0.2 + 0.8 * eased_nav;
    let slide_offset = match nav_dir {
        NavDirection::Right => 24.0 * (1.0 - eased_nav),
        NavDirection::Left => -24.0 * (1.0 - eased_nav),
    };

    let animated_grid = div()
        .id("cal-animated-grid")
        .relative()
        .left(px(slide_offset))
        .opacity(grid_opacity)
        .child(grid);

    div()
        .id("cal-mini-panel")
        .min_w(px(PANEL_MIN_W))
        .max_h(px(panel_h))
        .p_3()
        .gap_2()
        .border_1()
        .border_color(gpui::hsla(0.0, 0.0, 0.0, 0.0))
        .overflow_hidden()
        .flex()
        .flex_col()
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
                                .path("calendar-days.svg")
                                .size(px(15.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(theme.foreground())
                                .child(format!("{month_name} {view_year}")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .children(if !is_current_month {
                            Some(
                                div()
                                    .id("cal-today-btn")
                                    .px_2p5()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(theme.surface().opacity(0.6))
                                    .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                    .active(|s| s.bg(theme.surface()))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|_this, _, _, cx| {
                                        if cx.has_global::<AppState>() {
                                            cx.global::<AppState>().calendar.reset_to_today();
                                            spawn_nav_animation(cx);
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        div()
                                            .text_size(px(9.5))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme.accent())
                                            .child(today_text.clone()),
                                    ),
                            )
                        } else {
                            None
                        })
                        .child(
                            div()
                                .id("cal-prev-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(22.0))
                                .h(px(22.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.6))
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .active(|s| s.bg(theme.surface()))
                                .cursor_pointer()
                                .on_click(cx.listener(|_this, _, _, cx| {
                                    if cx.has_global::<AppState>() {
                                        cx.global::<AppState>().calendar.prev_month();
                                        spawn_nav_animation(cx);
                                        cx.notify();
                                    }
                                }))
                                .child(
                                    svg()
                                        .path("chevron-left.svg")
                                        .size(px(11.0))
                                        .text_color(theme.foreground()),
                                ),
                        )
                        .child(
                            div()
                                .id("cal-next-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(22.0))
                                .h(px(22.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.6))
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .active(|s| s.bg(theme.surface()))
                                .cursor_pointer()
                                .on_click(cx.listener(|_this, _, _, cx| {
                                    if cx.has_global::<AppState>() {
                                        cx.global::<AppState>().calendar.next_month();
                                        spawn_nav_animation(cx);
                                        cx.notify();
                                    }
                                }))
                                .child(
                                    svg()
                                        .path("chevron-right.svg")
                                        .size(px(11.0))
                                        .text_color(theme.foreground()),
                                ),
                        ),
                ),
        )
        .child(animated_grid)
        .into_any_element()
}
