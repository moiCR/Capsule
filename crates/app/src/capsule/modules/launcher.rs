use gpui::{
    EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Render, ScrollHandle, Window, div,
    prelude::*, px, svg,
};
use services::{AppState, Application, LauncherService};
use ui::theme::Theme;

use crate::capsule::widgets::launcher::{
    app_item::render_app_item, calc_item::render_calc_item, search_input::render_search_input,
};

pub enum LauncherEvent {
    Close,
}

pub struct LauncherModule {
    service: LauncherService,
    query: String,
    apps: Vec<Application>,
    calc_result: Option<String>,
    pub selected_index: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    pub mouse_moved: bool,
}

impl EventEmitter<LauncherEvent> for LauncherModule {}

impl LauncherModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = cx.global::<AppState>().launcher.clone();
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let initial_apps = service.search("");

        Self {
            service,
            query: String::new(),
            apps: initial_apps,
            calc_result: None,
            selected_index: 0,
            focus_handle,
            scroll_handle,
            mouse_moved: false,
        }
    }

    pub fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
        self.calc_result = None;
        self.selected_index = 0;
        self.mouse_moved = false;
        self.apps = self.service.search("");
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    fn total_items(&self) -> usize {
        self.apps.len() + usize::from(self.calc_result.is_some())
    }

    fn update_search(&mut self, new_query: String, cx: &mut Context<Self>) {
        self.query = new_query;
        self.calc_result = services::launcher::calculator::evaluate(&self.query);
        self.apps = self.service.search(&self.query);
        self.selected_index = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        let total = self.total_items();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn select_prev(&mut self, cx: &mut Context<Self>) {
        let total = self.total_items();
        if total > 0 {
            if self.selected_index == 0 {
                self.selected_index = total - 1;
            } else {
                self.selected_index -= 1;
            }
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn launch_selected(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(result) = &self.calc_result
            && self.selected_index == 0
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(result.clone()));
            self.reset_search(cx);
            cx.emit(LauncherEvent::Close);
            return true;
        }

        let app_index = if self.calc_result.is_some() {
            self.selected_index.saturating_sub(1)
        } else {
            self.selected_index
        };

        if let Some(app) = self.apps.get(app_index) {
            if let Err(err) = app.launch() {
                eprintln!("Failed to launch {}: {err}", app.name);
            }
            self.reset_search(cx);
            cx.emit(LauncherEvent::Close);
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
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if ctrl {
            match key {
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        let clean_text: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !clean_text.is_empty() {
                            let mut new_q = self.query.clone();
                            new_q.push_str(&clean_text);
                            self.update_search(new_q, cx);
                        }
                    }
                    return;
                }
                "c" => {
                    if !self.query.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(self.query.clone()));
                    }
                    return;
                }
                "x" => {
                    if !self.query.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(self.query.clone()));
                        self.update_search(String::new(), cx);
                    }
                    return;
                }
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
            "enter" => {
                self.launch_selected(cx);
            }
            "escape" => {
                self.reset_search(cx);
                cx.emit(LauncherEvent::Close);
            }
            "down" => {
                self.mouse_moved = false;
                self.select_next(cx);
            }
            "up" => {
                self.mouse_moved = false;
                self.select_prev(cx);
            }
            "backspace" => {
                if !self.query.is_empty() {
                    let mut new_q = self.query.clone();
                    new_q.pop();
                    self.update_search(new_q, cx);
                }
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1 && !ctrl {
                    let mut new_q = self.query.clone();
                    new_q.push_str(text);
                    self.update_search(new_q, cx);
                }
            }
        }
    }
}

impl Render for LauncherModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let no_apps = if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .language
                .get("launcher.no_apps")
        } else {
            "No se encontraron aplicaciones".to_string()
        };

        let has_calc = self.calc_result.is_some();
        let is_empty = self.apps.is_empty() && !has_calc;

        let mut app_list = div()
            .id("launcher-app-list")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_scroll()
            .gap(px(4.0));

        if let Some(ref calc_res) = self.calc_result {
            let is_selected = self.selected_index == 0;
            app_list = app_list.child(render_calc_item(
                calc_res,
                &self.query,
                is_selected,
                &theme,
                cx,
            ));
        }

        for (idx, app) in self.apps.iter().enumerate() {
            let item_index = if has_calc { idx + 1 } else { idx };
            let is_selected = item_index == self.selected_index;
            app_list = app_list.child(render_app_item(item_index, app, is_selected, &theme, cx));
        }

        let content = if is_empty {
            div()
                .id("launcher-empty-state")
                .flex()
                .flex_col()
                .flex_1()
                .items_center()
                .justify_center()
                .gap_2()
                .child(
                    svg()
                        .path("search.svg")
                        .size(px(24.0))
                        .text_color(theme.foreground_muted().opacity(0.4)),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child(no_apps),
                )
                .into_any_element()
        } else {
            app_list.into_any_element()
        };

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
            .w(px(380.0))
            .max_h(px(360.0))
            .p_3()
            .gap_2()
            .child(render_search_input(&self.query, &theme, cx))
            .child(content)
    }
}
