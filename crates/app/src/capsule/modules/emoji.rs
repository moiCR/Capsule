use gpui::{
    Context, EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    ScrollHandle, Window, div, prelude::*, px, svg,
};
use services::{EmojiItem, EmojiService};
use ui::theme::Theme;

pub enum EmojiEvent {
    Close,
}

pub struct EmojiModule {
    service: EmojiService,
    query: String,
    category_filter: Option<String>,
    items: Vec<EmojiItem>,
    filtered_items: Vec<EmojiItem>,
    pub selected_index: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pub mouse_moved: bool,
}

impl EventEmitter<EmojiEvent> for EmojiModule {}

impl EmojiModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = EmojiService::new();
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let initial_items = service.load_emojis();

        Self {
            service,
            query: String::new(),
            category_filter: None,
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
        self.category_filter = None;
        self.items = self.service.load_emojis();
        self.filter_items();
        self.selected_index = 0;
        self.mouse_moved = false;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn filter_items(&mut self) {
        let q = self.query.trim().to_lowercase();
        let cat = self.category_filter.as_deref();

        self.filtered_items = self
            .items
            .iter()
            .filter(|item| {
                if let Some(cat_name) = cat {
                    if !item.category.eq_ignore_ascii_case(cat_name) {
                        return false;
                    }
                }

                if q.is_empty() {
                    return true;
                }

                if item.name.to_lowercase().contains(&q)
                    || item.category.to_lowercase().contains(&q)
                {
                    return true;
                }

                item.keywords.iter().any(|kw| kw.to_lowercase().contains(&q))
            })
            .cloned()
            .collect();
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

    fn set_category(&mut self, category: Option<String>, cx: &mut Context<Self>) {
        self.category_filter = category;
        self.filter_items();
        self.selected_index = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
            let row = self.selected_index / 8;
            self.scroll_handle.scroll_to_item(row);
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
            let row = self.selected_index / 8;
            self.scroll_handle.scroll_to_item(row);
            cx.notify();
        }
    }

    fn select_row_down(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            if self.selected_index + 8 < self.filtered_items.len() {
                self.selected_index += 8;
            } else {
                self.selected_index = (self.selected_index + 8) % self.filtered_items.len();
            }
            let row = self.selected_index / 8;
            self.scroll_handle.scroll_to_item(row);
            cx.notify();
        }
    }

    fn select_row_up(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            if self.selected_index >= 8 {
                self.selected_index -= 8;
            } else {
                let remainder = self.selected_index;
                let last_full = (self.filtered_items.len() / 8) * 8;
                if last_full + remainder < self.filtered_items.len() {
                    self.selected_index = last_full + remainder;
                } else if last_full >= 8 {
                    self.selected_index = last_full - 8 + remainder;
                }
            }
            let row = self.selected_index / 8;
            self.scroll_handle.scroll_to_item(row);
            cx.notify();
        }
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            self.service.copy_emoji(&item.emoji);
            cx.emit(EmojiEvent::Close);
            true
        } else {
            false
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event.keystroke.key.as_str() {
            "right" => {
                self.mouse_moved = false;
                self.select_next(cx);
            }
            "left" => {
                self.mouse_moved = false;
                self.select_prev(cx);
            }
            "down" => {
                self.mouse_moved = false;
                self.select_row_down(cx);
            }
            "up" => {
                self.mouse_moved = false;
                self.select_row_up(cx);
            }
            "enter" => {
                self.copy_selected(cx);
            }
            "escape" => {
                cx.emit(EmojiEvent::Close);
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

impl Render for EmojiModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let query = self.query.clone();
        let selected_index = self.selected_index;
        let is_empty = self.filtered_items.is_empty();

        let categories = [
            ("all", "Todos"),
            ("people", "Emociones"),
            ("nature", "Naturaleza"),
            ("food", "Comida"),
            ("activity", "Actividad"),
            ("travel", "Viajes"),
            ("objects", "Objetos"),
            ("symbols", "Símbolos"),
            ("flags", "Banderas"),
        ];

        let mut cat_bar = div().flex().flex_row().items_center().gap_1().overflow_hidden();
        for (cat_id, cat_label) in categories {
            let is_cat_active = match (cat_id, self.category_filter.as_deref()) {
                ("all", None) => true,
                (cat, Some(active)) => cat.eq_ignore_ascii_case(active),
                _ => false,
            };

            let cat_val = if cat_id == "all" {
                None
            } else {
                Some(cat_id.to_string())
            };

            cat_bar = cat_bar.child(
                div()
                    .id(format!("cat-{cat_id}"))
                    .px_2p5()
                    .py_1()
                    .rounded_lg()
                    .text_size(px(11.0))
                    .cursor_pointer()
                    .bg(if is_cat_active {
                        theme.accent()
                    } else {
                        theme.surface().opacity(0.4)
                    })
                    .text_color(if is_cat_active {
                        theme.background()
                    } else {
                        theme.foreground_muted()
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_category(cat_val.clone(), cx);
                    }))
                    .child(cat_label),
            );
        }

        let mut grid_rows = div()
            .id("emoji-grid-container")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_scroll()
            .gap_1p5();

        let chunk_size = 8;
        for (row_idx, chunk) in self.filtered_items.chunks(chunk_size).enumerate() {
            let mut row = div().flex().flex_row().items_center().justify_start().gap_1p5();

            for (col_idx, item) in chunk.iter().enumerate() {
                let global_idx = row_idx * chunk_size + col_idx;
                let is_selected = global_idx == selected_index;
                let item_emoji = item.emoji.clone();

                row = row.child(
                    div()
                        .id(format!("emoji-{global_idx}"))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h(px(46.0))
                        .rounded_xl()
                        .cursor_pointer()
                        .bg(if is_selected {
                            theme.accent().opacity(0.25)
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
                                s.bg(theme.surface().opacity(0.7))
                            } else {
                                s
                            }
                        })
                        .on_mouse_move(cx.listener(move |this, _, _, cx| {
                            if !this.mouse_moved {
                                this.mouse_moved = true;
                            }
                            if this.selected_index != global_idx {
                                this.selected_index = global_idx;
                                cx.notify();
                            }
                        }))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.service.copy_emoji(&item_emoji);
                            cx.emit(EmojiEvent::Close);
                        }))
                        .child(
                            div()
                                .text_size(px(22.0))
                                .child(item.emoji.clone()),
                        ),
                );
            }

            grid_rows = grid_rows.child(row);
        }

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(460.0))
            .min_h(px(320.0))
            .max_h(px(480.0))
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
                                    .path("smile.svg")
                                    .size(px(18.0))
                                    .text_color(theme.accent()),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(13.0))
                                    .text_color(theme.foreground())
                                    .child("SELECTOR DE EMOJIS"),
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
                                "Buscar emoji por nombre o palabra clave...".to_string()
                            } else {
                                query
                            }),
                    ),
            )
            .child(cat_bar)
            .child(
                // Grid of items
                if is_empty {
                    div()
                        .id("emoji-empty-state")
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child("No se encontraron emojis que coincidan.")
                } else {
                    grid_rows
                },
            )
    }
}
