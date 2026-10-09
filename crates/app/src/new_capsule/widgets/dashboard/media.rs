use super::{DashboardAction as Action, icon_action};
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::widgets::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, PathBuilder, StyledImage, div, img, point,
    prelude::*, px, svg,
};
use services::MediaTrack;
use std::path::PathBuf;
use ui::theme::Theme;

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
        .cards_round
        .min(22.0);
    let card = div()
        .id("dashboard-media")
        .relative()
        .overflow_hidden()
        .h(px(super::MEDIA_HEIGHT))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .p(px(8.0))
        .gap(px(12.0))
        .justify_center()
        .rounded(px(radius))
        .bg(style::surface(theme));
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
        .h(px(12.0))
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
                        .aspect_ratio(super::RIGHT / super::MEDIA_HEIGHT)
                        .rounded(px(radius))
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .rounded(px(radius))
                        .backdrop_blur(px(18.0))
                        .bg(theme.background().opacity(0.62)),
                ),
        )
    });
    card.child(
        div()
            .relative()
            .w_full()
            .min_h(px(52.0))
            .flex_shrink_0()
            .px(px(12.0))
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(17.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(track.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .whitespace_nowrap()
                            .text_ellipsis()
                            .child(track.artist.clone()),
                    ),
            )
            .child(
                div()
                    .id("dashboard-play")
                    .size(px(36.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(theme.accent())
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| this.dispatch(Action::Media(0), cx)))
                    .child(
                        svg()
                            .path(if track.is_playing {
                                "pause.svg"
                            } else {
                                "play.svg"
                            })
                            .size(px(22.0))
                            .text_color(style::on_accent(theme)),
                    ),
            ),
    )
    .child(
        div()
            .relative()
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(icon_action(
                "dashboard-previous",
                "skip-back.svg",
                Action::Media(-1),
                track.can_go_previous,
                theme,
                cx,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h(px(32.0))
                    .flex()
                    .items_center()
                    .child(progress),
            )
            .child(icon_action(
                "dashboard-next",
                "skip-forward.svg",
                Action::Media(1),
                track.can_go_next,
                theme,
                cx,
            )),
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
