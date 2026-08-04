use gpui::{Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::{AppState, SniItem};
use std::path::PathBuf;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};
use crate::capsule::satellites::PANEL_W;

pub fn render_mini_panel(
    item: &SniItem,
    sni_idx: usize,
    anim_t: f32,
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
                        .child({
                            let lang = if cx.has_global::<ui::language::Language>() {
                                cx.global::<ui::language::Language>().clone()
                            } else {
                                ui::language::Language::default()
                            };
                            lang.tray.no_menu
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
                        .my_0p5()
                        .bg(theme.surface().opacity(0.5)),
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
                    .py_1()
                    .rounded(px(8.0))
                    .bg(theme.surface().opacity(0.3))
                    .hover(|style| {
                        if enabled {
                            style.bg(theme.surface())
                        } else {
                            style
                        }
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if enabled {
                            if let Some(ref mp) = path_c {
                                if cx.has_global::<AppState>() {
                                    let sni = cx.global::<AppState>().sni_host.clone();
                                    sni.trigger_menu(bus_c.clone(), mp.clone(), item_id);
                                }
                            }
                        }
                    }))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
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

    let opacity = anim_t.clamp(0.0, 1.0);

    let title_label = if item.title.is_empty() {
        item.id.clone()
    } else {
        item.title.clone()
    };

    div()
        .w(px(PANEL_W))
        .max_h(px(panel_h))
        .p_2p5()
        .gap_1p5()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(opacity)
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
                                .text_size(px(11.0))
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
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(theme.surface())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface()))
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
        .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
        .child(
            div()
                .id(("activate-sat-app", sni_idx as u32))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_1p5()
                .w_full()
                .py_1()
                .rounded(px(8.0))
                .bg(theme.accent().opacity(0.15))
                .border_1()
                .border_color(theme.accent().opacity(0.3))
                .cursor_pointer()
                .hover(|style| style.bg(theme.accent().opacity(0.25)))
                .on_click(cx.listener(move |_, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().sni_host.activate_item(sni_idx);
                    }
                }))
                .child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.accent())
                        .child({
                            let lang = if cx.has_global::<ui::language::Language>() {
                                cx.global::<ui::language::Language>().clone()
                            } else {
                                ui::language::Language::default()
                            };
                            lang.tray.open_app
                        }),
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
