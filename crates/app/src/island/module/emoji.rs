pub(crate) mod model;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::capsule::widgets::emoji as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render, Size,
    Subscription, Window, prelude::*, px,
};
use model::EmojiModel;
use services::EmojiService;
use ui::components::InputState;
use ui::theme::Theme;

pub(crate) struct EmojiModule {
    pub model: EmojiModel,
    pub search_input: InputState,
    service: EmojiService,
    focus: FocusHandle,
    _theme: Subscription,
}

impl EmojiModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = EmojiService::new();
        Self {
            model: EmojiModel::new(service.load_emojis()),
            search_input: InputState::new("").with_focused(true).with_max_len(120),
            service,
            focus: cx.focus_handle(),
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.search_input.clear();
        self.model.reset();
        cx.notify();
    }

    pub fn search(&mut self, query: String, cx: &mut Context<Self>) {
        if self.search_input.text() != query {
            self.search_input.set_text(&query);
        }
        self.model.search(query);
        cx.notify();
    }

    pub fn category(&mut self, category: Option<&'static str>, cx: &mut Context<Self>) {
        self.model.set_category(category);
        cx.notify();
    }

    pub fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.model.filtered.len() && self.model.selected != index {
            self.model.selected = index;
            cx.notify();
        }
    }

    pub fn navigate(&mut self, key: &str, cx: &mut Context<Self>) {
        let previous = self.model.selected;
        self.model.navigate(key);
        if previous != self.model.selected {
            cx.notify();
        }
    }

    pub fn copy_selected(&mut self, cx: &mut Context<Self>) {
        let Some(item) = self.model.selected_item() else {
            return;
        };
        let emoji = item.emoji.clone();
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji.clone()));
        let service = self.service.clone();
        services::spawn_blocking(move || service.copy_emoji(&emoji));
        cx.emit(CapsuleModuleEvent::Close);
    }

    fn key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let modifiers = &event.keystroke.modifiers;
        let control = modifiers.control || modifiers.platform;
        let shift = modifiers.shift;

        if key == "escape" {
            if !self.search_input.is_empty() {
                self.search_input.clear();
                self.search(String::new(), cx);
            } else {
                cx.emit(CapsuleModuleEvent::Close);
            }
            cx.stop_propagation();
            return;
        }

        if key == "enter" {
            self.copy_selected(cx);
            cx.stop_propagation();
            return;
        }

        if (key == "up" || key == "down" || key == "pageup" || key == "pagedown") && !control && !shift {
            self.navigate(key, cx);
            cx.stop_propagation();
            return;
        }

        if (key == "left" || key == "right" || key == "home" || key == "end")
            && !control
            && !shift
            && self.search_input.is_empty()
        {
            self.navigate(key, cx);
            cx.stop_propagation();
            return;
        }

        if self.search_input.handle_key_down(event, cx) {
            let text = self.search_input.text();
            self.search(text, cx);
        }
        cx.stop_propagation();
    }
}

impl EventEmitter<CapsuleModuleEvent> for EmojiModule {}

impl CapsuleModule for EmojiModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(px(widgets::WIDTH), px(widgets::HEIGHT))
    }
}

impl Render for EmojiModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(self, cx)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
    }
}
