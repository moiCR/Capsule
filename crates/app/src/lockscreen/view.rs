use gpui::{Context, KeyDownEvent, Render, Window, div, img, prelude::*, px, svg};
use services::AppState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ui::theme::Theme;

use super::components::{render_auth_form, render_clock, render_lyrics};

pub struct LockScreen {
    is_primary: bool,
    pub username: String,
    password: String,
    auth_failed: bool,
    is_checking: bool,
    pub focus_handle: gpui::FocusHandle,
    pub entry_start_time: Option<Instant>,
    last_track_key: String,
    last_track_pos_micros: u64,
    last_track_pos_time: Instant,
    pub current_lyric_idx: usize,
    pub lyric_change_time: Instant,
    pub blurred_wallpaper: Option<PathBuf>,
    fallback_blur_slot: Arc<Mutex<Option<PathBuf>>>,
}

impl LockScreen {
    pub fn new(cx: &mut Context<Self>, is_primary: bool, blur_path: Option<PathBuf>) -> Self {
        let username = std::env::var("USER").unwrap_or_else(|_| "User".to_string());
        let focus_handle = cx.focus_handle();

        if is_primary {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(250))
                        .await;
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            })
            .detach();
        }

        let blurred_wallpaper = blur_path.or_else(|| {
            if cx.has_global::<AppState>() {
                let wallpaper_service = &cx.global::<AppState>().wallpaper;
                wallpaper_service.get_blurred_wallpaper()
            } else {
                None
            }
        });

        let mut screen = Self {
            is_primary,
            username,
            password: String::new(),
            auth_failed: false,
            is_checking: false,
            focus_handle,
            entry_start_time: None,
            last_track_key: String::new(),
            last_track_pos_micros: 0,
            last_track_pos_time: Instant::now(),
            current_lyric_idx: 0,
            lyric_change_time: Instant::now(),
            blurred_wallpaper,
            fallback_blur_slot: Arc::new(Mutex::new(None)),
        };

        if screen.blurred_wallpaper.is_none() {
            screen.init_fallback_blur();
        }
        screen
    }

    fn init_fallback_blur(&mut self) {
        let wallpaper_path = dirs::home_dir().and_then(|h| {
            let conf = h.join(".config/capsule/current_wallpaper");
            std::fs::read_to_string(conf)
                .ok()
                .map(|s| PathBuf::from(s.trim()))
                .filter(|p| p.exists())
        });

        let Some(target) = wallpaper_path else {
            return;
        };

        let slot = self.fallback_blur_slot.clone();
        services::spawn_blocking(move || {
            let res = services::wallpaper::WallpaperService::create_or_get_blur(&target);
            if let Ok(mut guard) = slot.lock() {
                *guard = res;
            }
        });
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn submit_password(&mut self, cx: &mut Context<Self>) {
        if self.password.is_empty() || self.is_checking {
            return;
        }

        let pass = self.password.clone();
        self.is_checking = true;
        self.auth_failed = false;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let is_valid = services::spawn_blocking(move || {
                match services::PamService::authenticate_current_user(&pass) {
                    Ok(valid) => valid,
                    Err(error) => {
                        eprintln!("[PAM] Authentication failed: {error:#}");
                        false
                    }
                }
            })
            .await
            .unwrap_or(false);

            let _ = this.update(cx, |this: &mut Self, cx| {
                this.is_checking = false;
                if is_valid {
                    cx.defer(|cx| {
                        crate::panel::LockScreenPanel::close_all(cx);
                    });
                } else {
                    this.auth_failed = true;
                    this.password.clear();
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

impl Render for LockScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.blurred_wallpaper.is_none()
            && let Ok(mut guard) = self.fallback_blur_slot.lock()
            && let Some(path) = guard.take()
        {
            self.blurred_wallpaper = Some(path);
        }

        let theme = cx.global::<Theme>().clone();
        let lock_config = if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            app_state.config.get().lockscreen.clone()
        } else {
            services::LockScreenConfig::default()
        };

        if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            let track = app_state.mpris.get_current_track();
            if track.has_media {
                let id = format!("{} - {}", track.title, track.artist);
                let raw_pos = track.position_micros.unwrap_or(0).max(0) as u64;

                if self.last_track_key != id || raw_pos != self.last_track_pos_micros {
                    self.last_track_key = id;
                    self.last_track_pos_micros = raw_pos;
                    self.last_track_pos_time = Instant::now();
                }

                let estimated_pos = if track.is_playing {
                    raw_pos + self.last_track_pos_time.elapsed().as_micros() as u64
                } else {
                    raw_pos
                };
                let pos = Duration::from_micros(estimated_pos);

                if let Some(cached_opt) = app_state
                    .lyrics
                    .get_cached_lyrics(&track.title, &track.artist)
                {
                    if let Some(lyrics) = cached_opt.filter(|l| !l.synced_lines.is_empty()) {
                        let active_idx = lyrics
                            .synced_lines
                            .partition_point(|l| l.timestamp <= pos)
                            .saturating_sub(1);

                        if active_idx != self.current_lyric_idx {
                            self.current_lyric_idx = active_idx;
                            self.lyric_change_time = Instant::now();
                        }
                    }
                } else {
                    let dur_secs = track.length_micros.map(|l| (l.max(0) / 1_000_000) as u64);
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
            }
        }

        let start_time = *self.entry_start_time.get_or_insert_with(Instant::now);
        let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
        let anim_duration = 600.0;
        let entry_t = (elapsed / anim_duration).min(1.0);
        let eased = 1.0 - (1.0 - entry_t).powi(3);

        if entry_t < 1.0 {
            window.request_animation_frame();
        }

        let lyric_elapsed = self.lyric_change_time.elapsed().as_secs_f32() * 1000.0;
        let lyric_anim_progress = (lyric_elapsed / 300.0).min(1.0);
        if lyric_anim_progress < 1.0 {
            window.request_animation_frame();
        }

        let clock_y = (1.0 - eased) * -28.0;
        let clock_opacity = eased;

        let lyrics_y = (1.0 - eased) * 16.0;
        let lyrics_opacity = eased;

        let user_y = (1.0 - eased) * 36.0;
        let user_opacity = eased;

        let top_bar_opacity = eased;

        let (battery_opt, wifi_icon_opt) = if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            let battery = app_state.power.get_battery();
            let net = app_state.network.get_status();
            let wifi_icon = if net.ethernet_connected {
                Some("ethernet.svg")
            } else if net.wifi_enabled && !net.wifi_ssid.is_empty() {
                if net.wifi_signal > 60 {
                    Some("wifi-high.svg")
                } else if net.wifi_signal > 20 {
                    Some("wifi-low.svg")
                } else {
                    Some("wifi.svg")
                }
            } else if net.wifi_enabled {
                Some("wifi-zero.svg")
            } else {
                None
            };
            (battery, wifi_icon)
        } else {
            (None, None)
        };

        let top_bar = div()
            .absolute()
            .top(px(24.0))
            .right(px(32.0))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .opacity(top_bar_opacity)
            .when_some(battery_opt, |d, bat| {
                d.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(4.0))
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded(px(6.0))
                        .bg(theme.green().opacity(0.85))
                        .child(
                            div()
                                .font_family(theme.font_family())
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.background())
                                .child(bat.percentage.to_string()),
                        )
                        .child(
                            svg()
                                .path(if bat.is_charging {
                                    "battery-charging.svg"
                                } else if bat.percentage <= 15 {
                                    "battery-low.svg"
                                } else if bat.percentage <= 40 {
                                    "battery-medium.svg"
                                } else {
                                    "battery-full.svg"
                                })
                                .size(px(12.0))
                                .text_color(theme.background()),
                        ),
                )
            })
            .when_some(wifi_icon_opt, |d, icon| {
                d.child(
                    svg()
                        .path(icon)
                        .size(px(15.0))
                        .text_color(theme.foreground()),
                )
            });

        let content = if self.is_primary {
            div()
                .flex()
                .flex_col()
                .justify_between()
                .items_center()
                .size_full()
                .pt(px(120.0))
                .pb(px(100.0))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .opacity(clock_opacity)
                        .mt(px(clock_y))
                        .child(render_clock(&theme, &lock_config)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .w_full()
                        .max_w(px(520.0))
                        .px(px(20.0))
                        .opacity(lyrics_opacity)
                        .mt(px(lyrics_y))
                        .child(render_lyrics(
                            &theme,
                            self.current_lyric_idx,
                            lyric_anim_progress,
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .opacity(user_opacity)
                        .mt(px(user_y))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(46.0))
                                .h(px(46.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.45))
                                .border_1()
                                .border_color(theme.surface().opacity(0.35))
                                .child(
                                    svg()
                                        .path("user.svg")
                                        .size(px(22.0))
                                        .text_color(theme.foreground().opacity(0.95)),
                                ),
                        )
                        .child(
                            div()
                                .font_family(theme.font_family())
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_size(px(14.0))
                                .text_color(theme.foreground().opacity(0.9))
                                .child(self.username.clone()),
                        )
                        .child(render_auth_form(
                            &theme,
                            self.password.len(),
                            self.auth_failed,
                            self.is_checking,
                            cx,
                        )),
                )
        } else {
            div()
                .flex()
                .flex_col()
                .justify_center()
                .items_center()
                .size_full()
                .opacity(clock_opacity)
                .mt(px(clock_y))
                .child(render_clock(&theme, &lock_config))
        };

        div()
            .id("lockscreen-root")
            .key_context("LockScreen")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.is_checking {
                    return;
                }

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
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if this.is_primary {
                        window.focus(&this.focus_handle, cx);
                    }
                }),
            )
            .relative()
            .w_full()
            .h_full()
            .bg(theme.background())
            .when_some(self.blurred_wallpaper.as_ref(), |this, blur_path| {
                this.child(
                    img(blur_path.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover),
                )
            })
            .child(
                div()
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .bg(theme.background().opacity(0.35)),
            )
            .child(top_bar)
            .child(div().relative().size_full().child(content))
            .into_any_element()
    }
}

impl Drop for LockScreen {
    fn drop(&mut self) {
        if self.is_primary {
            crate::panel::LockScreenPanel::mark_closed();
        }
        if let Some(ref path) = self.blurred_wallpaper
            && path
                .to_string_lossy()
                .starts_with("/tmp/capsule_lock_blur_")
        {
            let _ = std::fs::remove_file(path);
        }
    }
}
