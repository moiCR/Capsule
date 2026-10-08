mod model;
mod worker;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::clipboard as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, Pixels, Render, ScrollHandle, Size,
    Subscription, Task, Window, prelude::*,
};
use model::ClipboardModel;
pub(crate) use model::ClipboardTab;
use services::{AppState, clipboard::ClipboardContent};
use ui::theme::Theme;
use worker::Request;

pub(crate) struct ClipboardModule {
    pub model: ClipboardModel,
    pub loading: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub pinned: Vec<String>,
    pub images: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>,
    pub scroll: ScrollHandle,
    focus: FocusHandle,
    active: bool,
    generation: u64,
    sender: tokio::sync::mpsc::UnboundedSender<Request>,
    _replies: Task<()>,
    _theme: Subscription,
    worker: tokio::task::JoinHandle<()>,
}

impl ClipboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (sender, mut receiver, worker) =
            worker::start(cx.global::<AppState>().clipboard.clone());
        let replies = cx.spawn(async move |this, cx| {
            while let Some(reply) = receiver.recv().await {
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let previous_size = module.size();
                        let error = if reply.finished || reply.error.is_some() {
                            reply.error.map(|error| module.text(&error, cx))
                        } else {
                            module.error.clone()
                        };
                        let changed = module.loading
                            || reply.finished
                            || module.error != error
                            || module.model.items != reply.items
                            || module.model.snippets != reply.snippets
                            || module.pinned != reply.pinned;
                        module.loading = false;
                        if reply.finished {
                            module.busy = false;
                        }
                        module.error = error;
                        module.pinned = reply.pinned;
                        let images_changed = module.images.len() != reply.images.len();
                        module.images = reply.images;
                        module.model.update(reply.items, reply.snippets);
                        if let Some((generation, content)) = reply.copied {
                            match content {
                                ClipboardContent::Text(text) => {
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text))
                                }
                                ClipboardContent::Image(bytes) => {
                                    let image =
                                        gpui::Image::from_bytes(gpui::ImageFormat::Png, bytes);
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_image(&image));
                                }
                            }
                            if module.active && generation == module.generation {
                                cx.emit(CapsuleModuleEvent::Close);
                            }
                        }
                        if module.active && (changed || images_changed) {
                            module
                                .scroll
                                .scroll_to_item(module.model.selected / widgets::COLUMNS);
                            if previous_size != module.size() {
                                cx.emit(CapsuleModuleEvent::SizeChanged(
                                    CapsuleModuleId::Clipboard,
                                ));
                            }
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            model: ClipboardModel::default(),
            loading: true,
            busy: false,
            error: None,
            pinned: Vec::new(),
            images: std::collections::HashMap::new(),
            scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
            active: false,
            generation: 0,
            sender,
            _replies: replies,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
            worker,
        }
    }
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            let _ = self.sender.send(Request::Active(active));
        }
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        self.model.search(String::new());
        self.loading = self.model.items.is_empty() && self.model.snippets.is_empty();
        self.error = None;
        self.scroll.scroll_to_item(0);
        let _ = self.sender.send(Request::Refresh);
        cx.notify();
    }
    pub fn text(&self, key: &str, cx: &gpui::App) -> String {
        cx.global::<AppState>().language.get(key)
    }
    pub fn search(&mut self, query: String, cx: &mut Context<Self>) {
        let size = self.size();
        self.model.search(query);
        self.scroll.scroll_to_item(0);
        if size != self.size() {
            cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Clipboard));
        }
        cx.notify();
    }
    pub fn switch_tab(&mut self, tab: ClipboardTab, cx: &mut Context<Self>) {
        if self.model.tab == tab {
            return;
        }
        let size = self.size();
        self.model.switch(tab);
        self.scroll.scroll_to_item(0);
        if size != self.size() {
            cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Clipboard));
        }
        cx.notify();
    }
    pub fn select(&mut self, index: usize, scroll: bool, cx: &mut Context<Self>) {
        if index >= self.model.filtered.len() || index == self.model.selected {
            return;
        }
        self.model.selected = index;
        if scroll {
            self.scroll.scroll_to_item(index / widgets::COLUMNS);
        }
        cx.notify();
    }
    fn request(&mut self, request: Request, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.error = None;
        if self.sender.send(request).is_ok() {
            self.busy = true;
        } else {
            self.error = Some(self.text("clipboard.action_error", cx));
        }
        cx.notify();
    }
    pub fn copy(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(&index) = self.model.filtered.get(index) else {
            return;
        };
        let request = match self.model.tab {
            ClipboardTab::History => {
                Request::Copy(self.model.items[index].clone(), self.generation)
            }
            ClipboardTab::Snippets => {
                Request::CopyText(self.model.snippets[index].content.clone(), self.generation)
            }
        };
        self.request(request, cx);
    }
    pub fn pin(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.model.tab != ClipboardTab::History {
            return;
        }
        let Some(&index) = self.model.filtered.get(index) else {
            return;
        };
        if !self.model.items[index].is_image {
            self.request(Request::Pin(self.model.items[index].clone()), cx);
        }
    }
    pub fn remove(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.model.tab != ClipboardTab::Snippets {
            return;
        }
        let Some(&index) = self.model.filtered.get(index) else {
            return;
        };
        self.request(Request::Remove(self.model.snippets[index].id.clone()), cx);
    }
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.request(Request::Clear, cx);
    }
    fn key_down(&mut self, event: &gpui::KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let key = event.keystroke.key.as_str();
        if control {
            match key {
                "u" => self.search(String::new(), cx),
                "w" => self.search(
                    self.model
                        .query
                        .trim_end()
                        .rsplit_once(' ')
                        .map(|(prefix, _)| prefix)
                        .unwrap_or("")
                        .to_string(),
                    cx,
                ),
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        let clean: String = text
                            .chars()
                            .filter(|character| !character.is_control())
                            .collect();
                        self.search(format!("{}{}", self.model.query, clean), cx);
                    }
                }
                "c" => self.copy(self.model.selected, cx),
                _ => {}
            }
            return;
        }
        match key {
            "escape" => cx.emit(CapsuleModuleEvent::Close),
            "enter" => self.copy(self.model.selected, cx),
            "tab" => self.switch_tab(
                if self.model.tab == ClipboardTab::History {
                    ClipboardTab::Snippets
                } else {
                    ClipboardTab::History
                },
                cx,
            ),
            "up" | "down" | "left" | "right" => {
                self.model.navigate(match key {
                    "up" => -(widgets::COLUMNS as i32),
                    "down" => widgets::COLUMNS as i32,
                    "left" => -1,
                    _ => 1,
                });
                self.scroll
                    .scroll_to_item(self.model.selected / widgets::COLUMNS);
                cx.notify();
            }
            "home" | "end" => self.select(
                if key == "home" {
                    0
                } else {
                    self.model.filtered.len().saturating_sub(1)
                },
                true,
                cx,
            ),
            "backspace" => {
                let mut query = self.model.query.clone();
                query.pop();
                self.search(query, cx);
            }
            _ if !event.keystroke.modifiers.alt => {
                if let Some(text) = &event.keystroke.key_char
                    && !text.chars().any(char::is_control)
                {
                    self.search(format!("{}{}", self.model.query, text), cx);
                }
            }
            _ => {}
        }
    }
}
impl Drop for ClipboardModule {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
impl EventEmitter<CapsuleModuleEvent> for ClipboardModule {}
impl CapsuleModule for ClipboardModule {
    fn size(&self) -> Size<Pixels> {
        let count = self.model.filtered.len();
        let rows = count.div_ceil(widgets::COLUMNS).min(2);
        let body = if count == 0 {
            80.0
        } else {
            rows as f32 * widgets::CARD_HEIGHT + widgets::GRID_GAP * rows.saturating_sub(1) as f32
        };
        gpui::size(
            gpui::px(widgets::WIDTH),
            gpui::px((widgets::BASE_HEIGHT + body).min(widgets::MAX_HEIGHT)),
        )
    }
}
impl Render for ClipboardModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        widgets::render(self, &theme, cx)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
    }
}
