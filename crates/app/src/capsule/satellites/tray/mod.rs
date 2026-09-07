use gpui::{Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::{AppState, SniItem};
use std::path::PathBuf;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};
use crate::capsule::satellites::PANEL_MIN_W;

pub fn render_mini_panel(
    item: &SniItem,
    sni_idx: usize,
    _anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> gpui::AnyElement {
    let bus_name = item.bus_name.clone();
    let menu_path = item.menu_path.clone();

    let mut menu_list = div()
        .id(("sat-menu-scroll", sni_idx as u32))
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1();

    if item.menu_items.is_empty() {
        menu_list = menu_list.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .py_2()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .child(if cx.has_global::<AppState>() {
                            cx.global::<AppState>().language.get("tray.no_menu")
                        } else {
                            "Sin menú disponible".to_string()
                        }),
                ),
        );
    } else {
        for (m_idx, m_item) in item.menu_items.iter().enumerate() {
            if m_item.is_separator {
                menu_list = menu_list.child(
                    div()
                        .w_full()
                        .h(px(1.0))
                        .my_1()
                        .bg(theme.surface().opacity(0.35)),
                );
                continue;
            }

            let item_id = m_item.id;
            let label = m_item.label.clone();
            let enabled = m_item.enabled;
            let bus_c = bus_name.clone();
            let path_c = menu_path.clone();

            menu_list = menu_list.child(
                div()
                    .id(("sat-menu-item", (sni_idx * 1000 + m_idx) as u32))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded(px(8.0))
                    .bg(gpui::hsla(0.0, 0.0, 0.0, 0.0))
                    .hover(|style| {
                        if enabled {
                            style.bg(theme.surface().opacity(0.4))
                        } else {
                            style
                        }
                    })
                    .active(|style| {
                        if enabled {
                            style.bg(theme.surface().opacity(0.6))
                        } else {
                            style
                        }
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if !enabled {
                            return;
                        }
                        let Some(ref mp) = path_c else { return };
                        if cx.has_global::<AppState>() {
                            let sni = cx.global::<AppState>().sni_host.clone();
                            sni.trigger_menu(bus_c.clone(), mp.clone(), item_id);
                        }
                    }))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::NORMAL)
                            .text_color(if enabled {
                                theme.foreground()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(label),
                    ),
            );
        }
    }

    let title_label = if item.title.is_empty() {
        item.id.clone()
    } else {
        item.title.clone()
    };

    let radius = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.satellite_round
    } else {
        20.0
    };

    div()
        .min_w(px(PANEL_MIN_W))
        .max_h(px(panel_h))
        .p_3()
        .gap_2()
        .rounded(px(radius))
        .bg(theme.background().opacity(0.95))
        .border_1()
        .border_color(theme.surface().opacity(0.35))
        .shadow_lg()
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
                        .gap_1p5()
                        .child(render_app_icon(item, &title_label, theme))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .child(title_label),
                        ),
                )
                .child(
                    div()
                        .id(("close-sat-panel", sni_idx as u32))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(0.6))
                        .hover(|s| s.bg(theme.surface().opacity(0.9)))
                        .active(|s| s.bg(theme.surface()))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(DashboardEvent::TrayIconClicked(sni_idx));
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(10.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .child(menu_list)
        .into_any_element()
}

fn render_app_icon(item: &SniItem, label: &str, theme: &Theme) -> gpui::AnyElement {
    if let Some(ref icon_path) = item.icon_file_path {
        if icon_path.ends_with(".svg") {
            svg()
                .path(icon_path.clone())
                .size(px(16.0))
                .text_color(theme.foreground())
                .into_any_element()
        } else {
            img(PathBuf::from(icon_path.clone()))
                .size(px(16.0))
                .rounded_sm()
                .into_any_element()
        }
    } else {
        render_initials(label, theme)
    }
}

fn render_initials(label: &str, theme: &Theme) -> gpui::AnyElement {
    let initials: String = label
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect();
    let initials_upper = if initials.is_empty() {
        label.chars().take(2).collect::<String>().to_uppercase()
    } else {
        initials.to_uppercase()
    };

    div()
        .w(px(16.0))
        .h(px(16.0))
        .rounded_full()
        .bg(theme.accent().opacity(0.2))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(7.0))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.accent())
                .child(initials_upper),
        )
        .into_any_element()
}
