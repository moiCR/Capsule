use gpui::{IntoElement, div, prelude::*, px};
use ui::theme::Theme;

use super::CLOCK_WIDTH;

pub fn render_clock(time: &str, theme: &Theme) -> impl IntoElement {
    let (digits, period) = time
        .split_once(' ')
        .map_or((time, None), |(digits, period)| (digits, Some(period)));
    div()
        .w(px(CLOCK_WIDTH))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .text_size(px(13.0))
        .text_color(theme.foreground())
        .child(digits.to_owned())
        .when_some(period, |s, period| {
            s.child(
                div()
                    .text_size(px(8.0))
                    .line_height(px(9.0))
                    .text_color(theme.foreground_muted())
                    .child(period.to_owned()),
            )
        })
}
