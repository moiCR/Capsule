use gpui::{
    ColorExt, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, StyledImage, div, img, px, svg,
};
use services::{AppState, MprisService};
use std::path::PathBuf;
use ui::theme::Theme;

use crate::lockscreen::LockScreen;

pub fn render_lockscreen_media_player(
    theme: &Theme,
    cx: &mut Context<LockScreen>,
) -> impl IntoElement {
    let active_track = if cx.has_global::<AppState>() {
        cx.global::<AppState>().mpris.get_current_track()
    } else {
        services::MediaTrack::default()
    };

    if !active_track.has_media || active_track.title.is_empty() {
        return div().into_any_element();
    }

    let bus_name = active_track.bus_name.clone();
    let bus_prev = bus_name.clone();
    let bus_next = bus_name.clone();
    let bus_play = bus_name.clone();
    let is_playing = active_track.is_playing;

    let art_thumb = if let Some(ref art_path) = active_track.local_art_path {
        div()
            .w(px(46.0))
            .h(px(46.0))
            .rounded(px(12.0))
            .overflow_hidden()
            .flex_shrink_0()
            .border_1()
            .border_color(theme.outline_variant().opacity(0.3))
            .child(
                img(PathBuf::from(art_path))
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover),
            )
            .into_any_element()
    } else {
        div()
            .w(px(46.0))
            .h(px(46.0))
            .rounded(px(12.0))
            .bg(theme.primary_container())
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(
                svg()
                    .path("music.svg")
                    .size(px(20.0))
                    .text_color(theme.on_primary_container()),
            )
            .into_any_element()
    };

    let btn_hover_prev = theme.surface_container_highest();
    let play_hover = theme.primary();
    let btn_hover_next = theme.surface_container_highest();

    div()
        .id("lockscreen-media-player")
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .px(px(12.0))
        .py(px(8.0))
        .gap(px(12.0))
        .rounded(px(24.0))
        .bg(theme.surface_container_high().opacity(0.85))
        .border_1()
        .border_color(theme.outline_variant().opacity(0.4))
        .child(art_thumb)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .justify_center()
                .child(
                    div()
                        .font_family(theme.font_family())
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(theme.foreground())
                        .truncate()
                        .child(active_track.title),
                )
                .child(
                    div()
                        .font_family(theme.font_family())
                        .text_size(px(11.0))
                        .text_color(theme.on_surface_variant())
                        .truncate()
                        .child(active_track.artist),
                ),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .flex_shrink_0()
                .child(
                    div()
                        .id("ls-mpris-prev")
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(move |s| s.bg(btn_hover_prev))
                        .active(|s| s.opacity(0.6))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_prev.clone();
                            services::spawn_tokio(async move {
                                MprisService::previous_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("skip-back.svg")
                                .size(px(14.0))
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .id("ls-mpris-play-pause")
                        .w(px(38.0))
                        .h(px(38.0))
                        .rounded_full()
                        .bg(theme.primary_container())
                        .hover(move |s| s.bg(play_hover))
                        .active(|s| s.opacity(0.7))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_play.clone();
                            services::spawn_tokio(async move {
                                MprisService::play_pause_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path(if is_playing { "pause.svg" } else { "play.svg" })
                                .size(px(16.0))
                                .text_color(theme.on_primary_container()),
                        ),
                )
                .child(
                    div()
                        .id("ls-mpris-next")
                        .w(px(32.0))
                        .h(px(32.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(move |s| s.bg(btn_hover_next))
                        .active(|s| s.opacity(0.6))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            let b = bus_next.clone();
                            services::spawn_tokio(async move {
                                MprisService::next_bus(&b).await;
                            });
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("skip-forward.svg")
                                .size(px(14.0))
                                .text_color(theme.foreground()),
                        ),
                ),
        )
        .into_any_element()
}
