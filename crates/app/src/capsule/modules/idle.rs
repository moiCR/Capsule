use chrono::{Local, Timelike};
use gpui::{div, prelude::*, px, Context, Entity, FontWeight, IntoElement, Render, Task, Window};
use services::{LyricsService, MprisService, NotificationStore};
use std::time::{Duration, Instant};
use ui::theme::Theme;

use super::super::widgets::vizualizer::Visualizer;

fn ease_out_cubic(t: f32) -> f32 {
    let p = 1.0 - t.clamp(0.0, 1.0);
    1.0 - p * p * p
}

pub struct IdleModule {
    audio_active: bool,
    time_str: String,
    current_sys_notif_id: Option<u32>,
    sys_notif_text: Option<String>,
    current_track_id: Option<String>,
    track_toast_text: Option<String>,
    track_toast_timer: Option<Instant>,
    current_lyric_line: Option<String>,
    prev_lyric_line: Option<String>,
    lyric_anim_progress: f32,
    lyric_anim_start: Option<Instant>,
    lyric_anim_task: Option<Task<()>>,
    visualizer: Entity<Visualizer>,
    #[allow(dead_code)]
    lyrics_service: LyricsService,
}

impl IdleModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let visualizer = cx.new(Visualizer::new);

        // Observe visualizer so child updates trigger parent re-render
        cx.observe(&visualizer, |_, _, cx| {
            cx.notify();
        })
        .detach();

        let lyrics_service = LyricsService::new();

        // Update clock using native GPUI timer
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let now = Local::now();
                let time_str = format!("{:02}:{:02}", now.hour(), now.minute());

                let res = this.update(cx, |this: &mut Self, cx| {
                    if this.time_str != time_str {
                        this.time_str = time_str;
                        cx.notify();
                    }
                });
                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        // Sync lyrics & system notifications with MPRIS / D-Bus
        let lyrics_service_clone = lyrics_service.clone();
        cx.spawn(async move |this, cx| {
            loop {
                let notif_store = NotificationStore::global();
                let active_sys_notif = notif_store.get_latest_active_notification();

                let (new_sys_id, new_sys_text) = if let Some(n) = &active_sys_notif {
                    let msg = if n.body.is_empty() {
                        format!("🔔 {}: {}", n.app_name, n.summary)
                    } else {
                        format!("🔔 {}: {} - {}", n.app_name, n.summary, n.body)
                    };
                    (Some(n.id), Some(msg))
                } else {
                    (None, None)
                };

                let players = MprisService::fetch_all_players().await;
                let active_track = players
                    .iter()
                    .find(|p| p.is_playing && p.has_media);

                let mut lyric_line = None;
                let mut new_track_id = None;
                let mut new_toast_text = None;

                if let Some(track) = active_track {
                    let id = format!("{} - {}", track.title, track.artist);
                    new_track_id = Some(id);

                    let album = if track.album.is_empty() {
                        None
                    } else {
                        Some(track.album.as_str())
                    };

                    let duration_secs = track
                        .length_micros
                        .map(|micros| (micros / 1_000_000) as u64);

                    if let Some(lyrics) = lyrics_service_clone
                        .get_lyrics_for_track(&track.title, &track.artist, album, duration_secs)
                        .await
                    {
                        let pos_micros = track.position_micros.unwrap_or(0).max(0) as u64;
                        let pos_duration = Duration::from_micros(pos_micros);

                        if let Some(line) = lyrics.get_current_line(pos_duration) {
                            lyric_line = Some(line.to_string());
                        }
                    }

                    let toast = if track.artist.is_empty() || track.artist == "No media playing" {
                        format!("🎵 {}", track.title)
                    } else {
                        format!("🎵 {} • {}", track.title, track.artist)
                    };
                    new_toast_text = Some(toast);
                }

                let is_audio_playing = active_track.is_some();

                let res = this.update(cx, |this: &mut Self, cx| {
                    let mut changed = false;

                    if this.audio_active != is_audio_playing {
                        this.audio_active = is_audio_playing;
                        this.set_active(is_audio_playing, cx);
                        changed = true;
                    }

                    // Check for D-Bus System Notification change
                    if this.current_sys_notif_id != new_sys_id {
                        let old_text = this.get_active_display_text();
                        this.current_sys_notif_id = new_sys_id;
                        this.sys_notif_text = new_sys_text;
                        this.prev_lyric_line = old_text;
                        this.lyric_anim_progress = 0.0;
                        this.lyric_anim_start = Some(Instant::now());
                        this.start_anim_task(cx);
                        changed = true;
                    }

                    // Check for track change -> trigger Toast Notification
                    if let Some(ref track_id) = new_track_id {
                        if this.current_track_id.as_ref() != Some(track_id) {
                            this.current_track_id = Some(track_id.clone());
                            this.track_toast_text = new_toast_text.clone();
                            this.track_toast_timer = Some(Instant::now());

                            if this.sys_notif_text.is_none() {
                                this.prev_lyric_line = this.get_active_display_text();
                                this.lyric_anim_progress = 0.0;
                                this.lyric_anim_start = Some(Instant::now());
                                this.start_anim_task(cx);
                            }
                            changed = true;
                        }
                    } else if this.current_track_id.is_some() {
                        this.current_track_id = None;
                        this.track_toast_text = None;
                        this.track_toast_timer = None;
                        changed = true;
                    }

                    // Expire toast timer after 3.2s
                    if let Some(timer) = this.track_toast_timer {
                        if timer.elapsed() >= Duration::from_millis(3200) {
                            if this.sys_notif_text.is_none() {
                                this.prev_lyric_line = this.track_toast_text.clone();
                                this.lyric_anim_progress = 0.0;
                                this.lyric_anim_start = Some(Instant::now());
                                this.start_anim_task(cx);
                            }
                            this.track_toast_text = None;
                            this.track_toast_timer = None;
                            changed = true;
                        }
                    }

                    // Update lyric line
                    if this.current_lyric_line != lyric_line {
                        if this.sys_notif_text.is_none() && this.track_toast_text.is_none() {
                            let old_text = this.get_active_display_text();
                            this.prev_lyric_line = old_text;
                            this.lyric_anim_progress = 0.0;
                            this.lyric_anim_start = Some(Instant::now());
                            this.start_anim_task(cx);
                        }
                        this.current_lyric_line = lyric_line;
                        changed = true;
                    }

                    if changed {
                        cx.notify();
                    }
                });
                if res.is_err() {
                    break;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
            }
        })
        .detach();

        let now = Local::now();
        Self {
            audio_active: false,
            time_str: format!("{:02}:{:02}", now.hour(), now.minute()),
            current_sys_notif_id: None,
            sys_notif_text: None,
            current_track_id: None,
            track_toast_text: None,
            track_toast_timer: None,
            current_lyric_line: None,
            prev_lyric_line: None,
            lyric_anim_progress: 1.0,
            lyric_anim_start: None,
            lyric_anim_task: None,
            visualizer,
            lyrics_service,
        }
    }

    fn start_anim_task(&mut self, cx: &mut Context<Self>) {
        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 220.0;
            loop {
                cx.background_executor().timer(Duration::from_millis(16)).await;
                let finished = this
                    .update(cx, |this: &mut Self, cx| {
                        if let Some(start) = this.lyric_anim_start {
                            let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                            let p = (elapsed / duration_ms).min(1.0);
                            this.lyric_anim_progress = p;
                            cx.notify();
                            p >= 1.0
                        } else {
                            true
                        }
                    })
                    .unwrap_or(true);

                if finished {
                    this.update(cx, |this: &mut Self, _| {
                        this.prev_lyric_line = None;
                        this.lyric_anim_task = None;
                    })
                    .ok();
                    break;
                }
            }
        });
        self.lyric_anim_task = Some(anim_task);
    }

    fn get_active_display_text(&self) -> Option<String> {
        if let Some(sys_notif) = &self.sys_notif_text {
            Some(sys_notif.clone())
        } else if let Some(toast) = &self.track_toast_text {
            Some(toast.clone())
        } else if let Some(line) = &self.current_lyric_line {
            if !line.is_empty() {
                Some(line.clone())
            } else {
                Some(self.time_str.clone())
            }
        } else {
            Some(self.time_str.clone())
        }
    }

    #[allow(dead_code)]
    pub fn set_active(&mut self, active: bool, cx: &mut Context<Self>) {
        let _ = self.visualizer.update(cx, |viz, cx| {
            viz.set_active(active, cx);
        });
    }

    pub fn desired_dimensions(&self) -> (f32, f32) {
        let text_width = |text: &str| -> f32 {
            let char_count = text.chars().count();
            char_count as f32 * 7.8
        };

        let extra_spacing = if self.audio_active { 64.0 } else { 32.0 };
        let min_w = if self.audio_active { 104.0 } else { 90.0 };

        let curr_text = self.get_active_display_text().unwrap_or_else(|| self.time_str.clone());
        let curr_w = (text_width(&curr_text) + extra_spacing).max(min_w);

        if let Some(prev_text) = &self.prev_lyric_line {
            let prev_w = (text_width(prev_text) + extra_spacing).max(min_w);
            let t = ease_out_cubic(self.lyric_anim_progress);
            (
                prev_w + (curr_w - prev_w) * t,
                42.0,
            )
        } else {
            (curr_w, 42.0)
        }
    }

    #[allow(dead_code)]
    pub fn desired_width(&self) -> f32 {
        self.desired_dimensions().0
    }
}

impl Render for IdleModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let curr_text = self.get_active_display_text().unwrap_or_else(|| self.time_str.clone());

        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .h_full()
            .px(px(16.0))
            .overflow_hidden()
            .gap(px(8.0));

        if self.audio_active {
            row = row.child(div().flex_shrink_0().child(self.visualizer.clone()));
        }

        let p = ease_out_cubic(self.lyric_anim_progress);
        let opacity = if p < 1.0 { p } else { 1.0 };
        let offset = if p < 1.0 { (1.0 - p) * 6.0 } else { 0.0 };

        row.child(
            div()
                .flex_shrink_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .font_weight(FontWeight::BOLD)
                .text_size(px(13.0))
                .text_color(theme.foreground())
                .opacity(opacity)
                .top(px(offset))
                .child(curr_text),
        )
    }
}
