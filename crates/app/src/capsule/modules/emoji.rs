use gpui::{
    Context, EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    ScrollHandle, Window, div, prelude::*, px, svg,
};
use services::{EmojiItem, EmojiService};
use ui::theme::Theme;

use crate::capsule::widgets::emoji::{
    emoji_grid::render_emoji_cell, emoji_search::render_emoji_search,
};

const COLS: usize = 7;
const VISIBLE_ROWS: usize = 5;
const ITEMS_PER_PAGE: usize = COLS * VISIBLE_ROWS;

const CATEGORY_ICONS: &[(&str, &str, &str)] = &[
    ("all", "sparkles.svg", "Todos"),
    ("people", "sparkles.svg", "😊"),
    ("nature", "leaf.svg", "🌿"),
    ("food", "sparkles.svg", "🍔"),
    ("activity", "sparkles.svg", "⚽"),
    ("travel", "sparkles.svg", "✈️"),
    ("objects", "sparkles.svg", "💡"),
    ("symbols", "sparkles.svg", "🔣"),
    ("flags", "sparkles.svg", "🏁"),
];

pub enum EmojiEvent {
    Close,
}

pub struct EmojiModule {
    pub service: EmojiService,
    query: String,
    category_filter: Option<String>,
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

                item.keywords
                    .iter()
                    .any(|kw| kw.to_lowercase().contains(&q))
            })
            .collect();
    }

    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn update_search(&mut self, new_query: String, cx: &mut Context<Self>) {
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
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1
                    && !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.alt
                    && !event.keystroke.modifiers.platform
                {
                    let mut q = self.query.clone();
                    q.push_str(text);
                    self.update_search(q, cx);
                }
            }
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

        let mut cat_strip = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .gap(px(2.0))
            .overflow_hidden();

        for &(cat_id, icon_path, label) in CATEGORY_ICONS {
            let is_active = match (cat_id, self.category_filter.as_deref()) {
                ("all", None) => true,
                (cat, Some(active)) => cat.eq_ignore_ascii_case(active),
                _ => false,
            };

            let cat_val = if cat_id == "all" {
                None
            } else {
                Some(cat_id.to_string())
            };

            let use_emoji_label = cat_id != "all";

            cat_strip = cat_strip.child(
                div()
                    .id(format!("cat-{cat_id}"))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_1()
                    .py(px(6.0))
                    .rounded(px(12.0))
                    .cursor_pointer()
                    .bg(if is_active {
                        theme.accent().opacity(0.15)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .border_1()
                    .border_color(if is_active {
                        theme.accent().opacity(0.4)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| {
                        if !is_active {
                            s.bg(theme.surface().opacity(0.4))
                        } else {
                            s
                        }
                    })
                    .active(|s| s.opacity(0.7))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_category(cat_val.clone(), cx);
                    }))
                    .child(if use_emoji_label {
                        div()
                            .text_size(px(14.0))
                            .child(label.to_string())
                            .into_any_element()
                    } else {
                        svg()
                            .path(icon_path)
                            .size(px(14.0))
                            .text_color(if is_active {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            })
                            .into_any_element()
                    }),
            );
        }

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
            .gap(px(2.0));

        for (row_idx, chunk) in current_page_items.chunks(COLS).enumerate() {
            let mut row = div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(2.0));

            for (col_idx, item) in chunk.iter().enumerate() {
                let global_idx = page_start + row_idx * COLS + col_idx;
                let is_selected = global_idx == selected_index;

                row = row.child(render_emoji_cell(global_idx, item, is_selected, &theme, cx));
            }

            if chunk.len() < COLS {
                for _ in 0..(COLS - chunk.len()) {
                    row = row.child(div().w(px(56.0)).h(px(56.0)));
                }
            }

            grid_rows = grid_rows.child(row);
        }

        let selected_preview = self
            .filtered_items
            .get(selected_index)
            .map(|item| (item.emoji.clone(), item.name.clone(), item.category.clone()));

        let current_page_num = self.page + 1;

        let footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(36.0))
            .px_1()
            .child(if let Some((emoji, name, category)) = selected_preview {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .overflow_hidden()
                    .child(div().text_size(px(18.0)).child(emoji))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.foreground())
                                    .text_ellipsis()
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_size(px(9.0))
                                    .text_color(theme.foreground_muted())
                                    .text_ellipsis()
                                    .child(category),
                            ),
                    )
                    .into_any_element()
            } else {
                div().flex_1().into_any_element()
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        div()
                            .id("page-prev-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.surface().opacity(0.6)))
                            .active(|s| s.opacity(0.6))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.prev_page(cx);
                            }))
                            .child(
                                svg()
                                    .path("chevron-left.svg")
                                    .size(px(12.0))
                                    .text_color(theme.foreground_muted()),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted())
                            .child(format!("{current_page_num}/{total_pages}")),
                    )
                    .child(
                        div()
                            .id("page-next-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(24.0))
                            .h(px(24.0))
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.surface().opacity(0.6)))
                            .active(|s| s.opacity(0.6))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.next_page(cx);
                            }))
                            .child(
                                svg()
                                    .path("chevron-right.svg")
                                    .size(px(12.0))
                                    .text_color(theme.foreground_muted()),
                            ),
                    ),
            );

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(cx.listener(|this, _, _window, cx| {
                if !this.mouse_moved {
                    this.mouse_moved = true;
                    cx.notify();
                }
            }))
            .flex()
            .flex_col()
            .w(px(430.0))
            .max_h(px(480.0))
            .p_3p5()
            .gap_2p5()
            .child(render_emoji_search(&query, total_items, &theme, cx))
            .child(cat_strip)
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            .child(if is_empty {
                div()
                    .id("emoji-empty-state")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(div().text_size(px(32.0)).child("🔍"))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .child("No se encontraron emojis"),
                    )
            } else {
                grid_rows
            })
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            .child(footer)
    }
}
