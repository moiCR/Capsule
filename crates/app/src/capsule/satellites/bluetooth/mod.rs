use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;

pub fn compute_bluetooth_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 80.0;
    let items_h = if status.bluetooth_device_list.is_empty() {
        36.0
    } else {
        status.bluetooth_device_list.len() as f32 * 36.0
    };
    (base_h + items_h).clamp(150.0, 420.0)
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

    let (
        no_bt_found,
        bt_devices,
        connecting_str,
        connected_str,
        paired_str,
        unpaired_str,
        disconnect_str,
        scanning_str,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("quick_settings.no_bt_found"),
            lang.get("quick_settings.bt_devices"),
            lang.get("quick_settings.connecting"),
            lang.get("dashboard.connected"),
            lang.get("quick_settings.paired"),
            lang.get("quick_settings.unpaired"),
            lang.get("quick_settings.disconnect"),
            lang.get("quick_settings.scanning"),
        )
    } else {
        (
            "No hay dispositivos Bluetooth".to_string(),
            "Dispositivos Bluetooth".to_string(),
            "Conectando...".to_string(),
            "Conectado".to_string(),
            "Vinculado".to_string(),
            "No vinculado".to_string(),
            "Desconectar".to_string(),
            "Buscando...".to_string(),
        )
    };

    let mut dev_list = div()
        .id("bt-internal-scroll")
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1p5();

    if status.bluetooth_device_list.is_empty() {
        let empty_label = if status.is_scanning_bluetooth {
            scanning_str.clone()
        } else {
            no_bt_found
        };

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
                        .child(empty_label),
                ),
        );
    } else {
        for (idx, dev) in status.bluetooth_device_list.iter().enumerate() {
            let mac = dev.mac.clone();
            let name = if dev.name.is_empty() {
                dev.mac.clone()
            } else {
                dev.name.clone()
            };
            let is_conn = dev.is_connected;
            let is_paired = dev.is_paired;
            let is_connecting = status.connecting_bluetooth_mac.as_deref() == Some(&mac);

            let mac_click = mac.clone();
            let mac_disconnect = mac.clone();

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
                            if is_conn {
                                cx.global::<AppState>()
                                    .network
                                    .disconnect_bluetooth(&mac_disconnect);
                            } else {
                                cx.global::<AppState>()
                                    .network
                                    .connect_bluetooth(&mac_click);
                            }
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
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_1p5()
                            .child(if is_connecting {
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.accent())
                                    .child(connecting_str.clone())
                            } else if is_conn {
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.accent().opacity(0.85))
                                    .child(connected_str.clone())
                            } else if is_paired {
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(theme.foreground_muted())
                                    .child(paired_str.clone())
                            } else {
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::NORMAL)
                                    .text_color(theme.foreground_muted().opacity(0.7))
                                    .child(unpaired_str.clone())
                            })
                            .when(is_conn, |s| {
                                s.child(
                                    div()
                                        .id(("bt-disconnect-btn", idx as u32))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded(px(6.0))
                                        .bg(theme.surface().opacity(0.7))
                                        .hover(|h| h.bg(theme.surface().opacity(0.95)))
                                        .cursor_pointer()
                                        .text_size(px(8.5))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child(disconnect_str.clone()),
                                )
                            }),
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
                                .child(bt_devices),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .id("scan-bt-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(20.0))
                                .h(px(20.0))
                                .rounded_full()
                                .bg(if status.is_scanning_bluetooth {
                                    theme.accent().opacity(0.25)
                                } else {
                                    theme.surface().opacity(0.6)
                                })
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, _, cx| {
                                    if cx.has_global::<AppState>() {
                                        cx.global::<AppState>().network.start_bluetooth_scan();
                                    }
                                    cx.notify();
                                }))
                                .child(svg().path("rotate-ccw.svg").size(px(11.0)).text_color(
                                    if status.is_scanning_bluetooth {
                                        theme.accent()
                                    } else {
                                        theme.foreground_muted()
                                    },
                                )),
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
                ),
        )
        .child(dev_list)
        .into_any_element()
}
