use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, NetworkStatus, NotificationStore};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn render_quick_settings_section(
    dashboard_w: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let col_w = ((dashboard_w - 32.0 - 10.0) / 2.0).floor();

    let wifi_pill = render_wifi_pill(&status, theme, cx);
    let bt_pill = render_bluetooth_pill(&status, theme, cx);
    let peace_pill = render_peace_pill(theme, cx);
    let action_circles = render_action_circles(theme, cx);

    div()
        .id("quick-settings-section")
        .flex()
        .flex_col()
        .w_full()
        .gap_2p5()
        .child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .gap_2p5()
                .child(div().w(px(col_w)).child(wifi_pill))
                .child(div().w(px(col_w)).child(bt_pill)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .gap_2p5()
                .child(div().w(px(col_w)).child(peace_pill))
                .child(div().w(px(col_w)).child(action_circles)),
        )
}

fn render_wifi_pill(
    status: &NetworkStatus,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let is_on = status.wifi_enabled || status.ethernet_connected;
    let is_connected = !status.wifi_ssid.is_empty() || status.ethernet_connected;

    let icon_path = if status.ethernet_connected {
        "ethernet.svg"
    } else if !is_on || !is_connected {
        "wifi-zero.svg"
    } else if status.wifi_signal > 60 {
        "wifi-high.svg"
    } else if status.wifi_signal > 20 {
        "wifi-low.svg"
    } else {
        "wifi.svg"
    };

    let (title, connected_str, disabled_str, disconnected_str) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        let t = if status.ethernet_connected {
            lang.get("quick_settings.ethernet")
        } else {
            "Wi-Fi".to_string()
        };
        (
            t,
            lang.get("dashboard.connected"),
            lang.get("dashboard.disabled"),
            lang.get("dashboard.disconnected"),
        )
    } else {
        (
            if status.ethernet_connected {
                "Ethernet".to_string()
            } else {
                "Wi-Fi".to_string()
            },
            "Conectado".to_string(),
            "Desactivado".to_string(),
            "Desconectado".to_string(),
        )
    };

    let subtitle = if status.ethernet_connected {
        connected_str
    } else if !is_on {
        disabled_str
    } else if is_connected {
        status.wifi_ssid.clone()
    } else {
        disconnected_str
    };

    div()
        .id("wifi-pill-main")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_2()
        .rounded_full()
        .bg(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.35)
        })
        .border_1()
        .border_color(if is_on {
            theme.accent().opacity(0.4)
        } else {
            theme.surface().opacity(0.18)
        })
        .hover(|s| {
            if is_on {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.5))
            }
        })
        .child(
            div()
                .id("wifi-icon-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .bg(if is_on {
                    theme.background().opacity(0.18)
                } else {
                    theme.surface().opacity(0.45)
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().network.toggle_wifi();
                        cx.notify();
                    }
                }))
                .child(svg().path(icon_path).size(px(16.0)).text_color(if is_on {
                    theme.background()
                } else {
                    theme.foreground_muted()
                })),
        )
        .child(
            div()
                .id("wifi-pill-text")
                .flex()
                .flex_col()
                .flex_1()
                .pl_2()
                .overflow_hidden()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WifiChevronClicked);
                }))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_on {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(if is_on {
                            theme.background().opacity(0.8)
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .id("wifi-chevron-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded_full()
                .hover(|s| {
                    if is_on {
                        s.bg(theme.background().opacity(0.15))
                    } else {
                        s.bg(theme.surface().opacity(0.5))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WifiChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(12.0))
                        .text_color(if is_on {
                            theme.background().opacity(0.85)
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .into_any_element()
}

fn render_bluetooth_pill(
    status: &NetworkStatus,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let is_on = status.bluetooth_enabled;
    let is_connected = !status.bluetooth_device_name.is_empty();

    let (_connected_str, disabled_str, disconnected_str) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("dashboard.connected"),
            lang.get("dashboard.disabled"),
            lang.get("dashboard.disconnected"),
        )
    } else {
        (
            "Conectado".to_string(),
            "Desactivado".to_string(),
            "Desconectado".to_string(),
        )
    };

    let subtitle = if !is_on {
        disabled_str
    } else if is_connected {
        status.bluetooth_device_name.clone()
    } else {
        disconnected_str
    };

    div()
        .id("bluetooth-pill-main")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_2()
        .rounded_full()
        .bg(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.35)
        })
        .border_1()
        .border_color(if is_on {
            theme.accent().opacity(0.4)
        } else {
            theme.surface().opacity(0.18)
        })
        .hover(|s| {
            if is_on {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.5))
            }
        })
        .child(
            div()
                .id("bluetooth-icon-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .bg(if is_on {
                    theme.background().opacity(0.18)
                } else {
                    theme.surface().opacity(0.45)
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().network.toggle_bluetooth();
                        cx.notify();
                    }
                }))
                .child(
                    svg()
                        .path("bluetooth.svg")
                        .size(px(16.0))
                        .text_color(if is_on {
                            theme.background()
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .child(
            div()
                .id("bluetooth-pill-text")
                .flex()
                .flex_col()
                .flex_1()
                .pl_2()
                .overflow_hidden()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::BluetoothChevronClicked);
                }))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_on {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child("Bluetooth"),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(if is_on {
                            theme.background().opacity(0.8)
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .id("bluetooth-chevron-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(24.0))
                .rounded_full()
                .hover(|s| {
                    if is_on {
                        s.bg(theme.background().opacity(0.15))
                    } else {
                        s.bg(theme.surface().opacity(0.5))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::BluetoothChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(12.0))
                        .text_color(if is_on {
                            theme.background().opacity(0.85)
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .into_any_element()
}

fn render_peace_pill(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    let is_dnd = NotificationStore::global().is_dnd_enabled();

    let (dnd_title, activated_str, deactivated_str) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("dashboard.dnd"),
            lang.get("dashboard.enabled"),
            lang.get("dashboard.disabled"),
        )
    } else {
        (
            "No molestar".to_string(),
            "Activado".to_string(),
            "Desactivado".to_string(),
        )
    };

    let subtitle = if is_dnd {
        activated_str
    } else {
        deactivated_str
    };

    div()
        .id("peace-pill-main")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_2()
        .rounded_full()
        .bg(if is_dnd {
            theme.accent()
        } else {
            theme.surface().opacity(0.35)
        })
        .border_1()
        .border_color(if is_dnd {
            theme.accent().opacity(0.4)
        } else {
            theme.surface().opacity(0.18)
        })
        .hover(|s| {
            if is_dnd {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.5))
            }
        })
        .cursor_pointer()
        .on_click(cx.listener(|_, _, _, cx| {
            NotificationStore::global().toggle_dnd();
            cx.notify();
        }))
        .child(
            div()
                .id("peace-icon-btn")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .bg(if is_dnd {
                    theme.background().opacity(0.18)
                } else {
                    theme.surface().opacity(0.45)
                })
                .child(
                    svg()
                        .path("minus-circle.svg")
                        .size(px(16.0))
                        .text_color(if is_dnd {
                            theme.background()
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .child(
            div()
                .id("peace-pill-text")
                .flex()
                .flex_col()
                .flex_1()
                .pl_2()
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_dnd {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(dnd_title),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(if is_dnd {
                            theme.background().opacity(0.8)
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(div().w(px(24.0)))
        .into_any_element()
}

fn render_action_circles(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    div()
        .id("quick-action-dock")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_2()
        .rounded_full()
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.18))
        .child(
            div()
                .id("action-btn-theme")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.55)))
                .active(|s| s.bg(theme.surface().opacity(0.85)))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::SelectThemeRequested);
                }))
                .child(
                    svg()
                        .path("palette_2.svg")
                        .size(px(15.0))
                        .text_color(theme.foreground()),
                ),
        )
        .child(
            div()
                .id("action-btn-wallpaper")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.55)))
                .active(|s| s.bg(theme.surface().opacity(0.85)))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WallpaperRequested);
                }))
                .child(
                    svg()
                        .path("wallpaper.svg")
                        .size(px(15.0))
                        .text_color(theme.foreground()),
                ),
        )
        .child(
            div()
                .id("action-btn-settings")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.55)))
                .active(|s| s.bg(theme.surface().opacity(0.85)))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::SettingsRequested);
                }))
                .child(
                    svg()
                        .path("settings.svg")
                        .size(px(15.0))
                        .text_color(theme.foreground()),
                ),
        )
        .child(
            div()
                .id("action-btn-lock")
                .flex()
                .items_center()
                .justify_center()
                .size(px(34.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.55)))
                .active(|s| s.bg(theme.surface().opacity(0.85)))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    crate::panel::LockScreenPanel::open_all(cx);
                }))
                .child(
                    svg()
                        .path("lock.svg")
                        .size(px(15.0))
                        .text_color(theme.foreground()),
                ),
        )
        .into_any_element()
}
