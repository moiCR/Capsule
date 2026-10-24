use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{AnyElement, BoxShadow, ColorExt, Context, IntoElement, Window, div, point, px, svg};
use services::{AppState, NotificationStore, PowerProfile};
use ui::components::{Input, InputState, Slider};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::bar::{Bar, style};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardSubView {
    #[default]
    Home,
    Wifi,
    Bluetooth,
    AudioOutput,
    AudioInput,
}

pub struct DashboardState {
    pub view: DashboardSubView,
    pub overlay_view: Option<DashboardSubView>,
    pub overlay_progress: f32,
    pub overlay_anim_start: Option<Instant>,
    pub overlay_closing: bool,
    pub selected_ssid: Option<String>,
    pub password_input: InputState,
    pub volume_bounds: DimensionTracker,
    pub input_volume_bounds: DimensionTracker,
    pub brightness_bounds: DimensionTracker,
    pub brightness_preview: Option<u32>,
    pub dragging_volume: bool,
    pub dragging_input_volume: bool,
    pub dragging_brightness: bool,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            view: DashboardSubView::Home,
            overlay_view: None,
            overlay_progress: 0.0,
            overlay_anim_start: None,
            overlay_closing: false,
            selected_ssid: None,
            password_input: InputState::new("").with_max_len(128),
            volume_bounds: DimensionTracker::new(),
            input_volume_bounds: DimensionTracker::new(),
            brightness_bounds: DimensionTracker::new(),
            brightness_preview: None,
            dragging_volume: false,
            dragging_input_volume: false,
            dragging_brightness: false,
        }
    }
}

impl DashboardState {
    pub fn reset(&mut self) {
        self.view = DashboardSubView::Home;
        self.overlay_view = None;
        self.overlay_progress = 0.0;
        self.overlay_anim_start = None;
        self.overlay_closing = false;
        self.selected_ssid = None;
        self.password_input.clear();
        self.dragging_volume = false;
        self.dragging_input_volume = false;
        self.dragging_brightness = false;
        self.brightness_preview = None;
    }

    pub fn open_subview(&mut self, subview: DashboardSubView) {
        if subview == DashboardSubView::Home {
            self.close_subview();
            return;
        }
        self.view = subview;
        self.overlay_view = Some(subview);
        self.overlay_closing = false;
        self.overlay_anim_start = Some(Instant::now());
        if self.overlay_progress <= 0.01 {
            self.overlay_progress = 0.01;
        }
    }

    pub fn close_subview(&mut self) {
        if self.overlay_view.is_none() {
            self.view = DashboardSubView::Home;
            self.selected_ssid = None;
            return;
        }
        self.view = DashboardSubView::Home;
        self.selected_ssid = None;
        self.password_input.clear();
        self.overlay_closing = true;
        self.overlay_anim_start = Some(Instant::now());
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(start) = self.overlay_anim_start else {
            return false;
        };
        let duration = Duration::from_millis(180);
        let raw =
            (now.duration_since(start).as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - raw).powi(3);

        if self.overlay_closing {
            self.overlay_progress = (1.0 - eased).clamp(0.0, 1.0);
            if raw >= 1.0 {
                self.overlay_progress = 0.0;
                self.overlay_view = None;
                self.overlay_anim_start = None;
                self.overlay_closing = false;
                return false;
            }
            true
        } else {
            self.overlay_progress = eased.clamp(0.0, 1.0);
            if raw >= 1.0 {
                self.overlay_progress = 1.0;
                self.overlay_anim_start = None;
                return false;
            }
            true
        }
    }
}

pub fn slider_fraction(x: f32, left: f32, width: f32) -> Option<f32> {
    (width > 0.0).then(|| ((x - left) / width).clamp(0.0, 1.0))
}

pub fn render(state: &DashboardState, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let home = render_home(state, &theme, cx);

    let mut root = div()
        .id("shared-panel-dashboard")
        .size_full()
        .relative()
        .p(px(16.0))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(home);

    if let Some(subview) = state.overlay_view {
        let progress = state.overlay_progress.clamp(0.0, 1.0);
        let offset_y = (1.0 - progress) * 18.0;
        let sub_content = match subview {
            DashboardSubView::Wifi => render_wifi(state, &theme, cx),
            DashboardSubView::Bluetooth => render_bluetooth(&theme, cx),
            DashboardSubView::AudioOutput => render_audio_output(&theme, cx),
            DashboardSubView::AudioInput => render_audio_input(&theme, cx),
            DashboardSubView::Home => div().into_any_element(),
        };

        let overlay = div()
            .id("bar-dash-subview-backdrop")
            .absolute()
            .inset_0()
            .backdrop_blur(px(16.0 * progress))
            .bg(gpui::black().opacity(0.25 * progress))
            .p(px(14.0))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|bar, _, window, cx| {
                    bar.close_dashboard_subview(window, cx);
                }),
            )
            .child(
                div()
                    .id("bar-dash-subview-card")
                    .w_full()
                    .h_full()
                    .relative()
                    .top(px(offset_y))
                    .opacity(progress)
                    .rounded(px(16.0))
                    .bg(style::background(&theme).opacity(0.92))
                    .border_1()
                    .border_color(style::border(&theme).opacity(0.18))
                    .shadow(vec![BoxShadow {
                        color: gpui::black().opacity(0.40).into(),
                        offset: point(px(0.0), px(16.0)),
                        blur_radius: px(36.0),
                        spread_radius: px(0.0),
                        inset: false,
                    }])
                    .p(px(14.0))
                    .overflow_hidden()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(|_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(sub_content),
            );

        root = root.child(overlay);
    }

    root.into_any_element()
}

fn render_home(state: &DashboardState, theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
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
        lang_enabled.clone()
    } else {
        lang_disabled.clone()
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
        cx.listener(|bar, _, window, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.rescan_wifi();
            }
            bar.open_dashboard_subview(DashboardSubView::Wifi, window, cx);
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
        cx.listener(|bar, _, window, cx| {
            if let Some(s) = cx.try_global::<AppState>() {
                s.network.start_bluetooth_scan();
            }
            bar.open_dashboard_subview(DashboardSubView::Bluetooth, window, cx);
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
        cx.listener(|bar, _, window, cx| {
            bar.close_shared_panel(window, cx);
            crate::panel::LockScreenPanel::open_all(cx);
        }),
    );

    let settings_btn = quick_icon_tile(
        "bar-dash-tile-settings",
        "settings.svg",
        false,
        theme,
        cx.listener(|bar, _, window, cx| {
            bar.close_shared_panel(window, cx);
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
        cx.listener(|bar, event: &gpui::MouseDownEvent, _, cx| {
            bar.start_dashboard_volume(f32::from(event.position.x), cx);
        }),
        cx.listener(|bar, _: &gpui::MouseUpEvent, _, cx| {
            bar.finish_dashboard_sliders(cx);
        }),
        Some(Box::new(cx.listener(|bar, _, window, cx| {
            bar.open_dashboard_subview(DashboardSubView::AudioOutput, window, cx);
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
        cx.listener(|bar, event: &gpui::MouseDownEvent, _, cx| {
            bar.start_dashboard_input_volume(f32::from(event.position.x), cx);
        }),
        cx.listener(|bar, _: &gpui::MouseUpEvent, _, cx| {
            bar.finish_dashboard_sliders(cx);
        }),
        Some(Box::new(cx.listener(|bar, _, window, cx| {
            bar.open_dashboard_subview(DashboardSubView::AudioInput, window, cx);
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
            cx.listener(|bar, event: &gpui::MouseDownEvent, _, cx| {
                bar.start_dashboard_brightness(f32::from(event.position.x), cx);
            }),
            cx.listener(|bar, _: &gpui::MouseUpEvent, _, cx| {
                bar.finish_dashboard_sliders(cx);
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
    let (bg_color, border_col, icon_color, fg_sub) = if active {
        (
            style::surface(theme).opacity(0.85),
            theme.accent().opacity(0.40),
            theme.accent(),
            theme.accent(),
        )
    } else {
        (
            style::surface(theme).opacity(0.50),
            style::border(theme).opacity(0.18),
            theme.foreground_muted(),
            theme.foreground_muted(),
        )
    };
    let fg_main = theme.foreground();
    let card_hover_bg = if active {
        style::surface(theme)
    } else {
        style::surface(theme).opacity(0.70)
    };
    let chevron_hover_bg = style::hover(theme);

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
            theme.accent().opacity(0.18),
            theme.accent(),
            theme.accent().opacity(0.45),
        )
    } else {
        (
            style::background(theme).opacity(0.50),
            theme.foreground(),
            style::border(theme).opacity(0.16),
        )
    };
    let hover_bg = if active {
        theme.accent().opacity(0.28)
    } else {
        style::hover(theme)
    };

    div()
        .id(id)
        .flex_1()
        .h(px(52.0))
        .rounded(px(12.0))
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
    let hover_bg = style::hover(theme);
    let fill_ratio = (pct as f32 / 100.0).clamp(0.0, 1.0);
    let active_color = if active {
        theme.accent()
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
        .track_color(style::background(theme))
        .active_color(active_color)
        .on_mouse_down(on_track_down)
        .on_mouse_up(on_track_up);

    let mut row = div()
        .w_full()
        .h(px(54.0))
        .px(px(12.0))
        .rounded(px(14.0))
        .bg(style::surface(theme).opacity(0.65))
        .border_1()
        .border_color(style::border(theme).opacity(0.18))
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

fn render_wifi(state: &DashboardState, theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
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
                        .on_click(cx.listener(|bar, _, window, cx| {
                            bar.close_dashboard_subview(window, cx);
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
                    .on_click(cx.listener(move |bar, _, window, cx| {
                        if let Some(s) = cx.try_global::<AppState>() {
                            if is_connected {
                                s.network.disconnect_wifi(&ssid_click);
                            } else if is_open {
                                s.network.connect_wifi(&ssid_click, None);
                            } else {
                                bar.shared_panel.dashboard_state.selected_ssid =
                                    Some(ssid_click.clone());
                                bar.shared_panel.dashboard_state.password_input.clear();
                            }
                            bar.sync_window_state(window, cx);
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
                                .on_click(cx.listener(move |bar, _, window, cx| {
                                    let pwd =
                                        bar.shared_panel.dashboard_state.password_input.text();
                                    if let Some(s) = cx.try_global::<AppState>() {
                                        s.network.connect_wifi(&ssid_conn, Some(&pwd));
                                    }
                                    bar.shared_panel.dashboard_state.password_input.clear();
                                    bar.shared_panel.dashboard_state.selected_ssid = None;
                                    bar.sync_window_state(window, cx);
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

fn render_bluetooth(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
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
                        .on_click(cx.listener(|bar, _, window, cx| {
                            bar.close_dashboard_subview(window, cx);
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

fn render_audio_output(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
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
                .on_click(cx.listener(|bar, _, window, cx| {
                    bar.close_dashboard_subview(window, cx);
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

fn render_audio_input(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
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
                .on_click(cx.listener(|bar, _, window, cx| {
                    bar.close_dashboard_subview(window, cx);
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
