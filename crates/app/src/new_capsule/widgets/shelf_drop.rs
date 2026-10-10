use gpui::{IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

pub const WIDTH: f32 = 560.0;
pub const HEIGHT: f32 = 104.0;

pub fn render(theme: &Theme) -> impl IntoElement {
    div()
        .id("shelf-drop-module")
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("cloud-upload.svg")
                .size(px(32.0))
                .text_color(theme.foreground_muted()),
        )
}
