use gpui::{Context, FontWeight, IntoElement, StyledImage, div, img, prelude::*, px, svg};
use services::{MediaTrack, MprisService};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn render_media_player_widget(
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> gpui::AnyElement {
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
            .w(px(185.0))
            .h(px(102.0))
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
                    .child("Sin reproducción"),
            )
            .into_any_element();
    }

    let is_playing = active_track.is_playing;
    let art_thumb = if let Some(ref url) = active_track.art_url {
        let clean_path = if let Some(stripped) = url.strip_prefix("file://") {
            stripped
        } else {
            url.as_str()
        };
        let p = std::path::Path::new(clean_path);
        if p.exists() {
            div()
                .w(px(30.0))
                .h(px(30.0))
                .rounded(px(8.0))
                .overflow_hidden()
                .flex_shrink_0()
                .child(
                    img(clean_path.to_string())
                        .size_full()
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .into_any_element()
        } else {
            render_art_fallback(theme)
        }
    } else {
        render_art_fallback(theme)
    };

    let player_dots = if total_players > 1 {
        let mut dots = div().flex().flex_row().items_center().gap_1();
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
        dots.into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .id("media-player-card")
        .flex()
        .flex_col()
        .justify_between()
        .w(px(185.0))
        .h(px(102.0))
        .rounded(card_radius)
        .bg(theme.surface().opacity(0.45))
        .border_1()
        .border_color(theme.surface().opacity(0.25))
        .p_2p5()
        .overflow_hidden()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .overflow_hidden()
                .child(art_thumb)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .pl_2()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .truncate()
                                .child(active_track.title.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.foreground_muted())
                                .truncate()
                                .child(active_track.artist.clone()),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .pt_1()
                .child(player_dots)
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .id("mpris-prev")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(22.0))
                                .h(px(22.0))
                                .rounded_full()
                                .hover(|s| s.bg(theme.surface().opacity(0.45)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        let bus_name = this
                                            .get_selected_player()
                                            .map(|p| p.bus_name.clone())
                                            .unwrap_or_else(|| {
                                                "org.mpris.MediaPlayer2.spotify".to_string()
                                            });
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
                                        .size(px(12.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        )
                        .child(
                            div()
                                .id("mpris-play-pause")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(28.0))
                                .h(px(28.0))
                                .rounded_full()
                                .bg(theme.accent())
                                .hover(|s| s.opacity(0.85))
                                .active(|s| s.opacity(0.7))
                                .cursor_pointer()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        let bus_name =
                                            if let Some(active) = this.get_selected_player_mut() {
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
                                        .size(px(13.0))
                                        .text_color(theme.background()),
                                ),
                        )
                        .child(
                            div()
                                .id("mpris-next")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(22.0))
                                .h(px(22.0))
                                .rounded_full()
                                .hover(|s| s.bg(theme.surface().opacity(0.45)))
                                .cursor_pointer()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(|this, _, _window, cx| {
                                        let bus_name = this
                                            .get_selected_player()
                                            .map(|p| p.bus_name.clone())
                                            .unwrap_or_else(|| {
                                                "org.mpris.MediaPlayer2.spotify".to_string()
                                            });
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
                                        .size(px(12.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn render_art_fallback(theme: &Theme) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(30.0))
        .h(px(30.0))
        .rounded(px(8.0))
        .bg(theme.surface().opacity(0.6))
        .flex_shrink_0()
        .child(
            svg()
                .path("music.svg")
                .size(px(14.0))
                .text_color(theme.accent()),
        )
        .into_any_element()
}
