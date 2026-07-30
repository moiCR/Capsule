use gpui::{Context, FontWeight, IntoElement, black, div, prelude::*, px, svg};
use services::{MediaTrack, MprisService};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn render_media_player_widget(
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let card_radius = px(42.0);

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
                .bg(theme.surface().opacity(0.8))
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
                        services::log_info!("MPRIS", "Button prev clicked");
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
                        services::log_info!("MPRIS", "Button play-pause clicked");
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
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(28.0))
                        .h(px(28.0))
                        .bg(theme.accent().opacity(0.3))
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
                        services::log_info!("MPRIS", "Button next clicked");
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
                        .size(px(16.0))
                        .text_color(theme.foreground_muted()),
                ),
        );

    div()
        .id("media-player-card")
        .relative()
        .w_full()
        .rounded(card_radius)
        .border_1()
        .border_color(theme.surface())
        .when_some(active_track.local_art_path.clone(), |this, art_path| {
            this.child(
                gpui::img(std::path::PathBuf::from(art_path))
                    .absolute()
                    .inset_0()
                    .w_full()
                    .h_full()
                    .rounded(card_radius)
                    .object_fit(gpui::ObjectFit::Cover),
            )
        })
        .child(div().absolute().inset_0().rounded(card_radius).bg(
            if active_track.local_art_path.is_some() {
                black().opacity(0.65)
            } else {
                theme.background_alt()
            },
        ))
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .w_full()
                .p_3()
                .gap_2()
                .child(player_header)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .w_full()
                        .overflow_hidden()
                        .gap_1()
                        .pt_1()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(14.0))
                                .text_color(theme.foreground())
                                .truncate()
                                .child(if active_track.has_media {
                                    active_track.title.clone()
                                } else {
                                    let lang = if cx.has_global::<ui::language::Language>() {
                                        cx.global::<ui::language::Language>().clone()
                                    } else {
                                        ui::language::Language::default()
                                    };
                                    lang.dashboard.no_media
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
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
