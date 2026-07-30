use gpui::{
    EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Render, ScrollHandle, Window, div,
    prelude::*, px,
};
use services::{AppState, Application, LauncherService};
use ui::theme::Theme;

use crate::capsule::widgets::launcher::{
    app_item::render_app_item, search_input::render_search_input,
};

pub enum LauncherEvent {
    Close,
}

pub struct LauncherModule {
    service: LauncherService,
    query: String,
    apps: Vec<Application>,
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
            selected_index: 0,
            focus_handle,
            scroll_handle,
            mouse_moved: false,
        }
    }

    pub fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.query.clear();
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

    fn update_search(&mut self, new_query: String, cx: &mut Context<Self>) {
        self.query = new_query;
        self.apps = self.service.search(&self.query);
        self.selected_index = 0;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    fn select_next(&mut self, cx: &mut Context<Self>) {
        if !self.apps.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.apps.len();
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.apps.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.apps.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    fn launch_selected(&mut self, cx: &mut Context<Self>) -> bool {
        if let Some(app) = self.apps.get(self.selected_index) {
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
                    if let Some(item) = cx.read_from_clipboard() {
                        if let Some(text) = item.text() {
                            let clean_text: String =
                                text.chars().filter(|c| !c.is_control()).collect();
                            if !clean_text.is_empty() {
                                let mut new_q = self.query.clone();
                                new_q.push_str(&clean_text);
                                self.update_search(new_q, cx);
                            }
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

        let mut app_list = div()
            .id("launcher-app-list")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .flex_1()
            .overflow_scroll()
            .gap_1();

        for (idx, app) in self.apps.iter().enumerate() {
            let is_selected = idx == self.selected_index;
            app_list = app_list.child(render_app_item(idx, app, is_selected, &theme, cx));
        }

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
            .w(px(348.0))
            .max_h(px(500.0))
            .p_3p5()
            .gap_2p5()
            .child(render_search_input(
                &self.query,
                self.apps.len(),
                &theme,
                cx,
            ))
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            .child(app_list)
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .py_1()
                    .text_xs()
                    .text_color(theme.foreground_muted())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child("↑↓ navegar")
                            .child("·")
                            .child("↵ abrir"),
                    )
                    .child("esc cerrar"),
            )
    }
}
