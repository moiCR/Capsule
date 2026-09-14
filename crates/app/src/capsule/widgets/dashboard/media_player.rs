use gpui::{Context, FontWeight, IntoElement, StyledImage, div, img, prelude::*, px, svg};
use services::{MediaTrack, MprisService};
use std::path::Path;
use ui::theme::Theme;

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

pub fn render_media_player_widget(
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    prev_art_path: Option<&str>,
    anim_progress: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> gpui::AnyElement {
    let card_w = px(185.0);
    let card_h = px(102.0);
    let card_radius = px(if cx.has_global::<services::AppState>() {
        cx.global::<services::AppState>()
            .config
            .get()
            .ui
            .cards_round
    } else {
        22.0
    });

    if !active_track.has_media || active_track.title.is_empty() {
        return div()
            .id("media-player-card")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w(card_w)
            .h(card_h)
            .rounded(card_radius)
            .bg(theme.surface().opacity(0.45))
            .border_1()
            .border_color(theme.surface().opacity(0.25))
            .gap_1p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(38.0))
                    .h(px(38.0))
                    .rounded_full()
                    .bg(theme.surface().opacity(0.6))
                    .child(
                        svg()
                            .path("music.svg")
                            .size(px(18.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child(if cx.has_global::<services::AppState>() {
                        cx.global::<services::AppState>()
                            .language
                            .get("dashboard.no_media")
                    } else {
                        "Sin reproducción".to_string()
                    }),
            )
            .into_any_element();
    }

    let is_playing = active_track.is_playing;
    let art_path = resolve_art_path(active_track);
    let ratio = 185.0 / 102.0;

    let mut card = div()
        .id("media-player-card")
        .relative()
        .w(card_w)
        .h(card_h)
        .rounded(card_radius)
        .border_1()
        .border_color(theme.surface().opacity(0.25))
        .overflow_hidden()
        .bg(theme.surface().opacity(0.45));

    // Previous art layer during crossfade
    if anim_progress < 1.0 {
        if let Some(prev) = prev_art_path {
            let prev_opacity = (1.0 - anim_progress).clamp(0.0, 1.0);
            if prev_opacity > 0.01 {
                card = card.child(
                    div()
                        .absolute()
                        .inset_0()
                        .w(card_w)
                        .h(card_h)
                        .opacity(prev_opacity)
                        .child(
                            img(prev.to_string())
                                .w(card_w)
                                .h(card_h)
                                .aspect_ratio(ratio)
                                .object_fit(gpui::ObjectFit::Cover)
                                .rounded(card_radius),
                        ),
                );
            }
        }
    }

    // Current art layer
    if let Some(ref image_path) = art_path {
        let current_opacity = if prev_art_path.is_some() || anim_progress < 1.0 {
            anim_progress.clamp(0.0, 1.0)
        } else {
            1.0
        };

        card = card.child(
            div()
                .absolute()
                .inset_0()
                .w(card_w)
                .h(card_h)
                .opacity(current_opacity)
                .child(
                    img(image_path.clone())
                        .w(card_w)
                        .h(card_h)
                        .aspect_ratio(ratio)
                        .object_fit(gpui::ObjectFit::Cover)
                        .rounded(card_radius),
                ),
        );
    }

    // Scrim overlay for contrast
    if art_path.is_some() || prev_art_path.is_some() {
        let scrim_opacity = if art_path.is_some() {
            0.65
        } else {
            0.65 * (1.0 - anim_progress)
        };
        card = card.child(
            div()
                .absolute()
                .inset_0()
                .w(card_w)
                .h(card_h)
                .rounded(card_radius)
                .bg(theme.background().opacity(scrim_opacity)),
        );
    }

    // Player indicator dots (if multiple players)
    let player_dots = if total_players > 1 {
        let mut dots = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap_1();
        for i in 0..total_players {
            let is_active = i == selected_player_idx;
            dots = dots.child(
                div()
                    .w(px(if is_active { 8.0 } else { 3.0 }))
                    .h(px(3.0))
                    .rounded_full()
                    .bg(if is_active {
                        theme.accent()
                    } else {
                        theme.foreground_muted()
                    }),
            );
        }
        Some(dots)
    } else {
        None
    };

    // Text slide & fade animation
    let text_opacity = if anim_progress < 1.0 {
        anim_progress.clamp(0.2, 1.0)
    } else {
        1.0
    };
    let text_slide_y = if anim_progress < 1.0 {
        (1.0 - anim_progress) * 4.0
    } else {
        0.0
    };

    let title_text = if active_track.title.is_empty() {
        "Desconocido".to_string()
    } else {
        active_track.title.clone()
    };
    let artist_text = if active_track.artist.is_empty() {
        "Artista desconocido".to_string()
    } else {
        active_track.artist.clone()
    };

    // Centered track info (no badge)
    let track_info = div()
        .flex()
        .flex_col()
        .items_center()
        .w_full()
        .pt(px(text_slide_y))
        .opacity(text_opacity)
        .overflow_hidden()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(12.0))
                .text_color(theme.foreground())
                .text_center()
                .w_full()
                .truncate()
                .child(title_text),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(theme.foreground_muted())
                .text_center()
                .w_full()
                .truncate()
                .child(artist_text),
        );

    // Centered playback controls
    let controls = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .w_full()
        .gap_2p5()
        .child(
            div()
                .id("mpris-prev")
                .flex()
                .items_center()
                .justify_center()
                .w(px(24.0))
                .h(px(24.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.45)))
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

                        tokio::spawn(async move {
                            MprisService::previous_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path("skip-back.svg")
                        .size(px(13.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .child(
            div()
                .id("mpris-play-pause")
                .flex()
                .items_center()
                .justify_center()
                .w(px(30.0))
                .h(px(30.0))
                .rounded_full()
                .bg(theme.accent())
                .hover(|s| s.opacity(0.85))
                .active(|s| s.opacity(0.7))
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

                        tokio::spawn(async move {
                            MprisService::play_pause_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path(if is_playing { "pause.svg" } else { "play.svg" })
                        .size(px(14.0))
                        .text_color(theme.background()),
                ),
        )
        .child(
            div()
                .id("mpris-next")
                .flex()
                .items_center()
                .justify_center()
                .w(px(24.0))
                .h(px(24.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.45)))
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

                        tokio::spawn(async move {
                            MprisService::next_bus(&bus_name).await;
                        });
                    }),
                )
                .child(
                    svg()
                        .path("skip-forward.svg")
                        .size(px(13.0))
                        .text_color(theme.foreground_muted()),
                ),
        );

    let bottom_section = div()
        .flex()
        .flex_col()
        .items_center()
        .w_full()
        .gap_1()
        .child(controls)
        .when_some(player_dots, |parent, dots| parent.child(dots));

    card.child(
        div()
            .relative()
            .flex()
            .flex_col()
            .justify_between()
            .items_center()
            .size_full()
            .p_2p5()
            .child(track_info)
            .child(bottom_section),
    )
    .into_any_element()
}
