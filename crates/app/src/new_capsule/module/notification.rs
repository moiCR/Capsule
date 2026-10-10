use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::notification as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, KeyDownEvent, Pixels, Render, Size, Subscription, Task,
    Window, prelude::*,
};
use services::{AppState, NotificationItem, NotificationStore};
use std::time::Duration;
use ui::theme::Theme;

pub(crate) struct NotificationModule {
    pub items: Vec<NotificationItem>,
    pub latest: Option<NotificationItem>,
    pub history: bool,
    pub reply: Option<Reply>,
    active: bool,
    popup_height: f32,
    pub text_system: gpui::WindowTextSystem,
    focus: FocusHandle,
    _events: Task<()>,
    expiry: Option<Task<()>>,
    response: Option<Task<()>>,
    _theme: Subscription,
}

pub(crate) struct Reply {
    pub id: u32,
    pub text: String,
    pub pending: bool,
    pub failed: bool,
}

impl NotificationModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let store = NotificationStore::global();
        let mut changes = store.subscribe();
        let events = cx.spawn(async move |this, cx| {
            while changes.changed().await.is_ok() {
                if this
                    .update(cx, |module: &mut Self, cx| module.refresh(cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        let latest = store.get_latest_active_notification();
        let text_system = gpui::WindowTextSystem::new(cx.text_system().clone());
        let popup_height = widgets::popup_height(latest.as_ref(), false, false, cx, &text_system);
        Self {
            items: store.get_all_notifications(),
            latest,
            history: false,
            reply: None,
            active: false,
            popup_height,
            text_system,
            focus: cx.focus_handle(),
            _events: events,
            expiry: None,
            response: None,
            _theme: cx.observe_global::<Theme>(|module, cx| {
                module.update_popup_height(cx);
                cx.notify();
            }),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    fn update_popup_height(&mut self, cx: &mut Context<Self>) {
        let reply = self
            .reply
            .as_ref()
            .filter(|reply| self.latest.as_ref().is_some_and(|item| item.id == reply.id));
        let height = widgets::popup_height(
            self.latest.as_ref(),
            reply.is_some(),
            reply.is_some_and(|reply| reply.failed),
            cx,
            &self.text_system,
        );
        if self.popup_height != height {
            self.popup_height = height;
            if self.active && !self.history {
                cx.emit(CapsuleModuleEvent::SizeChanged(
                    CapsuleModuleId::Notification,
                ));
            }
        }
    }

    pub fn text(&self, key: &str, cx: &gpui::App) -> String {
        cx.global::<AppState>().language.get(key)
    }

    pub fn open(&mut self, history: bool, cx: &mut Context<Self>) {
        self.history = history;
        self.active = true;
        self.items = NotificationStore::global().get_all_notifications();
        self.latest = NotificationStore::global().get_latest_active_notification();
        self.update_popup_height(cx);
        self.schedule_expiry(cx);
        cx.notify();
    }

    pub fn set_active(&mut self, active: bool) {
        if self.active && !active {
            self.expiry = None;
            self.reply = None;
            self.response = None;
            NotificationStore::global().set_hovered(false);
        }
        self.active = active;
    }

    fn refresh(&mut self, cx: &mut Context<Self>) {
        let previous_id = self.latest.as_ref().map(|item| item.id);
        let store = NotificationStore::global();
        self.items = store.get_all_notifications();
        self.latest = store.get_latest_active_notification();
        if self.reply.as_ref().is_some_and(|reply| {
            !store.contains_notification(reply.id)
                || (!self.history && self.latest.as_ref().map(|item| item.id) != Some(reply.id))
        }) {
            self.reply = None;
            self.response = None;
            store.set_hovered(false);
        }
        self.schedule_expiry(cx);
        self.update_popup_height(cx);
        let next_id = self.latest.as_ref().map(|item| item.id);
        if previous_id != next_id {
            cx.emit(CapsuleModuleEvent::NotificationChanged);
        }
        cx.notify();
    }

    fn schedule_expiry(&mut self, cx: &mut Context<Self>) {
        self.expiry = None;
        if !self.active || self.history || self.reply.is_some() {
            return;
        }
        let Some(item) = &self.latest else {
            return;
        };
        let remaining = item.timeout.saturating_sub(item.received_at.elapsed());
        let delay = if remaining.is_zero() {
            Duration::from_secs(60)
        } else {
            remaining
        };
        self.expiry = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, |module, cx| module.refresh(cx));
        }));
    }

    pub fn dismiss(&mut self, id: u32) {
        NotificationStore::global().remove_notification(id);
    }

    pub fn action(&mut self, id: u32, key: String, window: &mut Window, cx: &mut Context<Self>) {
        if key != "inline-reply" {
            NotificationStore::global().invoke_action(id, key);
            return;
        }
        if !self
            .items
            .iter()
            .any(|item| item.id == id && item.actions.iter().any(|(key, _)| key == "inline-reply"))
        {
            return;
        }
        self.reply = Some(Reply {
            id,
            text: String::new(),
            pending: false,
            failed: false,
        });
        NotificationStore::global().set_hovered(true);
        window.focus(&self.focus, cx);
        cx.global::<AppState>().compositor.request_layer_focus();
        self.expiry = None;
        self.update_popup_height(cx);
        cx.notify();
    }

    pub fn cancel_reply(&mut self, cx: &mut Context<Self>) {
        if self.reply.as_ref().is_some_and(|reply| reply.pending) {
            return;
        }
        self.reply = None;
        self.update_popup_height(cx);
        NotificationStore::global().set_hovered(false);
        self.schedule_expiry(cx);
        cx.notify();
    }

    pub fn send_reply(&mut self, cx: &mut Context<Self>) {
        let Some(reply) = self
            .reply
            .as_mut()
            .filter(|reply| !reply.pending && !reply.text.trim().is_empty())
        else {
            return;
        };
        let id = reply.id;
        let text = reply.text.clone();
        reply.pending = true;
        reply.failed = false;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        services::spawn_tokio(async move {
            let result = NotificationStore::global().reply(id, text).await;
            let _ = sender.send(result);
        });
        self.response = Some(cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |module, cx| {
                if let Some(reply) = module.reply.as_mut().filter(|reply| reply.id == id) {
                    reply.pending = false;
                    reply.failed = !matches!(result, Ok(Ok(())));
                    if !reply.failed {
                        module.reply = None;
                    }
                    module.update_popup_height(cx);
                    cx.notify();
                }
            });
        }));
        cx.notify();
    }

    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            if self.reply.is_some() {
                self.cancel_reply(cx);
            } else {
                cx.emit(CapsuleModuleEvent::Close);
            }
            cx.stop_propagation();
            return;
        }
        if key == "enter" && self.reply.is_some() {
            self.send_reply(cx);
            cx.stop_propagation();
            return;
        }
        let Some(reply) = self.reply.as_mut().filter(|reply| !reply.pending) else {
            return;
        };
        let modifiers = &event.keystroke.modifiers;
        if modifiers.control || modifiers.platform {
            if key == "v" {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    reply
                        .text
                        .extend(text.chars().filter(|ch| !ch.is_control()));
                }
            } else if key == "backspace" {
                reply.text.clear();
            }
        } else if key == "backspace" {
            reply.text.pop();
        } else if key == "space" {
            reply.text.push(' ');
        } else if !modifiers.alt
            && let Some(text) = &event.keystroke.key_char
        {
            reply
                .text
                .extend(text.chars().filter(|ch| !ch.is_control()));
        }
        reply.failed = false;
        self.update_popup_height(cx);
        cx.stop_propagation();
        cx.notify();
    }
}

impl EventEmitter<CapsuleModuleEvent> for NotificationModule {}
impl CapsuleModule for NotificationModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(
            gpui::px(widgets::WIDTH),
            gpui::px(if self.history {
                widgets::HISTORY_HEIGHT
            } else {
                self.popup_height
            }),
        )
    }
}
impl Render for NotificationModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        gpui::div()
            .id("notification-module")
            .size_full()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .child(widgets::render(self, &theme, cx))
    }
}
