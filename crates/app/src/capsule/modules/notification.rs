use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*, px, svg};
use services::{NotificationItem, NotificationStore};
use ui::theme::Theme;

pub struct NotificationModule {
    active_item: Option<NotificationItem>,
}

impl NotificationModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.spawn(async move |this, cx| {
            loop {
                let store = NotificationStore::global();
                let latest = store.get_latest_active_notification();

                let res = this.update(cx, |this: &mut Self, cx| {
                    if this.active_item != latest {
                        this.active_item = latest;
                        cx.notify();
                    }
                });
                if res.is_err() {
                    break;
                }

                cx.background_executor()
                    .timer(std::time::Duration::from_millis(150))
                    .await;
            }
        })
        .detach();

        Self { active_item: None }
    }

    #[allow(dead_code)]
    pub fn set_item(&mut self, item: Option<NotificationItem>, cx: &mut Context<Self>) {
        if self.active_item != item {
            self.active_item = item;
            cx.notify();
        }
    }
}

impl Render for NotificationModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let (app_name, summary, body) = if let Some(item) = &self.active_item {
            (
                if item.app_name.is_empty() {
                    "Notificación".to_string()
                } else {
                    item.app_name.clone()
                },
                item.summary.clone(),
                item.body.clone(),
            )
        } else {
            (
                "Notificación".to_string(),
                "Resumen".to_string(),
                "Mensaje".to_string(),
            )
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h_full()
            .px(px(14.0))
            .py(px(10.0))
            .gap(px(12.0))
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(32.0))
                    .h(px(32.0))
                    .rounded_full()
                    .bg(theme.accent().opacity(0.2))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("info.svg")
                            .size(px(16.0))
                            .text_color(theme.accent()),
                    ),
            )
            // Right text column (App Name, Summary, Body)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .overflow_hidden()
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.foreground_muted())
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(app_name),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.foreground())
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(summary),
                    )
                    .child(if !body.is_empty() {
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .overflow_hidden()
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(body)
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    }),
            )
    }
}
