pub(crate) mod details;
pub(crate) mod header;
pub(crate) mod home;
pub(crate) mod media;
mod notifications;

use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, MotionDurationExt, Pixels, Size, div, ease_in_out,
    prelude::*, px, size, svg,
};
use ui::theme::Theme;

pub const WIDTH: f32 = 640.0;
pub const HEIGHT: f32 = 480.0;
pub const PADDING: f32 = 20.0;
pub const GAP: f32 = 16.0;
pub const HEADER: f32 = 44.0;
pub const LEFT: f32 = 224.0;
pub const RIGHT: f32 = 360.0;
pub const BODY: f32 = HEIGHT - PADDING * 2.0 - HEADER - GAP;
pub fn module_size() -> Size<Pixels> {
    size(px(WIDTH), px(HEIGHT))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DashboardView {
    Home,
    Wifi,
    Bluetooth,
    Calendar,
    Audio,
    Media,
    Tray { bus: String, path: String },
    Themes,
    Wallpapers,
}
impl DashboardView {
    pub(crate) fn satellite_id(&self) -> Option<crate::new_capsule::satellite::SatelliteId> {
        use crate::new_capsule::satellite::SatelliteId;
        match self {
            Self::Wifi => Some(SatelliteId::Wifi),
            Self::Bluetooth => Some(SatelliteId::Bluetooth),
            Self::Calendar => Some(SatelliteId::Calendar),
            Self::Audio => Some(SatelliteId::Audio),
            Self::Media => Some(SatelliteId::Media),
            Self::Tray { bus, path } => Some(SatelliteId::Tray {
                bus: bus.clone(),
                path: path.clone(),
            }),
            _ => None,
        }
    }

    pub fn title_key(&self) -> &'static str {
        match self {
            Self::Home => "dashboard_new.title",
            Self::Wifi => "quick_settings.wifi_networks",
            Self::Bluetooth => "quick_settings.bt_devices",
            Self::Calendar => "dashboard_new.calendar",
            Self::Audio => "dashboard_new.audio_output",
            Self::Media => "settings.tab_media",
            Self::Tray { .. } => "dashboard_new.tray",
            Self::Themes => "dashboard_new.themes",
            Self::Wallpapers => "dashboard_new.wallpapers",
        }
    }
}
#[derive(Clone)]
pub(crate) enum DashboardAction {
    View(DashboardView),
    Back,
    Close,
    CloseSatellite(crate::new_capsule::satellite::SatelliteId),
    Wifi,
    Bluetooth,
    Dnd,
    Mute,
    Sink(String),
    ScanWifi,
    ScanBluetooth,
    SelectWifi(String),
    ConnectWifi,
    BluetoothDevice(String),
    Month(i8),
    Settings,
    Lock,
    ClearNotifications,
    RemoveNotification(u32),
    Notification(u32, String),
    Player(String),
    Media(i8),
    Seek(f64),
    TrayActivate { bus: String, path: String },
    TrayMenu(String, String, i32),
}

pub(crate) fn button(
    id: impl Into<gpui::ElementId>,
    label: String,
    icon: &'static str,
    action: DashboardAction,
    active: bool,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let duration = std::time::Duration::from_millis(
        cx.global::<services::AppState>()
            .config
            .get()
            .ui
            .animation_duration_ms as u64,
    );
    let background = if active {
        theme.accent().opacity(0.18)
    } else {
        theme.surface().opacity(0.35)
    };
    let hover = theme.surface().opacity(0.6);
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.0))
        .min_h(px(36.0))
        .min_w_0()
        .px(px(10.0))
        .rounded(px(10.0))
        .bg(background)
        .cursor_pointer()
        .transitions(|t| t.bg(duration.with_easing(ease_in_out)))
        .hover(move |s| s.bg(hover))
        .on_click(cx.listener(move |this, _, _, cx| this.dispatch(action.clone(), cx)))
        .child(
            svg()
                .path(icon)
                .size(px(16.0))
                .flex_shrink_0()
                .text_color(if active {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }),
        )
        .when(!label.is_empty(), |s| {
            s.child(div().min_w_0().text_ellipsis().child(label))
        })
        .into_any_element()
}
pub(crate) fn icon_action(
    id: impl Into<gpui::ElementId>,
    icon: &'static str,
    action: DashboardAction,
    enabled: bool,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let hover = theme.surface().opacity(0.6);
    div()
        .id(id)
        .size(px(28.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .opacity(if enabled { 1.0 } else { 0.35 })
        .when(enabled, |s| {
            s.cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |this, _, _, cx| {
                    cx.stop_propagation();
                    this.dispatch(action.clone(), cx);
                }))
        })
        .child(
            svg()
                .path(icon)
                .size(px(14.0))
                .text_color(theme.foreground_muted()),
        )
        .into_any_element()
}

pub(crate) fn empty(text: String, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .p(px(16.0))
        .text_size(px(12.0))
        .text_color(theme.foreground_muted())
        .child(text)
        .into_any_element()
}
pub(crate) fn error(text: String, theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .text_size(px(12.0))
        .text_color(theme.red())
        .child(text)
        .into_any_element()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dimensions_match_columns_and_vertical_layout() {
        assert_eq!(PADDING * 2.0 + LEFT + GAP + RIGHT, WIDTH);
        assert_eq!(PADDING * 2.0 + HEADER + GAP + BODY, HEIGHT);
        assert_eq!(module_size(), size(px(640.0), px(480.0)));
    }
}
