use gpui::{
    AnyElement, Context, FontWeight, IntoElement, MouseButton, canvas, div, img, prelude::*, px,
    svg,
};
use services::{AppState, MprisService};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;
use crate::capsule::widgets::dashboard::media_player::resolve_art_path;

#[allow(dead_code)]
pub fn compute_media_panel_height() -> f32 {
    195.0
}

fn format_duration(seconds: i64) -> String {
    let s = seconds.max(0);
    let mins = s / 60;
    let secs = s % 60;
    format!("{:02}:{:02}", mins, secs)
}

pub fn render_media_mini_panel(
    panel_h: f32,
    module: &mut DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let total_players = module.media_players.len();
    let active_track = module
        .media_players
        .get(module.selected_player_idx)
        .cloned()
        .unwrap_or_default();

    if !active_track.has_media || active_track.title.is_empty() {
        return div()
            .min_w(px(PANEL_MIN_W))
            .max_h(px(panel_h))
            .p_4()
            .gap_2()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(40.0))
                    .rounded_full()
                    .bg(theme.surface().opacity(0.55))
                    .child(
                        svg()
                            .path("music.svg")
                            .size(px(20.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child(if cx.has_global::<AppState>() {
                        cx.global::<AppState>().language.get("dashboard.no_media")
                    } else {
                        "Sin reproducción".to_string()
                    }),
            )
            .into_any_element();
    }

    let is_playing = active_track.is_playing;
    let art_path = resolve_art_path(&active_track);
    let player_name = if !active_track.player_name.is_empty() {
        active_track.player_name.clone()
    } else {
        "Reproductor".to_string()
    };

    // Header: Player badge + Switcher (if multiple players)
    let mut header = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full();

    let player_badge = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1p5()
        .child(
            svg()
                .path("music.svg")
                .size(px(13.0))
                .text_color(theme.accent()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child(player_name),
        );

    header = header.child(player_badge);

    if total_players > 1 {
        let switcher = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(
                div()
                    .id("media-panel-prev-player")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(20.0))
                    .rounded_full()
                    .hover(|s| s.bg(theme.surface().opacity(0.45)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.prev_player();
                            cx.notify();
                        }),
                    )
                    .child(
                        svg()
                            .path("chevron-left.svg")
                            .size(px(12.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(theme.foreground_muted())
                    .child(format!(
                        "{}/{}",
                        module.selected_player_idx + 1,
                        total_players
                    )),
            )
            .child(
                div()
                    .id("media-panel-next-player")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(20.0))
                    .rounded_full()
                    .hover(|s| s.bg(theme.surface().opacity(0.45)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _window, cx| {
                            this.next_player();
                            cx.notify();
                        }),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(12.0))
                            .text_color(theme.foreground_muted()),
                    ),
            );
        header = header.child(switcher);
    }

    // Body: Album art + Track info
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

    let art_element = if let Some(ref path) = art_path {
        div()
            .size(px(58.0))
            .rounded(px(10.0))
            .overflow_hidden()
            .border_1()
            .border_color(theme.surface().opacity(0.35))
            .child(
                img(path.clone())
                    .size(px(58.0))
                    .aspect_ratio(1.0)
                    .object_fit(gpui::ObjectFit::Cover)
                    .rounded(px(10.0)),
            )
    } else {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(58.0))
            .rounded(px(10.0))
            .bg(theme.surface().opacity(0.55))
            .border_1()
            .border_color(theme.surface().opacity(0.25))
            .child(
                svg()
                    .path("music.svg")
                    .size(px(24.0))
                    .text_color(theme.foreground_muted()),
            )
    };

    let mut info_col = div()
        .flex()
        .flex_col()
        .justify_center()
        .flex_1()
        .overflow_hidden()
        .gap_0p5()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_size(px(12.5))
                .text_color(theme.foreground())
                .truncate()
                .child(title_text),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(theme.foreground_muted())
                .truncate()
                .child(artist_text),
        );

    if !active_track.album.is_empty() {
        info_col = info_col.child(
            div()
                .text_size(px(9.5))
                .text_color(theme.foreground_muted().opacity(0.75))
                .truncate()
                .child(active_track.album.clone()),
        );
    }

    let track_body = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .w_full()
        .child(art_element)
        .child(info_col);

    // Progress bar & Seeking
    let length_micros = active_track.length_micros.unwrap_or(0);
    let position_micros = active_track.position_micros.unwrap_or(0);
    let has_valid_duration = length_micros > 0;

    let progress_section = if has_valid_duration {
        let total_secs = length_micros / 1_000_000;
        let pos_secs = (position_micros / 1_000_000).clamp(0, total_secs);
        let progress_ratio = (pos_secs as f32 / total_secs as f32).clamp(0.0, 1.0);
        let bus_name = active_track.bus_name.clone();

        let time_labels = div()
            .flex()
            .flex_row()
            .justify_between()
            .items_center()
            .w_full()
            .child(
                div()
                    .text_size(px(9.5))
                    .text_color(theme.foreground_muted())
                    .child(format_duration(pos_secs)),
            )
            .child(
                div()
                    .text_size(px(9.5))
                    .text_color(theme.foreground_muted())
                    .child(format_duration(total_secs)),
            );

        let slider_tracker = module.media_slider_tracker.clone();
        let track_bar = div()
            .id("media-satellite-progress-bar")
            .relative()
            .w_full()
            .h(px(6.0))
            .rounded_full()
            .bg(theme.surface().opacity(0.55))
            .cursor_pointer()
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _, _| slider_tracker.track_bounds(bounds),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .h_full()
                    .w(gpui::DefiniteLength::Fraction(progress_ratio))
                    .rounded_full()
                    .bg(theme.accent()),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                    let bar_x = this.media_slider_tracker.left();
                    let bar_w = this.media_slider_tracker.width(0.0);
                    let x_val = f32::from(event.position.x);
                    let rel_x = (x_val - bar_x).max(0.0);
                    let ratio = if bar_w > 0.0 {
                        (rel_x / bar_w).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let target_secs = ratio as f64 * total_secs as f64;
                    let bus = bus_name.clone();

                    if let Some(track) = this.get_selected_player_mut() {
                        track.position_micros = Some((target_secs * 1_000_000.0) as i64);
                    }
                    this.touch_user_action();
                    cx.notify();

                    tokio::spawn(async move {
                        MprisService::seek_to(&bus, target_secs).await;
                    });
                }),
            );

        div()
            .flex()
            .flex_col()
            .w_full()
            .gap_1()
            .child(track_bar)
            .child(time_labels)
    } else {
        div()
    };

    // Playback Controls
    let controls = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .w_full()
        .gap_3()
        .child(
            div()
                .id("media-panel-prev")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.45)))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
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
                .id("media-panel-play-pause")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .bg(theme.accent())
                .hover(|s| s.opacity(0.85))
                .active(|s| s.opacity(0.7))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
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
                        .size(px(15.0))
                        .text_color(theme.background()),
                ),
        )
        .child(
            div()
                .id("media-panel-next")
                .flex()
                .items_center()
                .justify_center()
                .size(px(28.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.45)))
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
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

    div()
        .min_w(px(PANEL_MIN_W))
        .max_h(px(panel_h))
        .p_3()
        .gap_2p5()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(header)
        .child(track_body)
        .child(progress_section)
        .child(controls)
        .into_any_element()
}
