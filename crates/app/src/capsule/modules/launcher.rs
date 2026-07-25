use gpui::{
    EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render, ScrollHandle, Window,
    div, prelude::*, px, svg,
};
use services::{AppState, Application, LauncherService};
use ui::theme::Theme;

pub enum LauncherEvent {
    Close,
}

pub struct LauncherModule {
    service: LauncherService,
    query: String,
    apps: Vec<Application>,
    selected_index: usize,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    mouse_moved: bool,
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

        let service_clone = self.service.clone();
        cx.spawn(async move |this, cx| {
            let _ = service_clone.refresh().await;
            let _ = this.update(cx, |this: &mut Self, cx| {
                if this.query.is_empty() {
                    this.apps = this.service.search("");
                    cx.notify();
                }
            });
        })
        .detach();

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

    pub fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        match key {
            "down" | "arrowdown" => self.select_next(cx),
            "up" | "arrowup" => self.select_prev(cx),
            "enter" => {
                self.launch_selected(cx);
            }
            "backspace" => {
                if !self.query.is_empty() {
                    let mut q = self.query.clone();
                    q.pop();
                    self.update_search(q, cx);
                }
            }
            "escape" => {
                self.reset_search(cx);
                cx.emit(LauncherEvent::Close);
            }
            _ => {
                if let Some(keystroke_text) = &event.keystroke.key_char {
                    if !keystroke_text.chars().any(|c| c.is_control()) {
                        let mut q = self.query.clone();
                        q.push_str(keystroke_text);
                        self.update_search(q, cx);
                    }
                }
            }
        }
    }
}

impl Render for LauncherModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let placeholder = if self.query.is_empty() {
            "Buscar aplicación..."
        } else {
            ""
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
            .size_full()
            .p_3p5()
            .gap_2p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .w_full()
                    .px_3p5()
                    .py_2p5()
                    .bg(theme.surface())
                    .rounded(px(42.0))
                    .child(
                        svg()
                            .path("search.svg")
                            .w_4()
                            .h_4()
                            .text_color(theme.foreground_muted()),
                    )
                    .child(div().flex_1().text_sm().child(if self.query.is_empty() {
                        div()
                            .text_color(theme.foreground_muted())
                            .child(placeholder)
                    } else {
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .child(self.query.clone())
                    }))
                    .child(if !self.query.is_empty() {
                        div()
                            .id("clear-search-btn")
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.reset_search(cx);
                            }))
                            .child(
                                svg()
                                    .path("close.svg")
                                    .w_3p5()
                                    .h_3p5()
                                    .text_color(theme.foreground_muted()),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted())
                            .child(format!("{}", self.apps.len()))
                            .into_any_element()
                    }),
            )
            // Divider
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            // App List
            .child(
                div()
                    .id("launcher-app-list")
                    .track_scroll(&self.scroll_handle)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_scroll()
                    .gap_1()
                    .children(self.apps.iter().enumerate().map(|(idx, app)| {
                        let is_selected = idx == self.selected_index;
                        let bg_color = if is_selected {
                            theme.surface()
                        } else {
                            gpui::hsla(0.0, 0.0, 0.0, 0.0)
                        };

                        let text_color = theme.foreground();
                        let muted_color = theme.foreground_muted();
                        let app_clone = app.clone();

                        div()
                            .id(idx)
                            .flex()
                            .items_center()
                            .gap_3()
                            .px_3()
                            .py_2()
                            .rounded_lg()
                            .bg(bg_color)
                            .cursor_pointer()
                            .on_hover(cx.listener(move |this, &hovered, _window, cx| {
                                if hovered && this.mouse_moved && this.selected_index != idx {
                                    this.selected_index = idx;
                                    cx.notify();
                                }
                            }))
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                let _ = app_clone.launch();
                                this.reset_search(cx);
                                cx.emit(LauncherEvent::Close);
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w_8()
                                    .h_8()
                                    .rounded_md()
                                    .bg(theme.background_alt())
                                    .overflow_hidden()
                                    .child(if let Some(icon_path) = &app.icon_path {
                                        gpui::img(icon_path.clone()).w_5().h_5().into_any_element()
                                    } else {
                                        svg()
                                            .path("sparkles.svg")
                                            .w_4()
                                            .h_4()
                                            .text_color(theme.accent())
                                            .into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(text_color)
                                            .child(app.name.clone()),
                                    )
                                    .child(if let Some(desc) = &app.generic_name {
                                        div().text_xs().text_color(muted_color).child(desc.clone())
                                    } else if let Some(comment) = &app.comment {
                                        div()
                                            .text_xs()
                                            .text_color(muted_color)
                                            .child(comment.clone())
                                    } else {
                                        div()
                                            .text_xs()
                                            .text_color(muted_color)
                                            .child(app.exec.clone())
                                    }),
                            )
                    })),
            )
            // Footer Shortcuts Hint
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
