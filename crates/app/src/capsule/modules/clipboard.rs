use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Render, ScrollHandle, Window,
    div, prelude::*, px, svg,
};
use services::{ClipboardItem, ClipboardService, Snippet};
use ui::theme::Theme;

use crate::capsule::widgets::clipboard::{
    header::render_header, history_item::render_history_item, snippet_item::render_snippet_item,
};

pub enum ClipboardEvent {
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardTab {
    History,
    Snippets,
}

pub struct ClipboardModule {
    pub service: ClipboardService,
    pub query: String,
    pub current_tab: ClipboardTab,
    items: Vec<ClipboardItem>,
    filtered_items: Vec<ClipboardItem>,
    snippets: Vec<Snippet>,
    filtered_snippets: Vec<Snippet>,
    pub selected_index: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pub mouse_moved: bool,
}

impl EventEmitter<ClipboardEvent> for ClipboardModule {}

impl ClipboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = ClipboardService::new();
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let initial_items = service.fetch_history();
        let initial_snippets = service.get_snippets();

        Self {
            service,
            query: String::new(),
            current_tab: ClipboardTab::History,
            items: initial_items.clone(),
            filtered_items: initial_items,
            snippets: initial_snippets.clone(),
            filtered_snippets: initial_snippets,
            selected_index: 0,
            focus_handle,
            scroll_handle,
            mouse_moved: false,
        }
    }

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
        self.items = self.service.fetch_history();
        self.snippets = self.service.get_snippets();
        self.filter_items();
        self.filter_snippets();
        self.selected_index = 0;
        self.mouse_moved = false;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab: ClipboardTab, cx: &mut Context<Self>) {
        if self.current_tab != tab {
            self.current_tab = tab;
            self.selected_index = 0;
            self.mouse_moved = false;
            self.scroll_handle.scroll_to_item(0);
            cx.notify();
        }
    }

    fn filter_items(&mut self) {
        if self.query.trim().is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| {
                    if item.is_image {
                        item.preview.to_lowercase().contains(&q)
                            || "imagen".contains(&q)
                            || "image".contains(&q)
                    } else {
                        item.preview.to_lowercase().contains(&q)
                    }
                })
                .cloned()
                .collect();
        }
    }

    fn filter_snippets(&mut self) {
        if self.query.trim().is_empty() {
            self.filtered_snippets = self.snippets.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_snippets = self
                .snippets
                .iter()
                .filter(|snippet| {
                    snippet.title.to_lowercase().contains(&q)
                        || snippet.content.to_lowercase().contains(&q)
                })
                .cloned()
                .collect();
        }
    }

    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn update_search(&mut self, new_query: String, cx: &mut Context<Self>) {
        self.query = new_query;
        self.filter_items();
        self.filter_snippets();
        self.selected_index = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn current_count(&self) -> usize {
        match self.current_tab {
            ClipboardTab::History => self.filtered_items.len(),
            ClipboardTab::Snippets => self.filtered_snippets.len(),
        }
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        let count = self.current_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn select_prev(&mut self, cx: &mut Context<Self>) {
        let count = self.current_count();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) -> bool {
        match self.current_tab {
            ClipboardTab::History => {
                if let Some(item) = self.filtered_items.get(self.selected_index) {
                    self.service.copy_item(item);
                    cx.emit(ClipboardEvent::Close);
                    true
                } else {
                    false
                }
            }
            ClipboardTab::Snippets => {
                if let Some(snippet) = self.filtered_snippets.get(self.selected_index) {
                    self.service.copy_text(&snippet.content);
                    cx.emit(ClipboardEvent::Close);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.service.clear_history();
        self.reload_items(cx);
    }

    pub fn toggle_pin_history(&mut self, item: &ClipboardItem, cx: &mut Context<Self>) {
        if self.service.is_pinned(&item.preview) {
            let id_to_remove = self
                .snippets
                .iter()
                .find(|snippet| snippet.content == item.preview)
                .map(|snippet| snippet.id.clone());
            if let Some(id) = id_to_remove {
                self.service.remove_snippet(&id);
            }
        } else {
            self.service.pin_from_history(item);
        }

        self.snippets = self.service.get_snippets();
        self.filter_snippets();
        cx.notify();
    }

    pub fn remove_snippet(&mut self, id: &str, cx: &mut Context<Self>) {
        self.service.remove_snippet(id);
        self.snippets = self.service.get_snippets();
        self.filter_snippets();
        let count = self.filtered_snippets.len();
        if self.selected_index >= count && count > 0 {
            self.selected_index = count - 1;
        }
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if ctrl {
            match key {
                "u" => {
                    self.update_search(String::new(), cx);
                    return;
                }
                "w" => {
                    let trimmed = self.query.trim_end();
                    let new_q = if let Some(idx) = trimmed.rfind(' ') {
                        trimmed[..idx].to_string()
                    } else {
                        String::new()
                    };
                    self.update_search(new_q, cx);
                    return;
                }
                _ => {}
            }
        }

        match key {
            "tab" => {
                let next_tab = match self.current_tab {
                    ClipboardTab::History => ClipboardTab::Snippets,
                    ClipboardTab::Snippets => ClipboardTab::History,
                };
                self.switch_tab(next_tab, cx);
            }
            "down" => {
                self.mouse_moved = false;
                self.select_next(cx);
            }
            "up" => {
                self.mouse_moved = false;
                self.select_prev(cx);
            }
            "enter" => {
                self.copy_selected(cx);
            }
            "escape" => {
                cx.emit(ClipboardEvent::Close);
            }
            "backspace" => {
                if !self.query.is_empty() {
                    let mut q = self.query.clone();
                    q.pop();
                    self.update_search(q, cx);
                }
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1 && !ctrl && !event.keystroke.modifiers.alt {
                    let mut q = self.query.clone();
                    q.push_str(text);
                    self.update_search(q, cx);
                }
            }
        }
    }
}

impl Render for ClipboardModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let (empty_history, empty_snippets) = if cx.has_global::<services::AppState>() {
            let lang = &cx.global::<services::AppState>().language;
            (
                lang.get("clipboard.empty_history"),
                lang.get("clipboard.empty_snippets"),
            )
        } else {
            (
                "No hay elementos en el historial".to_string(),
                "No hay plantillas fijadas".to_string(),
            )
        };

        window.focus(&self.focus_handle, cx);

        let selected_index = self.selected_index;

        let content_list = match self.current_tab {
            ClipboardTab::History => {
                if self.filtered_items.is_empty() {
                    div()
                        .id("clipboard-empty-state")
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .text_size(px(13.0))
                        .text_color(theme.foreground_muted())
                        .child(empty_history)
                        .into_any_element()
                } else {
                    let mut list = div()
                        .id("clipboard-item-list")
                        .track_scroll(&self.scroll_handle)
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_scroll()
                        .gap_1();

                    for (idx, item) in self.filtered_items.iter().enumerate() {
                        let is_selected = idx == selected_index;
                        let is_pinned = self.service.is_pinned(&item.preview);
                        list = list.child(render_history_item(
                            idx,
                            item,
                            is_selected,
                            is_pinned,
                            &theme,
                            cx,
                        ));
                    }
                    list.into_any_element()
                }
            }
            ClipboardTab::Snippets => {
                if self.filtered_snippets.is_empty() {
                    div()
                        .id("snippets-empty-state")
                        .flex()
                        .flex_col()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .py_8()
                        .gap_2()
                        .child(
                            svg()
                                .path("pin.svg")
                                .size(px(24.0))
                                .text_color(theme.foreground_muted().opacity(0.4)),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child(empty_snippets),
                        )
                        .into_any_element()
                } else {
                    let mut list = div()
                        .id("snippets-item-list")
                        .track_scroll(&self.scroll_handle)
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_scroll()
                        .gap_1();

                    for (idx, snippet) in self.filtered_snippets.iter().enumerate() {
                        let is_selected = idx == selected_index;
                        list =
                            list.child(render_snippet_item(idx, snippet, is_selected, &theme, cx));
                    }
                    list.into_any_element()
                }
            }
        };

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(380.0))
            .max_h(px(360.0))
            .p_3()
            .gap_2()
            .overflow_hidden()
            .child(render_header(&self.query, self.current_tab, &theme, cx))
            .child(content_list)
    }
}
