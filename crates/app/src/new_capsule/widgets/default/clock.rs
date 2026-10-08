use gpui::{IntoElement, div, prelude::*, px};
use ui::theme::Theme;

use super::CLOCK_WIDTH;

pub fn render_clock(time: &str, theme: &Theme) -> impl IntoElement {
    div()
        .w(px(CLOCK_WIDTH))
        .flex_shrink_0()
        .flex()
        .justify_center()
        .text_size(px(13.0))
        .text_color(theme.foreground())
        .child(time.to_string())
}
