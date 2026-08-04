use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_W;

pub fn compute_volume_panel_height(sink_count: usize) -> f32 {
    let base = 50.0;
    let item_h = 36.0;
    (base + (sink_count as f32 * item_h)).clamp(100.0, 300.0)
}

pub fn render_volume_mini_panel(
    anim_t: f32,
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
        .max_h(px(panel_h))
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
                        .child({
                            let lang = if cx.has_global::<ui::language::Language>() {
                                cx.global::<ui::language::Language>().clone()
                            } else {
                                ui::language::Language::default()
                            };
                            lang.volume.audio_output
                        }),
                ),
        )
        .child(list_col)
        .into_any_element()
}
