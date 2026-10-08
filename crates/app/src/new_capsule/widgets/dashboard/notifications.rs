use super::{DashboardAction as Action, icon_action};
use crate::new_capsule::module::dashboard::DashboardModule;
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
        .h(px(28.0))
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
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(module.text("dashboard.notifications_title", cx)),
                )
                .when(count > 0, |s| {
                    s.child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .child(count.to_string()),
                    )
                }),
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
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(6.0));
    if count == 0 {
        list = list.child(super::empty(
            module.text("dashboard.no_notifications", cx),
            theme,
        ));
    }
    let radius = cx
        .global::<services::AppState>()
        .config
        .get()
        .ui
        .cards_round
        .min(14.0);
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
            .p(px(10.0))
            .rounded(px(radius))
            .bg(theme.surface().opacity(0.18))
            .flex()
            .gap(px(10.0))
            .child(
                div()
                    .size(px(28.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(icon),
            );
        let mut content = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .h(px(20.0))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .text_ellipsis()
                            .child(notification.app_name.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(theme.foreground_muted())
                            .child(age),
                    ),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_ellipsis()
                    .child(notification.summary.clone()),
            )
            .when(!notification.body.is_empty(), |s| {
                s.child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .max_h(px(36.0))
                        .overflow_hidden()
                        .child(notification.body.clone()),
                )
            });
        let mut actions = div().flex().flex_wrap().gap(px(6.0));
        let mut has_actions = false;
        for (key, label) in &notification.actions {
            if key == "default" || label.trim().is_empty() {
                continue;
            }
            has_actions = true;
            let action = Action::Notification(notification.id, key.clone());
            let hover = theme.surface().opacity(0.65);
            actions = actions.child(
                div()
                    .id(format!("dashboard-notification-{}-{key}", notification.id))
                    .min_h(px(26.0))
                    .px(px(8.0))
                    .rounded(px(8.0))
                    .flex()
                    .items_center()
                    .text_size(px(11.0))
                    .text_color(theme.foreground())
                    .bg(theme.surface().opacity(0.35))
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
            let hover = theme.surface().opacity(0.3);
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
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(header)
        .child(list)
        .into_any_element()
}
