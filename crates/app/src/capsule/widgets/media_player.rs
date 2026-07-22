use gpui::{div, prelude::*, px, svg, Context, FontWeight, IntoElement};
use services::{MediaTrack, MprisService};
use std::time::Duration;
use ui::theme::Theme;

use crate::capsule::modules::idle_hover::IdleHoverModule;

pub fn render_media_player_widget(
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    theme: &Theme,
    cx: &mut Context<IdleHoverModule>,
) -> impl IntoElement {
    let art_element = if let Some(art_path) = &active_track.local_art_path {
        div()
            .w(px(64.0))
            .h(px(64.0))
            .rounded(px(12.0))
            .overflow_hidden()
            .flex_none()
            .child(
                gpui::img(std::path::PathBuf::from(art_path))
                    .w(px(64.0))
                    .h(px(64.0))
                    .object_fit(gpui::ObjectFit::Cover),
            )
    } else {
        div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(64.0))
            .h(px(64.0))
            .bg(theme.surface())
            .rounded(px(12.0))
            .flex_none()
            .child(
                svg()
                    .path("music.svg")
                    .size(px(24.0))
                    .text_color(theme.foreground_muted()),
            )
    };

    let player_header = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            div()
                .flex()
                .items_center()
                .px_2()
                .py_0p5()
                .bg(theme.surface())
                .rounded_full()
                .child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.accent())
                        .child(if active_track.has_media {
                            active_track.player_name.clone()
                        } else {
                            "Media".to_string()
                        }),
                ),
        )
        .child(if total_players > 1 {
            let mut dots = div().flex().flex_row().items_center().gap_1p5();
            for i in 0..total_players {
                let is_active = i == selected_player_idx;
                dots = dots.child(
                    div()
                        .w(px(if is_active { 12.0 } else { 5.0 }))
                        .h(px(5.0))
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
        });

    let media_controls = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_4()
        .pt_1()
        .child(
            div()
                .id("mpris-prev")
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        if let Some(active) = this.get_selected_player() {
                            let bus_name = active.bus_name.clone();
                            this.touch_user_action();
                            cx.notify();

                            cx.spawn(async move |this, cx| {
                                MprisService::previous_bus(&bus_name).await;
                                cx.background_executor()
                                    .timer(Duration::from_millis(150))
                                    .await;
                                let players = MprisService::fetch_all_players().await;
                                this.update(cx, |this, cx| {
                                    this.update_players(players);
                                    cx.notify();
                                })
                                .ok();
                            })
                            .detach();
                        }
                    }),
                )
                .child(
                    svg()
                        .path("skip-back.svg")
                        .size(px(16.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .child(
            div()
                .id("mpris-play-pause")
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        if let Some(active) = this.get_selected_player_mut() {
                            let bus_name = active.bus_name.clone();
                            active.is_playing = !active.is_playing;
                            this.touch_user_action();
                            cx.notify();

                            cx.spawn(async move |this, cx| {
                                MprisService::play_pause_bus(&bus_name).await;
                                cx.background_executor()
                                    .timer(Duration::from_millis(150))
                                    .await;
                                let players = MprisService::fetch_all_players().await;
                                this.update(cx, |this, cx| {
                                    this.update_players(players);
                                    cx.notify();
                                })
                                .ok();
                            })
                            .detach();
                        }
                    }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(28.0))
                        .h(px(28.0))
                        .bg(theme.accent().opacity(0.2))
                        .rounded_full()
                        .child(
                            svg()
                                .path(if active_track.is_playing {
                                    "pause.svg"
                                } else {
                                    "play.svg"
                                })
                                .size(px(14.0))
                                .text_color(theme.accent()),
                        ),
                ),
        )
        .child(
            div()
                .id("mpris-next")
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _window, cx| {
                        if let Some(active) = this.get_selected_player() {
                            let bus_name = active.bus_name.clone();
                            this.touch_user_action();
                            cx.notify();

                            cx.spawn(async move |this, cx| {
                                MprisService::next_bus(&bus_name).await;
                                cx.background_executor()
                                    .timer(Duration::from_millis(150))
                                    .await;
                                let players = MprisService::fetch_all_players().await;
                                this.update(cx, |this, cx| {
                                    this.update_players(players);
                                    cx.notify();
                                })
                                .ok();
                            })
                            .detach();
                        }
                    }),
                )
                .child(
                    svg()
                        .path("skip-forward.svg")
                        .size(px(16.0))
                        .text_color(theme.foreground_muted()),
                ),
        );

    div()
        .id("media-player-card")
        .flex()
        .flex_col()
        .w_full()
        .bg(theme.background_alt())
        .border_1()
        .border_color(theme.surface())
        .rounded(px(18.0))
        .p_3()
        .gap_2()
        .child(player_header)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .w_full()
                .gap_3()
                .child(art_element)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .w_full()
                        .overflow_hidden()
                        .gap_0p5()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(theme.foreground())
                                .truncate()
                                .child(if active_track.has_media {
                                    active_track.title.clone()
                                } else {
                                    "No media playing".to_string()
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.foreground_muted())
                                .truncate()
                                .child(if active_track.has_media {
                                    active_track.artist.clone()
                                } else {
                                    "Silence".to_string()
                                }),
                        )
                        .child(media_controls),
                ),
        )
}
