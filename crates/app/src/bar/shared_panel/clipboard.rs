use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, ScrollHandle, Window, div, img, px, svg};
use services::{AppState, ClipboardItem, Snippet};
use ui::components::{Input, InputState};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub const CLIPBOARD_COLUMNS: usize = 2;
pub const CARD_WIDTH: f32 = 266.0;
pub const CARD_HEIGHT: f32 = 154.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardTab {
    #[default]
    History,
    Snippets,
}

pub struct ClipboardState {
    pub search_input: InputState,
    pub tab: ClipboardTab,
    pub selected: usize,
    pub items: Vec<ClipboardItem>,
    pub snippets: Vec<Snippet>,
    pub scroll: ScrollHandle,
}

impl Default for ClipboardState {
    fn default() -> Self {
        Self {
            search_input: InputState::new("").with_max_len(120),
            tab: ClipboardTab::History,
            selected: 0,
            items: Vec::new(),
            snippets: Vec::new(),
            scroll: ScrollHandle::new(),
        }
    }
}

impl ClipboardState {
    pub fn filtered_history(&self) -> Vec<&ClipboardItem> {
        let q = self.search_input.text().to_lowercase();
        self.items
            .iter()
            .filter(|item| {
                if q.is_empty() {
                    true
                } else {
                    item.preview.to_lowercase().contains(&q)
                }
            })
            .collect()
    }

    pub fn filtered_snippets(&self) -> Vec<&Snippet> {
        let q = self.search_input.text().to_lowercase();
        self.snippets
            .iter()
            .filter(|s| {
                if q.is_empty() {
                    true
                } else {
                    s.title.to_lowercase().contains(&q) || s.content.to_lowercase().contains(&q)
                }
            })
            .collect()
    }
}

pub fn render(state: &ClipboardState, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let language = cx.try_global::<AppState>().map(|s| s.language.clone());

    let placeholder = language
        .as_ref()
        .map(|l| l.get("clipboard.search_placeholder"))
        .unwrap_or_else(|| "Buscar en el historial...".to_string());

    let history_label = language
        .as_ref()
        .map(|l| l.get("clipboard.history_tab"))
        .unwrap_or_else(|| "Historial".to_string());

    let snippets_label = language
        .as_ref()
        .map(|l| l.get("clipboard.snippets_tab"))
        .unwrap_or_else(|| "Plantillas".to_string());

    let is_history = state.tab == ClipboardTab::History;
    let is_searching = !state.search_input.is_empty();

    let empty_label = if is_searching {
        language
            .as_ref()
            .map(|l| l.get("clipboard.no_results"))
            .unwrap_or_else(|| "No hay coincidencias".to_string())
    } else if is_history {
        language
            .as_ref()
            .map(|l| l.get("clipboard.empty_history"))
            .unwrap_or_else(|| "No hay elementos en el historial".to_string())
    } else {
        language
            .as_ref()
            .map(|l| l.get("clipboard.empty_snippets"))
            .unwrap_or_else(|| "No hay plantillas fijadas".to_string())
    };

    let copy_hint = language
        .as_ref()
        .map(|l| l.get("clipboard.copy_hint"))
        .unwrap_or_else(|| "↑↓←→ navegar · Enter copiar · Tab cambiar".to_string());

    let search_bar = div()
        .flex_1()
        .h(px(38.0))
        .px(px(8.0))
        .rounded_full()
        .bg(style::surface(&theme))
        .border_1()
        .border_color(style::border(&theme).opacity(0.35))
        .flex()
        .items_center()
        .child(
            Input::from_state("bar-clipboard-search", &state.search_input)
                .placeholder(placeholder)
                .cleanable(!state.search_input.is_empty())
                .w_full()
                .h_px(34.0)
                .text_size(px(13.0))
                .bg(gpui::transparent_black())
                .border_color(gpui::transparent_black())
                .focused_border_color(gpui::transparent_black())
                .prefix(
                    svg()
                        .path("search.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                )
                .on_clear(cx.listener(|bar, _, _, cx| {
                    bar.shared_panel.clipboard_state.search_input.clear();
                    bar.shared_panel.clipboard_state.selected = 0;
                    bar.shared_panel.clipboard_state.scroll.scroll_to_item(0);
                    cx.notify();
                })),
        );

    let clear_hover = theme.red().opacity(0.18);
    let clear_btn = div()
        .id("clip-clear-btn")
        .size(px(38.0))
        .rounded_full()
        .bg(style::surface(&theme))
        .border_1()
        .border_color(style::border(&theme).opacity(0.35))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(move |s| s.bg(clear_hover))
        .on_click(cx.listener(|bar, _, _, cx| {
            bar.clear_clipboard_history(cx);
        }))
        .child(
            svg()
                .path("trash.svg")
                .size(px(14.0))
                .text_color(theme.foreground_muted()),
        );

    let close_hover = style::hover(&theme);
    let close_btn = div()
        .id("clip-close-btn")
        .size(px(38.0))
        .rounded_full()
        .bg(style::surface(&theme))
        .border_1()
        .border_color(style::border(&theme).opacity(0.35))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .hover(move |s| s.bg(close_hover))
        .on_click(cx.listener(|bar, _, window, cx| {
            bar.close_shared_panel(window, cx);
        }))
        .child(
            svg()
                .path("close.svg")
                .size(px(13.0))
                .text_color(theme.foreground_muted()),
        );

    let mut top_row = div()
        .w_full()
        .h(px(38.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(search_bar);

    if is_history && !state.items.is_empty() {
        top_row = top_row.child(clear_btn);
    }
    top_row = top_row.child(close_btn);

    let history_items = state.filtered_history();
    let snippet_items = state.filtered_snippets();

    let tab_hover_1 = style::hover(&theme);
    let tab_history_btn = div()
        .id("clip-tab-history")
        .h(px(28.0))
        .px(px(12.0))
        .rounded_full()
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .font_weight(if is_history {
            gpui::FontWeight::BOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .bg(if is_history {
            theme.accent()
        } else {
            style::surface(&theme)
        })
        .text_color(if is_history {
            theme.on_accent()
        } else {
            theme.foreground_muted()
        })
        .hover(move |s| if is_history { s } else { s.bg(tab_hover_1) })
        .on_click(cx.listener(|bar, _, _, cx| {
            bar.shared_panel.clipboard_state.tab = ClipboardTab::History;
            bar.shared_panel.clipboard_state.selected = 0;
            bar.shared_panel.clipboard_state.scroll.scroll_to_item(0);
            cx.notify();
        }))
        .child(history_label);

    let tab_hover_2 = style::hover(&theme);
    let tab_snippets_btn = div()
        .id("clip-tab-snippets")
        .h(px(28.0))
        .px(px(12.0))
        .rounded_full()
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .font_weight(if !is_history {
            gpui::FontWeight::BOLD
        } else {
            gpui::FontWeight::MEDIUM
        })
        .bg(if !is_history {
            theme.accent()
        } else {
            style::surface(&theme)
        })
        .text_color(if !is_history {
            theme.on_accent()
        } else {
            theme.foreground_muted()
        })
        .hover(move |s| if !is_history { s } else { s.bg(tab_hover_2) })
        .on_click(cx.listener(|bar, _, _, cx| {
            bar.shared_panel.clipboard_state.tab = ClipboardTab::Snippets;
            bar.shared_panel.clipboard_state.selected = 0;
            bar.shared_panel.clipboard_state.scroll.scroll_to_item(0);
            cx.notify();
        }))
        .child(snippets_label);

    let count_text = if is_history {
        format!("{} elementos", history_items.len())
    } else {
        format!("{} plantillas", snippet_items.len())
    };

    let tabs_row = div()
        .w_full()
        .h(px(28.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(tab_history_btn)
                .child(tab_snippets_btn),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted().opacity(0.7))
                .child(count_text),
        );

    let mut gallery = div()
        .id("bar-clipboard-gallery")
        .w_full()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .overflow_y_scroll()
        .track_scroll(&state.scroll);

    if is_history {
        if history_items.is_empty() {
            gallery = gallery.items_center().justify_center().child(
                div()
                    .py(px(48.0))
                    .flex()
                    .flex_col()
                    .items_center()
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
                    ),
            );
        } else {
            for (row_index, pair) in history_items.chunks(CLIPBOARD_COLUMNS).enumerate() {
                let mut row = div()
                    .id(("bar-clip-row", row_index))
                    .w_full()
                    .h(px(CARD_HEIGHT))
                    .min_h(px(CARD_HEIGHT))
                    .flex_shrink_0()
                    .flex()
                    .gap(px(10.0));

                for (col_index, item) in pair.iter().enumerate() {
                    let index = row_index * CLIPBOARD_COLUMNS + col_index;
                    let is_selected = index == state.selected;
                    let item_id = item.id.clone();
                    let is_image = item.is_image;
                    let preview = item.preview.clone();
                    let card_hover = style::hover(&theme);
                    let pin_hover = theme.accent().opacity(0.2);
                    let del_hover = theme.red().opacity(0.2);

                    let card_body = if is_image {
                        let img_view = if let Some(ref path) = item.image_path {
                            img(path.clone())
                                .size_full()
                                .object_fit(gpui::ObjectFit::Cover)
                                .into_any_element()
                        } else {
                            div()
                                .size_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .path("wallpaper.svg")
                                        .size(px(24.0))
                                        .text_color(theme.foreground_muted()),
                                )
                                .into_any_element()
                        };

                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .w_full()
                                    .h(px(94.0))
                                    .flex_shrink_0()
                                    .rounded_t_xl()
                                    .overflow_hidden()
                                    .bg(style::surface(&theme))
                                    .child(img_view),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .px(px(10.0))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                svg()
                                                    .path("wallpaper.svg")
                                                    .size(px(12.0))
                                                    .text_color(theme.foreground_muted()),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .text_size(px(11.0))
                                                    .line_clamp(1)
                                                    .text_color(theme.foreground_muted())
                                                    .child(if preview.is_empty() {
                                                        "Imagen".to_string()
                                                    } else {
                                                        preview.clone()
                                                    }),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id(("clip-del-hist-img", index))
                                            .size(px(22.0))
                                            .rounded_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .hover(move |s| s.bg(del_hover))
                                            .on_click(cx.listener(move |bar, _, _, cx| {
                                                cx.stop_propagation();
                                                bar.remove_clipboard_item(&item_id, cx);
                                            }))
                                            .child(
                                                svg()
                                                    .path("close.svg")
                                                    .size(px(11.0))
                                                    .text_color(theme.foreground_muted()),
                                            ),
                                    ),
                            )
                    } else {
                        div()
                            .size_full()
                            .p(px(10.0))
                            .flex()
                            .flex_col()
                            .justify_between()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .h(px(22.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                svg()
                                                    .path("file-text.svg")
                                                    .size(px(12.0))
                                                    .text_color(theme.foreground_muted()),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(10.5))
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(theme.foreground_muted())
                                                    .child(format!("{:02}", index + 1)),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(4.0))
                                            .child(
                                                div()
                                                    .id(("clip-pin-hist", index))
                                                    .size(px(22.0))
                                                    .rounded_full()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .hover(move |s| s.bg(pin_hover))
                                                    .on_click(cx.listener(move |bar, _, _, cx| {
                                                        cx.stop_propagation();
                                                        bar.pin_clipboard_item(index, cx);
                                                    }))
                                                    .child(
                                                        svg()
                                                            .path("pin.svg")
                                                            .size(px(11.0))
                                                            .text_color(theme.foreground_muted()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .id(("clip-del-hist", index))
                                                    .size(px(22.0))
                                                    .rounded_full()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .cursor_pointer()
                                                    .hover(move |s| s.bg(del_hover))
                                                    .on_click(cx.listener(move |bar, _, _, cx| {
                                                        cx.stop_propagation();
                                                        bar.remove_clipboard_item(&item_id, cx);
                                                    }))
                                                    .child(
                                                        svg()
                                                            .path("close.svg")
                                                            .size(px(11.0))
                                                            .text_color(theme.foreground_muted()),
                                                    ),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_hidden()
                                    .text_size(px(11.5))
                                    .line_height(px(16.0))
                                    .line_clamp(4)
                                    .text_color(theme.foreground())
                                    .child(preview),
                            )
                    };

                    let card = div()
                        .id(("bar-clip-card", index))
                        .w(px(CARD_WIDTH))
                        .h(px(CARD_HEIGHT))
                        .flex_shrink_0()
                        .rounded_xl()
                        .bg(if is_selected {
                            style::active(&theme)
                        } else {
                            style::surface(&theme)
                        })
                        .border_1()
                        .border_color(if is_selected {
                            theme.accent()
                        } else {
                            style::border(&theme).opacity(0.35)
                        })
                        .hover(move |s| s.bg(card_hover))
                        .cursor_pointer()
                        .on_click(cx.listener(move |bar, _, window, cx| {
                            bar.activate_clipboard_history(index, window, cx);
                        }))
                        .child(card_body);

                    row = row.child(card);
                }

                if pair.len() < CLIPBOARD_COLUMNS {
                    row = row.child(div().w(px(CARD_WIDTH)).h(px(CARD_HEIGHT)).flex_shrink_0());
                }

                gallery = gallery.child(row);
            }
        }
    } else if snippet_items.is_empty() {
        gallery = gallery.items_center().justify_center().child(
            div()
                .py(px(48.0))
                .flex()
                .flex_col()
                .items_center()
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
                ),
        );
    } else {
        for (row_index, pair) in snippet_items.chunks(CLIPBOARD_COLUMNS).enumerate() {
            let mut row = div()
                .id(("bar-snip-row", row_index))
                .w_full()
                .h(px(CARD_HEIGHT))
                .min_h(px(CARD_HEIGHT))
                .flex_shrink_0()
                .flex()
                .gap(px(10.0));

            for (col_index, snippet) in pair.iter().enumerate() {
                let index = row_index * CLIPBOARD_COLUMNS + col_index;
                let is_selected = index == state.selected;
                let snippet_id = snippet.id.clone();
                let title = snippet.title.clone();
                let content = snippet.content.clone();
                let card_hover = style::hover(&theme);
                let del_hover = theme.red().opacity(0.2);

                let card = div()
                    .id(("bar-snippet-card", index))
                    .w(px(CARD_WIDTH))
                    .h(px(CARD_HEIGHT))
                    .flex_shrink_0()
                    .rounded_xl()
                    .bg(if is_selected {
                        style::active(&theme)
                    } else {
                        style::surface(&theme)
                    })
                    .border_1()
                    .border_color(if is_selected {
                        theme.accent()
                    } else {
                        style::border(&theme).opacity(0.35)
                    })
                    .hover(move |s| s.bg(card_hover))
                    .cursor_pointer()
                    .on_click(cx.listener(move |bar, _, window, cx| {
                        bar.activate_clipboard_snippet(index, window, cx);
                    }))
                    .child(
                        div()
                            .size_full()
                            .p(px(10.0))
                            .flex()
                            .flex_col()
                            .justify_between()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .h(px(22.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .child(
                                                svg()
                                                    .path("sparkles.svg")
                                                    .size(px(12.0))
                                                    .text_color(theme.accent()),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .text_size(px(11.5))
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .line_clamp(1)
                                                    .text_color(theme.foreground())
                                                    .child(title),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id(("clip-del-snip", index))
                                            .size(px(22.0))
                                            .rounded_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .hover(move |s| s.bg(del_hover))
                                            .on_click(cx.listener(move |bar, _, _, cx| {
                                                cx.stop_propagation();
                                                bar.remove_snippet_item(&snippet_id, cx);
                                            }))
                                            .child(
                                                svg()
                                                    .path("close.svg")
                                                    .size(px(11.0))
                                                    .text_color(theme.foreground_muted()),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_hidden()
                                    .text_size(px(11.5))
                                    .line_height(px(16.0))
                                    .line_clamp(4)
                                    .text_color(theme.foreground_muted())
                                    .child(content),
                            ),
                    );

                row = row.child(card);
            }

            if pair.len() < CLIPBOARD_COLUMNS {
                row = row.child(div().w(px(CARD_WIDTH)).h(px(CARD_HEIGHT)).flex_shrink_0());
            }

            gallery = gallery.child(row);
        }
    }

    let footer = div()
        .h(px(22.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted().opacity(0.65))
                .child(copy_hint),
        );

    div()
        .id("bar-clipboard-panel")
        .size_full()
        .p(px(18.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(top_row)
        .child(tabs_row)
        .child(gallery)
        .child(footer)
        .into_any_element()
}
