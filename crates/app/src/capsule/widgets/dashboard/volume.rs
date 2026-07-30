use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn compute_volume_panel_height(sink_count: usize) -> f32 {
    let base = 50.0;
    let item_h = 36.0;
    (base + (sink_count as f32 * item_h)).clamp(100.0, 300.0)
}

pub fn render_volume_widget(
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

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_3()
        .py_2()
        .rounded_xl()
        .bg(theme.surface().opacity(0.4))
        .gap_2p5()
        .child(
            // Mute / Volume icon button
            div()
                .id("volume-mute-btn")
                .flex()
                .flex_row()
                .items_center()
                .gap_1p5()
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
                        .size(px(16.0))
                        .text_color(if is_muted {
                            theme.foreground_muted()
                        } else {
                            theme.accent()
                        }),
                )
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.5))
                        .text_color(theme.foreground())
                        .w(px(32.0))
                        .child(format!("{vol_percentage}%")),
                ),
        )
        .child(
            // Interactive Volume Bar Slider
            div()
                .id("volume-slider-bar")
                .flex_1()
                .h(px(8.0))
                .rounded_full()
                .bg(theme.surface())
                .cursor_pointer()
                .overflow_hidden()
                .relative()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                        this.is_dragging_volume = true;
                        let win_w: f32 = window.bounds().size.width.into();
                        let pill_x = (win_w - 440.0) / 2.0;
                        let slider_start_x = pill_x + 92.0;
                        let slider_width = 288.0;

                        let x_val = f32::from(event.position.x);
                        let rel_x = x_val - slider_start_x;
                        let pct = ((rel_x / slider_width) * 100.0).clamp(0.0, 100.0) as u32;

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
                ),
        )
        .child(
            // Chevron right button to open audio output satellite panel
            div()
                .id("volume-chevron-btn")
                .flex()
                .items_center()
                .justify_center()
                .w(px(22.0))
                .h(px(22.0))
                .rounded_full()
                .bg(theme.surface())
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .cursor_pointer()
                .on_click(cx.listener(|_this, _, _, cx| {
                    cx.emit(DashboardEvent::VolumeChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(13.0))
                        .text_color(theme.foreground()),
                ),
        )
}

pub fn render_volume_mini_panel(
    anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    use super::panel_manager::PANEL_W;

    let sinks = if cx.has_global::<AppState>() {
        cx.global::<AppState>().system.get_status().audio_sinks
    } else {
        Vec::new()
    };

    let mut list_col = div()
        .flex()
        .flex_col()
        .w_full()
        .gap_1();

    for sink in &sinks {
        let is_def = sink.is_default;
        let sink_name = sink.name.clone();

        list_col = list_col.child(
            div()
                .id(format!("sink-item-{}", sink.name))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .px_2()
                .py_1p5()
                .rounded_lg()
                .bg(if is_def {
                    theme.accent().opacity(0.15)
                } else {
                    theme.surface().opacity(0.0)
                })
                .hover(|s| {
                    if is_def {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.5))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        let sys = cx.global::<AppState>().system.clone();
                        let target_name = sink_name.clone();
                        let this = cx.entity().downgrade();
                        cx.spawn(async move |_this, cx| {
                            let _ = sys.set_default_sink(&target_name).await;
                            let _ = this.update(cx, |_view, cx| cx.notify());
                        })
                        .detach();
                    }
                }))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .overflow_hidden()
                        .child(
                            svg()
                                .path("volume-2.svg")
                                .size(px(13.0))
                                .text_color(if is_def {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(if is_def {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if is_def {
                                    theme.accent()
                                } else {
                                    theme.foreground()
                                })
                                .truncate()
                                .child(sink.description.clone()),
                        ),
                ),
        );
    }

    div()
        .w(px(PANEL_W))
        .h(px(panel_h))
        .p_2p5()
        .gap_2()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(anim_t)
        .flex()
        .flex_col()
        .child(
            // Panel Header
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_1p5()
                .w_full()
                .child(
                    svg()
                        .path("volume-2.svg")
                        .size(px(14.0))
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(11.5))
                        .text_color(theme.foreground())
                        .child("Salida de audio"),
                ),
        )
        .child(list_col)
        .into_any_element()
}
