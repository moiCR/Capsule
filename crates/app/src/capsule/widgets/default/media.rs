use gpui::{AnyElement, Context, IntoElement, ScrollWheelEvent, div, img, prelude::*, px, svg};
use services::{MediaTrack, MprisService};
use ui::theme::Theme;

use crate::capsule::modules::default::DefaultModule;
use crate::capsule::widgets::dashboard::media_player::resolve_art_path;

pub fn render_media_dock(
    track: &MediaTrack,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> Option<AnyElement> {
    if !track.has_media || (track.title.is_empty() && !track.is_playing) {
        return None;
    }

    let bus_disc = track.bus_name.clone();
    let bus_prev = track.bus_name.clone();
    let bus_play = track.bus_name.clone();
    let bus_next = track.bus_name.clone();
    let bus_scroll = track.bus_name.clone();

    let art_path = resolve_art_path(track);

    let disc_visual = if let Some(art_path) = art_path {
        div().size(px(18.0)).rounded_full().overflow_hidden().child(
            img(art_path)
                .size(px(18.0))
                .rounded_full()
                .aspect_ratio(1.0)
                .object_fit(gpui::ObjectFit::Cover),
        )
    } else {
        div().size(px(18.0)).rounded_full().overflow_hidden().child(
            svg()
                .path("vinyl.svg")
                .size(px(18.0))
                .text_color(theme.accent()),
        )
    };

    let disc_btn = div()
        .id("media-disc-btn")
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .cursor_pointer()
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_disc.clone();
            services::spawn_tokio(async move {
                let _ = MprisService::play_pause_bus(&bus).await;
            });
        }))
        .child(disc_visual);

    let prev_btn = div()
        .id("media-btn-prev")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_prev.clone();
            services::spawn_tokio(async move {
                let _ = MprisService::previous_bus(&bus).await;
            });
        }))
        .child(
            svg()
                .path("skip-back.svg")
                .size(px(9.5))
                .text_color(theme.foreground_muted()),
        );

    let play_icon = if track.is_playing {
        "pause.svg"
    } else {
        "play.svg"
    };

    let play_btn = div()
        .id("media-btn-play")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_play.clone();
            services::spawn_tokio(async move {
                let _ = MprisService::play_pause_bus(&bus).await;
            });
        }))
        .child(
            svg()
                .path(play_icon)
                .size(px(10.0))
                .text_color(theme.foreground_muted()),
        );

    let next_btn = div()
        .id("media-btn-next")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_next.clone();
            services::spawn_tokio(async move {
                let _ = MprisService::next_bus(&bus).await;
            });
        }))
        .child(
            svg()
                .path("skip-forward.svg")
                .size(px(9.5))
                .text_color(theme.foreground_muted()),
        );

    Some(
        div()
            .id("default-media-dock")
            .flex()
            .flex_shrink_0()
            .flex_row()
            .items_center()
            .h(px(22.0))
            .px(px(4.0))
            .gap(px(2.0))
            .rounded(px(7.0))
            .bg(theme.surface().opacity(0.35))
            .border_1()
            .border_color(theme.surface().opacity(0.2))
            .on_scroll_wheel(cx.listener(move |_this, event: &ScrollWheelEvent, _, _| {
                let bus = bus_scroll.clone();
                let delta_y = match event.delta {
                    gpui::ScrollDelta::Lines(lines) => lines.y,
                    gpui::ScrollDelta::Pixels(pixels) => pixels.y.into(),
                };
                services::spawn_tokio(async move {
                    if delta_y > 0.0 {
                        let _ = MprisService::previous_bus(&bus).await;
                    } else if delta_y < 0.0 {
                        let _ = MprisService::next_bus(&bus).await;
                    }
                });
            }))
            .child(disc_btn)
            .child(prev_btn)
            .child(play_btn)
            .child(next_btn)
            .into_any_element(),
    )
}
