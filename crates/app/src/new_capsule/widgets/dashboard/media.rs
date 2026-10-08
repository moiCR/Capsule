use super::{DashboardAction as Action, DashboardView, icon_action};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, PathBuilder, StyledImage, div, img, point,
    prelude::*, px, svg,
};
use services::MediaTrack;
use std::path::PathBuf;
use ui::theme::Theme;

pub(crate) fn artwork(track: &MediaTrack, size: f32, theme: &Theme) -> AnyElement {
    let image = track.local_art_path.as_ref().map(PathBuf::from);
    let frame = div()
        .size(px(size))
        .flex_shrink_0()
        .rounded(px(12.0))
        .overflow_hidden()
        .bg(theme.surface())
        .flex()
        .items_center()
        .justify_center();
    if let Some(path) = image {
        frame
            .child(
                img(path)
                    .size(px(size))
                    .aspect_ratio(1.0)
                    .rounded(px(12.0))
                    .object_fit(gpui::ObjectFit::Cover),
            )
            .into_any_element()
    } else {
        frame
            .child(
                svg()
                    .path("music.svg")
                    .size(px(size * 0.4))
                    .text_color(theme.foreground_muted()),
            )
            .into_any_element()
    }
}

pub(crate) fn position(track: &MediaTrack, elapsed: std::time::Duration) -> i64 {
    let position = track.position_micros.unwrap_or(0).max(0);
    let elapsed = if track.is_playing {
        elapsed.as_micros().min(i64::MAX as u128) as i64
    } else {
        0
    };
    let next = position.saturating_add(elapsed);
    track
        .length_micros
        .filter(|length| *length > 0)
        .map_or(next, |length| next.min(length))
}

fn timestamp(micros: i64) -> String {
    let seconds = micros.max(0) / 1_000_000;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let radius = cx
        .global::<services::AppState>()
        .config
        .get()
        .ui
        .cards_round;
    let card = div()
        .id("dashboard-media")
        .relative()
        .overflow_hidden()
        .h(px(168.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .p(px(12.0))
        .gap(px(6.0))
        .rounded(px(radius))
        .bg(theme.surface().opacity(0.2));
    let Some(track) = module.player() else {
        return card
            .justify_center()
            .child(super::empty(module.text("dashboard.no_media", cx), theme))
            .into_any_element();
    };
    let position = position(track, module.snapshot_at.elapsed());
    let length = track.length_micros.unwrap_or(0).max(0);
    let fraction = if length > 0 {
        (position as f32 / length as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let phase = module.wave_phase;
    let amplitude = if track.is_playing { 3.0 } else { 0.0 };
    let active = theme.accent();
    let inactive = theme.foreground_muted().opacity(0.3);
    let bounds_tracker = module.seek_bounds.clone();
    let progress = div()
        .id("dashboard-seek")
        .relative()
        .w_full()
        .h(px(22.0))
        .flex_shrink_0()
        .when(length > 0, |s| {
            s.cursor_pointer().on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _: &gpui::MouseDownEvent, _, _| this.start_seek()),
            )
        })
        .child(
            gpui::canvas(
                move |bounds, _, _| bounds_tracker.track_bounds(bounds),
                move |bounds, _, window, _| {
                    let width: f32 = bounds.size.width.into();
                    let height: f32 = bounds.size.height.into();
                    let center = height / 2.0;
                    let played = width * fraction;
                    let mut rest = PathBuilder::stroke(px(2.0));
                    rest.move_to(bounds.origin + point(px(played), px(center)));
                    rest.line_to(bounds.origin + point(px(width), px(center)));
                    if let Ok(path) = rest.build() {
                        window.paint_path(path, inactive);
                    }
                    if played > 0.0 {
                        let mut wave = PathBuilder::stroke(px(2.0));
                        let steps = played.ceil() as usize;
                        for step in 0..=steps {
                            let x = (step as f32).min(played);
                            let envelope =
                                (x / 8.0).min(1.0) * ((played - x) / 8.0).clamp(0.0, 1.0);
                            let y = center
                                + (x / 22.0 * std::f32::consts::TAU - phase).sin()
                                    * amplitude
                                    * envelope;
                            let point = bounds.origin + point(px(x), px(y));
                            if step == 0 {
                                wave.move_to(point);
                            } else {
                                wave.line_to(point);
                            }
                        }
                        if let Ok(path) = wave.build() {
                            window.paint_path(path, active);
                        }
                    }
                },
            )
            .w_full()
            .h_full(),
        );
    let subtitle = [&track.artist, &track.album]
        .into_iter()
        .filter(|value| !value.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join(" · ");
    let card = card.when_some(track.local_art_path.as_ref(), |card, path| {
        card.child(
            div()
                .absolute()
                .inset_0()
                .child(
                    img(PathBuf::from(path))
                        .absolute()
                        .inset_0()
                        .size_full()
                        .aspect_ratio(super::RIGHT / 168.0)
                        .rounded(px(radius))
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(radius))
                        .backdrop_blur(px(18.0)),
                ),
        )
    });
    card.child(
        div()
            .relative()
            .h(px(48.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(div().flex_1())
            .child(
                div()
                    .id("dashboard-player-picker")
                    .flex_shrink_0()
                    .self_start()
                    .h(px(28.0))
                    .max_w(px(104.0))
                    .px(px(6.0))
                    .rounded(px(8.0))
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .cursor_pointer()
                    .hover({
                        let hover = theme.surface().opacity(0.5);
                        move |s| s.bg(hover)
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.dispatch(Action::View(DashboardView::Media), cx)
                    }))
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(11.0))
                            .text_ellipsis()
                            .child(track.player_name.clone()),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(12.0))
                            .flex_shrink_0()
                            .text_color(theme.foreground_muted()),
                    ),
            ),
    )
    .child(
        div()
            .relative()
            .h(px(44.0))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(8.0))
            .justify_between()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .when(!subtitle.is_empty(), |s| {
                        s.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.foreground_muted())
                                .text_ellipsis()
                                .child(subtitle),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_ellipsis()
                            .child(track.title.clone()),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .gap(px(6.0))
                    .child(icon_action(
                        "dashboard-previous",
                        "skip-back.svg",
                        Action::Media(-1),
                        track.can_go_previous,
                        theme,
                        cx,
                    ))
                    .child(div().rounded_full().bg(theme.accent().opacity(0.16)).child(
                        icon_action(
                            "dashboard-play",
                            if track.is_playing {
                                "pause.svg"
                            } else {
                                "play.svg"
                            },
                            Action::Media(0),
                            true,
                            theme,
                            cx,
                        ),
                    ))
                    .child(icon_action(
                        "dashboard-next",
                        "skip-forward.svg",
                        Action::Media(1),
                        track.can_go_next,
                        theme,
                        cx,
                    )),
            ),
    )
    .child(progress)
    .child(
        div()
            .relative()
            .flex()
            .justify_between()
            .text_size(px(10.0))
            .text_color(theme.foreground_muted())
            .child(timestamp(position))
            .child(timestamp(length)),
    )
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn progress_stops_when_paused_and_never_exceeds_the_track() {
        let mut track = MediaTrack {
            position_micros: Some(2_000_000),
            length_micros: Some(3_000_000),
            ..Default::default()
        };
        assert_eq!(
            position(&track, std::time::Duration::from_secs(4)),
            2_000_000
        );
        track.is_playing = true;
        assert_eq!(
            position(&track, std::time::Duration::from_secs(4)),
            3_000_000
        );
        track.position_micros = Some(-20);
        assert_eq!(position(&track, std::time::Duration::ZERO), 0);
    }
}
