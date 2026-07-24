use gpui::{Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::{AppState, SniItem};
use std::path::PathBuf;
use ui::theme::Theme;

use crate::capsule::modules::idle_hover::IdleHoverModule;

pub fn render_tray_widget(theme: &Theme, cx: &mut Context<IdleHoverModule>) -> impl IntoElement {
    let (items, selected_idx) = if cx.has_global::<AppState>() {
        let sni = cx.global::<AppState>().sni_host.clone();
        (sni.get_items(), sni.get_selected_idx())
    } else {
        (vec![], None)
    };

    let is_empty = items.is_empty();
    let container_height = if is_empty { px(56.0) } else { px(64.0) };

    let tray_content = if is_empty {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .size_full()
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.foreground_muted())
                    .child("Sin aplicaciones activas en la bandeja"),
            )
            .into_any_element()
    } else {
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2p5()
            .size_full()
            .overflow_x_hidden();

        for (idx, item) in items.iter().enumerate() {
            let item_idx = idx;
            let is_selected = selected_idx == Some(idx);
            let label = if item.title.is_empty() {
                item.id.clone()
            } else {
                item.title.clone()
            };

            let icon_element = render_app_icon(item, &label, theme);

            row = row.child(
                div()
                    .id(("tray-icon-btn", idx as u32))
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(38.0))
                    .h(px(38.0))
                    .rounded_full()
                    .bg(if is_selected {
                        theme.accent().opacity(0.25)
                    } else {
                        theme.surface().opacity(0.6)
                    })
                    .border_1()
                    .border_color(if is_selected {
                        theme.accent()
                    } else {
                        theme.surface()
                    })
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.surface()))
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let sni = cx.global::<AppState>().sni_host.clone();
                            if sni.get_selected_idx() == Some(item_idx) {
                                sni.set_selected_idx(None);
                            } else {
                                sni.set_selected_idx(Some(item_idx));
                            }
                            cx.notify();
                        }
                    }))
                    .child(icon_element),
            );
        }

        row.into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .w_full()
        .gap_1p5()
        .child(
            div().flex().flex_row().items_center().px_1().child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.foreground_muted())
                    .child("Bandeja del sistema"),
            ),
        )
        .child(
            div()
                .w_full()
                .h(container_height)
                .bg(theme.background_alt())
                .border_1()
                .border_color(theme.surface())
                .rounded(px(16.0))
                .p_2p5()
                .overflow_hidden()
                .child(tray_content),
        )
}

pub fn render_side_tray_panel(
    selected_item: &SniItem,
    selected_idx: usize,
    theme: &Theme,
    cx: &mut Context<IdleHoverModule>,
) -> gpui::AnyElement {
    let bus_name = selected_item.bus_name.clone();
    let menu_path = selected_item.menu_path.clone();

    let mut menu_list = div()
        .flex()
        .flex_col()
        .w_full()
        .size_full()
        .overflow_hidden()
        .gap_1();

    if selected_item.menu_items.is_empty() {
        menu_list = menu_list.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .py_6()
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child("Sin menú disponible"),
                ),
        );
    } else {
        for (m_idx, m_item) in selected_item.menu_items.iter().enumerate() {
            if m_item.is_separator {
                menu_list = menu_list.child(
                    div()
                        .w_full()
                        .h(px(1.0))
                        .my_1()
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
                    .id(("side-menu-item", m_idx as u32))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_3()
                    .py_2()
                    .rounded(px(12.0))
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
                            .text_size(px(12.0))
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

    div()
        .flex()
        .flex_col()
        .w(px(280.0))
        .h(px(500.0))
        .p_4()
        .gap_3()
        .rounded(px(28.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .child(
            // Header
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
                        .child(render_app_icon(selected_item, &selected_item.title, theme))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(theme.foreground())
                                .child(selected_item.title.clone()),
                        ),
                )
                .child(
                    div()
                        .id("close-side-panel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded_full()
                        .bg(theme.surface())
                        .cursor_pointer()
                        .on_click(cx.listener(|_, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                cx.global::<AppState>().sni_host.set_selected_idx(None);
                                cx.notify();
                            }
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(12.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
        .child(
            // Action button: Activate app window
            div()
                .id("activate-side-app")
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap_2()
                .w_full()
                .py_2()
                .rounded(px(12.0))
                .bg(theme.accent().opacity(0.15))
                .border_1()
                .border_color(theme.accent().opacity(0.3))
                .cursor_pointer()
                .hover(|style| style.bg(theme.accent().opacity(0.25)))
                .on_click(cx.listener(move |_, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().sni_host.activate_item(selected_idx);
                    }
                }))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.accent())
                        .child("Abrir aplicación"),
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
                .size(px(22.0))
                .text_color(theme.foreground())
                .into_any_element()
        } else {
            img(PathBuf::from(icon_path.clone()))
                .size(px(22.0))
                .rounded_md()
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
        .w(px(22.0))
        .h(px(22.0))
        .rounded_full()
        .bg(theme.accent().opacity(0.2))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(9.0))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.accent())
                .child(initials_upper),
        )
        .into_any_element()
}
