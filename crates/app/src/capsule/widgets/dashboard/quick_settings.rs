use gpui::{AnyElement, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, MediaTrack, NetworkStatus, NotificationStore};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::{DashboardEvent, DashboardModule};

pub fn render_quick_settings_section(
    active_track: &MediaTrack,
    total_players: usize,
    selected_player_idx: usize,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let status = if cx.has_global::<AppState>() {
        cx.global::<AppState>().network.get_status()
    } else {
        NetworkStatus::default()
    };

    let wifi_pill = render_wifi_pill(&status, theme, cx);
    let bt_pill = render_bluetooth_pill(&status, theme, cx);
    let media_player = super::media_player::render_media_player_widget(
        active_track,
        total_players,
        selected_player_idx,
        theme,
        cx,
    );
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
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .gap_2p5()
                        .child(wifi_pill)
                        .child(bt_pill),
                )
                .child(media_player),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .w_full()
                .gap_2p5()
                .child(div().flex_1().child(peace_pill))
                .child(action_circles),
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
        .px_1p5()
        .py_1()
        .rounded_full()
        .bg(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.45)
        })
        .border_1()
        .border_color(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.25)
        })
        .hover(|s| {
            if is_on {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.55))
            }
        })
        .child(
            div()
                .id("wifi-pill-toggle")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().network.toggle_wifi();
                        cx.notify();
                    }
                }))
                .child(svg().path(icon_path).size(px(18.0)).text_color(if is_on {
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
                .pl_2p5()
                .overflow_hidden()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WifiChevronClicked);
                }))
                .child(
                    div()
                        .text_size(px(12.5))
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
                            theme.background().opacity(0.75)
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
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .hover(|s| {
                    if is_on {
                        s.bg(theme.background().opacity(0.15))
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WifiChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(14.0))
                        .text_color(if is_on {
                            theme.background().opacity(0.8)
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

    let (bt_title, disabled_str, enabled_str) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("dashboard.bluetooth"),
            lang.get("dashboard.disabled"),
            lang.get("dashboard.enabled"),
        )
    } else {
        (
            "Bluetooth".to_string(),
            "Desactivado".to_string(),
            "Activado".to_string(),
        )
    };

    let subtitle = if !is_on {
        disabled_str
    } else if is_connected {
        status.bluetooth_device_name.clone()
    } else {
        enabled_str
    };

    div()
        .id("bluetooth-pill-main")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_1p5()
        .py_1()
        .rounded_full()
        .bg(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.45)
        })
        .border_1()
        .border_color(if is_on {
            theme.accent()
        } else {
            theme.surface().opacity(0.25)
        })
        .hover(|s| {
            if is_on {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.55))
            }
        })
        .child(
            div()
                .id("bt-pill-toggle")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
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
                        .size(px(18.0))
                        .text_color(if is_on {
                            theme.background()
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .child(
            div()
                .id("bt-pill-text")
                .flex()
                .flex_col()
                .flex_1()
                .pl_2p5()
                .overflow_hidden()
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::BluetoothChevronClicked);
                }))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(if is_on {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(bt_title),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(if is_on {
                            theme.background().opacity(0.75)
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .id("bt-chevron-btn")
                .flex()
                .items_center()
                .justify_center()
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .hover(|s| {
                    if is_on {
                        s.bg(theme.background().opacity(0.15))
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::BluetoothChevronClicked);
                }))
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(14.0))
                        .text_color(if is_on {
                            theme.background().opacity(0.8)
                        } else {
                            theme.foreground_muted()
                        }),
                ),
        )
        .into_any_element()
}

fn render_peace_pill(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    let is_dnd = NotificationStore::global().is_dnd_enabled();
    let (dnd_title, enabled_str, disabled_str) = if cx.has_global::<AppState>() {
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
    let subtitle = if is_dnd { enabled_str } else { disabled_str };

    div()
        .id("peace-pill-main")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(46.0))
        .px_1p5()
        .py_1()
        .rounded_full()
        .bg(if is_dnd {
            theme.accent()
        } else {
            theme.surface().opacity(0.45)
        })
        .border_1()
        .border_color(if is_dnd {
            theme.accent()
        } else {
            theme.surface().opacity(0.25)
        })
        .hover(|s| {
            if is_dnd {
                s.opacity(0.92)
            } else {
                s.bg(theme.surface().opacity(0.55))
            }
        })
        .cursor_pointer()
        .on_click(cx.listener(|_, _, _, cx| {
            NotificationStore::global().toggle_dnd();
            cx.notify();
        }))
        .child(
            div()
                .id("peace-pill-icon")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .child(
                    svg()
                        .path("minus-circle.svg")
                        .size(px(18.0))
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
                .pl_2p5()
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(12.5))
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
                            theme.background().opacity(0.75)
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(subtitle),
                ),
        )
        .child(div().w(px(26.0)))
        .into_any_element()
}

fn render_action_circles(theme: &Theme, cx: &mut Context<DashboardModule>) -> AnyElement {
    div()
        .id("quick-action-circles")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w(px(185.0))
        .h(px(46.0))
        .child(
            div()
                .id("action-btn-theme")
                .flex()
                .items_center()
                .justify_center()
                .w(px(36.0))
                .h(px(36.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::SelectThemeRequested);
                }))
                .child(
                    svg()
                        .path("palette_2.svg")
                        .size(px(16.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .id("action-btn-wallpaper")
                .flex()
                .items_center()
                .justify_center()
                .w(px(36.0))
                .h(px(36.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::WallpaperRequested);
                }))
                .child(
                    svg()
                        .path("wallpaper.svg")
                        .size(px(16.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .id("action-btn-settings")
                .flex()
                .items_center()
                .justify_center()
                .w(px(36.0))
                .h(px(36.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(DashboardEvent::SettingsRequested);
                }))
                .child(
                    svg()
                        .path("settings.svg")
                        .size(px(16.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .id("action-btn-lock")
                .flex()
                .items_center()
                .justify_center()
                .w(px(36.0))
                .h(px(36.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    crate::panel::LockScreenPanel::open_all(cx);
                }))
                .child(
                    svg()
                        .path("lock.svg")
                        .size(px(16.0))
                        .text_color(theme.accent()),
                ),
        )
        .into_any_element()
}
