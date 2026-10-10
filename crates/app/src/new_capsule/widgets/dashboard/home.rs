use super::{
    BOTTOM_PADDING, CONTENT, CONTROLS, DashboardAction as Action, FOOTER, GAP, LEFT, NOTIFICATIONS,
    PADDING, RIGHT, SLIDER, card,
};
use crate::new_capsule::satellite::SatelliteId as View;
use crate::new_capsule::{module::dashboard::DashboardModule, widgets::style};
use gpui::{AnyElement, Bounds, Context, IntoElement, div, fill, img, prelude::*, px, size, svg};
use std::path::PathBuf;
use ui::theme::Theme;

struct Connectivity {
    id: &'static str,
    title: String,
    subtitle: String,
    icon: &'static str,
    enabled: bool,
    toggle: Action,
    detail: Option<View>,
}

fn connectivity(
    control: Connectivity,
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
    let hover = style::hover(theme);
    let open = detail
        .clone()
        .map(Action::Satellite)
        .unwrap_or_else(|| toggle.clone());
    div()
        .id(id)
        .w_full()
        .h(px(CONTROLS))
        .flex_shrink_0()
        .rounded(px(18.0))
        .bg(style::surface(theme))
        .px(px(12.0))
        .flex()
        .items_center()
        .gap(px(10.0))
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| this.dispatch(open.clone(), cx)))
        .child(
            div()
                .id(format!("{id}-toggle"))
                .size(px(40.0))
                .flex_shrink_0()
                .rounded_full()
                .bg(if enabled {
                    theme.accent()
                } else {
                    style::background(theme)
                })
                .flex()
                .items_center()
                .justify_center()
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.dispatch(toggle.clone(), cx);
                }))
                .child(svg().path(icon).size(px(20.0)).text_color(if enabled {
                    style::on_accent(theme)
                } else {
                    theme.foreground_muted()
                })),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_ellipsis()
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(subtitle),
                ),
        )
        .when(detail.is_some(), |s| {
            s.child(
                svg()
                    .path("chevron-right.svg")
                    .size(px(14.0))
                    .flex_shrink_0()
                    .text_color(theme.foreground_muted()),
            )
        })
        .into_any_element()
}

fn slider(
    module: &DashboardModule,
    brightness: bool,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let value = if brightness {
        module
            .brightness_preview
            .unwrap_or(module.snapshot.audio.brightness)
    } else {
        module.snapshot.audio.volume
    }
    .min(100);
    let fraction = value as f32 / 100.0;
    let tracker = if brightness {
        module.brightness_bounds.clone()
    } else {
        module.volume_bounds.clone()
    };
    let active = if !brightness && module.snapshot.audio.is_muted {
        theme.foreground_muted()
    } else {
        theme.accent()
    };
    let track_color = style::background(theme);
    let hover = style::hover(theme);
    let track = div()
        .id(if brightness {
            "dashboard-brightness-slider"
        } else {
            "dashboard-volume-slider"
        })
        .relative()
        .flex_1()
        .min_w_0()
        .h(px(32.0))
        .rounded_full()
        .cursor_pointer()
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                if brightness {
                    this.start_brightness(event.position.x.into(), cx);
                } else {
                    this.start_volume(event.position.x.into(), cx);
                }
            }),
        )
        .child(
            gpui::canvas(
                move |bounds, _, _| {
                    let inner = bounds;
                    tracker.track_bounds(inner);
                    inner
                },
                move |bounds, inner, window, _| {
                    window.paint_quad(fill(bounds, track_color).corner_radii(px(16.0)));
                    let width = inner.size.width * fraction;
                    if width > px(0.0) {
                        let visible = Bounds {
                            origin: inner.origin,
                            size: size(width, inner.size.height),
                        };
                        window.with_content_mask(
                            Some(gpui::ContentMask {
                                bounds: visible,
                                fade_out: Default::default(),
                            }),
                            |window| {
                                window.paint_quad(
                                    fill(inner, active).corner_radii(inner.size.height / 2.0),
                                );
                            },
                        );
                    }
                },
            )
            .size_full(),
        )
        .child(
            div().absolute().left(px(10.0)).top(px(6.0)).child(
                svg()
                    .path(if brightness {
                        "sun.svg"
                    } else if module.snapshot.audio.is_muted {
                        "volume-x.svg"
                    } else {
                        "volume-2.svg"
                    })
                    .size(px(20.0))
                    .text_color(
                        if value > 0
                            && fraction
                                * (super::WIDTH
                                    - PADDING * 2.0
                                    - 40.0
                                    - if brightness { 0.0 } else { 42.0 })
                                >= 28.0
                        {
                            style::on_accent(theme)
                        } else {
                            theme.foreground_muted()
                        },
                    ),
            ),
        );
    card(theme, cx)
        .id(if brightness {
            "dashboard-brightness"
        } else {
            "dashboard-sound"
        })
        .w_full()
        .h(px(SLIDER))
        .flex_shrink_0()
        .px(px(12.0))
        .py(px(10.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .h(px(22.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .id(if brightness {
                            "dashboard-display-label"
                        } else {
                            "dashboard-mute"
                        })
                        .text_size(px(16.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .when(!brightness, |s| {
                            s.cursor_pointer().on_click(
                                cx.listener(|this, _, _, cx| this.dispatch(Action::Mute, cx)),
                            )
                        })
                        .child(module.text(
                            if brightness {
                                "dashboard_new.display"
                            } else {
                                "dashboard.sound"
                            },
                            cx,
                        )),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.foreground_muted())
                        .child(format!("{value}%")),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(32.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(track)
                .when(!brightness, |s| {
                    s.child(
                        div()
                            .id("dashboard-audio-output")
                            .size(px(32.0))
                            .flex_shrink_0()
                            .rounded_full()
                            .bg(style::background(theme))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dispatch(Action::Satellite(View::Audio), cx)
                            }))
                            .child(
                                svg()
                                    .path("chevron-down.svg")
                                    .size(px(20.0))
                                    .text_color(theme.foreground_muted()),
                            ),
                    )
                }),
        )
        .into_any_element()
}

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let network = &module.snapshot.network;
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
    let controls = div()
        .w(px(LEFT))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(GAP))
        .child(connectivity(
            Connectivity {
                id: "dashboard-wifi",
                title: if network.ethernet_connected {
                    module.text("quick_settings.ethernet", cx)
                } else {
                    "Wi-Fi".into()
                },
                subtitle: wifi_status,
                icon: if network.ethernet_connected {
                    "ethernet.svg"
                } else {
                    "wifi.svg"
                },
                enabled: network.ethernet_connected || network.wifi_enabled,
                toggle: Action::Wifi,
                detail: Some(View::Wifi),
            },
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
                detail: Some(View::Bluetooth),
            },
            theme,
            cx,
        ))
        .child(connectivity(
            Connectivity {
                id: "dashboard-dnd",
                title: module.text("dashboard.dnd", cx),
                subtitle: module.text(
                    if module.snapshot.dnd {
                        "dashboard.enabled"
                    } else {
                        "dashboard.disabled"
                    },
                    cx,
                ),
                icon: "moon.svg",
                enabled: module.snapshot.dnd,
                toggle: Action::Dnd,
                detail: None,
            },
            theme,
            cx,
        ));
    div()
        .id("dashboard-home")
        .relative()
        .w_full()
        .h(super::module_size(!module.snapshot.tray.is_empty()).height)
        .flex_shrink_0()
        .px(px(PADDING))
        .pt(px(PADDING))
        .pb(px(BOTTOM_PADDING))
        .flex()
        .flex_col()
        .gap(px(GAP))
        .child(super::header::render(module, theme, cx))
        .child(
            div()
                .w_full()
                .h(px(CONTENT))
                .flex_shrink_0()
                .flex()
                .gap(px(GAP))
                .child(controls)
                .child(
                    div()
                        .w(px(RIGHT))
                        .h_full()
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .gap(px(GAP))
                        .child(super::media::render(module, theme, cx)),
                ),
        )
        .child(slider(module, false, theme, cx))
        .child(
            div()
                .w_full()
                .min_w_0()
                .max_w_full()
                .min_h(px(NOTIFICATIONS))
                .h(px(NOTIFICATIONS))
                .flex_shrink_0()
                .overflow_hidden()
                .child(super::notifications::render(module, theme, cx)),
        )
        .when(!module.snapshot.tray.is_empty(), |s| {
            let mut tray = div()
                .id("dashboard-tray")
                .w_full()
                .h(px(FOOTER))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .border_t_1()
                .border_color(style::border(theme))
                .overflow_x_scroll();
            for item in &module.snapshot.tray {
                let bus = item.bus_name.clone();
                let path = item.object_path.clone();
                let icon = if let Some(path) = &item.icon_file_path {
                    img(PathBuf::from(path)).size(px(22.0)).into_any_element()
                } else {
                    div()
                        .text_size(px(12.0))
                        .child(item.title.chars().next().unwrap_or('?').to_string())
                        .into_any_element()
                };
                let hover = style::hover(theme);
                tray = tray.child(
                    div()
                        .id(format!(
                            "dashboard-tray-{}-{}",
                            item.bus_name, item.object_path
                        ))
                        .size(px(24.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded_full()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.dispatch(
                                Action::Satellite(View::Tray {
                                    bus: bus.clone(),
                                    path: path.clone(),
                                }),
                                cx,
                            )
                        }))
                        .child(icon),
                );
            }
            s.child(tray)
        })
        .when_some(module.navigation.error.as_ref(), |s, message| {
            s.child(
                div()
                    .absolute()
                    .bottom(px(BOTTOM_PADDING + FOOTER + GAP))
                    .left(px(PADDING))
                    .right(px(PADDING))
                    .p(px(8.0))
                    .rounded(px(12.0))
                    .bg(style::background(theme))
                    .child(super::error(message.clone(), theme)),
            )
        })
        .into_any_element()
}
