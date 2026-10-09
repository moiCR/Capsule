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

pub const WIDTH: f32 = 560.0;
pub const HEIGHT: f32 = 562.0;
pub const PADDING: f32 = 12.0;
pub const GAP: f32 = 8.0;
pub const HEADER: f32 = 28.0;
pub const LEFT: f32 = 268.0;
pub const RIGHT: f32 = WIDTH - PADDING * 2.0 - LEFT - GAP;
pub const CONTROLS: f32 = 68.0;
pub const CONTENT: f32 = CONTROLS * 3.0 + GAP * 2.0;
pub const MEDIA_HEIGHT: f32 = CONTENT;
pub const SLIDER: f32 = 80.0;
pub const NOTIFICATIONS: f32 = 150.0;
pub const FOOTER: f32 = 40.0;
pub const BOTTOM_PADDING: f32 = 0.0;
#[cfg(test)]
pub const BODY: f32 = HEIGHT - PADDING - BOTTOM_PADDING - HEADER - GAP;

pub fn module_size(has_tray: bool) -> Size<Pixels> {
    let height = if has_tray {
        HEIGHT
    } else {
        HEIGHT - FOOTER - GAP
    };
    size(px(WIDTH), px(height))
}

pub(crate) fn card(theme: &Theme, cx: &gpui::App) -> gpui::Div {
    super::style::card(
        theme,
        cx.global::<services::AppState>()
            .config
            .get()
            .ui
            .cards_round
            .min(22.0),
    )
}

#[allow(dead_code)]
pub(crate) fn separator(theme: &Theme) -> gpui::Div {
    div()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(theme.foreground().opacity(0.1))
}

#[derive(Clone)]
pub(crate) enum DashboardAction {
    Satellite(crate::new_capsule::satellite::SatelliteId),
    Close,
    CloseSatellite(crate::new_capsule::satellite::SatelliteId),
    Wifi,
    Bluetooth,
    Dnd,
    Mute,
    Brightness(u32),
    Sink(String),
    ScanWifi,
    ScanBluetooth,
    SelectWifi(String),
    ConnectWifi,
    BluetoothDevice(String),
    Month(i8),
    Settings,
    ClearNotifications,
    RemoveNotification(u32),
    Notification(u32, String),
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
        super::style::selected(theme)
    } else {
        super::style::surface(theme)
    };
    let hover = super::style::hover(theme);
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(8.0))
        .min_h(px(44.0))
        .min_w_0()
        .px(px(10.0))
        .rounded(px(18.0))
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
    let hover = super::style::hover(theme);
    div()
        .id(id)
        .size(px(32.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(8.0))
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
                .size(px(18.0))
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
        assert_eq!(PADDING + BOTTOM_PADDING + HEADER + GAP + BODY, HEIGHT);
        assert_eq!(CONTENT + SLIDER + NOTIFICATIONS + FOOTER + GAP * 3.0, BODY);
        assert_eq!(module_size(true), size(px(WIDTH), px(HEIGHT)));
        assert_eq!(
            module_size(false),
            size(px(WIDTH), px(HEIGHT - FOOTER - GAP))
        );
    }
}
