use gpui::{
    Context, EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    ScrollHandle, Window, div, prelude::*, px, svg,
};
use services::{ClipboardItem, ClipboardService};
use ui::theme::Theme;

pub enum ClipboardEvent {
    Close,
}

pub struct ClipboardModule {
    service: ClipboardService,
    query: String,
    items: Vec<ClipboardItem>,
    filtered_items: Vec<ClipboardItem>,
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

        Self {
            service,
            query: String::new(),
            items: initial_items.clone(),
            filtered_items: initial_items,
            selected_index: 0,
            focus_handle,
            scroll_handle,
            mouse_moved: false,
        }
    }

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
        self.items = self.service.fetch_history();
        self.filter_items();
        self.selected_index = 0;
        self.mouse_moved = false;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn filter_items(&mut self) {
        if self.query.trim().is_empty() {
            self.filtered_items = self.items.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_items = self
                .items
                .iter()
                .filter(|item| item.preview.to_lowercase().contains(&q))
                .cloned()
                .collect();
        }
    }

    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    fn update_search(&mut self, new_query: String, cx: &mut Context<Self>) {
        self.query = new_query;
        self.filter_items();
        self.selected_index = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            self.service.copy_item(item);
            cx.emit(ClipboardEvent::Close);
            true
        } else {
            false
        }
    }

    fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.service.clear_history();
        self.reload_items(cx);
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
        let (search_placeholder, empty_item, empty_history) =
            if cx.has_global::<services::AppState>() {
                let lang = &cx.global::<services::AppState>().language;
                (
                    lang.get("clipboard.search_placeholder"),
                    lang.get("clipboard.empty_item"),
                    lang.get("clipboard.empty_history"),
                )
            } else {
                (
                    "Buscar en el historial...".to_string(),
                    "[Elemento vacío]".to_string(),
                    "No hay elementos en el historial".to_string(),
                )
            };

        window.focus(&self.focus_handle, cx);

        let query = self.query.clone();
        let selected_index = self.selected_index;
        let is_empty = self.filtered_items.is_empty();
        let has_query = !query.is_empty();

        let header = div()
            .flex()
            .items_center()
            .gap_3()
            .w_full()
            .px_3()
            .py_2()
            .child(
                svg()
                    .path("search.svg")
                    .w_4()
                    .h_4()
                    .text_color(theme.foreground_muted().opacity(0.8)),
            )
            .child(div().flex_1().text_sm().child(if has_query {
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child(query.clone())
            } else {
                div()
                    .text_color(theme.foreground_muted().opacity(0.7))
                    .child(search_placeholder)
            }))
            .children(if has_query {
                Some(
                    div()
                        .id("clear-search-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface().opacity(0.8)))
                        .active(|s| s.opacity(0.6))
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.update_search(String::new(), cx);
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .w_3()
                                .h_3()
                                .text_color(theme.foreground_muted()),
                        ),
                )
            } else {
                None
            })
            .child(
                div()
                    .id("clip-clear-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(22.0))
                    .h(px(22.0))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.6))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear_all(cx);
                    }))
                    .child(
                        svg()
                            .path("trash.svg")
                            .size(px(13.0))
                            .text_color(theme.foreground_muted()),
                    ),
            );

        let mut list_container = div()
            .id("clipboard-item-list")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_scroll()
            .gap_1();

        for (idx, item) in self.filtered_items.iter().enumerate() {
            let is_selected = idx == selected_index;
            let item_clone = item.clone();
            let empty_text = empty_item.clone();

            let indicator_color = if is_selected {
                theme.accent()
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };

            let item_bg = if is_selected {
                theme.surface().opacity(0.55)
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };

            let row = div()
                .id(format!("clip-item-{idx}"))
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .gap_2p5()
                .px_2()
                .py(px(6.0))
                .rounded(px(12.0))
                .cursor_pointer()
                .bg(item_bg)
                .hover(|s| s.bg(theme.surface().opacity(0.4)))
                .active(|s| s.bg(theme.surface().opacity(0.6)))
                .on_mouse_move(cx.listener(move |this, _, _, cx| {
                    if !this.mouse_moved {
                        this.mouse_moved = true;
                    }
                    if this.selected_index != idx {
                        this.selected_index = idx;
                        cx.notify();
                    }
                }))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.service.copy_item(&item_clone);
                    cx.emit(ClipboardEvent::Close);
                }))
                .child(
                    div()
                        .w(px(3.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(indicator_color),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2p5()
                        .flex_1()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .text_color(theme.foreground_muted().opacity(0.7))
                                .child(format!("#{}", idx + 1)),
                        )
                        .child(
                            div()
                                .font_weight(if is_selected {
                                    FontWeight::SEMIBOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_size(px(13.0))
                                .text_color(if is_selected {
                                    theme.foreground()
                                } else {
                                    theme.foreground_muted()
                                })
                                .truncate()
                                .child(if item.preview.is_empty() {
                                    empty_text
                                } else {
                                    item.preview.clone()
                                }),
                        ),
                );

            list_container = list_container.child(row);
        }

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
            .child(header)
            .child(if is_empty {
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
            } else {
                list_container
            })
    }
}
