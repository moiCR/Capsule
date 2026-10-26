use gpui::{ColorExt, Context, KeyDownEvent, Render, Window, div, img, prelude::*, px, svg};
use services::AppState;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ui::components::InputState;
use ui::theme::Theme;

use super::components::{
    render_auth_form, render_clock, render_lockscreen_media_player, render_lyrics,
    render_power_menu,
};

pub struct LockScreen {
    is_primary: bool,
    pub username: String,
    pub password_input: InputState,
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
            password_input: InputState::new("").with_masked(true).with_focused(true),
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
        let pass = self.password_input.text();
        if pass.is_empty() || self.is_checking {
            return;
        }

        self.is_checking = true;
        self.auth_failed = false;
        cx.notify();

        let (tx, mut rx) = tokio::sync::oneshot::channel();
        std::thread::spawn(move || {
            let res =
                std::panic::catch_unwind(|| match services::PamService::authenticate_current_user(
                    &pass,
                ) {
                    Ok(valid) => valid,
                    Err(error) => {
                        eprintln!("[PAM] Authentication failed: {error:#}");
                        false
                    }
                })
                .unwrap_or(false);
            let _ = tx.send(res);
        });

        cx.spawn(async move |this, cx| {
            let mut is_valid = false;
            for _ in 0..100 {
                if let Ok(res) = rx.try_recv() {
                    is_valid = res;
                    break;
                }
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
            }

            let _ = this.update(cx, |this: &mut Self, cx| {
                this.is_checking = false;
                if is_valid {
                    cx.defer(|cx| {
                        crate::panel::LockScreenPanel::close_all(cx);
                    });
                } else {
                    this.auth_failed = true;
                    this.password_input.clear();
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
            .gap(px(12.0))
            .opacity(top_bar_opacity)
            .when_some(battery_opt, |d, bat| {
                let is_crit = bat.percentage <= 15;
                let bg_color = if is_crit {
                    theme.red().opacity(0.85)
                } else if bat.is_charging {
                    theme.green().opacity(0.85)
                } else {
                    theme.surface_container_high().opacity(0.85)
                };
                let text_color = if is_crit || bat.is_charging {
                    theme.background()
                } else {
                    theme.foreground()
                };
                d.child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(10.0))
                        .py(px(4.0))
                        .rounded_full()
                        .bg(bg_color)
                        .border_1()
                        .border_color(theme.outline_variant().opacity(0.4))
                        .child(
                            div()
                                .font_family(theme.font_family())
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(text_color)
                                .child(format!("{}%", bat.percentage)),
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
                                .size(px(14.0))
                                .text_color(text_color),
                        ),
                )
            })
            .when_some(wifi_icon_opt, |d, icon| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(34.0))
                        .h(px(34.0))
                        .rounded_full()
                        .bg(theme.surface_container_high().opacity(0.85))
                        .border_1()
                        .border_color(theme.outline_variant().opacity(0.4))
                        .child(
                            svg()
                                .path(icon)
                                .size(px(15.0))
                                .text_color(theme.foreground()),
                        ),
                )
            })
            .when(self.is_primary, |d| {
                d.child(render_power_menu(&theme))
            });

        let content = if self.is_primary {
            let has_lyrics = if cx.has_global::<AppState>() {
                let app_state = cx.global::<AppState>();
                let track = app_state.mpris.get_current_track();
                if track.has_media {
                    app_state
                        .lyrics
                        .get_cached_lyrics(&track.title, &track.artist)
                        .flatten()
                        .map(|l| !l.synced_lines.is_empty())
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            };

            let middle_widget = if has_lyrics {
                render_lyrics(&theme, self.current_lyric_idx, lyric_anim_progress, cx).into_any_element()
            } else {
                render_lockscreen_media_player(&theme, cx).into_any_element()
            };

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
                        .child(middle_widget),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(10.0))
                        .opacity(user_opacity)
                        .mt(px(user_y))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(52.0))
                                .h(px(52.0))
                                .rounded_full()
                                .bg(theme.surface_container_high().opacity(0.85))
                                .border_1()
                                .border_color(theme.outline_variant().opacity(0.4))
                                .child(
                                    svg()
                                        .path("user.svg")
                                        .size(px(24.0))
                                        .text_color(theme.on_surface_variant()),
                                ),
                        )
                        .child(
                            div()
                                .font_family(theme.font_family())
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_size(px(15.0))
                                .text_color(theme.foreground())
                                .child(self.username.clone()),
                        )
                        .child(render_auth_form(
                            &theme,
                            &self.password_input,
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
                        this.password_input.clear();
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

                match key {
                    "enter" | "return" | "numpad_enter" | "kp_enter" | "numpadenter" => {
                        this.submit_password(cx);
                    }
                    _ => {
                        if this.password_input.handle_key_down(event, cx) {
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
