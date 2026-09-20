use gpui::{Context, FontWeight, IntoElement, StyledImage, canvas, div, img, prelude::*, px, svg};
use services::{AppState, MediaTrack, MprisService};
use std::path::Path;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn resolve_art_path(track: &MediaTrack) -> Option<String> {
    track
        .local_art_path
        .as_deref()
        .map(Path::new)
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| {
            track.art_url.as_ref().and_then(|url| {
                let clean_path = url.strip_prefix("file://").unwrap_or(url.as_str());
                let p = Path::new(clean_path);
                if p.exists() {
                    Some(clean_path.to_string())
                } else {
                    None
                }
            })
        })
}

#[allow(clippy::too_many_arguments)]
pub fn render_media_player_widget(
    dashboard_w: f32,
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    prev_art_path: Option<&str>,
    anim_progress: f32,
    media_tracker: &DimensionTracker,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> gpui::AnyElement {
    let content_w = (dashboard_w - 32.0).max(100.0);
    let card_w = px(content_w);
    let card_radius_val = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        22.0
    };
    let card_radius = px(card_radius_val);
    let inner_radius = px((card_radius_val - 1.0).max(0.0));

    let has_media = active_track.has_media && !active_track.title.is_empty();
    let is_playing = has_media && active_track.is_playing;
    let art_path = if has_media {
        resolve_art_path(active_track)
    } else {
        None
    };
    let card_h = px(118.0);
    let art_size = px(94.0);

    // Outer Hero Card (identical dimensions and style whether playing or idle)
    let mut card = div()
        .id("media-player-hero-card")
        .relative()
        .w(card_w)
        .h(card_h)
        .rounded(card_radius)
        .border_1()
        .border_color(theme.surface().opacity(0.18))
        .overflow_hidden()
        .bg(theme.surface().opacity(0.35));

    // Ambient blurred album art background layer (contained strictly within inner bounds)
    let card_ratio = content_w / 118.0;

    if anim_progress < 1.0
        && let Some(prev) = prev_art_path
    {
        let prev_opacity = (1.0 - anim_progress).clamp(0.0, 1.0) * 0.18;
        if prev_opacity > 0.005 {
            card = card.child(
                div()
                    .absolute()
                    .inset_0()
                    .size_full()
                    .rounded(inner_radius)
                    .overflow_hidden()
                    .opacity(prev_opacity)
                    .child(
                        img(prev.to_string())
                            .size_full()
                            .aspect_ratio(card_ratio)
                            .object_fit(gpui::ObjectFit::Cover)
                            .rounded(inner_radius),
                    ),
            );
        }
    }

    if let Some(ref image_path) = art_path {
        let ambient_opacity = if prev_art_path.is_some() || anim_progress < 1.0 {
            0.18 * anim_progress.clamp(0.0, 1.0)
        } else {
            0.18
        };

        card = card.child(
            div()
                .absolute()
                .inset_0()
                .size_full()
                .rounded(inner_radius)
                .overflow_hidden()
                .opacity(ambient_opacity)
                .child(
                    img(image_path.clone())
                        .size_full()
                        .aspect_ratio(card_ratio)
                        .object_fit(gpui::ObjectFit::Cover)
                        .rounded(inner_radius),
                ),
        );
    }

    if art_path.is_some() || (prev_art_path.is_some() && anim_progress < 1.0) {
        let scrim_opacity = if art_path.is_some() {
            0.65
        } else {
            0.65 * (1.0 - anim_progress)
        };
        card = card.child(
            div()
                .absolute()
                .inset_0()
                .size_full()
                .rounded(inner_radius)
                .bg(theme.background().opacity(scrim_opacity)),
        );
    }

    // Left Column: Square Album Art or Placeholder Box
    let mut art_box = div()
        .id("media-art-container")
        .relative()
        .size(art_size)
        .rounded(px(16.0))
        .overflow_hidden()
        .border_1()
        .border_color(theme.surface().opacity(0.25))
        .bg(theme.surface().opacity(0.5));

    if has_media {
        // Crossfade previous art if still animating
        if anim_progress < 1.0
            && let Some(prev) = prev_art_path
        {
            let prev_opacity = (1.0 - anim_progress).clamp(0.0, 1.0);
            if prev_opacity > 0.01 {
                art_box = art_box.child(
                    div()
                        .absolute()
                        .inset_0()
                        .size_full()
                        .opacity(prev_opacity)
                        .child(
                            img(prev.to_string())
                                .size(art_size)
                                .aspect_ratio(1.0)
                                .object_fit(gpui::ObjectFit::Cover)
                                .rounded(px(16.0)),
                        ),
                );
            }
        }

        // Current art
        if let Some(ref image_path) = art_path {
            let current_opacity = if prev_art_path.is_some() || anim_progress < 1.0 {
                anim_progress.clamp(0.0, 1.0)
            } else {
                1.0
            };

            art_box = art_box.child(
                div()
                    .absolute()
                    .inset_0()
                    .size_full()
                    .opacity(current_opacity)
                    .child(
                        img(image_path.clone())
                            .size(art_size)
                            .aspect_ratio(1.0)
                            .object_fit(gpui::ObjectFit::Cover)
                            .rounded(px(16.0)),
                    ),
            );
        } else {
            // Fallback icon if no art
            art_box = art_box.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .child(
                        svg()
                            .path("music.svg")
                            .size(px(32.0))
                            .text_color(theme.foreground_muted().opacity(0.6)),
                    ),
            );
        }

        // Playing indicator badge on cover art
        if is_playing {
            art_box = art_box.child(
                div()
                    .absolute()
                    .right(px(6.0))
                    .bottom(px(6.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(22.0))
                    .rounded_full()
                    .bg(theme.accent())
                    .child(
                        svg()
                            .path("music.svg")
                            .size(px(11.0))
                            .text_color(theme.background()),
                    ),
            );
        }
    } else {
        // Standby art placeholder icon with same dimensions
        art_box = art_box.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(
                    svg()
                        .path("music.svg")
                        .size(px(32.0))
                        .text_color(theme.foreground_muted().opacity(0.45)),
                ),
        );
    }

    // Optional player switcher arrows if multiple media players are active
    let player_switcher = if has_media && total_players > 1 {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(20.0))
                    .rounded_full()
                    .hover(|s| s.bg(theme.surface().opacity(0.55)))
                    .cursor_pointer()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.prev_player();
                            cx.notify();
                        }),
                    )
                    .child(
                        svg()
                            .path("chevron-left.svg")
                            .size(px(10.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child(format!("{}/{}", selected_player_idx + 1, total_players)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(20.0))
                    .rounded_full()
                    .hover(|s| s.bg(theme.surface().opacity(0.55)))
                    .cursor_pointer()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.next_player();
                            cx.notify();
                        }),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(10.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
    } else {
        div()
    };

    // Track Title & Artist
    let (title_text, artist_text) = if has_media {
        let title = if active_track.title.is_empty() {
            "Desconocido".to_string()
        } else {
            active_track.title.clone()
        };
        let artist = if active_track.artist.is_empty() {
            "Artista desconocido".to_string()
        } else if !active_track.album.is_empty() && active_track.album != active_track.title {
            format!("{} • {}", active_track.artist, active_track.album)
        } else {
            active_track.artist.clone()
        };
        (title, artist)
    } else {
        let no_media = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("dashboard.no_media")
        } else {
            "Sin reproducción activa".to_string()
        };
        let no_player = if cx.has_global::<AppState>() {
            cx.global::<AppState>()
                .language
                .get("dashboard.no_player_active")
        } else {
            "Ningún reproductor activo".to_string()
        };
        (no_media, no_player)
    };

    let track_info = div()
        .flex()
        .flex_col()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(15.5))
                .text_color(if has_media {
                    theme.foreground()
                } else {
                    theme.foreground_muted()
                })
                .truncate()
                .child(title_text),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(
                    theme
                        .foreground_muted()
                        .opacity(if has_media { 1.0 } else { 0.65 }),
                )
                .truncate()
                .child(artist_text),
        );

    let top_row = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(track_info)
        .child(player_switcher);

    // Scrubber progress & timestamps
    let (pos_str, dur_str, progress_pct) = if has_media {
        match (active_track.position_micros, active_track.length_micros) {
            (Some(pos), Some(len)) if len > 0 => {
                let p = (pos as f64 / 1_000_000.0).max(0.0);
                let l = (len as f64 / 1_000_000.0).max(1.0);
                let pct = (p / l).clamp(0.0, 1.0) as f32;
                (
                    format!("{:02}:{:02}", (p as u64) / 60, (p as u64) % 60),
                    format!("{:02}:{:02}", (l as u64) / 60, (l as u64) % 60),
                    pct,
                )
            }
            (Some(pos), None) => {
                let p = (pos as f64 / 1_000_000.0).max(0.0);
                (
                    format!("{:02}:{:02}", (p as u64) / 60, (p as u64) % 60),
                    "--:--".to_string(),
                    0.0,
                )
            }
            _ => ("--:--".to_string(), "--:--".to_string(), 0.0),
        }
    } else {
        ("--:--".to_string(), "--:--".to_string(), 0.0)
    };

    let mut progress_bar = div()
        .id("media-progress-bar")
        .relative()
        .flex_1()
        .h(px(4.0))
        .rounded_full()
        .bg(theme.surface().opacity(0.4));

    if has_media && active_track.length_micros.is_some() {
        let slider_tracker = media_tracker.clone();
        let bus_name_seek = active_track.bus_name.clone();
        let length_micros_opt = active_track.length_micros;

        progress_bar = progress_bar
            .cursor_pointer()
            .child(
                canvas(
                    move |bounds, _, _| slider_tracker.track_bounds(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                    if let Some(total_micros) = length_micros_opt
                        && total_micros > 0
                    {
                        let click_x = f32::from(event.position.x);
                        let start_x = this.media_slider_tracker.left();
                        let width = this.media_slider_tracker.width(1.0);
                        if width > 0.0 {
                            let pct = ((click_x - start_x) / width).clamp(0.0, 1.0);
                            let target_secs = pct as f64 * (total_micros as f64 / 1_000_000.0);
                            let bus = bus_name_seek.clone();
                            services::spawn_tokio(async move {
                                MprisService::seek_to(&bus, target_secs).await;
                            });
                            this.touch_user_action();
                            cx.notify();
                        }
                    }
                }),
            );
    }

    if progress_pct > 0.0 {
        progress_bar = progress_bar.child(
            div()
                .h_full()
                .w(gpui::DefiniteLength::Fraction(progress_pct))
                .rounded_full()
                .bg(theme.accent()),
        );
    }

    let scrubber_row = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap_2()
        .child(
            div()
                .text_size(px(9.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground_muted())
                .child(pos_str),
        )
        .child(progress_bar)
        .child(
            div()
                .text_size(px(9.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground_muted())
                .child(dur_str),
        );

    // Playback buttons: All 3 styled consistently without bg
    let btn_color = if has_media {
        theme.foreground()
    } else {
        theme.foreground_muted().opacity(0.4)
    };

    let playback_controls = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .child(
            div()
                .id("mpris-prev")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.5)))
                .active(|s| s.bg(theme.surface().opacity(0.7)))
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        let bus_name = this
                            .get_selected_player()
                            .map(|p| p.bus_name.clone())
                            .unwrap_or_else(|| "org.mpris.MediaPlayer2.spotify".to_string());
                        this.touch_user_action();
                        cx.notify();

                        services::spawn_tokio(async move {
                            MprisService::previous_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path("skip-back.svg")
                        .size(px(14.0))
                        .text_color(btn_color),
                ),
        )
        .child(
            div()
                .id("mpris-play-pause")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.5)))
                .active(|s| s.bg(theme.surface().opacity(0.7)))
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        let bus_name = if let Some(active) = this.get_selected_player_mut() {
                            active.is_playing = !active.is_playing;
                            active.bus_name.clone()
                        } else {
                            "org.mpris.MediaPlayer2.spotify".to_string()
                        };
                        this.touch_user_action();
                        cx.notify();

                        services::spawn_tokio(async move {
                            MprisService::play_pause_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path(if is_playing { "pause.svg" } else { "play.svg" })
                        .size(px(14.0))
                        .text_color(btn_color),
                ),
        )
        .child(
            div()
                .id("mpris-next")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.5)))
                .active(|s| s.bg(theme.surface().opacity(0.7)))
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        let bus_name = this
                            .get_selected_player()
                            .map(|p| p.bus_name.clone())
                            .unwrap_or_else(|| "org.mpris.MediaPlayer2.spotify".to_string());
                        this.touch_user_action();
                        cx.notify();

                        services::spawn_tokio(async move {
                            MprisService::next_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path("skip-forward.svg")
                        .size(px(14.0))
                        .text_color(btn_color),
                ),
        );

    let bottom_section = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .gap_3()
        .child(scrubber_row)
        .child(playback_controls);

    let right_pane = div()
        .flex()
        .flex_col()
        .justify_between()
        .flex_1()
        .h(art_size)
        .overflow_hidden()
        .child(top_row)
        .child(bottom_section);

    let content_layer = div()
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .size_full()
        .p_3()
        .gap_3()
        .child(art_box)
        .child(right_pane);

    card.child(content_layer).into_any_element()
}
