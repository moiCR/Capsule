use super::super::DashboardAction as Action;
use super::{button, empty};
use super::{error, list};
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::widgets::style;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use ui::theme::Theme;
pub(super) fn wifi(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let network = &module.snapshot.network;
    let mut content = list("dashboard-wifi-list").child(
        div()
            .flex()
            .gap(px(8.0))
            .child(button(
                "dashboard-wifi-power",
                module.text(
                    if network.wifi_enabled {
                        "dashboard.enabled"
                    } else {
                        "dashboard.disabled"
                    },
                    cx,
                ),
                "wifi.svg",
                Action::Wifi,
                network.wifi_enabled,
                theme,
                cx,
            ))
            .child(button(
                "dashboard-wifi-scan",
                module.text(
                    if network.is_scanning_wifi {
                        "quick_settings.scanning"
                    } else {
                        "quick_settings.scan"
                    },
                    cx,
                ),
                "rotate-ccw.svg",
                Action::ScanWifi,
                false,
                theme,
                cx,
            )),
    );
    if let Some(message) = &network.wifi_error {
        content = content.child(error(message.clone(), theme));
    }
    if network.wifi_ap_list.is_empty() {
        content = content.child(empty(
            module.text("quick_settings.no_wifi_found", cx),
            theme,
        ));
    }
    for ap in &network.wifi_ap_list {
        let pending = network.connecting_wifi_ssid.as_deref() == Some(&ap.ssid);
        let label = format!(
            "{}  ·  {}%{}",
            ap.ssid,
            ap.signal,
            if pending {
                format!("  ·  {}", module.text("quick_settings.connecting", cx))
            } else {
                String::new()
            }
        );
        content = content.child(button(
            format!("dashboard-network-{}", ap.ssid),
            label,
            "wifi.svg",
            Action::SelectWifi(ap.ssid.clone()),
            ap.is_connected,
            theme,
            cx,
        ));
        if module.navigation.selected_ssid.as_deref() == Some(&ap.ssid) {
            content = content.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .id("dashboard-wifi-password")
                            .flex_1()
                            .min_h(px(40.0))
                            .p(px(10.0))
                            .rounded(px(10.0))
                            .bg(style::surface(theme))
                            .cursor_text()
                            .on_click(cx.listener(|this, _, window, cx| {
                                window.focus(&this.focus_handle(), cx)
                            }))
                            .child(if module.navigation.password.is_empty() {
                                module.text("quick_settings.password_placeholder", cx)
                            } else {
                                "•".repeat(module.navigation.password.chars().count())
                            }),
                    )
                    .child(button(
                        "dashboard-wifi-connect",
                        module.text("quick_settings.connect", cx),
                        "chevron-right.svg",
                        Action::ConnectWifi,
                        true,
                        theme,
                        cx,
                    )),
            );
        }
    }
    content.into_any_element()
}
pub(super) fn bluetooth(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let network = &module.snapshot.network;
    let mut content = list("dashboard-bluetooth-list").child(
        div()
            .flex()
            .gap(px(8.0))
            .child(button(
                "dashboard-bluetooth-power",
                module.text(
                    if network.bluetooth_enabled {
                        "dashboard.enabled"
                    } else {
                        "dashboard.disabled"
                    },
                    cx,
                ),
                "bluetooth.svg",
                Action::Bluetooth,
                network.bluetooth_enabled,
                theme,
                cx,
            ))
            .child(button(
                "dashboard-bluetooth-scan",
                module.text(
                    if network.is_scanning_bluetooth {
                        "quick_settings.scanning"
                    } else {
                        "quick_settings.scan"
                    },
                    cx,
                ),
                "rotate-ccw.svg",
                Action::ScanBluetooth,
                false,
                theme,
                cx,
            )),
    );
    if let Some(message) = &network.bluetooth_error {
        content = content.child(error(message.clone(), theme));
    }
    if network.bluetooth_device_list.is_empty() {
        content = content.child(empty(module.text("quick_settings.no_bt_found", cx), theme));
    }
    for device in &network.bluetooth_device_list {
        let status = if network.connecting_bluetooth_mac.as_deref() == Some(&device.mac) {
            "quick_settings.connecting"
        } else if device.is_connected {
            "dashboard.connected"
        } else if device.is_paired {
            "quick_settings.paired"
        } else {
            "quick_settings.unpaired"
        };
        content = content.child(button(
            format!("dashboard-device-{}", device.mac),
            format!("{} · {}", device.name, module.text(status, cx)),
            "bluetooth.svg",
            Action::BluetoothDevice(device.mac.clone()),
            device.is_connected,
            theme,
            cx,
        ));
    }
    content.into_any_element()
}
