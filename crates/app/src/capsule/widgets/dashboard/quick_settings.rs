use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn render_quick_settings_widget(
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let wifi_card = if status.ethernet_connected {
        // Ethernet connected special case: no toggle, no chevron, no mini panel
        render_ethernet_card(&status, theme)
    } else {
        render_wifi_card(&status, theme, cx)
    };

    let bt_card = render_bluetooth_card(&status, theme, cx);

    div()
        .id("quick-settings-row")
        .flex()
        .flex_row()
        .w_full()
        .gap_2p5()
        .child(wifi_card)
        .child(bt_card)
}

fn render_ethernet_card(status: &NetworkStatus, theme: &Theme) -> AnyElement {
    div()
        .id("ethernet-card-main")
        .flex_1()
        .h(px(72.0))
        .rounded(px(18.0))
        .bg(theme.surface().opacity(0.4))
        .border_1()
        .border_color(theme.surface().opacity(0.3))
        .p_3()
        .flex()
        .flex_col()
        .justify_between()
        .child(
            // Top row
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    svg()
                        .path("ethernet.svg")
                        .size(px(16.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            // Bottom row
            div()
                .flex()
                .flex_col()
                .w_full()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground())
                        .child("Ethernet"),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .child(if status.ethernet_name.is_empty() {
                            "Conectado".to_string()
                        } else {
                            status.ethernet_name.clone()
                        }),
                ),
        )
        .into_any_element()
}

fn render_wifi_card(
    status: &NetworkStatus,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let is_on = status.wifi_enabled;
    let is_connected = !status.wifi_ssid.is_empty();

    let icon_path = if !is_on {
        "wifi-zero.svg"
    } else if status.wifi_signal > 60 {
        "wifi-high.svg"
    } else if status.wifi_signal > 20 {
        "wifi-low.svg"
    } else {
        "wifi.svg"
    };

    let subtitle = if !is_on {
        "Desactivado".to_string()
    } else if is_connected {
        status.wifi_ssid.clone()
    } else {
        "Desconectado".to_string()
    };

    div()
        .id("wifi-card-main")
        .flex_1()
        .h(px(72.0))
        .rounded(px(18.0))
        .bg(theme.surface().opacity(0.4))
        .border_1()
        .border_color(theme.surface().opacity(0.3))
        .p_3()
        .flex()
        .flex_col()
        .justify_between()
        .child(
            // Top row: Icon + Toggle Switch
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(svg().path(icon_path).size(px(16.0)).text_color(if is_on {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }))
                .child(
                    // Toggle Switch
                    div()
                        .id("wifi-toggle-switch")
                        .flex()
                        .items_center()
                        .w(px(34.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(if is_on {
                            theme.accent()
                        } else {
                            theme.surface()
                        })
                        .p_0p5()
                        .cursor_pointer()
                        .on_click(cx.listener(|_, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                cx.global::<AppState>().network.toggle_wifi();
                                cx.notify();
                            }
                        }))
                        .child(
                            div()
                                .w(px(14.0))
                                .h(px(14.0))
                                .rounded_full()
                                .bg(gpui::white())
                                .ml(if is_on { px(15.0) } else { px(0.0) }),
                        ),
                ),
        )
        .child(
            // Bottom row: Title/Subtitle + Chevron Arrow
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Wi-Fi"),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.foreground_muted())
                                .child(subtitle),
                        ),
                )
                .child(
                    // Chevron arrow button
                    div()
                        .id("wifi-chevron-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface()))
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.emit(DashboardEvent::WifiChevronClicked);
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("chevron-right.svg")
                                .size(px(12.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .into_any_element()
}

fn render_bluetooth_card(
    status: &NetworkStatus,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let is_on = status.bluetooth_enabled;
    let is_connected = !status.bluetooth_device_name.is_empty();

    let subtitle = if !is_on {
        "Desactivado".to_string()
    } else if is_connected {
        status.bluetooth_device_name.clone()
    } else {
        "Desconectado".to_string()
    };

    div()
        .id("bt-card-main")
        .flex_1()
        .h(px(72.0))
        .rounded(px(18.0))
        .bg(theme.surface().opacity(0.4))
        .border_1()
        .border_color(theme.surface().opacity(0.3))
        .p_3()
        .flex()
        .flex_col()
        .justify_between()
        .child(
            // Top row: Icon + Toggle Switch
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    svg()
                        .path("bluetooth.svg")
                        .size(px(16.0))
                        .text_color(if is_on {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        }),
                )
                .child(
                    // Toggle Switch
                    div()
                        .id("bt-toggle-switch")
                        .flex()
                        .items_center()
                        .w(px(34.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(if is_on {
                            theme.accent()
                        } else {
                            theme.surface()
                        })
                        .p_0p5()
                        .cursor_pointer()
                        .on_click(cx.listener(|_, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                cx.global::<AppState>().network.toggle_bluetooth();
                                cx.notify();
                            }
                        }))
                        .child(
                            div()
                                .w(px(14.0))
                                .h(px(14.0))
                                .rounded_full()
                                .bg(gpui::white())
                                .ml(if is_on { px(15.0) } else { px(0.0) }),
                        ),
                ),
        )
        .child(
            // Bottom row: Title/Subtitle + Chevron Arrow
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Bluetooth"),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.foreground_muted())
                                .child(subtitle),
                        ),
                )
                .child(
                    // Chevron arrow button
                    div()
                        .id("bt-chevron-btn")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface()))
                        .on_click(cx.listener(|_, _, _, cx| {
                            cx.emit(DashboardEvent::BluetoothChevronClicked);
                            cx.notify();
                        }))
                        .child(
                            svg()
                                .path("chevron-right.svg")
                                .size(px(12.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .into_any_element()
}

/// Compute Wi-Fi satellite panel height.
pub fn compute_wifi_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 75.0;
    let items_h = if status.wifi_ap_list.is_empty() {
        30.0
    } else {
        status.wifi_ap_list.len() as f32 * 32.0
    };
    (base_h + items_h).clamp(140.0, 380.0)
}

/// Render Wi-Fi satellite panel orbiting the Dashboard.
pub fn render_wifi_mini_panel(
    anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    use super::panel_manager::PANEL_W;

    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let opacity = anim_t;

    let mut ap_list = div()
        .id("wifi-sat-list")
        .flex()
        .flex_col()
        .w_full()
        .flex_1()
        .overflow_scroll()
        .gap_1();

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
                        .child("No hay redes Wi-Fi encontradas"),
                ),
        );
    } else {
        for (idx, ap) in status.wifi_ap_list.iter().enumerate() {
            let ssid = ap.ssid.clone();
            let is_conn = ap.is_connected;
            let icon_p = if ap.signal > 60 {
                "wifi-high.svg"
            } else if ap.signal > 20 {
                "wifi-low.svg"
            } else {
                "wifi-zero.svg"
            };

            let ssid_click = ssid.clone();

            ap_list = ap_list.child(
                div()
                    .id(("wifi-ap-item", idx as u32))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .py_1p5()
                    .rounded(px(8.0))
                    .bg(if is_conn {
                        theme.accent().opacity(0.15)
                    } else {
                        theme.surface().opacity(0.3)
                    })
                    .hover(|s| s.bg(theme.surface()))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>().network.connect_wifi(&ssid_click);
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(svg().path(icon_p).size(px(14.0)).text_color(if is_conn {
                                theme.accent()
                            } else {
                                theme.foreground()
                            }))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(if is_conn {
                                        FontWeight::BOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(if is_conn {
                                        theme.accent()
                                    } else {
                                        theme.foreground()
                                    })
                                    .child(ssid),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(9.0))
                            .text_color(theme.foreground_muted())
                            .child(if is_conn { "Conectado" } else { "" }),
                    ),
            );
        }
    }

    div()
        .w(px(PANEL_W))
        .h(px(panel_h))
        .p_2p5()
        .gap_1p5()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(opacity)
        .flex()
        .flex_col()
        .child(
            // Header
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
                                .text_size(px(11.0))
                                .text_color(theme.foreground())
                                .child("Redes Wi-Fi"),
                        ),
                )
                .child(
                    div()
                        .id("close-wifi-panel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(theme.surface())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface()))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(DashboardEvent::WifiChevronClicked);
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(10.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
        .child(ap_list)
        .into_any_element()
}

/// Compute Bluetooth satellite panel height.
pub fn compute_bluetooth_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 75.0;
    let items_h = if status.bluetooth_device_list.is_empty() {
        30.0
    } else {
        status.bluetooth_device_list.len() as f32 * 32.0
    };
    (base_h + items_h).clamp(140.0, 380.0)
}

/// Render Bluetooth satellite panel orbiting the Dashboard.
pub fn render_bluetooth_mini_panel(
    anim_t: f32,
    panel_h: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    use super::panel_manager::PANEL_W;

    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let opacity = anim_t;

    let mut dev_list = div()
        .id("bt-sat-list")
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
                        .child("No hay dispositivos Bluetooth"),
                ),
        );
    } else {
        for (idx, dev) in status.bluetooth_device_list.iter().enumerate() {
            let mac = dev.mac.clone();
            let name = dev.name.clone();
            let is_conn = dev.is_connected;
            let mac_click = mac.clone();

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
                    .rounded(px(8.0))
                    .bg(if is_conn {
                        theme.accent().opacity(0.15)
                    } else {
                        theme.surface().opacity(0.3)
                    })
                    .hover(|s| s.bg(theme.surface()))
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
                            .child(svg().path("bluetooth.svg").size(px(14.0)).text_color(
                                if is_conn {
                                    theme.accent()
                                } else {
                                    theme.foreground()
                                },
                            ))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(if is_conn {
                                        FontWeight::BOLD
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
                            .text_color(theme.foreground_muted())
                            .child(if is_conn { "Conectado" } else { "" }),
                    ),
            );
        }
    }

    div()
        .w(px(PANEL_W))
        .h(px(panel_h))
        .p_2p5()
        .gap_1p5()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(opacity)
        .flex()
        .flex_col()
        .child(
            // Header
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
                                .text_size(px(11.0))
                                .text_color(theme.foreground())
                                .child("Dispositivos Bluetooth"),
                        ),
                )
                .child(
                    div()
                        .id("close-bt-panel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(18.0))
                        .h(px(18.0))
                        .rounded_full()
                        .bg(theme.surface())
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.surface()))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(DashboardEvent::BluetoothChevronClicked);
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(10.0))
                                .text_color(theme.foreground_muted()),
                        ),
                ),
        )
        .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
        .child(dev_list)
        .into_any_element()
}
