use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, Window, div, px, svg};
use services::{AppState, NotificationStore, PowerProfile};
use ui::components::Slider;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::{DashboardModule, DashboardSubView};
use crate::bar::{shared_panel::SharedPanelEvent, style};

pub fn render_home(
    state: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let app_state = cx.try_global::<AppState>();
    let net = app_state
        .map(|s| s.network.get_status())
        .unwrap_or_default();
    let sys = app_state.map(|s| s.system.get_status()).unwrap_or_default();
    let has_battery = app_state.and_then(|s| s.power.get_battery()).is_some();
    let profile = app_state
        .map(|s| s.power.get_active_profile())
        .unwrap_or(PowerProfile::Balanced);
    let dnd = NotificationStore::global().is_dnd_enabled();

    let (lang_net, lang_enabled, lang_disabled, lang_perf, lang_bal, lang_saver, lang_bt) =
        if let Some(s) = app_state {
            let l = &s.language;
            let net_lbl = if l.current_target() == "es" {
                "Red".to_string()
            } else {
                "Network".to_string()
            };
            (
                net_lbl,
                l.get("dashboard.enabled"),
                l.get("dashboard.disabled"),
                l.get("power.performance"),
                l.get("power.balanced"),
                l.get("power.power_saver"),
                l.get("dashboard.bluetooth"),
            )
        } else {
            (
                "Red".to_string(),
                "Activado".to_string(),
                "Desactivado".to_string(),
                "Rendimiento".to_string(),
                "Equilibrado".to_string(),
                "Ahorro".to_string(),
                "Bluetooth".to_string(),
            )
        };

    let wifi_active = net.ethernet_connected || net.wifi_enabled;
    let wifi_icon = if net.ethernet_connected {
        "ethernet.svg"
    } else if !net.wifi_enabled || net.wifi_ssid.is_empty() {
        "wifi-zero.svg"
    } else if net.wifi_signal > 55 {
        "wifi-high.svg"
    } else {
        "wifi-low.svg"
    };
    let wifi_subtitle = if net.ethernet_connected {
        "Ethernet".to_string()
    } else if !net.wifi_ssid.is_empty() {
        net.wifi_ssid.clone()
    } else if net.wifi_enabled {
        lang_enabled.clone()
    } else {
        lang_disabled.clone()
    };

    let bt_active = net.bluetooth_enabled;
    let bt_icon = "bluetooth.svg";
    let bt_subtitle = if !net.bluetooth_device_name.is_empty() {
        net.bluetooth_device_name.clone()
    } else if bt_active {
        lang_enabled
    } else {
        lang_disabled
    };

    let net_card = top_pill_card(
        "bar-dash-net-card",
        &lang_net,
        &wifi_subtitle,
        wifi_icon,
        wifi_active,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.toggle_wifi();
                cx.notify();
            }
        }),
        cx.listener(|this, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.rescan_wifi();
            }
            this.open_subview(DashboardSubView::Wifi, cx);
        }),
    );

    let bt_card = top_pill_card(
        "bar-dash-bt-card",
        &lang_bt,
        &bt_subtitle,
        bt_icon,
        bt_active,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.toggle_bluetooth();
                cx.notify();
            }
        }),
        cx.listener(|this, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.start_bluetooth_scan();
            }
            this.open_subview(DashboardSubView::Bluetooth, cx);
        }),
    );

    let top_row = div()
        .w_full()
        .h(px(72.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(net_card)
        .child(bt_card);

    let dnd_icon = if dnd { "bell-off.svg" } else { "bell.svg" };
    let (prof_icon, _prof_label, prof_active) = match profile {
        PowerProfile::Performance => ("zap.svg", lang_perf, true),
        PowerProfile::Balanced => ("scale.svg", lang_bal, false),
        PowerProfile::PowerSaver => ("leaf.svg", lang_saver, true),
    };
    let next_profile = match profile {
        PowerProfile::Balanced => PowerProfile::Performance,
        PowerProfile::Performance => PowerProfile::PowerSaver,
        PowerProfile::PowerSaver => PowerProfile::Balanced,
    };

    let dnd_btn = quick_icon_tile(
        "bar-dash-tile-dnd",
        dnd_icon,
        dnd,
        theme,
        cx.listener(|_, _, _, cx| {
            NotificationStore::global().toggle_dnd();
            cx.notify();
        }),
    );

    let prof_btn = quick_icon_tile(
        "bar-dash-tile-profile",
        prof_icon,
        prof_active,
        theme,
        cx.listener(move |_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.power.set_active_profile(next_profile.clone());
                cx.notify();
            }
        }),
    );

    let mute_btn = quick_icon_tile(
        "bar-dash-tile-mute",
        if sys.is_muted {
            "volume-x.svg"
        } else {
            "volume-2.svg"
        },
        !sys.is_muted && sys.volume > 0,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                let system = s.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.toggle_mute().await;
                });
                cx.notify();
            }
        }),
    );

    let mic_btn = quick_icon_tile(
        "bar-dash-tile-mic",
        if sys.is_input_muted {
            "mic-off.svg"
        } else {
            "mic.svg"
        },
        !sys.is_input_muted && sys.input_volume > 0,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                let system = s.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.toggle_input_mute().await;
                });
                cx.notify();
            }
        }),
    );

    let lock_btn = quick_icon_tile(
        "bar-dash-tile-lock",
        "lock.svg",
        false,
        theme,
        cx.listener(|_, _, _, cx| {
            cx.emit(SharedPanelEvent::Close);
            crate::panel::LockScreenPanel::open_all(cx);
        }),
    );

    let settings_btn = quick_icon_tile(
        "bar-dash-tile-settings",
        "settings.svg",
        false,
        theme,
        cx.listener(|_, _, _, cx| {
            cx.emit(SharedPanelEvent::Close);
            crate::panel::SettingsPanel::open(cx);
        }),
    );

    let tiles_box = div()
        .w_full()
        .flex_shrink_0()
        .p(px(10.0))
        .rounded(px(14.0))
        .bg(style::surface(theme).opacity(0.65))
        .border_1()
        .border_color(style::border(theme).opacity(0.18))
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .child(dnd_btn)
        .child(prof_btn)
        .child(mute_btn)
        .child(mic_btn)
        .child(lock_btn)
        .child(settings_btn);

    let vol_pct = sys.volume.min(100);
    let in_pct = sys.input_volume.min(100);
    let vol_icon = if sys.is_muted || vol_pct == 0 {
        "volume-x.svg"
    } else {
        "volume-2.svg"
    };
    let mic_icon = if sys.is_input_muted || in_pct == 0 {
        "mic-off.svg"
    } else {
        "mic.svg"
    };

    let out_slider_card = slider_card(
        "bar-dash-out-slider",
        vol_icon,
        !sys.is_muted && vol_pct > 0,
        vol_pct,
        &state.volume_bounds,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                let system = s.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.toggle_mute().await;
                });
                cx.notify();
            }
        }),
        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
            this.start_volume(f32::from(event.position.x), cx);
        }),
        cx.listener(|this, _: &gpui::MouseUpEvent, _, cx| {
            this.finish_sliders(cx);
        }),
        Some(Box::new(cx.listener(|this, _, _, cx| {
            this.open_subview(DashboardSubView::AudioOutput, cx);
        }))),
    );

    let in_slider_card = slider_card(
        "bar-dash-in-slider",
        mic_icon,
        !sys.is_input_muted && in_pct > 0,
        in_pct,
        &state.input_volume_bounds,
        theme,
        cx.listener(|_, _, _, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                let system = s.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.toggle_input_mute().await;
                });
                cx.notify();
            }
        }),
        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
            this.start_input_volume(f32::from(event.position.x), cx);
        }),
        cx.listener(|this, _: &gpui::MouseUpEvent, _, cx| {
            this.finish_sliders(cx);
        }),
        Some(Box::new(cx.listener(|this, _, _, cx| {
            this.open_subview(DashboardSubView::AudioInput, cx);
        }))),
    );

    let mut sliders_col = div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(out_slider_card)
        .child(in_slider_card);

    if has_battery {
        let bright_pct = state
            .brightness_preview
            .unwrap_or(sys.brightness)
            .clamp(5, 100);
        let bright_card = slider_card(
            "bar-dash-bright-slider",
            "sun.svg",
            true,
            bright_pct,
            &state.brightness_bounds,
            theme,
            |_, _, _| {},
            cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                this.start_brightness(f32::from(event.position.x), cx);
            }),
            cx.listener(|this, _: &gpui::MouseUpEvent, _, cx| {
                this.finish_sliders(cx);
            }),
            None,
        );
        sliders_col = sliders_col.child(bright_card);
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(top_row)
        .child(tiles_box)
        .child(sliders_col)
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn top_pill_card(
    id_prefix: &'static str,
    title: &str,
    subtitle: &str,
    icon: &'static str,
    active: bool,
    theme: &Theme,
    on_toggle: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
    on_open_subview: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    let (bg_color, border_col, icon_color, fg_main, fg_sub) = if active {
        (
            theme.primary_container(),
            theme.primary_container(),
            theme.primary(),
            theme.on_primary_container(),
            theme.on_primary_container().opacity(0.85),
        )
    } else {
        (
            theme.surface_container_low(),
            theme.outline_variant(),
            theme.foreground_muted(),
            theme.foreground(),
            theme.foreground_muted(),
        )
    };
    let card_hover_bg = if active {
        theme.hover_layer(theme.primary_container())
    } else {
        theme.surface_container_high()
    };
    let chevron_hover_bg = theme.hover_layer(gpui::transparent_black());

    div()
        .id((id_prefix, 0usize))
        .flex_1()
        .h_full()
        .pl(px(10.0))
        .pr(px(12.0))
        .rounded(px(14.0))
        .bg(bg_color)
        .border_1()
        .border_color(border_col)
        .flex()
        .items_center()
        .gap(px(10.0))
        .cursor_pointer()
        .hover(move |s| s.bg(card_hover_bg))
        .on_click(on_toggle)
        .child(
            div()
                .w(px(3.5))
                .h(px(24.0))
                .flex_shrink_0()
                .rounded_full()
                .bg(if active {
                    theme.accent()
                } else {
                    gpui::transparent_black()
                }),
        )
        .child(
            svg()
                .path(icon)
                .size(px(22.0))
                .flex_shrink_0()
                .text_color(icon_color),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(1.5))
                .child(
                    div()
                        .text_size(px(14.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(fg_main)
                        .line_clamp(1)
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(fg_sub)
                        .line_clamp(1)
                        .child(subtitle.to_string()),
                ),
        )
        .child(
            div()
                .id((id_prefix, 1usize))
                .size(px(28.0))
                .flex_shrink_0()
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(chevron_hover_bg))
                .on_click(move |ev, window, cx| {
                    cx.stop_propagation();
                    on_open_subview(ev, window, cx);
                })
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .into_any_element()
}

fn quick_icon_tile(
    id: &'static str,
    icon: &'static str,
    active: bool,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
) -> AnyElement {
    let (bg, fg, border_col) = if active {
        (
            theme.primary_container(),
            theme.on_primary_container(),
            theme.primary_container(),
        )
    } else {
        (
            theme.surface_container_high(),
            theme.foreground(),
            theme.outline_variant(),
        )
    };
    let hover_bg = if active {
        theme.hover_layer(theme.primary_container())
    } else {
        theme.surface_container_highest()
    };

    div()
        .id(id)
        .flex_1()
        .h(px(52.0))
        .rounded(px(14.0))
        .bg(bg)
        .border_1()
        .border_color(border_col)
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_click(on_click)
        .child(svg().path(icon).size(px(20.0)).text_color(fg))
        .into_any_element()
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn slider_card(
    id_prefix: &'static str,
    icon: &'static str,
    active: bool,
    pct: u32,
    bounds: &DimensionTracker,
    theme: &Theme,
    on_icon_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static,
    on_track_down: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut gpui::App) + 'static,
    on_track_up: impl Fn(&gpui::MouseUpEvent, &mut Window, &mut gpui::App) + 'static,
    on_gear_click: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static>>,
) -> AnyElement {
    let hover_bg = theme.hover_layer(gpui::transparent_black());
    let fill_ratio = (pct as f32 / 100.0).clamp(0.0, 1.0);
    let active_color = if active {
        theme.primary()
    } else {
        theme.foreground_muted()
    };
    let icon_color = if active {
        theme.foreground()
    } else {
        theme.foreground_muted()
    };

    let slider = Slider::new(id_prefix)
        .fraction(fill_ratio)
        .height(30.0)
        .flex_1(true)
        .label(format!("{pct}%"))
        .track_bounds(bounds)
        .track_color(theme.surface_container_highest())
        .active_color(active_color)
        .on_mouse_down(on_track_down)
        .on_mouse_up(on_track_up);

    let mut row = div()
        .w_full()
        .h(px(54.0))
        .px(px(12.0))
        .rounded(px(16.0))
        .bg(theme.surface_container_low())
        .border_1()
        .border_color(theme.outline_variant())
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .id((id_prefix, 0usize))
                .size(px(34.0))
                .flex_shrink_0()
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(on_icon_click)
                .child(svg().path(icon).size(px(20.0)).text_color(icon_color)),
        )
        .child(slider);

    if let Some(gear_fn) = on_gear_click {
        row = row.child(
            div()
                .id((id_prefix, 2usize))
                .size(px(34.0))
                .flex_shrink_0()
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .on_click(gear_fn)
                .child(
                    svg()
                        .path("settings.svg")
                        .size(px(18.0))
                        .text_color(theme.foreground_muted()),
                ),
        );
    }

    row.into_any_element()
}
