use chrono::{Datelike, Local, NaiveDate};
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, calendar::NavDirection};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_W;

pub fn compute_calendar_panel_height() -> f32 {
    250.0
}

fn spawn_nav_animation(cx: &mut Context<DashboardModule>) {
    let frame_ms = if cx.has_global::<AppState>() {
        cx.global::<AppState>().compositor.get_frame_duration_ms()
    } else {
        16
    };
    let this = cx.entity().downgrade();
    cx.spawn(async move |_this, cx| {
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(220) {
            tokio::time::sleep(std::time::Duration::from_millis(frame_ms)).await;
            if this.update(cx, |_view, cx| cx.notify()).is_err() {
                break;
            }
        }
    })
    .detach();
}

pub fn render_calendar_mini_panel(
    anim_t: f32,
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

    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let month_name = lang
        .datetime
        .months
        .get((view_month as usize).saturating_sub(1))
        .cloned()
        .unwrap_or_default();

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

    let mut day_cells: Vec<AnyElement> = Vec::new();

    for _ in 0..start_weekday {
        day_cells.push(
            div()
                .w(px(22.0))
                .h(px(22.0))
                .flex()
                .items_center()
                .justify_center()
                .into_any_element(),
        );
    }

    for day in 1..=days_in_month {
        let is_today = is_current_month && day == now_day;
        day_cells.push(
            div()
                .w(px(22.0))
                .h(px(22.0))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(if is_today {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.0)
                })
                .child(
                    div()
                        .text_size(px(10.0))
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
                )
                .into_any_element(),
        );
    }

    let day_headers = ["L", "M", "M", "J", "V", "S", "D"];

    let mut grid = div().flex().flex_col().w_full().gap_1();

    let mut header_row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_1();

    for d in day_headers {
        header_row = header_row.child(
            div()
                .w(px(22.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground_muted())
                        .child(d),
                ),
        );
    }

    grid = grid.child(header_row);

    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    for cell in day_cells {
        current_chunk.push(cell);
        if current_chunk.len() == 7 {
            chunks.push(std::mem::take(&mut current_chunk));
        }
    }
    if !current_chunk.is_empty() {
        chunks.push(current_chunk);
    }

    for row_chunk in chunks {
        let chunk_len = row_chunk.len();
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px_1();

        for cell in row_chunk {
            row = row.child(cell);
        }

        if chunk_len < 7 {
            for _ in 0..(7 - chunk_len) {
                row = row.child(
                    div()
                        .w(px(22.0))
                        .h(px(22.0))
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
        .relative()
        .left(px(slide_offset))
        .opacity(grid_opacity)
        .child(grid);

    div()
        .w(px(PANEL_W))
        .max_h(px(panel_h))
        .p_2p5()
        .gap_2()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(anim_t)
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
                        .gap_1p5()
                        .child(
                            svg()
                                .path("calendar-days.svg")
                                .size(px(14.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .child(format!("{month_name} {view_year}")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1()
                        .children(if !is_current_month {
                            Some(
                                div()
                                    .id("cal-today-btn")
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(theme.surface().opacity(0.5))
                                    .hover(|s| s.bg(theme.surface()))
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
                                            .text_size(px(9.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.accent())
                                            .child(lang.datetime.today.clone()),
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
                                .w(px(18.0))
                                .h(px(18.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.5))
                                .hover(|s| s.bg(theme.surface()))
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
                                .w(px(18.0))
                                .h(px(18.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.5))
                                .hover(|s| s.bg(theme.surface()))
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
