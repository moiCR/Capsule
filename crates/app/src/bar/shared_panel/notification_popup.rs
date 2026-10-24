use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, Window, div, img, px, svg};
use services::NotificationStore;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(id: u32, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let notif_opt = NotificationStore::global()
        .get_latest_active_notification()
        .filter(|n| n.id == id)
        .or_else(|| {
            NotificationStore::global()
                .get_all_notifications()
                .into_iter()
                .find(|n| n.id == id)
        });

    let Some(item) = notif_opt else {
        return div().into_any_element();
    };

    let icon_el = if !item.app_icon.is_empty()
        && (item.app_icon.starts_with('/') || item.app_icon.contains('.'))
    {
        img(PathBuf::from(&item.app_icon))
            .size(px(22.0))
            .rounded_md()
            .into_any_element()
    } else {
        svg()
            .path("bell.svg")
            .size(px(18.0))
            .text_color(theme.accent())
            .into_any_element()
    };

    let app_name = if item.app_name.is_empty() {
        "Sistema".to_string()
    } else {
        item.app_name.clone()
    };

    let hover_icon = style::hover(&theme);
    let hover_chip = style::hover(&theme);

    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(26.0))
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
                        .child(app_name),
                ),
        )
        .child(
            div()
                .id("popup-notif-close")
                .size(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_icon))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(13.0))
                        .text_color(theme.foreground_muted()),
                )
                .on_click(cx.listener(move |bar, _, window, cx| {
                    NotificationStore::global().remove_notification(id);
                    bar.close_shared_panel(window, cx);
                })),
        );

    let mut content = div()
        .id("shared-panel-notification-popup")
        .size_full()
        .px(px(24.0))
        .py(px(14.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .font_family(theme.font_family())
        .on_hover(cx.listener(|_, hovered, _, _| {
            NotificationStore::global().set_hovered(*hovered);
        }))
        .child(header);

    if !item.summary.is_empty() {
        content = content.child(
            div()
                .text_size(px(14.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .line_clamp(1)
                .child(item.summary),
        );
    }

    if !item.body.is_empty() {
        content = content.child(
            div()
                .text_size(px(13.0))
                .text_color(theme.foreground_muted())
                .line_clamp(2)
                .child(item.body),
        );
    }

    if !item.actions.is_empty() {
        let mut actions_row = div().flex().items_center().gap(px(8.0)).pt(px(4.0));
        for (idx, (key, label)) in item.actions.into_iter().enumerate() {
            let key_clone = key.clone();
            let label_str = if !label.is_empty() {
                label
            } else {
                "Abrir".to_string()
            };
            let chip_hover = hover_chip;
            actions_row = actions_row.child(
                div()
                    .id(format!("notif-popup-act-{id}-{idx}"))
                    .h(px(28.0))
                    .px(px(12.0))
                    .rounded_full()
                    .bg(style::surface(&theme))
                    .hover(move |s| s.bg(chip_hover))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child(label_str)
                    .on_click(cx.listener(move |bar, _, window, cx| {
                        NotificationStore::global().invoke_action(id, key_clone.clone());
                        NotificationStore::global().remove_notification(id);
                        bar.close_shared_panel(window, cx);
                    })),
            );
        }
        content = content.child(actions_row);
    }

    content.into_any_element()
}
