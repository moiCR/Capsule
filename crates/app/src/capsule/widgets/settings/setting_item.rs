use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use ui::theme::Theme;

pub fn render_setting_row(
    title: &str,
    description: &str,
    control: impl IntoElement,
    cards_round: f32,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_4()
        .py_3()
        .rounded(px(cards_round))
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.5))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .pr_4()
                .gap(px(2.0))
                .child(
                    div()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_size(px(13.0))
                        .text_color(theme.foreground())
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child(description.to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .flex_shrink_0()
                .child(control),
        )
}

pub fn render_section_header(title: &str, subtitle: &str, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .mb_4()
        .child(
            div()
                .font_weight(gpui::FontWeight::BOLD)
                .text_size(px(18.0))
                .text_color(theme.foreground())
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(subtitle.to_string()),
        )
        .into_any_element()
}
