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

    let card_radius = px(if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        22.0
    });

    let slider_tracker = tracker.clone();

    let sound_label = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get("dashboard.sound")
    } else {
        "Sonido".to_string()
    };

    div()
        .id("sound-card-main")
        .flex()
        .flex_col()
        .w_full()
        .p_3()
        .rounded(card_radius)
        .bg(theme.surface().opacity(0.45))
        .border_1()
        .border_color(theme.surface().opacity(0.25))
        .gap_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(12.5))
                        .text_color(theme.foreground())
                        .child(sound_label),
                )
                .child(
                    div()
                        .id("volume-chevron-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(22.0))
                        .h(px(22.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(0.6))
                        .hover(|s| s.bg(theme.surface().opacity(0.9)))
                        .active(|s| s.bg(theme.surface()))
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            cx.emit(DashboardEvent::VolumeChevronClicked);
                        }))
                        .child(
                            svg()
                                .path("chevron-right.svg")
                                .size(px(13.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .child(
            div()
                .id("volume-slider-bar")
                .relative()
                .flex()
                .items_center()
                .w_full()
                .h(px(42.0))
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
                            let pill_x = (win_w - 490.0) / 2.0;
                            (pill_x + 28.0, 434.0)
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
                        .left(px(12.0))
                        .top(px(12.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(18.0))
                        .h(px(18.0))
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let sys = cx.global::<AppState>().system.clone();
                                let this = cx.entity().downgrade();
                                cx.spawn(async move |_this, cx| {
                                    let _ = sys.toggle_mute().await;
                                    let _ = this.update(cx, |_view, cx| cx.notify());
                                })
                                .detach();
                            }
                        }))
                        .child(
                            svg()
                                .path(icon_path)
                                .size(px(18.0))
                                .text_color(if is_muted {
                                    theme.foreground_muted()
                                } else {
                                    theme.background()
                                }),
                        ),
                ),
        )
}
