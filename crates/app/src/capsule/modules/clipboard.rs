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
        match event.keystroke.key.as_str() {
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
            ch if ch.len() == 1
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt
                && !event.keystroke.modifiers.platform =>
            {
                let mut q = self.query.clone();
                q.push_str(ch);
                self.update_search(q, cx);
            }
            _ => {}
        }
    }
}

impl Render for ClipboardModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let lang = if cx.has_global::<ui::language::Language>() {
            cx.global::<ui::language::Language>().clone()
        } else {
            ui::language::Language::default()
        };

        window.focus(&self.focus_handle, cx);

        let query = self.query.clone();
        let selected_index = self.selected_index;
        let is_empty = self.filtered_items.is_empty();

        let mut list_container = div()
            .id("clipboard-item-list")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_scroll()
            .gap_1p5();

        for (idx, item) in self.filtered_items.iter().enumerate() {
            let is_selected = idx == selected_index;
            let item_clone = item.clone();
            let empty_text = lang.clipboard.empty_item.clone();

            let row = div()
                .id(format!("clip-item-{idx}"))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .rounded_xl()
                .cursor_pointer()
                .bg(if is_selected {
                    theme.accent().opacity(0.2)
                } else {
                    theme.surface().opacity(0.3)
                })
                .border_1()
                .border_color(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.0)
                })
                .hover(|s| {
                    if !is_selected {
                        s.bg(theme.surface().opacity(0.6))
                    } else {
                        s
                    }
                })
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
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2p5()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.0))
                                .text_color(theme.foreground_muted())
                                .child(format!("#{}", idx + 1)),
                        )
                        .child(
                            div()
                                .font_weight(if is_selected {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_size(px(12.0))
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
            .w_full()
            .h_full()
            .p_4()
            .gap_3()
            .child(
                // Header
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                svg()
                                    .path("clipboard-list.svg")
                                    .size(px(18.0))
                                    .text_color(theme.accent()),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(13.0))
                                    .text_color(theme.foreground())
                                    .child(lang.clipboard.title),
                            ),
                    )
                    .child(
                        div()
                            .id("clip-clear-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .p_1p5()
                            .rounded_full()
                            .bg(theme.surface())
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.surface().opacity(0.8)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_all(cx);
                            }))
                            .child(
                                svg()
                                    .path("trash.svg")
                                    .size(px(13.0))
                                    .text_color(theme.foreground_muted()),
                            ),
                    ),
            )
            .child(
                // Search bar
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .rounded_xl()
                    .bg(theme.surface().opacity(0.5))
                    .border_1()
                    .border_color(theme.surface().opacity(0.8))
                    .child(
                        svg()
                            .path("search.svg")
                            .size(px(14.0))
                            .text_color(theme.foreground_muted()),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(if query.is_empty() {
                                theme.foreground_muted()
                            } else {
                                theme.foreground()
                            })
                            .child(if query.is_empty() {
                                lang.clipboard.search_placeholder
                            } else {
                                query
                            }),
                    ),
            )
            .child(
                // List of items
                if is_empty {
                    div()
                        .id("clipboard-empty-state")
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child(lang.clipboard.empty_history)
                } else {
                    list_container
                },
            )
    }
}
