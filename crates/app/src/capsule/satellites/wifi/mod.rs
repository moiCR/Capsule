use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;
use crate::capsule::satellites::PANEL_W;

pub fn compute_wifi_panel_height(status: &NetworkStatus) -> f32 {
    let base_h = 75.0;
    let items_h = if status.wifi_ap_list.is_empty() {
        30.0
    } else {
        status.wifi_ap_list.len() as f32 * 32.0
    };
    (base_h + items_h).clamp(140.0, 380.0)
}

pub fn render_wifi_mini_panel(
    anim_t: f32,
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

    let mut ap_list = div()
        .id("wifi-internal-scroll")
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
                        .child(lang.quick_settings.no_wifi_found),
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
        .max_h(px(panel_h))
        .p_2p5()
        .gap_1p5()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .overflow_hidden()
        .opacity(anim_t)
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
                                .text_size(px(11.0))
                                .text_color(theme.foreground())
                                .child(lang.quick_settings.wifi_networks),
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
        )
        .child(ap_list)
        .into_any_element()
}
