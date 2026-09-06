use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::NotificationStore;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn render_notifications_widget(
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let notifications = NotificationStore::global().get_all_notifications();
    let is_empty = notifications.is_empty();

    let clear_all_btn = if !is_empty {
        div()
            .id("clear-all-notifs")
            .cursor_pointer()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full()
            .hover(|s| s.bg(theme.surface().opacity(0.45)))
            .on_click(cx.listener(|_, _, _, cx| {
                NotificationStore::global().clear_all_notifications();
                cx.notify();
            }))
            .child(
                svg()
                    .path("trash.svg")
                    .size(px(11.0))
                    .text_color(theme.foreground_muted()),
            )
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(theme.foreground_muted())
                    .child("Limpiar todo"),
            )
            .into_any_element()
    } else {
        div().into_any_element()
    };

    let content = if is_empty {
        div()
            .flex()
            .items_center()
            .w_full()
            .py_2()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.foreground_muted().opacity(0.7))
                    .child("Sin notificaciones"),
            )
            .into_any_element()
    } else {
        let mut list = div()
            .id("notifications-scroll-list")
            .flex()
            .flex_col()
            .w_full()
            .max_h(px(130.0))
            .gap_1p5()
            .overflow_y_scroll();

        for item in notifications.iter().rev() {
            let notif_id = item.id;
            let body_text = if item.body.is_empty() {
                item.summary.clone()
            } else {
                format!("{}: {}", item.summary, item.body)
            };

            let notif_item = div()
                .flex()
                .flex_col()
                .w_full()
                .px_2p5()
                .py_2()
                .rounded(px(10.0))
                .bg(theme.surface().opacity(0.4))
                .hover(|s| s.bg(theme.surface().opacity(0.6)))
                .gap_1()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.foreground())
                                .child(item.app_name.clone()),
                        )
                        .child(
                            div()
                                .id(("dismiss-notif", notif_id as usize))
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(16.0))
                                .h(px(16.0))
                                .rounded_full()
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.surface()))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    NotificationStore::global().remove_notification(notif_id);
                                    cx.notify();
                                }))
                                .child(
                                    svg()
                                        .path("close.svg")
                                        .size(px(10.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .line_height(gpui::relative(1.2))
                        .child(body_text),
                );

            list = list.child(notif_item);
        }

        list.into_any_element()
    };

    div()
        .id("notifications-section")
        .flex()
        .flex_col()
        .w_full()
        .gap_1()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.foreground_muted())
                        .child("Notificaciones"),
                )
                .child(clear_all_btn),
        )
        .child(content)
}
