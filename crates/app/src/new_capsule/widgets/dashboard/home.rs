use super::{BODY, DashboardAction as Action, DashboardView as View, GAP, LEFT, RIGHT, button};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, MotionDurationExt, div, ease_in_out, prelude::*,
    px, relative, svg,
};
use ui::theme::Theme;

struct Connectivity {
    id: &'static str,
    title: String,
    subtitle: String,
    icon: &'static str,
    enabled: bool,
    toggle: Action,
    detail: View,
}
fn connectivity(
    control: Connectivity,
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let Connectivity {
        id,
        title,
        subtitle,
        icon,
        enabled,
        toggle,
        detail,
    } = control;
    let background = if enabled {
        theme.accent().opacity(0.14)
    } else {
        theme.surface().opacity(0.3)
    };
    let hover = theme.surface().opacity(0.55);
    let toggle_id = format!("{id}-toggle");
    div()
        .id(id)
        .h(px(62.0))
        .flex_shrink_0()
        .w_full()
        .flex()
        .items_center()
        .px(px(12.0))
        .gap(px(10.0))
        .rounded(px(14.0))
        .bg(background)
        .transitions(|t| t.bg(module.snapshot.duration.with_easing(ease_in_out)))
        .child(button(
            toggle_id,
            String::new(),
            icon,
            toggle,
            enabled,
            theme,
            cx,
        ))
        .child(
            div()
                .id(format!("{id}-details"))
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(6.0))
                .h_full()
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.dispatch(Action::View(detail.clone()), cx)
                }))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(3.0))
                        .child(div().text_size(px(14.0)).child(title))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.foreground_muted())
                                .text_ellipsis()
                                .child(subtitle),
                        ),
                )
                .child(
                    svg()
                        .path("chevron-right.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .into_any_element()
}

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let network = &module.snapshot.network;
    let wifi = if network.ethernet_connected {
        module.text("quick_settings.ethernet", cx)
    } else {
        "Wi-Fi".into()
    };
    let wifi_status = if network.ethernet_connected {
        module.text("dashboard.connected", cx)
    } else if !network.wifi_enabled {
        module.text("dashboard.disabled", cx)
    } else if network.wifi_ssid.is_empty() {
        module.text("dashboard.disconnected", cx)
    } else {
        network.wifi_ssid.clone()
    };
    let bluetooth_status = if !network.bluetooth_enabled {
        module.text("dashboard.disabled", cx)
    } else if network.bluetooth_device_name.is_empty() {
        module.text("dashboard.disconnected", cx)
    } else {
        network.bluetooth_device_name.clone()
    };
    let mut left = div()
        .w(px(LEFT))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(connectivity(
            Connectivity {
                id: "dashboard-wifi",
                title: wifi,
                subtitle: wifi_status,
                icon: if network.ethernet_connected {
                    "ethernet.svg"
                } else {
                    "wifi.svg"
                },
                enabled: network.wifi_enabled || network.ethernet_connected,
                toggle: Action::Wifi,
                detail: View::Wifi,
            },
            module,
            theme,
            cx,
        ))
        .child(connectivity(
            Connectivity {
                id: "dashboard-bluetooth",
                title: "Bluetooth".into(),
                subtitle: bluetooth_status,
                icon: "bluetooth.svg",
                enabled: network.bluetooth_enabled,
                toggle: Action::Bluetooth,
                detail: View::Bluetooth,
            },
            module,
            theme,
            cx,
        ))
        .child(button(
            "dashboard-dnd",
            module.text("dashboard.dnd", cx),
            "moon.svg",
            Action::Dnd,
            module.snapshot.dnd,
            theme,
            cx,
        ));
    let audio = &module.snapshot.audio;
    let output = audio
        .audio_sinks
        .iter()
        .find(|sink| sink.is_default)
        .map(|sink| sink.description.clone())
        .unwrap_or_else(|| module.text("dashboard_new.audio_output", cx));
    let bounds = module.volume_bounds.clone();
    let slider = div()
        .id("dashboard-volume-slider")
        .relative()
        .w_full()
        .h(px(24.0))
        .flex()
        .items_center()
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                this.start_volume(event.position.x.into(), cx)
            }),
        )
        .child(
            gpui::canvas(
                move |bounds_rect, _, _| bounds.track_bounds(bounds_rect),
                |_, _, _, _| {},
            )
            .absolute()
            .size_full(),
        )
        .child(
            div()
                .w_full()
                .h(px(6.0))
                .rounded_full()
                .bg(theme.surface())
                .child(
                    div()
                        .id("dashboard-volume-fill")
                        .h_full()
                        .w(relative(audio.volume.min(100) as f32 / 100.0))
                        .rounded_full()
                        .bg(if audio.is_muted {
                            theme.foreground_muted()
                        } else {
                            theme.accent()
                        })
                        .transitions(|t| {
                            t.bg(module.snapshot.duration.with_easing(ease_in_out))
                                .w(module.snapshot.duration.with_easing(ease_in_out))
                        }),
                ),
        );
    left = left.child(
        div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .p(px(10.0))
            .rounded(px(14.0))
            .bg(theme.surface().opacity(0.25))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(button(
                        "dashboard-mute",
                        module.text("dashboard.volume", cx),
                        if audio.is_muted {
                            "volume-x.svg"
                        } else {
                            "volume-2.svg"
                        },
                        Action::Mute,
                        audio.is_muted,
                        theme,
                        cx,
                    ))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .child(format!("{}%", audio.volume)),
                    ),
            )
            .child(slider)
            .child(button(
                "dashboard-audio-output",
                output,
                "chevron-right.svg",
                Action::View(View::Audio),
                false,
                theme,
                cx,
            )),
    );
    let mut shortcuts = div().flex().gap(px(6.0));
    for (id, key, icon, action) in [
        (
            "themes",
            "dashboard_new.themes",
            "palette_2.svg",
            Action::View(View::Themes),
        ),
        (
            "wallpapers",
            "dashboard_new.wallpapers",
            "wallpaper.svg",
            Action::View(View::Wallpapers),
        ),
        (
            "settings",
            "dashboard_new.settings",
            "settings.svg",
            Action::Settings,
        ),
        ("lock", "dashboard_new.lock", "lock.svg", Action::Lock),
    ] {
        let label = module.text(key, cx);
        shortcuts = shortcuts.child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .child(button(
                    format!("dashboard-{id}"),
                    String::new(),
                    icon,
                    action,
                    false,
                    theme,
                    cx,
                ))
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(label),
                ),
        );
    }
    left = left.child(shortcuts);
    if module.satellites.is_empty()
        && let Some(message) = &module.navigation.error
    {
        left = left.child(super::error(message.clone(), theme));
    }
    let right = div()
        .w(px(RIGHT))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(super::media::render(module, theme, cx))
        .child(super::notifications::render(module, theme, cx));
    div()
        .id("dashboard-home")
        .flex()
        .w_full()
        .h(px(BODY))
        .flex_shrink_0()
        .gap(px(GAP))
        .child(left)
        .child(right)
        .into_any_element()
}
