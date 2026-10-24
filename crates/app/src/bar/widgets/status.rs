use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, Pixels, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(
    status_center: &std::rc::Rc<std::cell::Cell<Pixels>>,
    theme: &Theme,
    cx: &mut Context<Bar>,
) -> AnyElement {
    let (network_icon, vol_icon, battery_info, privacy_info) =
        if let Some(state) = cx.try_global::<AppState>() {
            let net = state.network.get_status();
            let net_icon = if net.ethernet_connected {
                "ethernet.svg"
            } else if !net.wifi_enabled || net.wifi_ssid.is_empty() {
                "wifi-zero.svg"
            } else if net.wifi_signal > 60 {
                "wifi-high.svg"
            } else if net.wifi_signal > 20 {
                "wifi-low.svg"
            } else {
                "wifi.svg"
            };

            let sys = state.system.get_status();
            let v_icon = if sys.is_muted || sys.volume == 0 {
                "volume-x.svg"
            } else {
                "volume-2.svg"
            };

            let bat = state.power.get_battery().map(|b| {
                let icon = if b.is_charging {
                    "battery-charging.svg"
                } else if b.percentage <= 15 {
                    "battery-low.svg"
                } else if b.percentage <= 40 {
                    "battery-medium.svg"
                } else {
                    "battery-full.svg"
                };
                (icon, b.percentage, b.is_charging)
            });

            let capture = state.system.get_capture_status();
            let privacy = (capture.camera_in_use, capture.microphone_in_use);

            (net_icon, v_icon, bat, privacy)
        } else {
            ("wifi.svg", "volume-2.svg", None, (false, false))
        };

    let hover_bg = style::hover(theme);
    let anchor = std::rc::Rc::new(std::cell::Cell::new(px(0.0)));
    let anchor_click = anchor.clone();
    let bar_status_center = status_center.clone();
    let anchor_sync = anchor.clone();

    let tracker = gpui::canvas(
        move |bounds, _, _| {
            let cx_pos = bounds.origin.x + bounds.size.width / 2.0;
            anchor_sync.set(cx_pos);
            bar_status_center.set(cx_pos);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full();

    let mut status = div()
        .id("bar-widget-status")
        .relative()
        .child(tracker)
        .flex()
        .items_center()
        .gap(px(8.0))
        .h(px(style::WIDGET_HEIGHT))
        .px(px(10.0))
        .rounded_full()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |bar, event: &gpui::MouseDownEvent, window, cx| {
                let center_x = anchor_click.get();
                let target = if center_x > px(0.0) {
                    center_x
                } else {
                    event.position.x
                };
                bar.toggle_shared_panel(
                    crate::bar::shared_panel::SharedPanelModule::Dashboard,
                    target,
                    window,
                    cx,
                );
            }),
        )
        .child(
            svg()
                .path(network_icon)
                .size(px(15.0))
                .text_color(theme.foreground()),
        )
        .child(
            svg()
                .path(vol_icon)
                .size(px(15.0))
                .text_color(theme.foreground()),
        );

    if privacy_info.0 {
        status = status.child(
            svg()
                .path("camera.svg")
                .size(px(13.5))
                .text_color(theme.green()),
        );
    }

    if privacy_info.1 {
        status = status.child(
            svg()
                .path("mic.svg")
                .size(px(13.5))
                .text_color(theme.yellow()),
        );
    }

    if let Some((bat_icon, pct, is_charging)) = battery_info {
        let bat_color = if is_charging {
            theme.accent()
        } else if pct <= 15 {
            theme.red()
        } else {
            theme.foreground()
        };

        status = status
            .child(
                div()
                    .w(px(1.0))
                    .h(px(12.0))
                    .bg(style::border(theme).opacity(0.45)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.0))
                    .child(svg().path(bat_icon).size(px(15.0)).text_color(bat_color))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(format!("{pct}%")),
                    ),
            );
    }

    status.into_any_element()
}
