use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, Window, div, px, svg};
use services::{AppState, EmojiItem, EmojiService};
use ui::components::{Input, InputState};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub const EMOJIS_PER_PAGE: usize = 35;
pub const EMOJI_COLUMNS: usize = 7;
pub const EMOJI_ROWS: usize = 5;

pub const CATEGORIES: &[(Option<&'static str>, &str)] = &[
    (None, ""),
    (Some("people"), "😊"),
    (Some("animals"), "🐾"),
    (Some("nature"), "🌿"),
    (Some("food"), "🍔"),
    (Some("activity"), "⚽"),
    (Some("travel"), "✈️"),
    (Some("objects"), "💡"),
    (Some("symbols"), "🔣"),
    (Some("flags"), "🏁"),
];

pub struct EmojiState {
    pub search_input: InputState,
    pub category: Option<&'static str>,
    pub selected: usize,
    pub page: usize,
}

impl Default for EmojiState {
    fn default() -> Self {
        Self {
            search_input: InputState::new("").with_max_len(60),
            category: None,
            selected: 0,
            page: 0,
        }
    }
}

impl EmojiState {
    pub fn filtered_emojis(&self) -> Vec<&'static EmojiItem> {
        let all = EmojiService::new().load_emojis();
        let q = self.search_input.text().to_lowercase();
        all.iter()
            .filter(|e| {
                if let Some(cat) = self.category
                    && !e.category.eq_ignore_ascii_case(cat)
                {
                    return false;
                }
                if q.is_empty() {
                    return true;
                }
                e.name.to_lowercase().contains(&q)
                    || e.emoji.contains(&q)
                    || e.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .collect()
    }
}

pub fn render(state: &EmojiState, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let language = cx.try_global::<AppState>().map(|s| s.language.clone());

    let placeholder = language
        .as_ref()
        .map(|l| l.get("emoji.search_placeholder"))
        .unwrap_or_else(|| "Buscar emoji...".to_string());

    let empty_label = language
        .as_ref()
        .map(|l| l.get("emoji.no_emojis"))
        .unwrap_or_else(|| "No se encontraron emojis".to_string());

    let search_bar = div()
        .w_full()
        .h(px(40.0))
        .flex_shrink_0()
        .px(px(4.0))
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            svg()
                .path("search.svg")
                .size(px(16.0))
                .text_color(theme.foreground_muted()),
        )
        .child(
            Input::from_state("bar-emoji-search", &state.search_input)
                .placeholder(placeholder)
                .cleanable(!state.search_input.is_empty())
                .w_full()
                .h_px(36.0)
                .text_size(px(13.5))
                .bg(gpui::transparent_black())
                .border_color(gpui::transparent_black())
                .focused_border_color(gpui::transparent_black())
                .on_clear(cx.listener(|bar, _, _, cx| {
                    bar.shared_panel.emoji_state.search_input.clear();
                    bar.shared_panel.emoji_state.selected = 0;
                    bar.shared_panel.emoji_state.page = 0;
                    cx.notify();
                })),
        );

    let divider = div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(&theme).opacity(0.16));

    let cat_hover_bg = style::surface(&theme).opacity(0.35);
    let mut cat_row = div()
        .w_full()
        .h(px(28.0))
        .flex()
        .items_center()
        .gap(px(4.0));

    for (cat_idx, &(cat, icon)) in CATEGORIES.iter().enumerate() {
        let is_active = state.category == cat;

        let content = if cat.is_none() {
            svg()
                .path("sparkles.svg")
                .size(px(14.0))
                .text_color(if is_active {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                })
                .into_any_element()
        } else {
            div()
                .font_family("Noto Color Emoji")
                .text_size(px(15.0))
                .line_height(px(18.0))
                .child(icon)
                .into_any_element()
        };

        let cat_pill = div()
            .id(("bar-emoji-cat", cat_idx))
            .flex_1()
            .min_w_0()
            .h_full()
            .rounded(px(7.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .bg(if is_active {
                style::surface(&theme).opacity(0.85)
            } else {
                gpui::transparent_black()
            })
            .border_1()
            .border_color(if is_active {
                style::border(&theme).opacity(0.18)
            } else {
                gpui::transparent_black()
            })
            .hover(move |s| if is_active { s } else { s.bg(cat_hover_bg) })
            .on_click(cx.listener(move |bar, _, _, cx| {
                bar.shared_panel.emoji_state.category = cat;
                bar.shared_panel.emoji_state.page = 0;
                bar.shared_panel.emoji_state.selected = 0;
                cx.notify();
            }))
            .child(content);

        cat_row = cat_row.child(cat_pill);
    }

    let all_filtered = state.filtered_emojis();
    let total_pages = all_filtered.len().div_ceil(EMOJIS_PER_PAGE).max(1);
    let current_page = state.page.min(total_pages - 1);

    let page_items: Vec<&EmojiItem> = all_filtered
        .into_iter()
        .skip(current_page * EMOJIS_PER_PAGE)
        .take(EMOJIS_PER_PAGE)
        .collect();

    let grid_container = if page_items.is_empty() {
        div()
            .id("bar-emoji-empty")
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10.0))
            .child(
                svg()
                    .path("search.svg")
                    .size(px(26.0))
                    .text_color(theme.foreground_muted().opacity(0.35)),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(theme.foreground_muted())
                    .child(empty_label),
            )
    } else {
        let mut grid = div()
            .id("bar-emoji-grid")
            .w_full()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(6.0));

        let rows_count = page_items.len().div_ceil(EMOJI_COLUMNS);
        for row_idx in 0..rows_count {
            let mut row = div().w_full().flex().items_center().gap(px(6.0));

            for col_idx in 0..EMOJI_COLUMNS {
                let item_idx = row_idx * EMOJI_COLUMNS + col_idx;
                if let Some(item) = page_items.get(item_idx) {
                    let is_selected = item_idx == state.selected;
                    let emoji_char = item.emoji.clone();
                    let hover_cell_bg = style::surface(&theme).opacity(0.35);

                    let cell = div()
                        .id(("bar-emoji-cell", item_idx))
                        .w(px(66.0))
                        .h(px(56.0))
                        .flex_shrink_0()
                        .rounded(px(10.0))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(if is_selected {
                            style::surface(&theme).opacity(0.85)
                        } else {
                            gpui::transparent_black()
                        })
                        .border_1()
                        .border_color(if is_selected {
                            style::border(&theme).opacity(0.24)
                        } else {
                            gpui::transparent_black()
                        })
                        .hover(move |s| if !is_selected { s.bg(hover_cell_bg) } else { s })
                        .on_click(cx.listener(move |bar, _, window, cx| {
                            bar.copy_emoji(&emoji_char, window, cx);
                        }))
                        .child(
                            div()
                                .font_family("Noto Color Emoji")
                                .text_size(px(28.0))
                                .line_height(px(32.0))
                                .child(item.emoji.clone()),
                        );

                    row = row.child(cell);
                } else {
                    row = row.child(div().w(px(66.0)).h(px(56.0)).flex_shrink_0());
                }
            }

            grid = grid.child(row);
        }

        grid
    };

    let selected_item = page_items.get(state.selected).copied();

    let btn_hover_1 = style::hover(&theme);
    let prev_btn = div()
        .id("emoji-prev-btn")
        .size(px(26.0))
        .rounded(px(6.0))
        .bg(style::surface(&theme).opacity(0.60))
        .border_1()
        .border_color(style::border(&theme).opacity(0.16))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .opacity(if current_page > 0 { 1.0 } else { 0.3 })
        .when(current_page > 0, |s| {
            s.hover(move |s| s.bg(btn_hover_1))
                .on_click(cx.listener(|bar, _, _, cx| {
                    if bar.shared_panel.emoji_state.page > 0 {
                        bar.shared_panel.emoji_state.page -= 1;
                        bar.shared_panel.emoji_state.selected = 0;
                        cx.notify();
                    }
                }))
        })
        .child(
            svg()
                .path("chevron-left.svg")
                .size(px(12.0))
                .text_color(theme.foreground_muted()),
        );

    let btn_hover_2 = style::hover(&theme);
    let next_btn = div()
        .id("emoji-next-btn")
        .size(px(26.0))
        .rounded(px(6.0))
        .bg(style::surface(&theme).opacity(0.60))
        .border_1()
        .border_color(style::border(&theme).opacity(0.16))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .opacity(if current_page + 1 < total_pages {
            1.0
        } else {
            0.3
        })
        .when(current_page + 1 < total_pages, |s| {
            s.hover(move |s| s.bg(btn_hover_2))
                .on_click(cx.listener(|bar, _, _, cx| {
                    let total_pages = bar.emoji_total_pages();
                    if bar.shared_panel.emoji_state.page + 1 < total_pages {
                        bar.shared_panel.emoji_state.page += 1;
                        bar.shared_panel.emoji_state.selected = 0;
                        cx.notify();
                    }
                }))
        })
        .child(
            svg()
                .path("chevron-right.svg")
                .size(px(12.0))
                .text_color(theme.foreground_muted()),
        );

    let page_indicator = div()
        .text_size(px(11.5))
        .text_color(theme.foreground_muted())
        .child(format!("{}/{}", current_page + 1, total_pages));

    let footer = div()
        .h(px(32.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .when_some(selected_item, |element, item| {
                    element
                        .child(
                            div()
                                .font_family("Noto Color Emoji")
                                .text_size(px(20.0))
                                .line_height(px(24.0))
                                .child(item.emoji.clone()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(11.5))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .line_clamp(1)
                                .text_color(theme.foreground_muted())
                                .child(item.name.clone()),
                        )
                }),
        )
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(prev_btn)
                .child(page_indicator)
                .child(next_btn),
        );

    div()
        .id("bar-emoji-panel")
        .size_full()
        .p(px(14.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(search_bar)
        .child(divider)
        .child(cat_row)
        .child(grid_container)
        .child(footer)
        .into_any_element()
}
