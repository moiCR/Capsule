use gpui::{
    Context, EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    ScrollHandle, Window, div, prelude::*, px, svg,
};
use services::{EmojiItem, EmojiService};
use ui::theme::Theme;

const ITEMS_PER_PAGE: usize = 40; // 8 columns x 5 rows = 40 visible emojis per page (perfect zero-clipping layout)
const COLS: usize = 8;

pub enum EmojiEvent {
    Close,
}

pub struct EmojiModule {
    service: EmojiService,
    query: String,
    category_filter: Option<String>,
    category_offset: usize,
    items: &'static [EmojiItem],
    filtered_items: Vec<&'static EmojiItem>,
    pub selected_index: usize,
    pub page: usize,
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
        let all_emojis = service.load_emojis().as_slice();
        let filtered: Vec<&'static EmojiItem> = all_emojis.iter().collect();

        Self {
            service,
            query: String::new(),
            category_filter: None,
            category_offset: 0,
            items: all_emojis,
            filtered_items: filtered,
            selected_index: 0,
            page: 0,
            focus_handle,
            scroll_handle,
            mouse_moved: false,
        }
    }

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
        self.category_filter = None;
        self.category_offset = 0;
        self.items = self.service.load_emojis().as_slice();
        self.filter_items();
        self.selected_index = 0;
        self.page = 0;
        self.mouse_moved = false;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    pub fn clear_cache(&mut self) {
        self.filtered_items.clear();
        self.query.clear();
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
        self.page = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn set_category(&mut self, category: Option<String>, cx: &mut Context<Self>) {
        self.category_filter = category;
        self.filter_items();
        self.selected_index = 0;
        self.page = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
            self.page = self.selected_index / ITEMS_PER_PAGE;
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
            self.page = self.selected_index / ITEMS_PER_PAGE;
            cx.notify();
        }
    }

    fn select_row_down(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            if self.selected_index + COLS < self.filtered_items.len() {
                self.selected_index += COLS;
            } else {
                self.selected_index = (self.selected_index + COLS) % self.filtered_items.len();
            }
            self.page = self.selected_index / ITEMS_PER_PAGE;
            cx.notify();
        }
    }

    fn select_row_up(&mut self, cx: &mut Context<Self>) {
        if !self.filtered_items.is_empty() {
            if self.selected_index >= COLS {
                self.selected_index -= COLS;
            } else {
                let remainder = self.selected_index;
                let last_full = (self.filtered_items.len() / COLS) * COLS;
                if last_full + remainder < self.filtered_items.len() {
                    self.selected_index = last_full + remainder;
                } else if last_full >= COLS {
                    self.selected_index = last_full - COLS + remainder;
                }
            }
            self.page = self.selected_index / ITEMS_PER_PAGE;
            cx.notify();
        }
    }

    fn next_page(&mut self, cx: &mut Context<Self>) {
        let total_pages = (self.filtered_items.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE.max(1);
        if total_pages > 0 {
            self.page = (self.page + 1) % total_pages;
            self.selected_index = self.page * ITEMS_PER_PAGE;
            self.scroll_handle.scroll_to_item(0);
            cx.notify();
        }
    }

    fn prev_page(&mut self, cx: &mut Context<Self>) {
        let total_pages = (self.filtered_items.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE.max(1);
        if total_pages > 0 {
            if self.page == 0 {
                self.page = total_pages - 1;
            } else {
                self.page -= 1;
            }
            self.selected_index = self.page * ITEMS_PER_PAGE;
            self.scroll_handle.scroll_to_item(0);
            cx.notify();
        }
    }

    fn copy_selected(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            self.service.copy_emoji(&item.emoji);
            self.clear_cache();
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
            "pageup" => {
                self.prev_page(cx);
            }
            "pagedown" => {
                self.next_page(cx);
            }
            "enter" => {
                self.copy_selected(cx);
            }
            "escape" => {
                self.clear_cache();
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
        let total_items = self.filtered_items.len();
        let is_empty = total_items == 0;

        let total_pages = if total_items == 0 {
            1
        } else {
            (total_items + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE
        };

        if self.page >= total_pages {
            self.page = 0;
        }

        let categories = [
            ("all", "Todos"),
            ("people", "Emociones"),
            ("nature", "Naturaleza"),
            ("food", "Comida"),
            ("activity", "Actividades"),
            ("travel", "Viajes"),
            ("objects", "Objetos"),
            ("symbols", "Símbolos"),
            ("flags", "Banderas"),
        ];

        let visible_cat_count = 4;
        let max_offset = categories.len().saturating_sub(visible_cat_count);
        let category_offset = self.category_offset.min(max_offset);

        let can_prev_cat = category_offset > 0;
        let can_next_cat = category_offset < max_offset;

        // Spacious Unified Category Wrapper
        let mut cat_bar = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px_2()
            .py_1()
            .rounded_2xl()
            .bg(theme.surface().opacity(0.35))
            .border_1()
            .border_color(theme.surface().opacity(0.6))
            .gap_1p5();

        // Chevron Left Button (flex_none prevents squishing)
        cat_bar = cat_bar.child(
            div()
                .id("cat-prev-btn")
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.0))
                .h(px(28.0))
                .rounded_xl()
                .bg(if can_prev_cat {
                    theme.surface().opacity(0.6)
                } else {
                    theme.surface().opacity(0.2)
                })
                .cursor_pointer()
                .hover(|s| {
                    if can_prev_cat {
                        s.bg(theme.surface().opacity(0.9))
                    } else {
                        s
                    }
                })
                .active(|s| if can_prev_cat { s.opacity(0.6) } else { s })
                .on_click(cx.listener(move |this, _, _, cx| {
                    if this.category_offset > 0 {
                        this.category_offset -= 1;
                        cx.notify();
                    }
                }))
                .child(
                    svg()
                        .path("chevron-left.svg")
                        .size(px(13.0))
                        .text_color(if can_prev_cat {
                            theme.foreground()
                        } else {
                            theme.foreground_muted().opacity(0.3)
                        }),
                ),
        );

        let mut cat_items_container = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1p5()
            .flex_1()
            .justify_between()
            .overflow_hidden();

        let visible_cats = &categories[category_offset..(category_offset + visible_cat_count).min(categories.len())];

        for (cat_id, cat_label) in visible_cats {
            let is_cat_active = match (*cat_id, self.category_filter.as_deref()) {
                ("all", None) => true,
                (cat, Some(active)) => cat.eq_ignore_ascii_case(active),
                _ => false,
            };

            let cat_val = if *cat_id == "all" {
                None
            } else {
                Some((*cat_id).to_string())
            };

            cat_items_container = cat_items_container.child(
                div()
                    .id(format!("cat-{cat_id}"))
                    .px_2p5()
                    .py_1()
                    .rounded_xl()
                    .text_size(px(11.0))
                    .font_weight(if is_cat_active {
                        FontWeight::BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .cursor_pointer()
                    .bg(if is_cat_active {
                        theme.accent()
                    } else {
                        theme.surface().opacity(0.3)
                    })
                    .text_color(if is_cat_active {
                        theme.background()
                    } else {
                        theme.foreground_muted()
                    })
                    .hover(|s| {
                        if !is_cat_active {
                            s.bg(theme.surface().opacity(0.7))
                                .text_color(theme.foreground())
                        } else {
                            s
                        }
                    })
                    .active(|s| s.opacity(0.7))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_category(cat_val.clone(), cx);
                    }))
                    .child(*cat_label),
            );
        }

        cat_bar = cat_bar.child(cat_items_container);

        // Chevron Right Button (flex_none prevents squishing or clipping)
        cat_bar = cat_bar.child(
            div()
                .id("cat-next-btn")
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.0))
                .h(px(28.0))
                .rounded_xl()
                .bg(if can_next_cat {
                    theme.surface().opacity(0.6)
                } else {
                    theme.surface().opacity(0.2)
                })
                .cursor_pointer()
                .hover(|s| {
                    if can_next_cat {
                        s.bg(theme.surface().opacity(0.9))
                    } else {
                        s
                    }
                })
                .active(|s| if can_next_cat { s.opacity(0.6) } else { s })
                .on_click(cx.listener(move |this, _, _, cx| {
                    if this.category_offset + visible_cat_count < categories.len() {
                        this.category_offset += 1;
                        cx.notify();
                    }
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(13.0))
                        .text_color(if can_next_cat {
                            theme.foreground()
                        } else {
                            theme.foreground_muted().opacity(0.3)
                        }),
                ),
        );

        // Render current page items (48 items per page: 8 cols x 6 rows)
        let page_start = self.page * ITEMS_PER_PAGE;
        let page_end = (page_start + ITEMS_PER_PAGE).min(total_items);
        let current_page_items = if page_start < total_items {
            &self.filtered_items[page_start..page_end]
        } else {
            &[]
        };

        let mut grid_rows = div()
            .id("emoji-grid-container")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .gap_2();

        for (row_idx, chunk) in current_page_items.chunks(COLS).enumerate() {
            let mut row = div().flex().flex_row().items_center().justify_start().gap_2();

            for (col_idx, item) in chunk.iter().enumerate() {
                let global_idx = page_start + row_idx * COLS + col_idx;
                let is_selected = global_idx == selected_index;
                let item_emoji = item.emoji.clone();

                row = row.child(
                    div()
                        .id(format!("emoji-{global_idx}"))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(54.0))
                        .h(px(40.0))
                        .rounded_2xl()
                        .cursor_pointer()
                        .bg(if is_selected {
                            theme.accent().opacity(0.25)
                        } else {
                            theme.surface().opacity(0.35)
                        })
                        .border_1()
                        .border_color(if is_selected {
                            theme.accent()
                        } else {
                            theme.surface().opacity(0.0)
                        })
                        .hover(|s| {
                            if !is_selected {
                                s.bg(theme.surface().opacity(0.75))
                            } else {
                                s
                            }
                        })
                        .active(|s| s.opacity(0.7))
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
                            this.clear_cache();
                            cx.emit(EmojiEvent::Close);
                        }))
                        .child(
                            div()
                                .text_size(px(20.0))
                                .child(item.emoji.clone()),
                        ),
                );
            }

            grid_rows = grid_rows.child(row);
        }

        // Page navigation controls
        let current_page_num = self.page + 1;
        let pagination_bar = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .pt_1()
            .child(
                div()
                    .id("page-prev-btn")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_3()
                    .py_1p5()
                    .rounded_xl()
                    .bg(theme.surface().opacity(0.4))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.7))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.prev_page(cx);
                    }))
                    .child(
                        svg()
                            .path("chevron-left.svg")
                            .size(px(13.0))
                            .text_color(theme.foreground()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child("Anterior"),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.foreground_muted())
                    .child(format!("{current_page_num} / {total_pages}")),
            )
            .child(
                div()
                    .id("page-next-btn")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_3()
                    .py_1p5()
                    .rounded_xl()
                    .bg(theme.surface().opacity(0.4))
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.7))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.next_page(cx);
                    }))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child("Siguiente"),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(13.0))
                            .text_color(theme.foreground()),
                    ),
            );

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(520.0))
            .h(px(500.0))
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
                                    .path("sparkles.svg")
                                    .size(px(18.0))
                                    .text_color(theme.accent()),
                            )
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_size(px(14.0))
                                    .text_color(theme.foreground())
                                    .child("EMOJIS"),
                            ),
                    )
                    .child(
                        div()
                            .px_2p5()
                            .py_0p5()
                            .rounded_full()
                            .bg(theme.surface().opacity(0.4))
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted())
                            .child(format!("{total_items} disponibles")),
                    ),
            )
            .child(
                // Search bar
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2p5()
                    .px_3p5()
                    .py_2()
                    .rounded_2xl()
                    .bg(theme.surface().opacity(0.45))
                    .border_1()
                    .border_color(theme.surface().opacity(0.7))
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
                // Grid of items or empty state
                if is_empty {
                    div()
                        .id("emoji-empty-state")
                        .flex()
                        .flex_1()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child("No se encontraron emojis.")
                } else {
                    grid_rows
                },
            )
            .child(pagination_bar)
    }
}
