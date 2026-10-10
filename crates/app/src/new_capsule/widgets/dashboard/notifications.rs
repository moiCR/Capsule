use super::{DashboardAction as Action, icon_action};
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::widgets::style;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, img, prelude::*, px, svg};
use std::path::PathBuf;
use ui::theme::Theme;

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let count = module.snapshot.notifications.len();
    let header = div()
        .w_full()
        .min_w_0()
        .h(px(32.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(module.text("dashboard.notifications_title", cx)),
                )
                .child(
                    div()
                        .px(px(6.0))
                        .py(px(1.0))
                        .rounded_full()
                        .bg(style::surface(theme))
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child(count.to_string()),
                ),
        )
        .when(count > 0, |s| {
            s.child(icon_action(
                "dashboard-clear-notifications",
                "trash.svg",
                Action::ClearNotifications,
                true,
                theme,
                cx,
            ))
        });
    let mut list = div()
        .id("dashboard-notifications-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .min_w_0()
        .max_w_full()
        .overflow_x_hidden()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(6.0));
    if count == 0 {
        list = list.child(
            div()
                .w_full()
                .flex_1()
                .min_h(px(32.0))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(10.0))
                .child(
                    svg()
                        .path("bell.svg")
                        .size(px(18.0))
                        .text_color(theme.foreground_muted().opacity(0.65)),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.foreground_muted())
                        .child(module.text("dashboard.no_notifications", cx)),
                ),
        );
    }
    for notification in module.snapshot.notifications.iter().rev() {
        let icon = if notification.app_icon.starts_with('/') {
            if notification.app_icon.ends_with(".svg") {
                svg()
                    .path(notification.app_icon.clone())
                    .size(px(18.0))
                    .text_color(theme.foreground_muted())
                    .into_any_element()
            } else {
                img(PathBuf::from(&notification.app_icon))
                    .size(px(18.0))
                    .into_any_element()
            }
        } else {
            svg()
                .path("bell.svg")
                .size(px(16.0))
                .text_color(theme.foreground_muted())
                .into_any_element()
        };
        let minutes = notification.received_at.elapsed().as_secs() / 60;
        let age = if minutes == 0 {
            String::new()
        } else if minutes < 60 {
            format!("{minutes}m")
        } else {
            format!("{}h", minutes / 60)
        };
        let mut row = div()
            .id(format!("dashboard-notification-{}", notification.id))
            .flex_shrink_0()
            .w_full()
            .min_w_0()
            .max_w_full()
            .overflow_hidden()
            .min_h(px(64.0))
            .p(px(8.0))
            .rounded(px(style::INNER_RADIUS))
            .bg(style::surface(theme))
            .flex()
            .items_start()
            .gap(px(10.0))
            .child(
                div()
                    .size(px(28.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(style::background(theme))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon),
            );
        let mut content = div()
            .flex_1()
            .min_w_0()
            .max_w_full()
            .overflow_hidden()
            .whitespace_normal()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .min_h(px(22.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .line_height(px(18.0))
                            .child(notification.summary.clone()),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .child(age),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .max_w_full()
                    .flex_shrink_0()
                    .text_size(px(12.0))
                    .line_height(px(18.0))
                    .text_color(theme.foreground_muted())
                    .child(if notification.body.is_empty() {
                        notification.app_name.clone()
                    } else {
                        format!("{} · {}", notification.app_name, notification.body)
                    }),
            );
        let mut actions = div()
            .w_full()
            .min_w_0()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .gap(px(6.0));
        let mut has_actions = false;
        for (key, label) in &notification.actions {
            if key == "default" || label.trim().is_empty() {
                continue;
            }
            has_actions = true;
            let action = Action::Notification(notification.id, key.clone());
            let hover = style::hover(theme);
            actions = actions.child(
                div()
                    .id(format!("dashboard-notification-{}-{key}", notification.id))
                    .min_w_0()
                    .max_w_full()
                    .min_h(px(26.0))
                    .px(px(8.0))
                    .rounded(px(8.0))
                    .flex()
                    .items_center()
                    .text_size(px(11.0))
                    .text_color(theme.foreground())
                    .bg(style::background(theme))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        cx.stop_propagation();
                        this.dispatch(action.clone(), cx);
                    }))
                    .child(label.clone()),
            );
        }
        if has_actions {
            content = content.child(actions);
        }
        if notification.actions.iter().any(|(key, _)| key == "default") {
            let id = notification.id;
            let hover = style::hover(theme);
            row = row
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.dispatch(Action::Notification(id, "default".into()), cx)
                }));
        }
        row = row.child(content).child(icon_action(
            format!("dashboard-dismiss-{}", notification.id),
            "close.svg",
            Action::RemoveNotification(notification.id),
            true,
            theme,
            cx,
        ));
        list = list.child(row);
    }
    div()
        .id("dashboard-notifications")
        .flex_1()
        .min_h_0()
        .w_full()
        .min_w_0()
        .max_w_full()
        .h_full()
        .px(px(2.0))
        .overflow_hidden()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(header)
        .child(list)
        .into_any_element()
}
