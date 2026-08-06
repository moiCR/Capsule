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

    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let notifications_box = div()
        .id("notifications-container-box")
        .flex()
        .flex_col()
        .w_full()
        .h(px(130.0))
        .flex_shrink_0()
        .bg(theme.background_alt())
        .border_1()
        .border_color(theme.surface())
        .rounded(px(24.0))
        .overflow_hidden();

    let clear_all_btn = if !is_empty {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .w_full()
            .px_1()
            .child(
                div()
                    .id("clear-all-notifs")
                    .cursor_pointer()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_1()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .hover(|style| style.bg(theme.surface()))
                    .on_click(cx.listener(|_, _, _, cx| {
                        NotificationStore::global().clear_all_notifications();
                        cx.notify();
                    }))
                    .child(
                        svg()
                            .path("trash.svg")
                            .size(px(12.0))
                            .text_color(theme.foreground_muted()),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(theme.foreground_muted())
                            .child(lang.dashboard.clear_all),
                    ),
            )
            .into_any_element()
    } else {
        div().into_any_element()
    };

    let box_content = if is_empty {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .size_full()
            .gap_2()
            .child(
                svg()
                    .path("bell-off.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground_muted()),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.foreground_muted())
                    .child(lang.dashboard.no_notifications),
            )
            .into_any_element()
    } else {
        let notif_count = notifications.len();
        let mut list = div()
            .id("notifs-internal-scroll")
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll();

        for (idx, item) in notifications.iter().rev().enumerate() {
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
                .px_3p5()
                .py_2p5()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .child(item.app_name.clone()),
                        )
                        .child(
                            div()
                                .id(("delete-notif", notif_id))
                                .cursor_pointer()
                                .p_0p5()
                                .rounded_full()
                                .hover(|style| style.bg(theme.surface()))
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    NotificationStore::global().remove_notification(notif_id);
                                    cx.notify();
                                }))
                                .child(
                                    svg()
                                        .path("close.svg")
                                        .size(px(11.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child(body_text),
                );

            list = list.child(notif_item);

            if idx < notif_count - 1 {
                list = list.child(
                    div()
                        .w_full()
                        .h(px(1.0))
                        .bg(theme.surface().opacity(0.3)),
                );
            }
        }
        list.into_any_element()
    };

    if is_empty {
        notifications_box.child(box_content).into_any_element()
    } else {
        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_1p5()
            .child(clear_all_btn)
            .child(notifications_box.child(box_content))
            .into_any_element()
    }
}
