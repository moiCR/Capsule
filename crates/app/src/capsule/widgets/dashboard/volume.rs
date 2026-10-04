use gpui::{Context, FontWeight, IntoElement, canvas, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn render_volume_widget(
    tracker: &DimensionTracker,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let (volume, is_muted) = if cx.has_global::<AppState>() {
        let status = cx.global::<AppState>().system.get_status();
        (status.volume, status.is_muted)
    } else {
        (50, false)
    };

    let icon_path = if is_muted || volume == 0 {
        "volume-x.svg"
    } else {
        "volume-2.svg"
    };

    let vol_percentage = volume.min(100);
    let slider_tracker = tracker.clone();

    div()
        .id("sound-card-main")
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(46.0))
        .gap_2p5()
        .child(
            div()
                .id("volume-slider-bar")
                .relative()
                .flex_1()
                .h(px(46.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.35))
                .border_1()
                .border_color(theme.surface().opacity(0.18))
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
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.is_dragging_volume = true;
                        let slider_x = this.slider_tracker.left();
                        let slider_w = this.slider_tracker.width(0.0);
                        let (start_x, width) = if slider_w > 0.0 {
                            (slider_x, slider_w)
                        } else {
                            let win_w: f32 = window.bounds().size.width.into();
                            let pill_x = (win_w - 540.0) / 2.0;
                            (pill_x + 16.0, 540.0 - 32.0 - 46.0 - 10.0)
                        };

                        let x_val = f32::from(event.position.x);
                        let rel_x = x_val - start_x;
                        let pct = ((rel_x / width) * 100.0).clamp(0.0, 100.0) as u32;

                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>().system.set_volume_fast(pct);
                        }
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .h_full()
                        .w(gpui::DefiniteLength::Fraction(
                            (vol_percentage as f32 / 100.0).clamp(0.0, 1.0),
                        ))
                        .rounded_full()
                        .bg(if is_muted {
                            theme.foreground_muted().opacity(0.4)
                        } else {
                            theme.accent()
                        }),
                )
                .child(
                    div()
                        .id("volume-mute-icon-btn")
                        .absolute()
                        .left(px(6.0))
                        .top(px(6.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(34.0))
                        .rounded_full()
                        .hover(|s| s.bg(theme.background().opacity(0.2)))
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let sys = cx.global::<AppState>().system.clone();
                                services::spawn_tokio(async move {
                                    let _ = sys.toggle_mute().await;
                                });
                                cx.notify();
                            }
                        }))
                        .child(
                            svg()
                                .path(icon_path)
                                .size(px(16.0))
                                .text_color(if is_muted {
                                    theme.foreground_muted()
                                } else {
                                    theme.background()
                                }),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .right(px(14.0))
                        .top(px(0.0))
                        .bottom(px(0.0))
                        .flex()
                        .items_center()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if vol_percentage > 85 {
                            theme.background()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(format!("{vol_percentage}%")),
                ),
        )
        .child(
            div()
                .id("volume-chevron-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(46.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.35))
                .border_1()
                .border_color(theme.surface().opacity(0.18))
                .hover(|s| s.bg(theme.surface().opacity(0.65)))
                .active(|s| s.bg(theme.surface().opacity(0.85)))
                .cursor_pointer()
                .on_click(cx.listener(|_this, _, _, cx| {
                    cx.emit(DashboardEvent::VolumeChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
}
