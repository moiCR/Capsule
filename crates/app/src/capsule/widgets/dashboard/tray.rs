use gpui::{Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::{AppState, SniItem};
use std::path::PathBuf;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn render_tray_widget(
    open_panel_indices: &[usize],
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let items = if cx.has_global::<AppState>() {
        cx.global::<AppState>().sni_host.get_items()
    } else {
        vec![]
    };
    let open_indices = open_panel_indices.to_vec();

    if items.is_empty() {
        return div().into_any_element();
    }

    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_1()
        .size_full()
        .overflow_x_hidden();

    for (idx, item) in items.iter().enumerate() {
        let item_idx = idx;
        let is_open = open_indices.contains(&idx);
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
                .w(px(24.0))
                .h(px(24.0))
                .rounded_sm()
                .bg(if is_open {
                    theme.accent().opacity(0.2)
                } else {
                    theme.surface().opacity(0.0)
                })
                .cursor_pointer()
                .hover(|style| style.bg(theme.surface().opacity(0.5)))
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.emit(DashboardEvent::TrayIconClicked(item_idx));
                }))
                .child(icon_element),
        );
    }

    row.into_any_element()
}

pub fn compute_panel_height(item: &SniItem) -> f32 {
    let base_h = 95.0;
    let items_h = if item.menu_items.is_empty() {
        30.0
    } else {
        item.menu_items.len() as f32 * 28.0
    };
    (base_h + items_h).clamp(125.0, 460.0)
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
