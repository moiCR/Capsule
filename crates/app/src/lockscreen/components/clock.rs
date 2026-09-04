use chrono::Local;
use gpui::{Element, ParentElement, Styled, div, px};
use services::LockScreenConfig;
use ui::theme::Theme;

pub fn render_clock(theme: &Theme, config: &LockScreenConfig) -> impl Element {
    let now = Local::now();
    let time_str = now.format(&config.time_format).to_string();
    let date_str = now.format(&config.date_format).to_string();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .font_family(theme.font_family())
                .text_size(px(72.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .child(time_str),
        )
        .child(
            div()
                .font_family(theme.font_family())
                .text_size(px(20.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.foreground_muted())
                .child(date_str),
        )
}
