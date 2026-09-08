use gpui::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent, Render,
    Task, Window, div, prelude::*, px, svg,
};
use services::AppState;
use std::time::{Duration, Instant};
use ui::theme::Theme;
use ui::theme::theme_manager::{ThemeItem, ThemeManager};

use crate::capsule::widgets::select_theme::theme_card::{get_theme_card_props, render_theme_card};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectThemeEvent {
    ThemeSelected,
}

pub struct SelectThemeModule {
    focus_handle: FocusHandle,
    themes: Vec<ThemeItem>,
    query: String,
    pub selected_idx: usize,
    pub anim_progress: f32,
    pub anim_direction: f32,
    pub is_animating: bool,
    pub anim_task: Option<Task<()>>,
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
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
            anim_progress: 1.0,
            anim_direction: 0.0,
            is_animating: false,
            anim_task: None,
        }
    }

    pub fn refresh_themes(&mut self, cx: &mut Context<Self>) {
        self.themes = Self::load_themes(cx);
        self.anim_progress = 1.0;
        self.anim_direction = 0.0;
        self.is_animating = false;
        self.anim_task = None;
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

    pub fn navigate(&mut self, dir: f32, new_idx: usize, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if filtered.is_empty() {
            return;
        }
        let total = filtered.len();
        self.selected_idx = new_idx % total;
        self.anim_direction = dir;
        self.anim_progress = 0.0;
        self.is_animating = true;
        self.anim_task = None;

        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };

        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 220.0;
            let start = Instant::now();
            loop {
                let frame_dur = if let Some(ref comp) = compositor {
                    comp.get_frame_duration()
                } else {
                    Duration::from_millis(16)
                };

                cx.background_executor().timer(frame_dur).await;

                let finished = this
                    .update(cx, |module: &mut Self, cx| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let p = (elapsed / duration_ms).min(1.0);
                        module.anim_progress = p;
                        if p >= 1.0 {
                            module.is_animating = false;
                        }
                        cx.notify();
                        p >= 1.0
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });

        self.anim_task = Some(anim_task);
    }

    pub fn select_prev(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if filtered.len() > 1 {
            let total = filtered.len();
            let next_idx = if self.selected_idx == 0 {
                total - 1
            } else {
                self.selected_idx - 1
            };
            self.navigate(-1.0, next_idx, cx);
        }
    }

    pub fn select_next(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_themes();
        if filtered.len() > 1 {
            let total = filtered.len();
            let next_idx = (self.selected_idx + 1) % total;
            self.navigate(1.0, next_idx, cx);
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
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
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
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
                    cx.notify();
                    return;
                }
                "v" => {
                    if let Some(item) = cx.read_from_clipboard()
                        && let Some(text) = item.text()
                    {
                        let clean_text: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !clean_text.is_empty() {
                            self.query.push_str(&clean_text);
                            self.selected_idx = 0;
                            self.anim_progress = 1.0;
                            self.anim_direction = 0.0;
                            self.is_animating = false;
                            self.anim_task = None;
                            cx.notify();
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
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
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
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
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
        if total > 0 && self.selected_idx >= total {
            self.selected_idx = 0;
        }
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
            .gap(px(10.0))
            .h(px(104.0));

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
        } else if total == 1 {
            let item = &filtered[0];
            let props = get_theme_card_props(0.0);
            let card = render_theme_card(
                ElementId::from("slot-center"),
                item,
                0,
                0,
                &props,
                &theme,
                cx,
            );
            carousel_row = carousel_row.child(card);
        } else {
            let eased = if self.is_animating {
                ease_out_cubic(self.anim_progress)
            } else {
                1.0
            };

            let shift = (1.0 - eased) * self.anim_direction;

            for offset in -2i32..=2i32 {
                let idx = (self.selected_idx as i32 + offset).rem_euclid(total as i32) as usize;
                let item = &filtered[idx];

                let vis_pos = offset as f32 + shift;
                let abs_pos = vis_pos.abs();
                let props = get_theme_card_props(abs_pos);

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
                    item,
                    idx,
                    offset,
                    &props,
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
