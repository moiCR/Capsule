pub(crate) mod model;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::new_capsule::widgets::emoji as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render, Size,
    Subscription, Window, prelude::*, px,
};
use model::EmojiModel;
use services::EmojiService;
use ui::theme::Theme;

pub(crate) struct EmojiModule {
    pub model: EmojiModel,
    service: EmojiService,
    focus: FocusHandle,
    _theme: Subscription,
}

impl EmojiModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = EmojiService::new();
        Self {
            model: EmojiModel::new(service.load_emojis()),
            service,
            focus: cx.focus_handle(),
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.model.reset();
        cx.notify();
    }

    pub fn search(&mut self, query: String, cx: &mut Context<Self>) {
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
        if key == "escape" {
            cx.emit(CapsuleModuleEvent::Close);
        } else if control {
            match key {
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        let clean: String = text.chars().filter(|ch| !ch.is_control()).collect();
                        self.search(format!("{}{}", self.model.query, clean), cx);
                    }
                }
                "u" => self.search(String::new(), cx),
                "w" => {
                    let query = self
                        .model
                        .query
                        .trim_end()
                        .rsplit_once(' ')
                        .map_or("", |(prefix, _)| prefix)
                        .to_owned();
                    self.search(query, cx);
                }
                _ => {}
            }
        } else {
            match key {
                "enter" => self.copy_selected(cx),
                "left" | "right" | "up" | "down" | "home" | "end" | "pageup" | "pagedown" => {
                    self.navigate(key, cx)
                }
                "backspace" => {
                    let mut query = self.model.query.clone();
                    query.pop();
                    self.search(query, cx);
                }
                "space" => self.search(format!("{} ", self.model.query), cx),
                _ if !modifiers.alt => {
                    if let Some(text) = &event.keystroke.key_char
                        && !text.chars().any(char::is_control)
                    {
                        self.search(format!("{}{}", self.model.query, text), cx);
                    }
                }
                _ => {}
            }
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
