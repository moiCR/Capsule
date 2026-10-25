use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{
    App, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render,
    Size, Window, div, img, px, svg,
};
use services::{AppState, MediaTrack, MprisService};
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub struct MediaModule {
    focus: FocusHandle,
}

impl MediaModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
        }
    }
}

impl EventEmitter<SharedPanelEvent> for MediaModule {}

impl SharedPanelItem for MediaModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        Size {
            width: px(430.0),
            height: px(250.0),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

fn format_duration(micros: i64) -> String {
    let secs = (micros / 1_000_000).max(0);
    let m = secs / 60;
    let s = secs % 60;
    format!("{m}:{s:02}")
}

impl Render for MediaModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let (track, volume) = if let Some(state) = cx.try_global::<AppState>() {
            let t = state.mpris.get_current_track();
            let v = state.system.get_status().volume;
            (t, v)
        } else {
            (MediaTrack::default(), 50)
        };

        if track.title.is_empty() {
            return div()
                .id("shared-panel-media-empty")
                .track_focus(&self.focus)
                .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                    if event.keystroke.key == "escape" {
                        cx.emit(SharedPanelEvent::Close);
                    }
                }))
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(12.0))
                .child(
                    svg()
                        .path("music.svg")
                        .size(px(38.0))
                        .text_color(theme.foreground_muted().opacity(0.4)),
                )
                .child(
                    div()
                        .text_size(px(14.5))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground_muted())
                        .child("Sin reproducción activa"),
                )
                .into_any_element();
        }

        let pos = track.position_micros.unwrap_or(0).max(0);
        let len = track.length_micros.unwrap_or(0).max(0);
        let fraction = if len > 0 {
            (pos as f32 / len as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let pos_str = format_duration(pos);
        let len_str = if len > 0 {
            format_duration(len)
        } else {
            "--:--".to_string()
        };

        let art_element = if let Some(art_path) = &track.local_art_path {
            img(PathBuf::from(art_path))
                .size(px(68.0))
                .rounded(px(12.0))
                .border_1()
                .border_color(style::border(&theme).opacity(0.18))
                .into_any_element()
        } else {
            div()
                .size(px(68.0))
                .rounded(px(12.0))
                .bg(style::surface(&theme).opacity(0.60))
                .border_1()
                .border_color(style::border(&theme).opacity(0.18))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    svg()
                        .path("music.svg")
                        .size(px(28.0))
                        .text_color(theme.accent()),
                )
                .into_any_element()
        };

        let bus_play = track.bus_name.clone();
        let bus_prev = track.bus_name.clone();
        let bus_next = track.bus_name.clone();
        let bus_seek = track.bus_name.clone();

        let play_icon = if track.is_playing {
            "pause.svg"
        } else {
            "play.svg"
        };

        let hover_btn = style::hover(&theme);
        let prev_hover = hover_btn;
        let next_hover = hover_btn;

        let seek_len = len;
        let bar_w = 390.0;
        let progress_bar = div()
            .id("shared-media-seek")
            .w_full()
            .h(px(5.0))
            .rounded_full()
            .bg(style::surface(&theme).opacity(0.60))
            .cursor_pointer()
            .relative()
            .child(
                div()
                    .h_full()
                    .rounded_full()
                    .bg(theme.accent())
                    .w(px(fraction * bar_w)),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |_, event: &gpui::MouseDownEvent, _, _| {
                    if seek_len > 0 {
                        let click_pct = (f32::from(event.position.x) / bar_w).clamp(0.0, 1.0);
                        let target_secs = (seek_len as f64 / 1_000_000.0) * click_pct as f64;
                        let bus = bus_seek.clone();
                        services::spawn_tokio(async move {
                            MprisService::seek_to(&bus, target_secs).await;
                        });
                    }
                }),
            );

        div()
            .id("shared-panel-media")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SharedPanelEvent::Close);
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .justify_between()
            .p(px(16.0))
            .gap(px(12.0))
            .font_family(theme.font_family())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(14.0))
                    .child(art_element)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(3.0))
                            .child(
                                div()
                                    .text_size(px(15.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.foreground())
                                    .line_clamp(1)
                                    .child(track.title),
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .text_color(theme.foreground_muted())
                                    .line_clamp(1)
                                    .child(track.artist),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(progress_bar)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(11.5))
                            .text_color(theme.foreground_muted())
                            .child(pos_str)
                            .child(len_str),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                svg()
                                    .path("volume-2.svg")
                                    .size(px(15.0))
                                    .text_color(theme.foreground_muted()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.foreground_muted())
                                    .child(format!("{volume}%")),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("shared-media-prev-btn")
                                    .size(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(8.0))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(prev_hover))
                                    .child(
                                        svg()
                                            .path("skip-back.svg")
                                            .size(px(15.0))
                                            .text_color(theme.foreground()),
                                    )
                                    .on_click(cx.listener(move |_, _, _, _| {
                                        let bus = bus_prev.clone();
                                        services::spawn_tokio(async move {
                                            MprisService::previous_bus(&bus).await;
                                        });
                                    })),
                            )
                            .child(
                                div()
                                    .id("shared-media-play-btn")
                                    .size(px(38.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .bg(theme.accent())
                                    .cursor_pointer()
                                    .child(
                                        svg()
                                            .path(play_icon)
                                            .size(px(17.0))
                                            .text_color(theme.on_accent()),
                                    )
                                    .on_click(cx.listener(move |_, _, _, _| {
                                        let bus = bus_play.clone();
                                        services::spawn_tokio(async move {
                                            MprisService::play_pause_bus(&bus).await;
                                        });
                                    })),
                            )
                            .child(
                                div()
                                    .id("shared-media-next-btn")
                                    .size(px(32.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(8.0))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(next_hover))
                                    .child(
                                        svg()
                                            .path("skip-forward.svg")
                                            .size(px(15.0))
                                            .text_color(theme.foreground()),
                                    )
                                    .on_click(cx.listener(move |_, _, _, _| {
                                        let bus = bus_next.clone();
                                        services::spawn_tokio(async move {
                                            MprisService::next_bus(&bus).await;
                                        });
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}
