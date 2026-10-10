use super::style;
use crate::new_capsule::module::{CapsuleModuleEvent, notification::NotificationModule};
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::{NotificationItem, NotificationStore};
use std::path::PathBuf;
use ui::theme::Theme;

pub const WIDTH: f32 = 440.0;
pub const HISTORY_HEIGHT: f32 = 400.0;

fn visible_action(key: &str, label: &str, replying: bool) -> bool {
    !key.trim().is_empty()
        && (!label.trim().is_empty() || key == "default" || key == "inline-reply")
        && !(replying && key == "inline-reply")
}

fn action_label(key: &str, label: &str, cx: &gpui::App) -> String {
    if !label.trim().is_empty() {
        return label.to_owned();
    }
    cx.global::<services::AppState>()
        .language
        .get(if key == "inline-reply" {
            "notifications_module.reply"
        } else {
            "notifications_module.open"
        })
}

fn measure(
    text: &str,
    font_size: f32,
    weight: FontWeight,
    wrap: bool,
    cx: &gpui::App,
    text_system: &gpui::WindowTextSystem,
) -> gpui::Size<gpui::Pixels> {
    let theme = cx.global::<Theme>();
    let runs = [gpui::TextRun {
        len: text.len(),
        font: gpui::Font {
            weight,
            ..gpui::font(theme.font_family())
        },
        color: theme.foreground(),
        ..Default::default()
    }];
    text_system
        .shape_text(
            text.to_owned(),
            px(font_size),
            &runs,
            wrap.then(|| px(WIDTH - 32.0)),
            wrap.then_some(4),
        )
        .map(|shaped| {
            let line_height = px(if font_size == 15.0 { 21.0 } else { 18.0 });
            let mut size = shaped.size(line_height);
            if wrap {
                size.height = size.height.min(line_height * 4.0);
            }
            size
        })
        .unwrap_or_else(|_| gpui::size(px(text.chars().count() as f32 * font_size), px(84.0)))
}

fn actions_height(
    item: &NotificationItem,
    replying: bool,
    cx: &gpui::App,
    text_system: &gpui::WindowTextSystem,
) -> f32 {
    let mut rows: usize = 0;
    let mut used = 0.0;
    let width = WIDTH - 32.0;
    for (key, label) in &item.actions {
        if !visible_action(key, label, replying) {
            continue;
        }
        let label = action_label(key, label, cx);
        let button =
            (f32::from(measure(&label, 11.0, FontWeight::NORMAL, false, cx, text_system).width)
                + 30.0)
                .min(width);
        if rows == 0 || used + 6.0 + button > width {
            rows += 1;
            used = button;
        } else {
            used += 6.0 + button;
        }
    }
    let rows = rows.min(3);
    rows as f32 * 28.0 + rows.saturating_sub(1) as f32 * 6.0
}

pub fn popup_height(
    item: Option<&NotificationItem>,
    replying: bool,
    failed: bool,
    cx: &gpui::App,
    text_system: &gpui::WindowTextSystem,
) -> f32 {
    let content = item.map_or(144.0, |item| {
        let summary = f32::from(
            measure(
                &item.summary,
                15.0,
                FontWeight::SEMIBOLD,
                true,
                cx,
                text_system,
            )
            .height,
        )
        .max(21.0);
        let body = if item.body.is_empty() {
            0.0
        } else {
            5.0 + f32::from(
                measure(&item.body, 13.0, FontWeight::NORMAL, true, cx, text_system).height,
            )
        };
        let actions = actions_height(item, replying, cx, text_system);
        32.0 + 32.0 + 10.0 + summary + body + if actions > 0.0 { 10.0 + actions } else { 0.0 }
    });
    content.max(126.0)
        + if replying {
            84.0 + if failed { 24.0 } else { 0.0 }
        } else {
            0.0
        }
}

pub fn render(
    module: &NotificationModule,
    theme: &Theme,
    cx: &mut Context<NotificationModule>,
) -> AnyElement {
    let mut root = div()
        .size_full()
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(12.0));
    if module.history {
        root = root.child(
            div()
                .h(px(36.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(module.text("dashboard.notifications_title", cx)),
                        )
                        .child(style::chip(
                            "notification-count",
                            module.items.len().to_string(),
                            false,
                            theme,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(6.0))
                        .when(!module.items.is_empty(), |s| {
                            s.child(
                                style::circle_button(
                                    "notification-clear",
                                    "trash.svg",
                                    30.0,
                                    theme,
                                )
                                .on_click(cx.listener(
                                    |_, _, _, _| {
                                        NotificationStore::global().clear_all_notifications()
                                    },
                                )),
                            )
                        })
                        .child(
                            style::circle_button("notification-close", "close.svg", 30.0, theme)
                                .on_click(
                                    cx.listener(|_, _, _, cx| cx.emit(CapsuleModuleEvent::Close)),
                                ),
                        ),
                ),
        );
        let mut list = div()
            .id("notification-history")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(10.0));
        if module.items.is_empty() {
            list = list.child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .child(
                        svg()
                            .path("bell.svg")
                            .size(px(28.0))
                            .text_color(theme.foreground_muted()),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.foreground_muted())
                            .child(module.text("dashboard.no_notifications", cx)),
                    ),
            );
        }
        for item in module.items.iter().rev() {
            list = list.child(card(module, item, false, theme, cx));
        }
        root.child(list).into_any_element()
    } else if let Some(item) = &module.latest {
        root.child(card(module, item, true, theme, cx))
            .into_any_element()
    } else {
        root.into_any_element()
    }
}

fn card(
    module: &NotificationModule,
    item: &NotificationItem,
    popup: bool,
    theme: &Theme,
    cx: &mut Context<NotificationModule>,
) -> AnyElement {
    let id = item.id;
    let apps = cx.global::<services::AppState>().launcher.get_apps();
    let icon_path = if item.app_icon.starts_with('/') {
        Some(PathBuf::from(&item.app_icon))
    } else {
        apps.iter()
            .find(|app| {
                app.icon_name.as_deref() == Some(item.app_icon.as_str())
                    || app.name.eq_ignore_ascii_case(&item.app_name)
                    || app.id.eq_ignore_ascii_case(&item.app_name)
            })
            .and_then(|app| app.icon_path.clone())
    };
    let icon = if let Some(path) = icon_path {
        img(path).size(px(22.0)).into_any_element()
    } else {
        svg()
            .path("bell.svg")
            .size(px(20.0))
            .text_color(theme.accent())
            .into_any_element()
    };
    let mut content = div()
        .id(format!("notification-content-{id}"))
        .w_full()
        .min_w_0()
        .when(popup, |s| s.flex_1().min_h_0().overflow_y_scroll())
        .flex()
        .flex_col()
        .gap(px(5.0))
        .child(
            div()
                .w_full()
                .flex_shrink_0()
                .text_size(px(15.0))
                .font_weight(FontWeight::SEMIBOLD)
                .line_height(px(21.0))
                .child(item.summary.clone()),
        )
        .when(!item.body.is_empty(), |s| {
            s.child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(theme.foreground_muted())
                    .child(item.body.clone()),
            )
        });
    if item.actions.iter().any(|(key, _)| key == "default") {
        content = content
            .cursor_pointer()
            .on_click(cx.listener(move |module, _, window, cx| {
                module.action(id, "default".into(), window, cx)
            }));
    }
    let mut row = div()
        .id(format!("notification-card-{id}"))
        .min_w_0()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .when(popup, |s| {
            s.flex_1().min_h_0().on_hover(
                cx.listener(|_, hovered, _, _| NotificationStore::global().set_hovered(*hovered)),
            )
        })
        .when(!popup, |s| {
            s.flex_shrink_0()
                .min_h(px(100.0))
                .p(px(14.0))
                .rounded(px(style::CARD_RADIUS))
                .bg(style::surface(theme))
        })
        .child(
            div()
                .h(px(32.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(10.0))
                .child(
                    div()
                        .size(px(32.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(style::selected(theme))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(item.app_name.clone()),
                )
                .child(
                    style::circle_button(
                        format!("notification-dismiss-{id}"),
                        "close.svg",
                        28.0,
                        theme,
                    )
                    .on_click(cx.listener(move |module, _, _, cx| {
                        cx.stop_propagation();
                        module.dismiss(id);
                    })),
                ),
        )
        .child(content);
    let replying = module.reply.as_ref().is_some_and(|reply| reply.id == id);
    let mut actions = div()
        .id(format!("notification-actions-{id}"))
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .gap(px(6.0))
        .when(popup, |s| {
            s.h(px(actions_height(item, replying, cx, &module.text_system)))
                .overflow_y_scroll()
        });
    let mut has_actions = false;
    for (key, label) in &item.actions {
        if !visible_action(key, label, replying) {
            continue;
        }
        has_actions = true;
        let label = action_label(key, label, cx);
        let key = key.clone();
        actions = actions.child(
            style::chip(
                format!("notification-action-{id}-{key}"),
                label,
                false,
                theme,
            )
            .max_w_full()
            .text_ellipsis()
            .overflow_hidden()
            .on_click(cx.listener(move |module, _, window, cx| {
                cx.stop_propagation();
                module.action(id, key.clone(), window, cx);
            })),
        );
    }
    if has_actions {
        row = row.child(actions);
    }
    if let Some(reply) = module.reply.as_ref().filter(|reply| reply.id == id) {
        let enabled = !reply.pending && !reply.text.trim().is_empty();
        row = row.child(
            div()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .h(px(36.0))
                        .px(px(12.0))
                        .rounded(px(style::INNER_RADIUS))
                        .bg(style::surface(theme))
                        .border_1()
                        .border_color(theme.accent())
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .child(div().text_size(px(13.0)).text_ellipsis().child(
                            if reply.text.is_empty() {
                                module.text("notifications_module.reply_placeholder", cx)
                            } else {
                                reply.text.clone()
                            },
                        )),
                )
                .child(
                    div()
                        .h(px(32.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            style::chip(
                                format!("notification-reply-cancel-{id}"),
                                module.text("notifications_module.cancel", cx),
                                false,
                                theme,
                            )
                            .on_click(cx.listener(|module, _, _, cx| module.cancel_reply(cx))),
                        )
                        .child(
                            style::primary_button(
                                format!("notification-reply-send-{id}"),
                                module.text(
                                    if reply.pending {
                                        "notifications_module.sending"
                                    } else {
                                        "notifications_module.send"
                                    },
                                    cx,
                                ),
                                enabled,
                                theme,
                            )
                            .on_click(cx.listener(|module, _, _, cx| module.send_reply(cx))),
                        ),
                )
                .when(reply.failed, |s| {
                    s.child(
                        div()
                            .text_size(px(12.0))
                            .line_height(px(18.0))
                            .text_color(theme.red())
                            .child(module.text("notifications_module.reply_failed", cx)),
                    )
                }),
        );
    }
    row.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_reserves_wrapped_summary_actions_and_reply_space() {
        gpui_platform::headless().with_assets(assets::Assets {}).run(|cx| {
            cx.set_global(Theme::default());
            let text_system = gpui::WindowTextSystem::new(cx.text_system().clone());
            let mut item = NotificationItem {
                id: 1,
                app_name: String::new(),
                app_icon: String::new(),
                summary: "Title".into(),
                body: String::new(),
                actions: vec![],
                received_at: std::time::Instant::now(),
                timeout: std::time::Duration::from_secs(5),
            };
            assert_eq!(popup_height(Some(&item), false, false, cx, &text_system), 126.0);
            item.summary = "Ajustado: subí el contador y corregí la altura de línea para centrar el número en la insignia. Binario recompilado; reinicia Capsule.".into();
            item.actions.push(("default".into(), "Abrir".into()));
            let summary_height = f32::from(measure(&item.summary, 15.0, FontWeight::SEMIBOLD, true, cx, &text_system).height);
            assert!(summary_height > 21.0);
            assert_eq!(popup_height(Some(&item), false, false, cx, &text_system), 32.0 + 32.0 + 10.0 + summary_height + 10.0 + 28.0);
            item.summary = "Title".into();
            item.body = "Long message\n".repeat(1000);
            item.actions = vec![("inline-reply".into(), "Reply".into())];
            assert_eq!(popup_height(Some(&item), false, false, cx, &text_system), 210.0);
            assert_eq!(popup_height(Some(&item), true, false, cx, &text_system), 256.0);
            assert_eq!(popup_height(Some(&item), true, true, cx, &text_system), 280.0);
            item.actions = (0..6).map(|i| (format!("action-{i}"), "Very long action label ".repeat(20))).collect();
            assert_eq!(actions_height(&item, false, cx, &text_system), 96.0);
            let app = cx.to_async();
            cx.foreground_executor().spawn(async move {
                app.background_executor().timer(std::time::Duration::from_millis(10)).await;
                app.update(|cx| cx.quit());
            }).detach();
        });
    }
}
