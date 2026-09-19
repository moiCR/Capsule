use gpui::{IntoElement, div, prelude::*};
use ui::theme::Theme;

use super::flip_clock::FlipClock;

pub fn render_clock_widget(flip_clock: &FlipClock, theme: &Theme) -> impl IntoElement {
    div()
        .id("default-clock")
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .child(flip_clock.render(theme))
}
