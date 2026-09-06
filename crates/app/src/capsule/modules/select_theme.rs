use gpui::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent, Render,
    Window, div, prelude::*, px, svg,
};
use ui::theme::Theme;
use ui::theme::theme_manager::{ThemeItem, ThemeManager};

use crate::capsule::widgets::select_theme::theme_card::render_theme_card;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectThemeEvent {
    ThemeSelected,
}

pub struct SelectThemeModule {
    focus_handle: FocusHandle,
    themes: Vec<ThemeItem>,
    query: String,
    pub selected_idx: usize,
}

impl SelectThemeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let themes = Self::load_themes(cx);
        let selected_idx = if cx.has_global::<ThemeManager>() {
            let curr = &cx.global::<ThemeManager>().current_theme;
            themes.iter().position(|t| &t.theme == curr).unwrap_or(0)
        } else {
            0
        };
        Self {
            focus_handle,
            themes,
            query: String::new(),
            selected_idx,
        }
    }

    pub fn refresh_themes(&mut self, cx: &mut Context<Self>) {
        self.themes = Self::load_themes(cx);
        if cx.has_global::<ThemeManager>() {
            let curr = &cx.global::<ThemeManager>().current_theme;
            if let Some(pos) = self.themes.iter().position(|t| &t.theme == curr) {
                self.selected_idx = pos;
            }
        }
        cx.notify();
    }

    fn load_themes(cx: &mut Context<Self>) -> Vec<ThemeItem> {
        if cx.has_global::<ThemeManager>() {
            cx.global::<ThemeManager>().list_themes()
        } else {
            Vec::new()
        }
    }

    pub fn filtered_themes(&self) -> Vec<ThemeItem> {
        if self.query.is_empty() {
            self.themes.clone()
        } else {
            let q = self.query.to_lowercase();
            self.themes
                .iter()
                .filter(|t| t.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }

    pub fn select_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if cx.has_global::<ThemeManager>() {
            cx.global_mut::<ThemeManager>().set_theme(theme);
            cx.set_global(cx.global::<ThemeManager>().current_theme.clone());
        }
        cx.emit(SelectThemeEvent::ThemeSelected);
    }

    pub fn set_selected_idx(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.selected_idx = idx;
        cx.notify();
    }

    pub fn select_prev(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if !filtered.is_empty() {
            let total = filtered.len();
            self.selected_idx = if self.selected_idx == 0 {
                total - 1
            } else {
                self.selected_idx - 1
            };
            cx.notify();
        }
    }

    pub fn select_next(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if !filtered.is_empty() {
            let total = filtered.len();
            self.selected_idx = (self.selected_idx + 1) % total;
            cx.notify();
        }
    }

    pub fn apply_selected(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if !filtered.is_empty() {
            let idx = self.selected_idx.min(filtered.len() - 1);
            let theme = filtered[idx].theme.clone();
            self.select_theme(theme, cx);
        }
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
                    self.query.clear();
                    self.selected_idx = 0;
                    cx.notify();
                    return;
                }
                "w" => {
                    let trimmed = self.query.trim_end();
                    let new_q = if let Some(idx) = trimmed.rfind(' ') {
                        trimmed[..idx].to_string()
                    } else {
                        String::new()
                    };
                    self.query = new_q;
                    self.selected_idx = 0;
                    cx.notify();
                    return;
                }
                "v" => {
                    if let Some(item) = cx.read_from_clipboard() {
                        if let Some(text) = item.text() {
                            let clean_text: String =
                                text.chars().filter(|c| !c.is_control()).collect();
                            if !clean_text.is_empty() {
                                self.query.push_str(&clean_text);
                                self.selected_idx = 0;
                                cx.notify();
                            }
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        match key {
            "escape" => {
                cx.emit(SelectThemeEvent::ThemeSelected);
            }
            "left" => {
                self.select_prev(cx);
            }
            "right" => {
                self.select_next(cx);
            }
            "enter" => {
                self.apply_selected(cx);
            }
            "backspace" => {
                if !self.query.is_empty() {
                    self.query.pop();
                    self.selected_idx = 0;
                    cx.notify();
                }
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1 && !ctrl {
                    self.query.push_str(text);
                    self.selected_idx = 0;
                    cx.notify();
                }
            }
        }
    }
}

impl EventEmitter<SelectThemeEvent> for SelectThemeModule {}

impl Focusable for SelectThemeModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectThemeModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let (search_placeholder, no_themes, apply_hint) = if cx.has_global::<services::AppState>() {
            let lang = &cx.global::<services::AppState>().language;
            (
                lang.get("themes.search_placeholder"),
                lang.get("themes.no_themes"),
                lang.get("themes.apply_hint"),
            )
        } else {
            (
                "Buscar temas...".to_string(),
                "No se encontraron temas".to_string(),
                "↵ Aplicar".to_string(),
            )
        };

        window.focus(&self.focus_handle, cx);

        let filtered = self.filtered_themes();
        let total = filtered.len();
        let current_pos = if total == 0 {
            0
        } else {
            (self.selected_idx % total) + 1
        };

        let has_query = !self.query.is_empty();

        let header = div()
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
                    .gap_2p5()
                    .child(
                        svg()
                            .path("search.svg")
                            .size(px(14.0))
                            .text_color(theme.foreground_muted().opacity(0.8)),
                    )
                    .child(if has_query {
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(self.query.clone())
                    } else {
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.foreground_muted().opacity(0.5))
                            .child(search_placeholder)
                    }),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted().opacity(0.8))
                    .child(format!("{}/{}", current_pos, total)),
            );

        let mut carousel_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .gap(px(12.0))
            .overflow_hidden();

        if total == 0 {
            carousel_row = carousel_row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(84.0))
                    .text_size(px(13.0))
                    .text_color(theme.foreground_muted())
                    .child(no_themes),
            );
        } else if total >= 5 {
            let active_idx = self.selected_idx % total;
            for offset in -2i32..=2i32 {
                let idx = (active_idx as i32 + offset).rem_euclid(total as i32) as usize;
                let slot_key = match offset {
                    -2 => "slot-prev2",
                    -1 => "slot-prev1",
                    0 => "slot-center",
                    1 => "slot-next1",
                    2 => "slot-next2",
                    _ => "slot-other",
                };
                let card = render_theme_card(
                    ElementId::from(slot_key),
                    &filtered[idx],
                    offset == 0,
                    idx,
                    &theme,
                    cx,
                );
                carousel_row = carousel_row.child(card);
            }
        } else {
            let active_idx = self.selected_idx % total;
            for (i, item) in filtered.iter().enumerate() {
                let card = render_theme_card(
                    ElementId::NamedInteger("theme-card".into(), i as u64),
                    item,
                    i == active_idx,
                    i,
                    &theme,
                    cx,
                );
                carousel_row = carousel_row.child(card);
            }
        }

        let footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .w_full()
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted().opacity(0.6))
                    .child(apply_hint),
            );

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .justify_between()
            .w(px(680.0))
            .h(px(180.0))
            .px(px(24.0))
            .py(px(16.0))
            .overflow_hidden()
            .child(header)
            .child(carousel_row)
            .child(footer)
    }
}
