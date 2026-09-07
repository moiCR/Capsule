use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;

pub fn compute_volume_panel_height(sink_count: usize) -> f32 {
    let base = 65.0;
    let item_h = 32.0;
    (base + (sink_count as f32 * item_h)).clamp(100.0, 320.0)
}

pub fn render_volume_mini_panel(
    _anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let sinks = if cx.has_global::<AppState>() {
        cx.global::<AppState>().system.get_status().audio_sinks
    } else {
        Vec::new()
    };

    let mut list_col = div()
        .id("volume-internal-scroll")
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1();

    for (idx, sink) in sinks.iter().enumerate() {
        let is_def = sink.is_default;
        let sink_name = sink.name.clone();

        let indicator_color = if is_def {
            theme.accent()
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        };

        list_col =
            list_col.child(
                div()
                    .id(("sink-item", idx as u32))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded(px(10.0))
                    .bg(if is_def {
                        theme.surface().opacity(0.55)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| s.bg(theme.surface().opacity(0.4)))
                    .active(|s| s.bg(theme.surface().opacity(0.6)))
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
                                div()
                                    .w(px(3.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .bg(indicator_color),
                            )
                            .child(svg().path("volume-2.svg").size(px(14.0)).text_color(
                                if is_def {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                },
                            ))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(if is_def {
                                        FontWeight::SEMIBOLD
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
                    )
                    .child(
                        div()
                            .text_size(px(9.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent().opacity(0.85))
                            .child(if is_def { "Activo" } else { "" }),
                    ),
            );
    }

    let radius = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.satellite_round
    } else {
        20.0
    };

    div()
        .min_w(px(PANEL_MIN_W))
        .max_h(px(panel_h))
        .p_3()
        .gap_2()
        .rounded(px(radius))
        .bg(theme.background().opacity(0.95))
        .border_1()
        .border_color(theme.surface().opacity(0.35))
        .shadow_lg()
        .overflow_hidden()
        .flex()
        .flex_col()
        .child(
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
                        .child(if cx.has_global::<AppState>() {
                            cx.global::<AppState>().language.get("volume.audio_output")
                        } else {
                            "Salida de audio".to_string()
                        }),
                ),
        )
        .child(list_col)
        .into_any_element()
}
