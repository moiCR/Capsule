use gpui::{
    Context, Element, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, svg,
};
use services::{AppState, MprisService};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::lockscreen::LockScreen;

static LAST_MEDIA_STATE: Mutex<Option<(String, u64, Instant)>> = Mutex::new(None);

pub fn render_lockscreen_media_player(
    theme: &Theme,
    cx: &mut Context<LockScreen>,
) -> impl Element {
    let active_track = if cx.has_global::<AppState>() {
        cx.global::<AppState>().mpris.get_current_track()
    } else {
        services::MediaTrack::default()
    };

    if !active_track.has_media {
        return div().into_any_element();
    }

    let bus_name = active_track.bus_name.clone();
    let bus_prev = bus_name.clone();
    let bus_next = bus_name.clone();
    let bus_play = bus_name.clone();
    let is_playing = active_track.is_playing;

    div()
        .id("lockscreen-media-player")
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w(px(360.0))
        .px(px(20.0))
        .py(px(14.0))
        .rounded(px(20.0))
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.5))
        .shadow_md()
        .gap(px(8.0))
        // Track Title & Artist
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .w_full()
                .gap(px(2.0))
                .child(
                    div()
                        .font_family(theme.font_family())
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(14.0))
                        .text_color(theme.foreground())
                        .text_center()
                        .w_full()
                        .truncate()
                        .child(active_track.title.clone()),
                )
                .child(
                    div()
                        .font_family(theme.font_family())
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .text_center()
                        .w_full()
                        .truncate()
                        .child(active_track.artist.clone()),
                ),
        )
        // Media Controls
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .pt(px(2.0))
                // Prev
                .child(
                    div()
                        .id("ls-mpris-prev")
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_prev.clone();
                            tokio::spawn(async move {
                                MprisService::previous_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("skip-back.svg")
                                .size(px(16.0))
                                .text_color(theme.foreground_muted()),
                        ),
                )
                // Play / Pause
                .child(
                    div()
                        .id("ls-mpris-play-pause")
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_play.clone();
                            tokio::spawn(async move {
                                MprisService::play_pause_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(32.0))
                                .h(px(32.0))
                                .bg(theme.accent().opacity(0.25))
                                .rounded_full()
                                .child(
                                    svg()
                                        .path(if is_playing { "pause.svg" } else { "play.svg" })
                                        .size(px(15.0))
                                        .text_color(theme.accent()),
                                ),
                        ),
                )
                // Next
                .child(
                    div()
                        .id("ls-mpris-next")
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_next.clone();
                            tokio::spawn(async move {
                                MprisService::next_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("skip-forward.svg")
                                .size(px(16.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .into_any_element()
}

static LAST_LYRIC_ANIM: Mutex<Option<(usize, Instant)>> = Mutex::new(None);

pub fn render_lyrics_cascade(
    theme: &Theme,
    cx: &mut Context<LockScreen>,
) -> impl Element {
    let (active_track, lyrics_data) = if cx.has_global::<AppState>() {
        let app_state = cx.global::<AppState>();
        let track = app_state.mpris.get_current_track();

        let lyrics_opt = if track.has_media {
            if let Some(cached) = app_state
                .lyrics
                .get_cached_lyrics(&track.title, &track.artist)
            {
                cached
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
                None
            }
        } else {
            None
        };

        (track, lyrics_opt)
    } else {
        (services::MediaTrack::default(), None)
    };

    if !active_track.has_media {
        return div().into_any_element();
    }

    let id = format!("{} - {}", active_track.title, active_track.artist);
    let raw_pos = active_track.position_micros.unwrap_or(0).max(0) as u64;

    let estimated_pos_micros = if active_track.is_playing {
        if let Ok(mut guard) = LAST_MEDIA_STATE.lock() {
            if let Some((ref last_id, last_pos, last_time)) = *guard {
                if last_id == &id && raw_pos == last_pos {
                    raw_pos + last_time.elapsed().as_micros() as u64
                } else {
                    *guard = Some((id.clone(), raw_pos, Instant::now()));
                    raw_pos
                }
            } else {
                *guard = Some((id.clone(), raw_pos, Instant::now()));
                raw_pos
            }
        } else {
            raw_pos
        }
    } else {
        raw_pos
    };

    let pos = Duration::from_micros(estimated_pos_micros);

    let mut lines_col = div().flex().flex_col().gap(px(10.0)).w_full();

    if let Some(ref lyrics) = lyrics_data {
        if !lyrics.synced_lines.is_empty() {
            let active_idx = if let Some(idx) = lyrics
                .synced_lines
                .partition_point(|l| l.timestamp <= pos)
                .checked_sub(1)
            {
                idx
            } else {
                0
            };

            let anim_t = if let Ok(mut guard) = LAST_LYRIC_ANIM.lock() {
                if let Some((last_idx, last_start)) = *guard {
                    if last_idx != active_idx {
                        *guard = Some((active_idx, Instant::now()));
                        0.0
                    } else {
                        (last_start.elapsed().as_secs_f32() * 1000.0 / 250.0).clamp(0.0, 1.0)
                    }
                } else {
                    *guard = Some((active_idx, Instant::now()));
                    1.0
                }
            } else {
                1.0
            };

            let p = 1.0 - anim_t;
            let eased_t = 1.0 - p * p * p;

            let start_idx = active_idx.saturating_sub(2);
            let end_idx = (active_idx + 4).min(lyrics.synced_lines.len());

            for idx in start_idx..end_idx {
                let line = &lyrics.synced_lines[idx];
                let is_active = idx == active_idx;
                let is_past = idx < active_idx;

                if line.text.is_empty() {
                    continue;
                }

                let opacity = if is_active {
                    0.35 + 0.65 * eased_t
                } else if is_past {
                    (0.65 - 0.25 * eased_t).max(0.35)
                } else {
                    0.65
                };

                let font_size = if is_active {
                    13.0 + 3.0 * eased_t
                } else {
                    13.0
                };

                let y_offset = if is_active {
                    5.0 * (1.0 - eased_t)
                } else {
                    0.0
                };

                lines_col = lines_col.child(
                    div()
                        .font_family(theme.font_family())
                        .font_weight(if is_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_size(px(font_size))
                        .text_color(if is_active {
                            theme.accent()
                        } else if is_past {
                            theme.foreground_muted()
                        } else {
                            theme.foreground()
                        })
                        .opacity(opacity)
                        .relative()
                        .top(px(y_offset))
                        .child(line.text.clone()),
                );
            }
        }
    }

    div()
        .id("lockscreen-lyrics-cascade")
        .flex()
        .flex_col()
        .w(px(360.0))
        .p(px(16.0))
        .rounded(px(20.0))
        .bg(theme.surface().opacity(0.25))
        .border_1()
        .border_color(theme.surface().opacity(0.4))
        .shadow_lg()
        .child(lines_col)
        .into_any_element()
}
