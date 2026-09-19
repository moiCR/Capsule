use gpui::{Context, KeyDownEvent, Render, Task, Window, div, prelude::*, px, svg};
use services::AppState;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ui::theme::Theme;

use super::components::{
    render_auth_form, render_clock, render_lockscreen_media_player, render_lyrics,
    render_power_menu,
};

pub struct LockScreen {
    is_primary: bool,
    pub username: String,
    password: String,
    auth_failed: bool,
    is_checking: bool,
    should_close: bool,
    focus_handle: gpui::FocusHandle,
    pending_result: Option<Arc<Mutex<Option<bool>>>>,
    pub current_lyric_idx: usize,
    pub lyric_anim_progress: f32,
    lyric_anim_task: Option<Task<()>>,
}

impl LockScreen {
    pub fn new(cx: &mut Context<Self>, is_primary: bool) -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "User".to_string());
        let focus_handle = cx.focus_handle();

        if is_primary {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_secs(1))
                        .await;
                    let res = this.update(cx, |_, cx| {
                        cx.notify();
                    });
                    if res.is_err() {
                        break;
                    }
                }
            })
            .detach();

            cx.spawn(async move |this, cx| {
                let mut last_track_key = String::new();
                let mut last_pos_micros: u64 = 0;
                let mut last_pos_time = Instant::now();

                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(50))
                        .await;

                    let res = this.update(cx, |this: &mut Self, cx| {
                        if !cx.has_global::<AppState>() {
                            return;
                        }
                        let app_state = cx.global::<AppState>();
                        let track = app_state.mpris.get_current_track();

                        if !track.has_media || !track.is_playing {
                            return;
                        }

                        let id = format!("{} - {}", track.title, track.artist);
                        let raw_pos = track.position_micros.unwrap_or(0).max(0) as u64;

                        if last_track_key != id || raw_pos != last_pos_micros {
                            last_track_key = id;
                            last_pos_micros = raw_pos;
                            last_pos_time = Instant::now();
                        }

                        let estimated_pos = raw_pos + last_pos_time.elapsed().as_micros() as u64;
                        let pos = Duration::from_micros(estimated_pos);

                        if let Some(cached_opt) = app_state
                            .lyrics
                            .get_cached_lyrics(&track.title, &track.artist)
                        {
                            if let Some(lyrics) = cached_opt.filter(|l| !l.synced_lines.is_empty())
                            {
                                let active_idx = lyrics
                                    .synced_lines
                                    .partition_point(|l| l.timestamp <= pos)
                                    .saturating_sub(1);

                                if active_idx != this.current_lyric_idx {
                                    this.current_lyric_idx = active_idx;
                                    this.lyric_anim_progress = 0.0;
                                    this.start_lyric_animation(cx);
                                }
                            }
                        } else {
                            let dur_secs =
                                track.length_micros.map(|l| (l.max(0) / 1_000_000) as u64);
                            app_state.lyrics.fetch_lyrics_in_background(
                                track.title.clone(),
                                track.artist.clone(),
                                if track.album.is_empty() {
                                    None
                                } else {
                                    Some(track.album.clone())
                                },
                                dur_secs,
                            );
                        }
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
            username,
            password: String::new(),
            auth_failed: false,
            is_checking: false,
            should_close: false,
            focus_handle,
            pending_result: None,
            current_lyric_idx: 0,
            lyric_anim_progress: 1.0,
            lyric_anim_task: None,
        }
    }

    fn start_lyric_animation(&mut self, cx: &mut Context<Self>) {
        let frame_duration = if cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.get_frame_duration()
        } else {
            Duration::from_millis(16)
        };

        let anim_task = cx.spawn(async move |this, cx| {
            let start = Instant::now();
            let duration_ms = 220.0;

            loop {
                cx.background_executor().timer(frame_duration).await;
                let finished = this
                    .update(cx, |this: &mut Self, cx| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let p = (elapsed / duration_ms).min(1.0);
                        this.lyric_anim_progress = p;
                        cx.notify();
                        let done = p >= 1.0;
                        if done {
                            this.lyric_anim_task = None;
                        }
                        done
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });
        self.lyric_anim_task = Some(anim_task);
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

    pub fn submit_password(&mut self, cx: &mut Context<Self>) {
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
        let (lock_config, capsule_round) = if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            let cfg = app_state.config.get();
            (cfg.lockscreen.clone(), cfg.ui.capsule_round)
        } else {
            (services::LockScreenConfig::default(), 42.0)
        };

        let content = if self.is_primary {
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .h_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .w(px(380.0))
                        .p(px(24.0))
                        .gap(px(16.0))
                        .rounded(px(capsule_round))
                        .bg(theme.background())
                        .border_1()
                        .border_color(theme.surface().opacity(0.35))
                        .shadow_xl()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w(px(40.0))
                                        .h(px(40.0))
                                        .rounded_full()
                                        .bg(theme.surface().opacity(0.5))
                                        .border_1()
                                        .border_color(theme.surface().opacity(0.3))
                                        .child(
                                            svg()
                                                .path("lock.svg")
                                                .size(px(16.0))
                                                .text_color(theme.accent()),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_family(theme.font_family())
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_size(px(13.5))
                                        .text_color(theme.foreground())
                                        .child(self.username.clone()),
                                ),
                        )
                        .when(lock_config.show_clock, |d| {
                            d.child(render_clock(&theme, &lock_config))
                        })
                        .child(render_auth_form(
                            &theme,
                            self.password.len(),
                            self.auth_failed,
                            self.is_checking,
                            cx,
                        ))
                        .when(lock_config.show_media_player, |d| {
                            d.child(render_lockscreen_media_player(&theme, cx)).child(
                                render_lyrics(
                                    &theme,
                                    self.current_lyric_idx,
                                    self.lyric_anim_progress,
                                    cx,
                                ),
                            )
                        })
                        .child(render_power_menu(&theme)),
                )
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .h_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .w(px(320.0))
                        .p(px(24.0))
                        .gap(px(12.0))
                        .rounded(px(capsule_round))
                        .bg(theme.background())
                        .border_1()
                        .border_color(theme.surface().opacity(0.35))
                        .shadow_xl()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(36.0))
                                .h(px(36.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.5))
                                .border_1()
                                .border_color(theme.surface().opacity(0.3))
                                .child(
                                    svg()
                                        .path("lock.svg")
                                        .size(px(15.0))
                                        .text_color(theme.accent()),
                                ),
                        )
                        .when(lock_config.show_clock, |d| {
                            d.child(render_clock(&theme, &lock_config))
                        }),
                )
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
                            if let Some(text) =
                                cx.read_from_clipboard().and_then(|item| item.text())
                            {
                                let clean_text: String =
                                    text.chars().filter(|c| !c.is_control()).collect();
                                if !clean_text.is_empty() {
                                    this.password.push_str(&clean_text);
                                    this.auth_failed = false;
                                    cx.notify();
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
                        } else if let Some(c) =
                            k.chars().next().filter(|c| !c.is_control() && k.len() == 1)
                        {
                            this.password.push(c);
                            this.auth_failed = false;
                            cx.notify();
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
