use chrono::Local;
use gpui::{Element, ParentElement, Styled, div, px};
use services::LockScreenConfig;
use ui::theme::Theme;

pub fn render_clock(theme: &Theme, config: &LockScreenConfig) -> impl Element {
    let now = Local::now();
    let time_str = now.format(&config.time_format).to_string();
    let date_str = now.format("%A, %B %-d").to_string();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(
            div()
                .font_family(theme.font_family())
                .text_size(px(18.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.foreground().opacity(0.92))
                .child(date_str),
        )
        .child(
            div()
                .font_family(theme.font_family())
                .text_size(px(96.0))
                .line_height(px(96.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .child(time_str),
        )
}
