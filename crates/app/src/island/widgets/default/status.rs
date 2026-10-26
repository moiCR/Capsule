use gpui::{IntoElement, div, prelude::*, px, svg};
use services::{BatteryStatus, NetworkStatus};
use ui::theme::Theme;

use super::STATUS_WIDTH;

#[derive(Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    network_icon: &'static str,
    connected: bool,
    battery: Option<BatteryStatus>,
}

impl StatusSnapshot {
    pub fn new(network: &NetworkStatus, battery: Option<BatteryStatus>) -> Self {
        let connected =
            network.ethernet_connected || (network.wifi_enabled && !network.wifi_ssid.is_empty());
        let network_icon = if network.ethernet_connected {
            "ethernet.svg"
        } else if !connected {
            "wifi-zero.svg"
        } else if network.wifi_signal > 60 {
            "wifi-high.svg"
        } else if network.wifi_signal > 20 {
            "wifi-low.svg"
        } else {
            "wifi.svg"
        };
        Self {
            network_icon,
            connected,
            battery,
        }
    }
}

pub fn render_status(status: &StatusSnapshot, theme: &Theme) -> impl IntoElement {
    let battery = status.battery;
    let battery_icon = match battery {
        Some(value) if value.is_charging => "battery-charging.svg",
        Some(value) if value.percentage <= 15 => "battery-low.svg",
        Some(value) if value.percentage <= 40 => "battery-medium.svg",
        _ => "battery-full.svg",
    };
    let battery_color = match battery {
        Some(value) if value.is_charging => theme.accent(),
        Some(value) if value.percentage <= 15 => theme.red(),
        _ => theme.foreground(),
    };
    div()
        .w(px(STATUS_WIDTH))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_end()
        .gap(px(6.0))
        .child(
            svg()
                .path(status.network_icon)
                .size(px(15.0))
                .text_color(if status.connected {
                    theme.foreground()
                } else {
                    theme.foreground_muted()
                }),
        )
        .when(battery.is_some(), |row| {
            row.child(
                svg()
                    .path(battery_icon)
                    .size(px(15.0))
                    .text_color(battery_color),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_snapshot_tracks_only_visible_status() {
        let mut network = NetworkStatus::default();
        let disconnected = StatusSnapshot::new(&network, None);
        assert_eq!(disconnected.network_icon, "wifi-zero.svg");
        network.bluetooth_enabled = true;
        assert!(disconnected == StatusSnapshot::new(&network, None));
        network.wifi_enabled = true;
        network.wifi_ssid = "test".to_string();
        network.wifi_signal = 70;
        let connected = StatusSnapshot::new(&network, None);
        assert_eq!(connected.network_icon, "wifi-high.svg");
        assert!(connected.connected);
        network.ethernet_connected = true;
        assert_eq!(
            StatusSnapshot::new(&network, None).network_icon,
            "ethernet.svg"
        );
    }

    #[test]
    fn battery_snapshot_preserves_absent_charging_and_low_states() {
        for battery in [
            None,
            Some(BatteryStatus {
                percentage: 10,
                is_charging: false,
            }),
            Some(BatteryStatus {
                percentage: 80,
                is_charging: true,
            }),
        ] {
            assert_eq!(
                StatusSnapshot::new(&NetworkStatus::default(), battery).battery,
                battery
            );
        }
    }
}
