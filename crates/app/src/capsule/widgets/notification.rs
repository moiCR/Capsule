use crate::capsule::modules::notification::NotificationModule;
use gpui::{Context, IntoElement, div, prelude::*, px};
use services::{AppState, NotificationItem};
use ui::components::input::TextInputField;
use ui::theme::Theme;

pub fn visible_actions(item: &NotificationItem) -> impl Iterator<Item = &(String, String)> {
    item.actions.iter().filter(|(key, label)| {
        !key.trim().is_empty()
            && (key == "default" || key == "inline-reply" || !label.trim().is_empty())
    })
}

fn action_label<'a>(key: &str, label: &'a str, open_label: &'a str) -> &'a str {
    if key == "default" && label.trim().is_empty() {
        open_label
    } else {
        label.trim()
    }
}

pub fn render_actions(
    item: Option<&NotificationItem>,
    cx: &Context<NotificationModule>,
) -> impl IntoElement {
    let open_label = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .language
            .get("notifications_module.open")
    } else {
        "Open".to_string()
    };
    let reply_label = cx
        .global::<AppState>()
        .language
        .get("notifications_module.reply");
    let theme = cx.global::<Theme>();
    let foreground = theme.foreground_muted();
    let accent = theme.accent();
    let surface = theme.surface();
    div()
        .id("notification-actions")
        .max_h(px(76.0))
        .ml(px(20.0))
        .mr(px(20.0))
        .pb(px(10.0))
        .flex()
        .flex_wrap()
        .justify_end()
        .items_center()
        .gap(px(4.0))
        .overflow_y_scroll()
        .children(item.into_iter().flat_map(|item| {
            let open_label = &open_label;
            let reply_label = &reply_label;
            visible_actions(item)
                .enumerate()
                .map(move |(index, (key, label))| {
                    let id = item.id;
                    let key = key.clone();
                    div()
                        .id(("notification-action", index))
                        .max_w_full()
                        .px(px(10.0))
                        .py(px(5.0))
                        .rounded(px(12.0))
                        .text_size(px(11.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(foreground)
                        .cursor_pointer()
                        .hover(move |style| style.bg(surface.opacity(0.5)).text_color(accent))
                        .active(move |style| style.bg(surface).text_color(accent))
                        .child(
                            div()
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(if key == "inline-reply" && label.trim().is_empty() {
                                    reply_label.clone()
                                } else {
                                    action_label(&key, label, open_label).to_owned()
                                }),
                        )
                        .on_click(cx.listener(move |module, _, window, cx| {
                            cx.stop_propagation();
                            module.activate_action(id, &key, window, cx);
                        }))
                })
        }))
}

pub fn render_reply(
    text: &str,
    pending: bool,
    failed: bool,
    progress: f32,
    cx: &Context<NotificationModule>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let language = &cx.global::<AppState>().language;
    let eased = 1.0 - (1.0 - progress).powi(3);
    let enabled = !pending && !text.trim().is_empty();
    div()
        .mx(px(20.0))
        .pb(px(10.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .relative()
                .top(px((1.0 - eased) * 6.0))
                .opacity(eased)
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(
                    div().flex_1().min_w_0().overflow_hidden().child(
                        TextInputField::new("notification-reply-input", text.to_owned())
                            .placeholder(language.get("notifications_module.reply_placeholder")),
                    ),
                )
                .child(
                    div()
                        .id("notification-reply-send")
                        .px(px(8.0))
                        .py(px(6.0))
                        .text_size(px(11.0))
                        .text_color(if enabled {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        })
                        .opacity(if enabled { 1.0 } else { 0.5 })
                        .when(enabled, |el| {
                            el.cursor_pointer()
                                .on_click(cx.listener(|module, _, _, cx| {
                                    cx.stop_propagation();
                                    module.send_reply(cx);
                                }))
                        })
                        .child(language.get(if pending {
                            "notifications_module.sending"
                        } else {
                            "notifications_module.send"
                        })),
                )
                .when(!pending, |el| {
                    el.child(
                        div()
                            .id("notification-reply-cancel")
                            .px(px(6.0))
                            .py(px(6.0))
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .cursor_pointer()
                            .on_click(cx.listener(|module, _, window, cx| {
                                cx.stop_propagation();
                                module.cancel_reply(window, cx);
                            }))
                            .child(language.get("notifications_module.cancel")),
                    )
                }),
        )
        .when(failed, |el| {
            el.child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.red())
                    .child(language.get("notifications_module.reply_failed")),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_action_without_label_has_open_label() {
        assert_eq!(action_label("default", "", "Abrir"), "Abrir");
        assert_eq!(action_label("default", " \n ", "Open"), "Open");
    }

    #[test]
    fn application_labels_are_preserved() {
        assert_eq!(action_label("reply", "Responder", "Abrir"), "Responder");
        assert_eq!(action_label("default", "Abrir chat", "Abrir"), "Abrir chat");
    }
}
