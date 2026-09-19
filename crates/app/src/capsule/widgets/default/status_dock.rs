use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{BatteryStatus, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::default::{DefaultEvent, DefaultModule};

pub fn render_status_dock(
    network_status: &NetworkStatus,
    battery: Option<BatteryStatus>,
    is_dnd: bool,
    notif_count: usize,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> AnyElement {
    let mut row = div()
        .id("default-status-dock-row")
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0));

    let net_icon = if network_status.ethernet_connected {
        svg()
            .path("ethernet.svg")
            .size(px(13.0))
            .text_color(theme.foreground())
    } else if network_status.wifi_enabled && !network_status.wifi_ssid.is_empty() {
        let path = if network_status.wifi_signal > 60 {
            "wifi-high.svg"
        } else if network_status.wifi_signal > 20 {
            "wifi-low.svg"
        } else {
            "wifi.svg"
        };
        svg()
            .path(path)
            .size(px(13.0))
            .text_color(theme.foreground())
    } else {
        svg()
            .path("wifi-zero.svg")
            .size(px(13.0))
            .text_color(theme.foreground_muted().opacity(0.6))
    };
    row = row.child(net_icon);

    let bt_in_use = !network_status.bluetooth_device_name.is_empty()
        || network_status
            .bluetooth_device_list
            .iter()
            .any(|d| d.is_connected);

    if bt_in_use {
        let bt_icon = svg()
            .path("bluetooth.svg")
            .size(px(13.0))
            .text_color(theme.foreground());
        row = row.child(bt_icon);
    }

    if let Some(bat) = battery {
        let (bat_icon_name, bat_color) = if bat.is_charging {
            ("battery-charging.svg", theme.accent())
        } else if bat.percentage <= 15 {
            ("battery-low.svg", theme.red())
        } else if bat.percentage <= 40 {
            ("battery-medium.svg", theme.foreground())
        } else {
            ("battery-full.svg", theme.foreground())
        };

        row = row.child(
            svg()
                .path(bat_icon_name)
                .size(px(13.0))
                .text_color(bat_color),
        );
    }

    if is_dnd {
        let dnd_icon = svg()
            .path("minus-circle.svg")
            .size(px(13.0))
            .text_color(theme.accent());
        row = row.child(dnd_icon);
    }

    let bell_icon = svg()
        .path("bell.svg")
        .size(px(13.0))
        .text_color(theme.foreground());

    let notif_elem = if notif_count > 0 {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(3.0))
            .child(bell_icon)
            .child(
                div()
                    .font_family(theme.font_family())
                    .text_size(px(10.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.accent())
                    .child(notif_count.to_string()),
            )
            .into_any_element()
    } else {
        bell_icon.into_any_element()
    };
    row = row.child(notif_elem);

    let hover_bg = theme.surface().opacity(0.6);
    let hover_border = theme.surface().opacity(0.4);

    div()
        .id("default-status-dock")
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .h(px(22.0))
        .px(px(6.0))
        .rounded(px(7.0))
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.2))
        .cursor_pointer()
        .hover(move |style| style.bg(hover_bg).border_color(hover_border))
        .on_click(cx.listener(|_, _, _, cx| {
            cx.emit(DefaultEvent::StatusDockClicked);
        }))
        .child(row)
        .into_any_element()
}
