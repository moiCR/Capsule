use gpui::prelude::*;
use gpui::{
    App, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render,
    ScrollHandle, Size, Window, div, img, px, svg,
};
use services::{AppState, ClipboardItem, Snippet};
use ui::components::{Input, InputState};
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub const CLIPBOARD_COLUMNS: usize = 2;
pub const CARD_WIDTH: f32 = 266.0;
pub const CARD_HEIGHT: f32 = 154.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipboardTab {
    #[default]
    History,
    Snippets,
}

pub struct ClipboardModule {
    pub search_input: InputState,
    pub tab: ClipboardTab,
    pub selected: usize,
    pub items: Vec<ClipboardItem>,
    pub snippets: Vec<Snippet>,
    pub scroll: ScrollHandle,
    focus: FocusHandle,
}

pub type ClipboardState = ClipboardModule;

impl ClipboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            search_input: InputState::new("").with_max_len(120),
            tab: ClipboardTab::History,
            selected: 0,
            items: Vec::new(),
            snippets: Vec::new(),
            scroll: ScrollHandle::new(),
            focus: cx.focus_handle(),
        }
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            self.items = state.clipboard.fetch_history();
            self.snippets = state.clipboard.get_snippets();
        }
        self.search_input.clear();
        self.selected = 0;
        self.scroll.scroll_to_item(0);
        cx.notify();
    }

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

    pub fn activate_history(&mut self, idx: usize, cx: &mut Context<Self>) {
        let filtered = self.filtered_history();
        if let Some(item) = filtered.get(idx).copied() {
            let item_clone = item.clone();
            if !item.is_image {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(item.preview.clone()));
            }
            if let Some(state) = cx.try_global::<AppState>() {
                let cb = state.clipboard.clone();
                services::spawn_blocking(move || {
                    cb.copy_item(&item_clone);
                });
            }
            cx.emit(SharedPanelEvent::Close);
        }
    }

    pub fn activate_snippet(&mut self, idx: usize, cx: &mut Context<Self>) {
        let filtered = self.filtered_snippets();
        if let Some(snippet) = filtered.get(idx).copied() {
            let content = snippet.content.clone();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(content.clone()));
            services::spawn_blocking(move || {
                let _ = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut c| {
                        use std::io::Write;
                        if let Some(mut stdin) = c.stdin.take() {
                            let _ = stdin.write_all(content.as_bytes());
                        }
                        c.wait()
                    });
            });
            cx.emit(SharedPanelEvent::Close);
        }
    }

    pub fn pin_item(&mut self, idx: usize, cx: &mut Context<Self>) {
        let history = self.filtered_history();
        if let Some(item) = history.get(idx) {
            let item_clone = (*item).clone();
            if let Some(state) = cx.try_global::<AppState>() {
                let cb = state.clipboard.clone();
                services::spawn_blocking(move || {
                    let _ = cb.pin_from_history(&item_clone);
                });
                self.snippets = state.clipboard.get_snippets();
                cx.notify();
            }
        }
    }

    pub fn remove_item(&mut self, id: &str, cx: &mut Context<Self>) {
        self.items.retain(|i| i.id != id);
        cx.notify();
    }

    pub fn remove_snippet(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let id_str = id.to_string();
            let cb = state.clipboard.clone();
            services::spawn_blocking(move || {
                cb.remove_snippet(&id_str);
            });
        }
        self.snippets.retain(|s| s.id != id);
        cx.notify();
    }

    pub fn clear_history(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let cb = state.clipboard.clone();
            services::spawn_blocking(move || {
                cb.clear_history();
            });
        }
        self.items.clear();
        cx.notify();
    }

    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            cx.emit(SharedPanelEvent::Close);
            cx.stop_propagation();
            return;
        }

        let count = match self.tab {
            ClipboardTab::History => self.filtered_history().len(),
            ClipboardTab::Snippets => self.filtered_snippets().len(),
        };

        if key == "right" {
            if count > 0 {
                let cur = self.selected;
                let next = (cur + 1).min(count.saturating_sub(1));
                self.selected = next;
                let row = next / CLIPBOARD_COLUMNS;
                self.scroll.scroll_to_item(row);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if key == "left" {
            if count > 0 {
                let cur = self.selected;
                let next = cur.saturating_sub(1);
                self.selected = next;
                let row = next / CLIPBOARD_COLUMNS;
                self.scroll.scroll_to_item(row);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if key == "down" {
            if count > 0 {
                let cur = self.selected;
                let next = (cur + CLIPBOARD_COLUMNS).min(count.saturating_sub(1));
                self.selected = next;
                let row = next / CLIPBOARD_COLUMNS;
                self.scroll.scroll_to_item(row);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if key == "up" {
            if count > 0 {
                let cur = self.selected;
                let next = cur.saturating_sub(CLIPBOARD_COLUMNS);
                self.selected = next;
                let row = next / CLIPBOARD_COLUMNS;
                self.scroll.scroll_to_item(row);
                cx.notify();
            }
            cx.stop_propagation();
            return;
        }

        if key == "enter" || key == "return" {
            let sel = self.selected;
            match self.tab {
                ClipboardTab::History => {
                    self.activate_history(sel, cx);
                }
                ClipboardTab::Snippets => {
                    self.activate_snippet(sel, cx);
                }
            }
            cx.stop_propagation();
            return;
        }

        if key == "tab" {
            self.tab = match self.tab {
                ClipboardTab::History => ClipboardTab::Snippets,
                ClipboardTab::Snippets => ClipboardTab::History,
            };
            self.selected = 0;
            self.scroll.scroll_to_item(0);
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if self.search_input.handle_key_down(event, cx) {
            self.selected = 0;
            self.scroll.scroll_to_item(0);
            cx.notify();
            cx.stop_propagation();
        }
    }
}

impl EventEmitter<SharedPanelEvent> for ClipboardModule {}

impl SharedPanelItem for ClipboardModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        Size {
            width: px(580.0),
            height: px(540.0),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }

    fn needs_exclusive_keyboard(&self) -> bool {
        true
    }
}

impl Render for ClipboardModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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

        let is_history = self.tab == ClipboardTab::History;
        let is_searching = !self.search_input.is_empty();

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

        let search_bar = div().flex_1().h(px(40.0)).flex().items_center().child(
            Input::from_state("bar-clipboard-search", &self.search_input)
                .placeholder(placeholder)
                .prefix(
                    svg()
                        .path("search.svg")
                        .size(px(16.0))
                        .text_color(theme.foreground_muted())
                        .into_any_element(),
                )
                .padding_x(px(0.0))
                .gap(px(10.0))
                .w_full()
                .h_px(36.0)
                .text_size(px(13.5))
                .bg(gpui::transparent_black())
                .border_color(gpui::transparent_black())
                .focused_border_color(gpui::transparent_black()),
        );

        let clear_hover = theme.red().opacity(0.18);
        let clear_btn = div()
            .id("clip-clear-btn")
            .size(px(28.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(move |s| s.bg(clear_hover))
            .on_click(cx.listener(|this, _, _, cx| {
                this.clear_history(cx);
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
            .size(px(28.0))
            .rounded(px(8.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(move |s| s.bg(close_hover))
            .on_click(cx.listener(|_, _, _, cx| {
                cx.emit(SharedPanelEvent::Close);
            }))
            .child(
                svg()
                    .path("close.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground_muted()),
            );

        let mut top_row = div()
            .w_full()
            .h(px(40.0))
            .flex_shrink_0()
            .px(px(4.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(search_bar);

        if is_history && !self.items.is_empty() {
            top_row = top_row.child(clear_btn);
        }
        top_row = top_row.child(close_btn);

        let divider = div()
            .w_full()
            .h(px(1.0))
            .flex_shrink_0()
            .bg(style::border(&theme).opacity(0.16));

        let history_items: Vec<ClipboardItem> =
            self.filtered_history().into_iter().cloned().collect();
        let snippet_items: Vec<Snippet> = self.filtered_snippets().into_iter().cloned().collect();

        let tab_hover_1 = style::surface(&theme).opacity(0.40);
        let tab_history_btn = div()
            .id("clip-tab-history")
            .h(px(26.0))
            .px(px(10.0))
            .rounded(px(6.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(12.0))
            .font_weight(if is_history {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::MEDIUM
            })
            .bg(if is_history {
                style::surface(&theme).opacity(0.85)
            } else {
                gpui::transparent_black()
            })
            .border_1()
            .border_color(if is_history {
                style::border(&theme).opacity(0.18)
            } else {
                gpui::transparent_black()
            })
            .text_color(if is_history {
                theme.foreground()
            } else {
                theme.foreground_muted()
            })
            .hover(move |s| if is_history { s } else { s.bg(tab_hover_1) })
            .on_click(cx.listener(|this, _, _, cx| {
                this.tab = ClipboardTab::History;
                this.selected = 0;
                this.scroll.scroll_to_item(0);
                cx.notify();
            }))
            .child(history_label);

        let tab_hover_2 = style::surface(&theme).opacity(0.40);
        let tab_snippets_btn = div()
            .id("clip-tab-snippets")
            .h(px(26.0))
            .px(px(10.0))
            .rounded(px(6.0))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(12.0))
            .font_weight(if !is_history {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::MEDIUM
            })
            .bg(if !is_history {
                style::surface(&theme).opacity(0.85)
            } else {
                gpui::transparent_black()
            })
            .border_1()
            .border_color(if !is_history {
                style::border(&theme).opacity(0.18)
            } else {
                gpui::transparent_black()
            })
            .text_color(if !is_history {
                theme.foreground()
            } else {
                theme.foreground_muted()
            })
            .hover(move |s| if !is_history { s } else { s.bg(tab_hover_2) })
            .on_click(cx.listener(|this, _, _, cx| {
                this.tab = ClipboardTab::Snippets;
                this.selected = 0;
                this.scroll.scroll_to_item(0);
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
            .track_scroll(&self.scroll);

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
                        let is_selected = index == self.selected;
                        let item_id = item.id.clone();
                        let is_image = item.is_image;
                        let preview = item.preview.clone();
                        let card_hover_bg = style::surface(&theme).opacity(0.60);
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
                                                    div()
                                                        .w(px(3.5))
                                                        .h(px(14.0))
                                                        .rounded_full()
                                                        .flex_shrink_0()
                                                        .bg(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            gpui::transparent_black()
                                                        }),
                                                )
                                                .child(
                                                    svg()
                                                        .path("wallpaper.svg")
                                                        .size(px(12.0))
                                                        .text_color(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            theme.foreground_muted()
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .text_size(px(11.0))
                                                        .line_clamp(1)
                                                        .text_color(if is_selected {
                                                            theme.foreground()
                                                        } else {
                                                            theme.foreground_muted()
                                                        })
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
                                                .rounded(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .cursor_pointer()
                                                .hover(move |s| s.bg(del_hover))
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.remove_item(&item_id, cx);
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
                                                    div()
                                                        .w(px(3.5))
                                                        .h(px(14.0))
                                                        .rounded_full()
                                                        .flex_shrink_0()
                                                        .bg(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            gpui::transparent_black()
                                                        }),
                                                )
                                                .child(
                                                    svg()
                                                        .path("file-text.svg")
                                                        .size(px(12.0))
                                                        .text_color(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            theme.foreground_muted()
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .font_weight(if is_selected {
                                                            gpui::FontWeight::SEMIBOLD
                                                        } else {
                                                            gpui::FontWeight::MEDIUM
                                                        })
                                                        .text_color(if is_selected {
                                                            theme.foreground()
                                                        } else {
                                                            theme.foreground_muted()
                                                        })
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
                                                        .rounded(px(6.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .cursor_pointer()
                                                        .hover(move |s| s.bg(pin_hover))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                cx.stop_propagation();
                                                                this.pin_item(index, cx);
                                                            },
                                                        ))
                                                        .child(
                                                            svg()
                                                                .path("pin.svg")
                                                                .size(px(11.0))
                                                                .text_color(
                                                                    theme.foreground_muted(),
                                                                ),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .id(("clip-del-hist", index))
                                                        .size(px(22.0))
                                                        .rounded(px(6.0))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .cursor_pointer()
                                                        .hover(move |s| s.bg(del_hover))
                                                        .on_click(cx.listener(
                                                            move |this, _, _, cx| {
                                                                cx.stop_propagation();
                                                                this.remove_item(&item_id, cx);
                                                            },
                                                        ))
                                                        .child(
                                                            svg()
                                                                .path("close.svg")
                                                                .size(px(11.0))
                                                                .text_color(
                                                                    theme.foreground_muted(),
                                                                ),
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
                            .rounded(px(12.0))
                            .bg(if is_selected {
                                style::surface(&theme).opacity(0.85)
                            } else {
                                style::surface(&theme).opacity(0.40)
                            })
                            .border_1()
                            .border_color(if is_selected {
                                style::border(&theme).opacity(0.25)
                            } else {
                                style::border(&theme).opacity(0.14)
                            })
                            .hover(move |s| if !is_selected { s.bg(card_hover_bg) } else { s })
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.activate_history(index, cx);
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
                    let is_selected = index == self.selected;
                    let snippet_id = snippet.id.clone();
                    let title = snippet.title.clone();
                    let content = snippet.content.clone();
                    let card_hover_bg = style::surface(&theme).opacity(0.60);
                    let del_hover = theme.red().opacity(0.2);

                    let card = div()
                        .id(("bar-snippet-card", index))
                        .w(px(CARD_WIDTH))
                        .h(px(CARD_HEIGHT))
                        .flex_shrink_0()
                        .rounded(px(12.0))
                        .bg(if is_selected {
                            style::surface(&theme).opacity(0.85)
                        } else {
                            style::surface(&theme).opacity(0.40)
                        })
                        .border_1()
                        .border_color(if is_selected {
                            style::border(&theme).opacity(0.25)
                        } else {
                            style::border(&theme).opacity(0.14)
                        })
                        .hover(move |s| if !is_selected { s.bg(card_hover_bg) } else { s })
                        .cursor_pointer()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.activate_snippet(index, cx);
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
                                                    div()
                                                        .w(px(3.5))
                                                        .h(px(14.0))
                                                        .rounded_full()
                                                        .flex_shrink_0()
                                                        .bg(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            gpui::transparent_black()
                                                        }),
                                                )
                                                .child(
                                                    svg()
                                                        .path("sparkles.svg")
                                                        .size(px(12.0))
                                                        .text_color(if is_selected {
                                                            theme.accent()
                                                        } else {
                                                            theme.foreground_muted()
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .min_w_0()
                                                        .text_size(px(11.5))
                                                        .font_weight(if is_selected {
                                                            gpui::FontWeight::SEMIBOLD
                                                        } else {
                                                            gpui::FontWeight::MEDIUM
                                                        })
                                                        .line_clamp(1)
                                                        .text_color(theme.foreground())
                                                        .child(title),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .id(("clip-del-snip", index))
                                                .size(px(22.0))
                                                .rounded(px(6.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .cursor_pointer()
                                                .hover(move |s| s.bg(del_hover))
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    cx.stop_propagation();
                                                    this.remove_snippet(&snippet_id, cx);
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
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .size_full()
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(top_row)
            .child(divider)
            .child(tabs_row)
            .child(gallery)
            .child(footer)
    }
}
