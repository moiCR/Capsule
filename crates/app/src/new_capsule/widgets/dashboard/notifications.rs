use super::{DashboardAction as Action, button};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, prelude::*, px};
use ui::theme::Theme;
pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(36.0))
        .flex_shrink_0()
        .child(
            div()
                .text_size(px(14.0))
                .child(module.text("dashboard.notifications_title", cx)),
        )
        .when(!module.snapshot.notifications.is_empty(), |s| {
            s.child(button(
                "dashboard-clear-notifications",
                String::new(),
                "trash.svg",
                Action::ClearNotifications,
                false,
                theme,
                cx,
            ))
        });
    let mut list = div()
        .id("dashboard-notifications-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(8.0));
    if module.snapshot.notifications.is_empty() {
        list = list.child(super::empty(
            module.text("dashboard.no_notifications", cx),
            theme,
        ));
    }
    for notification in module.snapshot.notifications.iter().rev() {
        let mut row = div()
            .id(format!("dashboard-notification-{}", notification.id))
            .flex_shrink_0()
            .w_full()
            .p(px(12.0))
            .rounded(px(12.0))
            .bg(theme.surface().opacity(0.25))
            .flex()
            .flex_col()
            .gap(px(5.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .text_ellipsis()
                            .child(notification.app_name.clone()),
                    )
                    .child(button(
                        format!("dashboard-dismiss-{}", notification.id),
                        String::new(),
                        "close.svg",
                        Action::RemoveNotification(notification.id),
                        false,
                        theme,
                        cx,
                    )),
            )
            .child(
                div()
                    .text_size(px(14.0))
                    .text_ellipsis()
                    .child(notification.summary.clone()),
            )
            .when(!notification.body.is_empty(), |s| {
                s.child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .max_h(px(48.0))
                        .overflow_hidden()
                        .child(notification.body.clone()),
                )
            });
        let mut actions = div().flex().flex_wrap().gap(px(4.0));
        for (key, label) in &notification.actions {
            actions = actions.child(button(
                format!("dashboard-notification-{}-{key}", notification.id),
                label.clone(),
                "chevron-right.svg",
                Action::Notification(notification.id, key.clone()),
                false,
                theme,
                cx,
            ));
        }
        if !notification.actions.is_empty() {
            row = row.child(actions);
        }
        list = list.child(row);
    }
    div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(list)
        .into_any_element()
}
