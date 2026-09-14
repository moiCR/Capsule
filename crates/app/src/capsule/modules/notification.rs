use gpui::{
    Context, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render, Window, canvas, div,
    prelude::*, px, svg,
};
use services::{AppState, NotificationItem, NotificationStore};
use std::time::Instant;
use tokio::sync::oneshot;
use ui::theme::Theme;

pub const MAX_NOTIFICATION_WIDTH: f32 = 480.0;

pub struct NotificationModule {
    active_item: Option<NotificationItem>,
    expanded: bool,
    hovered: bool,
    reply: Option<ReplyState>,
    measured_dimensions: (f32, f32),
}

struct ReplyState {
    text: String,
    focus: FocusHandle,
    started_at: Instant,
    pending: Option<oneshot::Receiver<Result<(), String>>>,
    failed: bool,
}

impl NotificationModule {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            active_item: None,
            expanded: false,
            hovered: false,
            reply: None,
            measured_dimensions: (348.0, 68.0),
        }
    }

    pub fn set_expanded(&mut self, expanded: bool, cx: &mut Context<Self>) {
        self.hovered = expanded;
        if self.is_replying() && !expanded {
            return;
        }
        if self.expanded != expanded {
            self.expanded = expanded;
            NotificationStore::global().set_hovered(expanded);
            cx.notify();
        }
    }

    pub fn is_replying(&self) -> bool {
        self.reply.is_some()
    }

    pub fn deactivate(&mut self, cx: &mut Context<Self>) {
        self.reply = None;
        self.set_expanded(false, cx);
        cx.notify();
    }

    pub fn activate_action(
        &mut self,
        id: u32,
        key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(item) = self.active_item.as_ref().filter(|item| item.id == id) else {
            return;
        };
        if !item.actions.iter().any(|(action, _)| action == key) {
            return;
        }
        if key != "inline-reply" {
            NotificationStore::global().invoke_action(id, key.to_owned());
            return;
        }
        let focus = cx.focus_handle();
        window.activate_window();
        window.focus(&focus, cx);
        if cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.request_layer_focus();
        }
        self.reply = Some(ReplyState {
            text: String::new(),
            focus,
            started_at: Instant::now(),
            pending: None,
            failed: false,
        });
        self.expanded = true;
        NotificationStore::global().set_hovered(true);
        cx.notify();
    }

    pub fn cancel_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .reply
            .as_ref()
            .is_some_and(|reply| reply.pending.is_some())
        {
            return;
        }
        self.reply = None;
        self.set_expanded(self.hovered, cx);
        window.blur(cx);
        cx.notify();
    }

    pub fn send_reply(&mut self, cx: &mut Context<Self>) {
        let Some(item) = &self.active_item else {
            return;
        };
        let Some(reply) = self.reply.as_mut() else {
            return;
        };
        if reply.text.trim().is_empty() || reply.pending.is_some() {
            return;
        }
        let id = item.id;
        let text = reply.text.clone();
        let (sender, receiver) = oneshot::channel();
        reply.pending = Some(receiver);
        reply.failed = false;
        tokio::spawn(async move {
            let result = NotificationStore::global()
                .reply(id, text)
                .await
                .map_err(|error| error.to_string());
            let _ = sender.send(result);
        });
        cx.notify();
    }

    pub fn poll_reply(&mut self, cx: &mut Context<Self>) {
        if self.is_replying()
            && !self
                .active_item
                .as_ref()
                .is_some_and(|item| NotificationStore::global().contains_notification(item.id))
        {
            self.deactivate(cx);
            return;
        }
        let Some(reply) = self.reply.as_mut() else {
            return;
        };
        let Some(receiver) = reply.pending.as_mut() else {
            return;
        };
        match receiver.try_recv() {
            Ok(Ok(())) => {
                self.reply = None;
                cx.notify();
            }
            Ok(Err(error)) => {
                eprintln!("Failed to send notification reply: {error}");
                reply.pending = None;
                reply.failed = true;
                cx.notify();
            }
            Err(oneshot::error::TryRecvError::Closed) => {
                reply.pending = None;
                reply.failed = true;
                cx.notify();
            }
            Err(oneshot::error::TryRecvError::Empty) => {}
        }
    }

    fn handle_reply_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_replying() {
            return;
        }
        cx.stop_propagation();
        match event.keystroke.key.as_str() {
            "escape" => self.cancel_reply(window, cx),
            "enter" => self.send_reply(cx),
            _ => {
                let Some(reply) = self.reply.as_mut().filter(|reply| reply.pending.is_none())
                else {
                    return;
                };
                let modifiers = &event.keystroke.modifiers;
                if modifiers.control || modifiers.platform {
                    match event.keystroke.key.as_str() {
                        "v" => {
                            if let Some(text) =
                                cx.read_from_clipboard().and_then(|item| item.text())
                            {
                                reply
                                    .text
                                    .extend(text.chars().filter(|ch| !ch.is_control()));
                            }
                        }
                        "backspace" => reply.text.clear(),
                        _ => return,
                    }
                } else if event.keystroke.key == "backspace" {
                    reply.text.pop();
                } else if !modifiers.alt
                    && let Some(text) = &event.keystroke.key_char
                {
                    reply
                        .text
                        .extend(text.chars().filter(|ch| !ch.is_control()));
                }
                reply.failed = false;
                cx.notify();
            }
        }
    }

    pub fn desired_dimensions(&self) -> (f32, f32) {
        self.measured_dimensions
    }

    fn update_measured_dimensions(&mut self, width: f32, height: f32, cx: &mut Context<Self>) {
        if (width - self.measured_dimensions.0).abs() > 0.5
            || (height - self.measured_dimensions.1).abs() > 0.5
        {
            self.measured_dimensions = (width, height);
            cx.notify();
        }
    }

    fn has_actions(&self) -> bool {
        self.active_item.as_ref().is_some_and(|item| {
            crate::capsule::widgets::notification::visible_actions(item)
                .next()
                .is_some()
        })
    }

    pub fn set_item(&mut self, item: Option<NotificationItem>, cx: &mut Context<Self>) {
        if self.active_item != item {
            if self.active_item.as_ref().map(|item| item.id) != item.as_ref().map(|item| item.id) {
                self.reply = None;
            }
            self.active_item = item;
            if self.active_item.is_none() {
                self.expanded = false;
            }
            NotificationStore::global().set_hovered(self.expanded);
            cx.notify();
        }
    }
}

impl Render for NotificationModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let (def_app, def_summary, def_body) = if cx.has_global::<AppState>() {
            let lang = &cx.global::<AppState>().language;
            (
                lang.get("notifications_module.default_title"),
                lang.get("notifications_module.default_summary"),
                lang.get("notifications_module.default_body"),
            )
        } else {
            (
                "Notificación".to_string(),
                "Resumen".to_string(),
                "Mensaje".to_string(),
            )
        };

        let (app_name, summary, body) = if let Some(item) = &self.active_item {
            (
                if item.app_name.is_empty() {
                    def_app
                } else {
                    item.app_name.clone()
                },
                item.summary.clone(),
                item.body.clone(),
            )
        } else {
            (def_app, def_summary, def_body)
        };

        let entity = cx.entity().downgrade();
        let content = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .px(px(14.0))
            .py(px(10.0))
            .gap(px(12.0))
            .child(
                div()
                    .flex_shrink_0()
                    .w(px(32.0))
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("bell.svg")
                            .size(px(16.0))
                            .text_color(theme.accent()),
                    ),
            )
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
                            .when(!self.expanded, |el| el.whitespace_nowrap().text_ellipsis())
                            .when(self.expanded, |el| el.max_h(px(54.0)))
                            .child(body)
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    }),
            );

        div()
            .id("notification-module")
            .when_some(
                self.reply.as_ref().map(|reply| reply.focus.clone()),
                |el, focus| {
                    el.track_focus(&focus)
                        .on_key_down(cx.listener(Self::handle_reply_key))
                },
            )
            .relative()
            .min_w(px(348.0))
            .max_w(px(MAX_NOTIFICATION_WIDTH))
            .when(!self.expanded, |el| el.w(px(348.0)))
            .min_h(px(68.0))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(content)
            .child(
                canvas(
                    move |bounds, _, cx| {
                        let width = f32::from(bounds.size.width);
                        let height = f32::from(bounds.size.height);
                        let entity = entity.clone();
                        cx.defer(move |cx| {
                            let _ = entity.update(cx, |module, cx| {
                                module.update_measured_dimensions(width, height, cx)
                            });
                        });
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .inset_0(),
            )
            .when(self.expanded && self.has_actions(), |el| {
                if let Some(reply) = &self.reply {
                    let duration = if cx.has_global::<AppState>() {
                        cx.global::<AppState>()
                            .config
                            .get()
                            .ui
                            .animation_duration_ms as f32
                            / 1000.0
                    } else {
                        0.2
                    };
                    let progress =
                        (reply.started_at.elapsed().as_secs_f32() / duration.max(0.001)).min(1.0);
                    if progress < 1.0 || reply.pending.is_some() {
                        window.request_animation_frame();
                    }
                    el.child(crate::capsule::widgets::notification::render_reply(
                        &reply.text,
                        reply.pending.is_some(),
                        reply.failed,
                        progress,
                        cx,
                    ))
                } else {
                    el.child(crate::capsule::widgets::notification::render_actions(
                        self.active_item.as_ref(),
                        cx,
                    ))
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_without_actions_keeps_compact_dimensions() {
        let module = NotificationModule {
            active_item: None,
            expanded: true,
            hovered: true,
            reply: None,
            measured_dimensions: (348.0, 68.0),
        };
        assert_eq!(module.desired_dimensions(), (348.0, 68.0));
    }

    #[test]
    fn hover_does_not_guess_content_dimensions() {
        let mut module = NotificationModule {
            active_item: Some(NotificationItem {
                id: 1,
                app_name: String::new(),
                app_icon: String::new(),
                summary: String::new(),
                body: String::new(),
                actions: vec![("reply".into(), "Responder".into())],
                received_at: Instant::now(),
                timeout: std::time::Duration::from_secs(5),
            }),
            expanded: false,
            hovered: false,
            reply: None,
            measured_dimensions: (348.0, 68.0),
        };
        assert_eq!(module.desired_dimensions(), (348.0, 68.0));
        module.expanded = true;
        assert_eq!(module.desired_dimensions(), (348.0, 68.0));
        module.measured_dimensions = (382.0, 112.0);
        assert_eq!(module.desired_dimensions(), (382.0, 112.0));
        if let Some(item) = module.active_item.as_mut() {
            item.actions.clear();
        }
        assert_eq!(module.desired_dimensions(), (382.0, 112.0));
    }

    #[test]
    fn notification_layout_reports_content_size_without_polling() {
        use std::sync::{Arc, Mutex};
        struct LayoutView(gpui::Entity<NotificationModule>);
        impl Render for LayoutView {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .w(px(480.0))
                    .flex()
                    .items_start()
                    .child(self.0.clone())
            }
        }
        let measurements = Arc::new(Mutex::new(Vec::new()));
        let output = measurements.clone();
        gpui_platform::headless().with_assets(assets::Assets {}).run(move |cx| {
            cx.set_global(Theme::default());
            let window = cx.open_window(gpui::WindowOptions::default(), |_, cx| {
                let module = cx.new(NotificationModule::new);
                module.update(cx, |module, _| {
                    module.expanded = true;
                    module.active_item = Some(NotificationItem {
                        id: 1, app_name: "Test".into(), app_icon: String::new(), summary: "Short".into(), body: "Message".into(), actions: Vec::new(), received_at: Instant::now(), timeout: std::time::Duration::from_secs(5),
                    });
                });
                cx.new(|_| LayoutView(module))
            }).expect("headless window");
            let mut app = cx.to_async();
            cx.foreground_executor().spawn(async move {
                for long in [false, true, false] {
                    app.update_window(window.into(), |root, window, cx| {
                        let root = root.downcast::<LayoutView>().unwrap();
                        root.read(cx).0.clone().update(cx, |module, cx| {
                            if let Some(item) = module.active_item.as_mut() {
                                item.summary = if long { "A longer notification title that needs more horizontal room".into() } else { "Short".into() };
                                item.body = if long { "Long notification content with enough words to wrap into multiple lines inside the bounded notification capsule.".into() } else { "Message".into() };
                            }
                            cx.notify();
                        });
                        window.draw(cx).clear(cx);
                    }).unwrap();
                    app.update_window(window.into(), |root, _, cx| {
                        let root = root.downcast::<LayoutView>().unwrap();
                        output.lock().unwrap().push(root.read(cx).0.read(cx).desired_dimensions());
                    }).unwrap();
                }
                app.update(|cx| cx.quit());
            }).detach();
        });
        let sizes = measurements.lock().unwrap();
        assert_eq!(sizes.len(), 3);
        assert!(sizes[1].0 > sizes[0].0, "widths: {sizes:?}");
        assert!(sizes[1].1 > sizes[0].1, "heights: {sizes:?}");
        assert_eq!(sizes[0], sizes[2]);
        assert!(sizes[1].0 <= 480.0);
    }

    #[test]
    fn inline_reply_keeps_focus_and_draft_until_cancelled() {
        struct EmptyView;
        impl Render for EmptyView {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
            }
        }
        gpui_platform::headless().run(|cx| {
            let _window = cx
                .open_window(gpui::WindowOptions::default(), |window, cx| {
                    let module = cx.new(NotificationModule::new);
                    module.update(cx, |module, cx| {
                        let store = NotificationStore::global();
                        let id = store.add_notification(
                            "Test".into(),
                            String::new(),
                            "Title".into(),
                            String::new(),
                            5000,
                        );
                        let mut item = store
                            .get_latest_active_notification()
                            .expect("test notification");
                        item.actions = vec![
                            ("reply".into(), "Reply".into()),
                            ("inline-reply".into(), "Reply".into()),
                        ];
                        module.set_item(Some(item), cx);
                        module.activate_action(id, "reply", window, cx);
                        assert!(!module.is_replying());
                        module.activate_action(id, "inline-reply", window, cx);
                        assert!(module.is_replying());
                        assert!(module.reply.as_ref().unwrap().focus.is_focused(window));
                        module.set_expanded(false, cx);
                        assert!(module.expanded);
                        module.send_reply(cx);
                        assert!(module.reply.as_ref().unwrap().pending.is_none());
                        let (sender, receiver) = oneshot::channel();
                        let reply = module.reply.as_mut().unwrap();
                        reply.text = "Hello 🌍".into();
                        reply.pending = Some(receiver);
                        module.send_reply(cx);
                        module.cancel_reply(window, cx);
                        assert!(module.is_replying());
                        sender.send(Err("test failure".into())).unwrap();
                        module.poll_reply(cx);
                        let reply = module.reply.as_ref().unwrap();
                        assert!(reply.failed);
                        assert_eq!(reply.text, "Hello 🌍");
                        module.cancel_reply(window, cx);
                        assert!(!module.is_replying());
                        assert!(!module.expanded);
                        store.remove_notification(id);
                    });
                    cx.new(|_| EmptyView)
                })
                .expect("headless window");
            let app = cx.to_async();
            cx.foreground_executor()
                .spawn(async move {
                    app.update(|cx| cx.quit());
                })
                .detach();
        });
    }
}
