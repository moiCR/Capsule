use gpui::{AnyElement, IntoElement, div, prelude::*, px, svg};
use services::system::CaptureStatus;
use ui::theme::Theme;

pub fn render_privacy_dock(status: CaptureStatus, theme: &Theme) -> Option<AnyElement> {
    if !status.camera_in_use && !status.microphone_in_use {
        return None;
    }

    Some(
        div()
            .id("default-privacy-dock")
            .flex()
            .flex_shrink_0()
            .flex_row()
            .items_center()
            .justify_center()
            .h(px(22.0))
            .px(px(6.0))
            .gap(px(5.0))
            .rounded(px(7.0))
            .bg(theme.surface().opacity(0.35))
            .border_1()
            .border_color(theme.surface().opacity(0.2))
            .when(status.camera_in_use, |dock| {
                dock.child(
                    svg()
                        .path("camera.svg")
                        .size(px(13.0))
                        .text_color(theme.accent()),
                )
            })
            .when(status.microphone_in_use, |dock| {
                dock.child(
                    svg()
                        .path("mic.svg")
                        .size(px(13.0))
                        .text_color(theme.accent()),
                )
            })
            .into_any_element(),
    )
}
