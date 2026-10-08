use super::{DashboardAction as Action, button};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, MotionDurationExt, StyledImage, div, ease_in_out,
    img, prelude::*, px, relative,
};
use ui::theme::Theme;

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut card = div()
        .id("dashboard-media")
        .h(px(144.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .p(px(12.0))
        .gap(px(8.0))
        .rounded(px(14.0))
        .bg(theme.surface().opacity(0.25));
    let Some(track) = module.player() else {
        return card
            .justify_center()
            .child(super::empty(module.text("dashboard.no_media", cx), theme))
            .into_any_element();
    };
    let art = track
        .local_art_path
        .as_ref()
        .map(|path| {
            img(path.clone())
                .size(px(56.0))
                .object_fit(gpui::ObjectFit::Cover)
                .rounded(px(10.0))
                .into_any_element()
        })
        .unwrap_or_else(|| {
            div()
                .size(px(56.0))
                .rounded(px(10.0))
                .bg(theme.surface())
                .flex()
                .items_center()
                .justify_center()
                .child(
                    gpui::svg()
                        .path("music.svg")
                        .size(px(24.0))
                        .text_color(theme.foreground_muted()),
                )
                .into_any_element()
        });
    card = card.child(
        div()
            .flex()
            .h(px(56.0))
            .flex_shrink_0()
            .gap(px(12.0))
            .child(art)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(5.0))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_ellipsis()
                            .child(if track.title.is_empty() {
                                module.text("dashboard.no_media", cx)
                            } else {
                                track.title.clone()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .text_ellipsis()
                            .child(track.artist.clone()),
                    ),
            ),
    );
    let next_player = module
        .snapshot
        .players
        .iter()
        .position(|p| p.bus_name == track.bus_name)
        .and_then(|index| {
            module
                .snapshot
                .players
                .get((index + 1) % module.snapshot.players.len())
        })
        .map(|p| p.bus_name.clone())
        .unwrap_or_default();
    let controls = div()
        .flex()
        .items_center()
        .justify_between()
        .h(px(36.0))
        .flex_shrink_0()
        .child(button(
            "dashboard-player",
            track.player_name.clone(),
            "music.svg",
            Action::Player(next_player),
            false,
            theme,
            cx,
        ))
        .child(
            div()
                .flex()
                .gap(px(4.0))
                .child(
                    div()
                        .opacity(if track.can_go_previous { 1.0 } else { 0.3 })
                        .child(button(
                            "dashboard-previous",
                            String::new(),
                            "skip-back.svg",
                            Action::Media(-1),
                            false,
                            theme,
                            cx,
                        )),
                )
                .child(button(
                    "dashboard-play",
                    String::new(),
                    if track.is_playing {
                        "pause.svg"
                    } else {
                        "play.svg"
                    },
                    Action::Media(0),
                    track.is_playing,
                    theme,
                    cx,
                ))
                .child(
                    div()
                        .opacity(if track.can_go_next { 1.0 } else { 0.3 })
                        .child(button(
                            "dashboard-next",
                            String::new(),
                            "skip-forward.svg",
                            Action::Media(1),
                            false,
                            theme,
                            cx,
                        )),
                ),
        );
    let bounds = module.seek_bounds.clone();
    let fraction = match (track.position_micros, track.length_micros) {
        (Some(position), Some(length)) if length > 0 => {
            (position as f32 / length as f32).clamp(0.0, 1.0)
        }
        _ => 0.0,
    };
    card.child(controls)
        .child(
            div()
                .id("dashboard-seek")
                .relative()
                .w_full()
                .h(px(8.0))
                .flex_shrink_0()
                .cursor_pointer()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _: &gpui::MouseDownEvent, _, _| this.start_seek()),
                )
                .child(
                    gpui::canvas(
                        move |bounds_rect, _, _| bounds.track_bounds(bounds_rect),
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .child(
                    div()
                        .h(px(4.0))
                        .w_full()
                        .rounded_full()
                        .bg(theme.surface())
                        .child(
                            div()
                                .id("dashboard-seek-fill")
                                .h_full()
                                .w(relative(fraction))
                                .transitions(|t| {
                                    t.w(module.snapshot.duration.with_easing(ease_in_out))
                                })
                                .rounded_full()
                                .bg(theme.accent()),
                        ),
                ),
        )
        .into_any_element()
}
