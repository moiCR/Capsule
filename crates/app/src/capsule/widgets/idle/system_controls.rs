use gpui::{Context, FontWeight, IntoElement, div, prelude::*, svg};
use services::{SystemService, SystemStatus};
use ui::theme::Theme;

pub fn render_system_controls_widget<V: 'static>(
    status: &SystemStatus,
    service: &SystemService,
    theme: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let service_mute = service.clone();
    let service_vol_dec = service.clone();
    let service_vol_inc = service.clone();
    let service_bright_dec = service.clone();
    let service_bright_inc = service.clone();

    let current_vol = status.volume;
    let current_bright = status.brightness;
    let is_muted = status.is_muted;

    div()
        .flex()
        .flex_col()
        .w_full()
        .gap_2p5()
        .p_3()
        .bg(theme.surface())
        .rounded_xl()
        .border_1()
        .border_color(theme.background_alt())
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .w_full()
                        .child(
                            div()
                                .id("mute-toggle-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w_7()
                                .h_7()
                                .rounded_md()
                                .bg(if is_muted {
                                    theme.red_color.to_hsla()
                                } else {
                                    theme.background_alt()
                                })
                                .cursor_pointer()
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    let service = service_mute.clone();
                                    cx.spawn(async move |_this, _cx| {
                                        let _ = service.toggle_mute().await;
                                    })
                                    .detach();
                                }))
                                .child(
                                    svg()
                                        .path(if is_muted {
                                            "bell-off.svg"
                                        } else {
                                            "volume-2.svg"
                                        })
                                        .w_3p5()
                                        .h_3p5()
                                        .text_color(theme.foreground()),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .id("vol-dec-btn")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w_6()
                                        .h_6()
                                        .rounded_md()
                                        .bg(theme.background_alt())
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            let service = service_vol_dec.clone();
                                            let target = current_vol.saturating_sub(5);
                                            cx.spawn(async move |_this, _cx| {
                                                let _ = service.set_volume(target).await;
                                            })
                                            .detach();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.foreground())
                                                .child("-"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .h_2()
                                        .rounded_full()
                                        .bg(theme.background_alt())
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(gpui::DefiniteLength::Fraction(
                                                    (current_vol as f32 / 100.0).clamp(0.0, 1.0),
                                                ))
                                                .bg(theme.accent()),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("vol-inc-btn")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w_6()
                                        .h_6()
                                        .rounded_md()
                                        .bg(theme.background_alt())
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            let service = service_vol_inc.clone();
                                            let target = current_vol.saturating_add(5);
                                            cx.spawn(async move |_this, _cx| {
                                                let _ = service.set_volume(target).await;
                                            })
                                            .detach();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.foreground())
                                                .child("+"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .w_8()
                                .text_right()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground_muted())
                                .child(format!("{current_vol}%")),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .w_7()
                                .h_7()
                                .rounded_md()
                                .bg(theme.background_alt())
                                .child(
                                    svg()
                                        .path("sun.svg")
                                        .w_3p5()
                                        .h_3p5()
                                        .text_color(theme.foreground()),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .id("bright-dec-btn")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w_6()
                                        .h_6()
                                        .rounded_md()
                                        .bg(theme.background_alt())
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            let service = service_bright_dec.clone();
                                            let target = current_bright.saturating_sub(5);
                                            cx.spawn(async move |_this, _cx| {
                                                let _ = service.set_brightness(target).await;
                                            })
                                            .detach();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.foreground())
                                                .child("-"),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .h_2()
                                        .rounded_full()
                                        .bg(theme.background_alt())
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(gpui::DefiniteLength::Fraction(
                                                    (current_bright as f32 / 100.0).clamp(0.0, 1.0),
                                                ))
                                                .bg(theme.accent()),
                                        ),
                                )
                                .child(
                                    div()
                                        .id("bright-inc-btn")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .w_6()
                                        .h_6()
                                        .rounded_md()
                                        .bg(theme.background_alt())
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_, _, _, cx| {
                                            let service = service_bright_inc.clone();
                                            let target = current_bright.saturating_add(5);
                                            cx.spawn(async move |_this, _cx| {
                                                let _ = service.set_brightness(target).await;
                                            })
                                            .detach();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme.foreground())
                                                .child("+"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .w_8()
                                .text_right()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground_muted())
                                .child(format!("{current_bright}%")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .gap_2()
                .child(
                    div()
                        .id("sys-lock-btn")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.background_alt())
                        .hover(|style| style.bg(theme.surface()))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.spawn(async move |_this, _cx| {
                                let _ = SystemService::lock().await;
                            })
                            .detach();
                        }))
                        .child(
                            svg()
                                .path("lock.svg")
                                .w_4()
                                .h_4()
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .id("sys-suspend-btn")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.background_alt())
                        .hover(|style| style.bg(theme.surface()))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.spawn(async move |_this, _cx| {
                                let _ = SystemService::suspend().await;
                            })
                            .detach();
                        }))
                        .child(
                            svg()
                                .path("moon.svg")
                                .w_4()
                                .h_4()
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .id("sys-reboot-btn")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.background_alt())
                        .hover(|style| style.bg(theme.surface()))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.spawn(async move |_this, _cx| {
                                let _ = SystemService::reboot().await;
                            })
                            .detach();
                        }))
                        .child(
                            svg()
                                .path("rotate-cw.svg")
                                .w_4()
                                .h_4()
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .id("sys-poweroff-btn")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme.red_color.to_hsla().opacity(0.2))
                        .hover(|style| style.bg(theme.red_color.to_hsla().opacity(0.35)))
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.spawn(async move |_this, _cx| {
                                let _ = SystemService::poweroff().await;
                            })
                            .detach();
                        }))
                        .child(
                            svg()
                                .path("power.svg")
                                .w_4()
                                .h_4()
                                .text_color(theme.red_color.to_hsla()),
                        ),
                ),
        )
}
