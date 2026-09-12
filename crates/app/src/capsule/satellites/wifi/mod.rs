use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_MIN_W;

pub fn compute_wifi_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 80.0;
    let items_h = if status.wifi_ap_list.is_empty() {
        32.0
    } else {
        status.wifi_ap_list.len() as f32 * 36.0
    };
    let extra_h = if status.connecting_wifi_ssid.is_some() || status.wifi_error.is_some() {
        80.0
    } else {
        55.0
    };
    (base_h + items_h + extra_h).clamp(150.0, 420.0)
}

pub fn render_wifi_mini_panel(
    _anim_t: f32,
    panel_h: f32,
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let selected_ssid = module.wifi_selected_ssid.clone();
    let current_password = module.wifi_password_input.clone();

    let (
        no_wifi_found,
        wifi_networks,
        connect_str,
        disconnect_str,
        password_placeholder,
        connecting_str,
        connected_str,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("quick_settings.no_wifi_found"),
            lang.get("quick_settings.wifi_networks"),
            lang.get("quick_settings.connect"),
            lang.get("quick_settings.disconnect"),
            lang.get("quick_settings.password_placeholder"),
            lang.get("quick_settings.connecting"),
            lang.get("dashboard.connected"),
        )
    } else {
        (
            "No hay redes Wi-Fi encontradas".to_string(),
            "Redes Wi-Fi".to_string(),
            "Conectar".to_string(),
            "Desconectar".to_string(),
            "Contraseña...".to_string(),
            "Conectando...".to_string(),
            "Conectado".to_string(),
        )
    };

    let mut ap_list = div()
        .id("wifi-internal-scroll")
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1p5();

    if status.wifi_ap_list.is_empty() {
        ap_list = ap_list.child(
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
                        .child(no_wifi_found),
                ),
        );
    } else {
        for (idx, ap) in status.wifi_ap_list.iter().enumerate() {
            let ssid = ap.ssid.clone();
            let is_conn = ap.is_connected;
            let is_saved = ap.is_saved;
            let is_secured = !ap.security.is_empty();
            let is_selected = selected_ssid.as_deref() == Some(&ssid);
            let is_connecting = status.connecting_wifi_ssid.as_deref() == Some(&ssid);

            let icon_p = if ap.signal > 60 {
                "wifi-high.svg"
            } else if ap.signal > 20 {
                "wifi-low.svg"
            } else {
                "wifi-zero.svg"
            };

            let ssid_click = ssid.clone();
            let ssid_disconnect = ssid.clone();

            let indicator_color = if is_conn {
                theme.accent()
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };

            let mut item_container = div()
                .id(("wifi-ap-item-wrapper", idx as u32))
                .flex()
                .flex_col()
                .w_full()
                .gap_1();

            let row = div()
                .id(("wifi-ap-item", idx as u32))
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
                } else if is_selected {
                    theme.surface().opacity(0.35)
                } else {
                    gpui::hsla(0.0, 0.0, 0.0, 0.0)
                })
                .hover(|s| s.bg(theme.surface().opacity(0.4)))
                .active(|s| s.bg(theme.surface().opacity(0.6)))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    if is_conn {
                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>()
                                .network
                                .disconnect_wifi(&ssid_disconnect);
                        }
                    } else if is_saved || !is_secured {
                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>()
                                .network
                                .connect_wifi(&ssid_click, None);
                        }
                    } else {
                        this.select_wifi_for_password(ssid_click.clone(), cx);
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
                        .child(svg().path(icon_p).size(px(14.0)).text_color(if is_conn {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        }))
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
                                .child(ssid.clone()),
                        )
                        .when(is_secured, |s| {
                            s.child(
                                svg()
                                    .path("lock.svg")
                                    .size(px(11.0))
                                    .text_color(theme.foreground_muted().opacity(0.7)),
                            )
                        }),
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
                        } else {
                            div()
                        })
                        .when(is_conn, |s| {
                            s.child(
                                div()
                                    .id(("wifi-disconnect-btn", idx as u32))
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
                );

            item_container = item_container.child(row);

            if is_selected && !is_conn {
                let display_pwd = if current_password.is_empty() {
                    password_placeholder.clone()
                } else {
                    "•".repeat(current_password.len())
                };

                let prompt_box = div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded(px(8.0))
                    .bg(theme.surface().opacity(0.4))
                    .border_1()
                    .border_color(theme.accent().opacity(0.4))
                    .gap_1p5()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .w_full()
                            .px_2()
                            .py_1()
                            .rounded(px(6.0))
                            .bg(theme.background().opacity(0.8))
                            .gap_1p5()
                            .child(
                                svg()
                                    .path("lock.svg")
                                    .size(px(12.0))
                                    .text_color(theme.foreground_muted()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(10.5))
                                    .text_color(if current_password.is_empty() {
                                        theme.foreground_muted()
                                    } else {
                                        theme.foreground()
                                    })
                                    .child(display_pwd),
                            ),
                    )
                    .when_some(status.wifi_error.clone(), |s, err| {
                        s.child(
                            div()
                                .text_size(px(8.5))
                                .text_color(gpui::rgb(0xef4444))
                                .child(err),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_end()
                            .gap_1p5()
                            .child(
                                div()
                                    .id("wifi-pwd-cancel")
                                    .px_2()
                                    .py_1()
                                    .rounded(px(6.0))
                                    .bg(theme.surface().opacity(0.6))
                                    .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                    .cursor_pointer()
                                    .text_size(px(9.5))
                                    .text_color(theme.foreground_muted())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.cancel_wifi_password(cx);
                                    }))
                                    .child("Cancelar"),
                            )
                            .child(
                                div()
                                    .id("wifi-pwd-submit")
                                    .px_2()
                                    .py_1()
                                    .rounded(px(6.0))
                                    .bg(theme.accent())
                                    .hover(|s| s.opacity(0.9))
                                    .cursor_pointer()
                                    .text_size(px(9.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.background())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.submit_wifi_password(cx);
                                    }))
                                    .child(connect_str.clone()),
                            ),
                    );

                item_container = item_container.child(prompt_box);
            }

            ap_list = ap_list.child(item_container);
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
                                .path("wifi-high.svg")
                                .size(px(14.0))
                                .text_color(theme.accent()),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.5))
                                .text_color(theme.foreground())
                                .child(wifi_networks),
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
                                .id("rescan-wifi-btn")
                                .flex()
                                .items_center()
                                .justify_center()
                                .w(px(20.0))
                                .h(px(20.0))
                                .rounded_full()
                                .bg(if status.is_scanning_wifi {
                                    theme.accent().opacity(0.25)
                                } else {
                                    theme.surface().opacity(0.6)
                                })
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .cursor_pointer()
                                .on_click(cx.listener(|_, _, _, cx| {
                                    if cx.has_global::<AppState>() {
                                        cx.global::<AppState>().network.rescan_wifi();
                                    }
                                    cx.notify();
                                }))
                                .child(svg().path("rotate-ccw.svg").size(px(11.0)).text_color(
                                    if status.is_scanning_wifi {
                                        theme.accent()
                                    } else {
                                        theme.foreground_muted()
                                    },
                                )),
                        )
                        .child(
                            div()
                                .id("close-wifi-panel")
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
                                        cx.global::<AppState>().network.toggle_wifi();
                                    }
                                    cx.notify();
                                }))
                                .child(svg().path("power.svg").size(px(10.0)).text_color(
                                    if status.wifi_enabled {
                                        theme.accent()
                                    } else {
                                        theme.foreground_muted()
                                    },
                                )),
                        ),
                ),
        )
        .child(ap_list)
        .into_any_element()
}
