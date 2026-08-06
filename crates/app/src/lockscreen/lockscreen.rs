use gpui::{Context, KeyDownEvent, Render, Window, div, prelude::*, px};
use services::AppState;
use std::sync::{Arc, Mutex};
use ui::theme::Theme;

use super::components::{
    render_auth_form, render_clock, render_lockscreen_media_player, render_lyrics_cascade,
    render_power_menu,
};

pub struct LockScreen {
    is_primary: bool,
    _username: String,
    password: String,
    auth_failed: bool,
    is_checking: bool,
    should_close: bool,
    focus_handle: gpui::FocusHandle,
    pending_result: Option<Arc<Mutex<Option<bool>>>>,
}

impl LockScreen {
    pub fn new(cx: &mut Context<Self>, is_primary: bool) -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "User".to_string());
        let focus_handle = cx.focus_handle();

        if is_primary {
            let compositor = if cx.has_global::<AppState>() {
                Some(cx.global::<AppState>().compositor.clone())
            } else {
                None
            };

            cx.spawn(async move |this, cx| {
                loop {
                    let frame_duration = if let Some(ref comp) = compositor {
                        comp.get_frame_duration()
                    } else {
                        std::time::Duration::from_millis(16)
                    };

                    cx.background_executor().timer(frame_duration).await;
                    let res = this.update(cx, |_, cx| {
                        cx.notify();
                    });
                    if res.is_err() {
                        break;
                    }
                }
            })
            .detach();
        }

        Self {
            is_primary,
            _username: username,
            password: String::new(),
            auth_failed: false,
            is_checking: false,
            should_close: false,
            focus_handle,
            pending_result: None,
        }
    }

    pub fn poll_result(&mut self, cx: &mut Context<Self>) {
        if !self.is_checking {
            return;
        }

        let finished_res = if let Some(ref slot) = self.pending_result {
            if let Ok(guard) = slot.lock() {
                *guard
            } else {
                None
            }
        } else {
            None
        };

        if let Some(is_valid) = finished_res {
            self.is_checking = false;
            self.pending_result = None;
            if is_valid {
                self.should_close = true;
            } else {
                self.auth_failed = true;
                self.password.clear();
            }
            cx.notify();
        }
    }

    fn submit_password(&mut self, cx: &mut Context<Self>) {
        if self.password.is_empty() || self.is_checking {
            return;
        }

        let pass = self.password.clone();
        let result_slot = Arc::new(Mutex::new(None));
        self.pending_result = Some(result_slot.clone());
        self.is_checking = true;
        self.auth_failed = false;
        cx.notify();

        tokio::spawn(async move {
            let is_valid = tokio::task::spawn_blocking(move || {
                services::PamService::authenticate_current_user(&pass).unwrap_or(false)
            })
            .await
            .unwrap_or(false);

            if let Ok(mut guard) = result_slot.lock() {
                *guard = Some(is_valid);
            }
        });
    }
}

impl Render for LockScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.poll_result(cx);

        if self.should_close {
            window.remove_window();
            return div().into_any_element();
        }

        if self.is_primary {
            window.focus(&self.focus_handle, cx);
        }
        let theme = cx.global::<Theme>().clone();

        let content = if self.is_primary {
            div()
                .relative()
                .w_full()
                .h_full()
                // Center: Clock & Date - Exact Screen Center
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .h_full()
                        .child(render_clock(&theme)),
                )
                // Bottom-Center: Password Input Box
                .child(
                    div()
                        .absolute()
                        .bottom(px(32.0))
                        .w_full()
                        .flex()
                        .justify_center()
                        .child(render_auth_form(
                            &theme,
                            self.password.len(),
                            self.auth_failed,
                            self.is_checking,
                        )),
                )
                // Bottom-Left: Lyrics Cascade & Media Player
                .child(
                    div()
                        .absolute()
                        .bottom(px(32.0))
                        .left(px(32.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(render_lyrics_cascade(&theme, cx))
                        .child(render_lockscreen_media_player(&theme, cx)),
                )
                // Bottom-Right: Power Menu with SVG icons
                .child(
                    div()
                        .absolute()
                        .bottom(px(32.0))
                        .right(px(32.0))
                        .child(render_power_menu(&theme)),
                )
        } else {
            // Secondary screens show a clean ambient clock
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .child(render_clock(&theme))
        };

        div()
            .id("lockscreen-root")
            .key_context("LockScreen")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();

                match key {
                    "escape" | "Escape" | "esc" | "\u{1b}" => {
                        this.password.clear();
                        this.auth_failed = false;
                        cx.notify();
                        return;
                    }
                    _ => {}
                }

                if !this.is_primary {
                    return;
                }
                window.focus(&this.focus_handle, cx);
                let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

                if ctrl {
                    match key {
                        "v" => {
                            if let Some(item) = cx.read_from_clipboard() {
                                if let Some(text) = item.text() {
                                    let clean_text: String =
                                        text.chars().filter(|c| !c.is_control()).collect();
                                    if !clean_text.is_empty() {
                                        this.password.push_str(&clean_text);
                                        this.auth_failed = false;
                                        cx.notify();
                                    }
                                }
                            }
                            return;
                        }
                        "u" | "w" | "c" | "x" => {
                            this.password.clear();
                            this.auth_failed = false;
                            cx.notify();
                            return;
                        }
                        _ => return,
                    }
                }

                if event.keystroke.modifiers.alt {
                    return;
                }

                match key {
                    "enter" | "return" | "numpad_enter" | "kp_enter" | "numpadenter" => {
                        this.submit_password(cx);
                    }
                    "backspace" => {
                        this.password.pop();
                        this.auth_failed = false;
                        cx.notify();
                    }
                    "space" => {
                        this.password.push(' ');
                        this.auth_failed = false;
                        cx.notify();
                    }
                    k => {
                        if let Some(ref ch) = event.keystroke.key_char {
                            let clean: String = ch.chars().filter(|c| !c.is_control()).collect();
                            if !clean.is_empty() {
                                this.password.push_str(&clean);
                                this.auth_failed = false;
                                cx.notify();
                            }
                        } else if k.chars().count() == 1 {
                            let c = k.chars().next().unwrap();
                            if !c.is_control() {
                                this.password.push(c);
                                this.auth_failed = false;
                                cx.notify();
                            }
                        }
                    }
                }
            }))
            .w_full()
            .h_full()
            .bg(theme.background())
            .child(content)
            .into_any_element()
    }
}

impl Drop for LockScreen {
    fn drop(&mut self) {
        if self.is_primary {
            crate::panel::LockScreenPanel::mark_closed();
            if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
                let _ = std::process::Command::new("hyprctl")
                    .args(["eval", "hl.dsp.submap(\"reset\")"])
                    .spawn();
            }
        }
    }
}
