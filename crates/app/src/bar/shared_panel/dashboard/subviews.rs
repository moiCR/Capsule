use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, px, svg};
use services::AppState;
use ui::components::Input;
use ui::theme::Theme;

use super::DashboardModule;
use crate::bar::{shared_panel::SharedPanelEvent, style};

pub fn render_wifi(
    state: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let app_state = cx.try_global::<AppState>();
    let net = app_state
        .map(|s| s.network.get_status())
        .unwrap_or_default();

    let (title, scan_label, empty_label, pass_placeholder, connect_label) =
        if let Some(s) = app_state {
            let l = &s.language;
            (
                l.get("quick_settings.wifi_networks"),
                if net.is_scanning_wifi {
                    l.get("quick_settings.scanning")
                } else {
                    l.get("quick_settings.scan")
                },
                l.get("quick_settings.no_wifi_found"),
                l.get("quick_settings.password_placeholder"),
                l.get("quick_settings.connect"),
            )
        } else {
            (
                "Redes Wi-Fi".to_string(),
                "Buscar".to_string(),
                "No hay redes Wi-Fi encontradas".to_string(),
                "Contraseña...".to_string(),
                "Conectar".to_string(),
            )
        };

    let hover_bg = style::hover(theme);
    let header = div()
        .h(px(38.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("bar-dash-wifi-back")
                        .size(px(30.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_subview(cx);
                        }))
                        .child(
                            svg()
                                .path("chevron-left.svg")
                                .size(px(16.0))
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(14.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground())
                        .child(title),
                ),
        )
        .child(
            div()
                .id("bar-dash-wifi-rescan")
                .h(px(28.0))
                .px(px(10.0))
                .rounded(px(8.0))
                .bg(style::surface(theme).opacity(0.60))
                .border_1()
                .border_color(style::border(theme).opacity(0.18))
                .flex()
                .items_center()
                .gap(px(6.0))
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(cx.listener(|_, _, _, cx| {
                    if let Some(s) = cx.try_global::<AppState>() {
                        s.network.rescan_wifi();
                        cx.notify();
                    }
                }))
                .child(
                    svg()
                        .path("rotate-ccw.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground())
                        .child(scan_label),
                ),
        );

    let divider = div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme).opacity(0.16));

    let mut list = div()
        .id("bar-dash-wifi-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(4.0));

    if net.wifi_ap_list.is_empty() {
        list = list.items_center().justify_center().child(
            div()
                .text_size(px(13.5))
                .text_color(theme.foreground_muted())
                .child(empty_label),
        );
    } else {
        for (idx, ap) in net.wifi_ap_list.iter().enumerate() {
            let ssid = ap.ssid.clone();
            let ssid_click = ssid.clone();
            let is_connected = ap.is_connected;
            let is_open = ap.security.is_empty() || ap.security == "--" || ap.is_saved;

            list = list.child(
                div()
                    .id(("bar-dash-wifi-ap", idx))
                    .w_full()
                    .h(px(46.0))
                    .flex_shrink_0()
                    .pl(px(6.0))
                    .pr(px(10.0))
                    .rounded(px(10.0))
                    .bg(if is_connected {
                        style::surface(theme).opacity(0.85)
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if is_connected {
                        style::border(theme).opacity(0.18)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(move |s| {
                        if !is_connected {
                            s.bg(style::surface(theme).opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(s) = cx.try_global::<AppState>() {
                            if is_connected {
                                s.network.disconnect_wifi(&ssid_click);
                            } else if is_open {
                                s.network.connect_wifi(&ssid_click, None);
                            } else {
                                this.selected_ssid = Some(ssid_click.clone());
                                this.password_input.clear();
                            }
                            cx.emit(SharedPanelEvent::SyncWindow);
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w_0()
                            .child(
                                div()
                                    .w(px(3.5))
                                    .h(px(18.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(if is_connected {
                                        theme.accent()
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(svg().path("wifi.svg").size(px(16.0)).text_color(
                                if is_connected {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                },
                            ))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(if is_connected {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::MEDIUM
                                    })
                                    .text_color(theme.foreground())
                                    .line_clamp(1)
                                    .child(ssid.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(if is_connected {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(if is_connected {
                                "Conectado".to_string()
                            } else {
                                format!("{}%", ap.signal)
                            }),
                    ),
            );

            if state.selected_ssid.as_deref() == Some(&ap.ssid) {
                let ssid_conn = ap.ssid.clone();
                list = list.child(
                    div()
                        .w_full()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            Input::from_state("bar-dash-wifi-pass", &state.password_input)
                                .placeholder(pass_placeholder.clone())
                                .masked(true)
                                .focused(true)
                                .h_px(38.0)
                                .flex_1(true)
                                .rounded(px(12.0))
                                .bg(style::surface(theme)),
                        )
                        .child(
                            div()
                                .id("bar-dash-wifi-connect-submit")
                                .h(px(38.0))
                                .px(px(14.0))
                                .rounded_xl()
                                .bg(theme.accent())
                                .text_color(theme.on_accent())
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    let pwd = this.password_input.text();
                                    if let Some(s) = cx.try_global::<AppState>() {
                                        s.network.connect_wifi(&ssid_conn, Some(&pwd));
                                    }
                                    this.password_input.clear();
                                    this.selected_ssid = None;
                                    cx.emit(SharedPanelEvent::SyncWindow);
                                    cx.notify();
                                }))
                                .child(connect_label.clone()),
                        ),
                );
            }
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(divider)
        .child(list)
        .into_any_element()
}

pub fn render_bluetooth(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    let app_state = cx.try_global::<AppState>();
    let net = app_state
        .map(|s| s.network.get_status())
        .unwrap_or_default();

    let (title, scan_label, empty_label) = if let Some(s) = app_state {
        let l = &s.language;
        (
            l.get("quick_settings.bt_devices"),
            if net.is_scanning_bluetooth {
                l.get("quick_settings.scanning")
            } else {
                l.get("quick_settings.scan")
            },
            l.get("quick_settings.no_bt_found"),
        )
    } else {
        (
            "Dispositivos Bluetooth".to_string(),
            "Buscar".to_string(),
            "No hay dispositivos Bluetooth".to_string(),
        )
    };

    let hover_bg = style::hover(theme);
    let header = div()
        .h(px(38.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .id("bar-dash-bt-back")
                        .size(px(30.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.close_subview(cx);
                        }))
                        .child(
                            svg()
                                .path("chevron-left.svg")
                                .size(px(16.0))
                                .text_color(theme.foreground()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(14.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground())
                        .child(title),
                ),
        )
        .child(
            div()
                .id("bar-dash-bt-rescan")
                .h(px(28.0))
                .px(px(10.0))
                .rounded(px(8.0))
                .bg(style::surface(theme).opacity(0.60))
                .border_1()
                .border_color(style::border(theme).opacity(0.18))
                .flex()
                .items_center()
                .gap(px(6.0))
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(cx.listener(|_, _, _, cx| {
                    if let Some(s) = cx.try_global::<AppState>() {
                        s.network.start_bluetooth_scan();
                        cx.notify();
                    }
                }))
                .child(
                    svg()
                        .path("rotate-ccw.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground())
                        .child(scan_label),
                ),
        );

    let divider = div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme).opacity(0.16));

    let mut list = div()
        .id("bar-dash-bt-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(4.0));

    if net.bluetooth_device_list.is_empty() {
        list = list.items_center().justify_center().child(
            div()
                .text_size(px(13.5))
                .text_color(theme.foreground_muted())
                .child(empty_label),
        );
    } else {
        for (idx, dev) in net.bluetooth_device_list.iter().enumerate() {
            let mac = dev.mac.clone();
            let is_connected = dev.is_connected;

            list = list.child(
                div()
                    .id(("bar-dash-bt-dev", idx))
                    .w_full()
                    .h(px(46.0))
                    .flex_shrink_0()
                    .pl(px(6.0))
                    .pr(px(10.0))
                    .rounded(px(10.0))
                    .bg(if is_connected {
                        style::surface(theme).opacity(0.85)
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if is_connected {
                        style::border(theme).opacity(0.18)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(move |s| {
                        if !is_connected {
                            s.bg(style::surface(theme).opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if let Some(s) = cx.try_global::<AppState>() {
                            if is_connected {
                                s.network.disconnect_bluetooth(&mac);
                            } else {
                                s.network.connect_bluetooth(&mac);
                            }
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w_0()
                            .child(
                                div()
                                    .w(px(3.5))
                                    .h(px(18.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(if is_connected {
                                        theme.accent()
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(svg().path("bluetooth.svg").size(px(16.0)).text_color(
                                if is_connected {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                },
                            ))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(if is_connected {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::MEDIUM
                                    })
                                    .text_color(theme.foreground())
                                    .line_clamp(1)
                                    .child(dev.name.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(if is_connected {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(if is_connected { "Conectado" } else { "" }),
                    ),
            );
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(divider)
        .child(list)
        .into_any_element()
}

pub fn render_audio_output(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    let app_state = cx.try_global::<AppState>();
    let sys = app_state.map(|s| s.system.get_status()).unwrap_or_default();

    let (title, empty_label) = if let Some(s) = app_state {
        let l = &s.language;
        (
            l.get("dashboard_new.audio_output"),
            l.get("dashboard_new.no_outputs"),
        )
    } else {
        (
            "Salida de audio".to_string(),
            "No hay salidas de audio disponibles".to_string(),
        )
    };

    let hover_bg = style::hover(theme);
    let header = div()
        .h(px(42.0))
        .w_full()
        .flex_shrink_0()
        .px(px(4.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .id("bar-dash-audio-back")
                .size(px(28.0))
                .rounded(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.close_subview(cx);
                }))
                .child(
                    svg()
                        .path("chevron-left.svg")
                        .size(px(16.0))
                        .text_color(theme.foreground()),
                ),
        )
        .child(
            div()
                .text_size(px(14.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child(title),
        );

    let divider = div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme).opacity(0.16));

    let mut list = div()
        .id("bar-dash-audio-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(4.0));

    if sys.audio_sinks.is_empty() {
        list = list.items_center().justify_center().child(
            div()
                .text_size(px(13.5))
                .text_color(theme.foreground_muted())
                .child(empty_label),
        );
    } else {
        for (idx, sink) in sys.audio_sinks.iter().enumerate() {
            let sink_name = sink.name.clone();
            let is_selected = sink.is_default;
            let desc_lower = sink.description.to_lowercase();
            let icon = if desc_lower.contains("headphone")
                || desc_lower.contains("auricular")
                || desc_lower.contains("headset")
            {
                "headphones.svg"
            } else if desc_lower.contains("hdmi")
                || desc_lower.contains("displayport")
                || desc_lower.contains("monitor")
            {
                "monitor.svg"
            } else {
                "volume-2.svg"
            };

            list = list.child(
                div()
                    .id(("bar-dash-sink", idx))
                    .w_full()
                    .h(px(46.0))
                    .flex_shrink_0()
                    .pl(px(6.0))
                    .pr(px(10.0))
                    .rounded(px(10.0))
                    .bg(if is_selected {
                        style::surface(theme).opacity(0.85)
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if is_selected {
                        style::border(theme).opacity(0.18)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(move |s| {
                        if !is_selected {
                            s.bg(style::surface(theme).opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if let Some(s) = cx.try_global::<AppState>() {
                            let system = s.system.clone();
                            let name = sink_name.clone();
                            services::spawn_tokio(async move {
                                let _ = system.set_default_sink(&name).await;
                            });
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w_0()
                            .flex_1()
                            .child(
                                div()
                                    .w(px(3.5))
                                    .h(px(18.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(if is_selected {
                                        theme.accent()
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(svg().path(icon).size(px(16.0)).text_color(if is_selected {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            }))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.0))
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(if is_selected {
                                                gpui::FontWeight::SEMIBOLD
                                            } else {
                                                gpui::FontWeight::MEDIUM
                                            })
                                            .text_color(theme.foreground())
                                            .line_clamp(1)
                                            .child(sink.description.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.foreground_muted())
                                            .line_clamp(1)
                                            .child(sink.name.clone()),
                                    ),
                            ),
                    ),
            );
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(divider)
        .child(list)
        .into_any_element()
}

pub fn render_audio_input(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    let app_state = cx.try_global::<AppState>();
    let sys = app_state.map(|s| s.system.get_status()).unwrap_or_default();

    let (title, empty_label) = if let Some(s) = app_state {
        let l = &s.language;
        let t = if l.current_target() == "es" {
            "Entrada de audio".to_string()
        } else {
            "Audio Input".to_string()
        };
        let e = if l.current_target() == "es" {
            "No hay micrófonos disponibles".to_string()
        } else {
            "No microphones available".to_string()
        };
        (t, e)
    } else {
        (
            "Entrada de audio".to_string(),
            "No hay micrófonos disponibles".to_string(),
        )
    };

    let hover_bg = style::hover(theme);
    let header = div()
        .h(px(42.0))
        .w_full()
        .flex_shrink_0()
        .px(px(4.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .id("bar-dash-audio-in-back")
                .size(px(28.0))
                .rounded(px(8.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.close_subview(cx);
                }))
                .child(
                    svg()
                        .path("chevron-left.svg")
                        .size(px(16.0))
                        .text_color(theme.foreground()),
                ),
        )
        .child(
            div()
                .text_size(px(14.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child(title),
        );

    let divider = div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme).opacity(0.16));

    let mut list = div()
        .id("bar-dash-audio-in-list")
        .flex_1()
        .min_h_0()
        .w_full()
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(4.0));

    if sys.audio_sources.is_empty() {
        list = list.items_center().justify_center().child(
            div()
                .text_size(px(13.5))
                .text_color(theme.foreground_muted())
                .child(empty_label),
        );
    } else {
        for (idx, source) in sys.audio_sources.iter().enumerate() {
            let source_name = source.name.clone();
            let is_selected = source.is_default;

            list = list.child(
                div()
                    .id(("bar-dash-source", idx))
                    .w_full()
                    .h(px(46.0))
                    .flex_shrink_0()
                    .pl(px(6.0))
                    .pr(px(10.0))
                    .rounded(px(10.0))
                    .bg(if is_selected {
                        style::surface(theme).opacity(0.85)
                    } else {
                        gpui::transparent_black()
                    })
                    .border_1()
                    .border_color(if is_selected {
                        style::border(theme).opacity(0.18)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(move |s| {
                        if !is_selected {
                            s.bg(style::surface(theme).opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .flex()
                    .items_center()
                    .justify_between()
                    .cursor_pointer()
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if let Some(s) = cx.try_global::<AppState>() {
                            let system = s.system.clone();
                            let name = source_name.clone();
                            services::spawn_tokio(async move {
                                let _ = system.set_default_source(&name).await;
                            });
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w_0()
                            .flex_1()
                            .child(
                                div()
                                    .w(px(3.5))
                                    .h(px(18.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(if is_selected {
                                        theme.accent()
                                    } else {
                                        gpui::transparent_black()
                                    }),
                            )
                            .child(svg().path("mic.svg").size(px(16.0)).text_color(
                                if is_selected {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.0))
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(if is_selected {
                                                gpui::FontWeight::SEMIBOLD
                                            } else {
                                                gpui::FontWeight::MEDIUM
                                            })
                                            .text_color(theme.foreground())
                                            .line_clamp(1)
                                            .child(source.description.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.foreground_muted())
                                            .line_clamp(1)
                                            .child(source.name.clone()),
                                    ),
                            ),
                    ),
            );
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(header)
        .child(divider)
        .child(list)
        .into_any_element()
}
