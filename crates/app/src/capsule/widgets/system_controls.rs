use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{SystemService, SystemStatus};
use ui::theme::Theme;

pub fn render_system_controls_widget<V: 'static>(
    status: &SystemStatus,
    service: &SystemService,
    theme: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let service_vol = service.clone();
    let service_mute = service.clone();
    let service_bright = service.clone();
    let service_lock = service.clone();
    let service_suspend = service.clone();
    let service_reboot = service.clone();
    let service_poweroff = service.clone();

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
        // Quick Sliders (Volume & Brightness)
        .child(
            div()
                .flex()
                .flex_col()
                .w_full()
                .gap_2()
                // Volume Row
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
                                .on_click(cx.listener(move |_, _, _window, _cx| {
                                    let s = service_mute.clone();
                                    tokio::spawn(async move {
                                        let _ = s.toggle_mute().await;
                                    });
                                }))
                                .child(
                                    svg()
                                        .path(if is_muted {
                                            "bell-off.svg"
                                        } else {
                                            "music.svg"
                                        })
                                        .w_3p5()
                                        .h_3p5()
                                        .text_color(theme.foreground()),
                                ),
                        )
                        // Volume Bar
                        .child(
                            div()
                                .id("vol-bar")
                                .flex_1()
                                .h_2p5()
                                .rounded_full()
                                .bg(theme.background_alt())
                                .overflow_hidden()
                                .cursor_pointer()
                                .on_click(cx.listener(
                                    move |_, event: &gpui::ClickEvent, _window, _cx| {
                                        let s = service_vol.clone();
                                        // Estimate click position percentage
                                        let new_vol = ((current_vol + 10) % 100).max(10);
                                        let _ = event;
                                        tokio::spawn(async move {
                                            let _ = s.set_volume(new_vol).await;
                                        });
                                    },
                                ))
                                .child(
                                    div()
                                        .h_full()
                                        .w(px((current_vol as f32 / 100.0) * 200.0))
                                        .bg(theme.accent()),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.foreground_muted())
                                .child(format!("{current_vol}%")),
                        ),
                )
                // Brightness Row
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
                                        .text_color(theme.foreground_muted()),
                                ),
                        )
                        .child(
                            div()
                                .id("bright-bar")
                                .flex_1()
                                .h_2p5()
                                .rounded_full()
                                .bg(theme.background_alt())
                                .overflow_hidden()
                                .cursor_pointer()
                                .on_click(cx.listener(move |_, _event, _window, _cx| {
                                    let s = service_bright.clone();
                                    let new_bright = ((current_bright + 20) % 100).max(20);
                                    tokio::spawn(async move {
                                        let _ = s.set_brightness(new_bright).await;
                                    });
                                }))
                                .child(
                                    div()
                                        .h_full()
                                        .w(px((current_bright as f32 / 100.0) * 200.0))
                                        .bg(theme.foreground()),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.foreground_muted())
                                .child(format!("{current_bright}%")),
                        ),
                ),
        )
        // Power Actions Bar (Lock, Suspend, Reboot, Shutdown)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .pt_1()
                // Lock Button
                .child(
                    div()
                        .id("power-lock")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme.background_alt())
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, _| {
                            let _ = service_lock;
                            tokio::spawn(async move {
                                let _ = SystemService::lock().await;
                            });
                        }))
                        .child(
                            svg()
                                .path("moon.svg")
                                .w_3p5()
                                .h_3p5()
                                .text_color(theme.foreground()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child("Bloquear"),
                        ),
                )
                // Suspend Button
                .child(
                    div()
                        .id("power-suspend")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme.background_alt())
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, _| {
                            let _ = service_suspend;
                            tokio::spawn(async move {
                                let _ = SystemService::suspend().await;
                            });
                        }))
                        .child(
                            svg()
                                .path("sparkles.svg")
                                .w_3p5()
                                .h_3p5()
                                .text_color(theme.foreground()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child("Suspender"),
                        ),
                )
                // Reboot Button
                .child(
                    div()
                        .id("power-reboot")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme.background_alt())
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, _| {
                            let _ = service_reboot;
                            tokio::spawn(async move {
                                let _ = SystemService::reboot().await;
                            });
                        }))
                        .child(
                            svg()
                                .path("chevron-right.svg")
                                .w_3p5()
                                .h_3p5()
                                .text_color(theme.foreground()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child("Reiniciar"),
                        ),
                )
                // Power Off Button
                .child(
                    div()
                        .id("power-off")
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme.red_color.to_hsla())
                        .cursor_pointer()
                        .on_click(cx.listener(move |_, _, _, _| {
                            let _ = service_poweroff;
                            tokio::spawn(async move {
                                let _ = SystemService::poweroff().await;
                            });
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .w_3p5()
                                .h_3p5()
                                .text_color(theme.foreground()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Apagar"),
                        ),
                ),
        )
}
