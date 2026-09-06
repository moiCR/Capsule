use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;

pub fn compute_bluetooth_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 75.0;
    let items_h = if status.bluetooth_device_list.is_empty() {
        30.0
    } else {
        status.bluetooth_device_list.len() as f32 * 32.0
    };
    (base_h + items_h).clamp(140.0, 380.0)
}

pub fn render_bluetooth_mini_panel(
    _anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let mut dev_list = div()
        .id("bt-internal-scroll")
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1();

    if status.bluetooth_device_list.is_empty() {
        dev_list = dev_list.child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_full()
                .py_3()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .child(lang.quick_settings.no_bt_found),
                ),
        );
    } else {
        for (idx, dev) in status.bluetooth_device_list.iter().enumerate() {
            let mac = dev.mac.clone();
            let name = dev.name.clone();
            let is_conn = dev.is_connected;
            let mac_click = mac.clone();

            let indicator_color = if is_conn {
                theme.accent()
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };

            dev_list = dev_list.child(
                div()
                    .id(("bt-dev-item", idx as u32))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded(px(10.0))
                    .bg(if is_conn {
                        theme.surface().opacity(0.55)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| s.bg(theme.surface().opacity(0.4)))
                    .active(|s| s.bg(theme.surface().opacity(0.6)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>()
                                .network
                                .connect_bluetooth(&mac_click);
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(3.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .bg(indicator_color),
                            )
                            .child(svg().path("bluetooth.svg").size(px(14.0)).text_color(
                                if is_conn {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                },
                            ))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(if is_conn {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(if is_conn {
                                        theme.accent()
                                    } else {
                                        theme.foreground()
                                    })
                                    .child(name),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(9.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent().opacity(0.85))
                            .child(if is_conn { "Conectado" } else { "" }),
                    ),
            );
        }
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
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .child(
                            svg()
                                .path("bluetooth.svg")
                                .size(px(14.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .child(lang.quick_settings.bt_devices),
                        ),
                )
                .child(
                    div()
                        .id("close-bt-panel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(0.6))
                        .hover(|s| s.bg(theme.surface().opacity(0.9)))
                        .cursor_pointer()
                        .on_click(cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                cx.global::<AppState>().network.toggle_bluetooth();
                            }
                            cx.notify();
                        }))
                        .child(svg().path("power.svg").size(px(10.0)).text_color(
                            if status.bluetooth_enabled {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            },
                        )),
                ),
        )
        .child(dev_list)
        .into_any_element()
}
