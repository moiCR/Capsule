use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::emoji::EmojiModule;

pub fn render_emoji_search(
    query: &str,
    total_items: usize,
    theme: &Theme,
    cx: &mut Context<EmojiModule>,
) -> impl IntoElement {
    let has_query = !query.is_empty();

    div()
        .flex()
        .items_center()
        .gap_2p5()
        .w_full()
        .px_3p5()
        .py_2p5()
        .bg(theme.surface())
        .rounded(px(42.0))
        .child(
            svg()
                .path("search.svg")
                .w_4()
                .h_4()
                .text_color(if has_query {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }),
        )
        .child(div().flex_1().text_sm().child(if has_query {
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child(query.to_string())
        } else {
            div()
                .text_color(theme.foreground_muted())
                .child("Buscar emoji...")
        }))
        .child(if has_query {
            div()
                .id("emoji-clear-search")
                .cursor_pointer()
                .hover(|s| s.opacity(0.7))
                .active(|s| s.opacity(0.5))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.update_search(String::new(), cx);
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .w_3p5()
                        .h_3p5()
                        .text_color(theme.foreground_muted()),
                )
                .into_any_element()
        } else {
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground_muted())
                .child(format!("{total_items}"))
                .into_any_element()
        })
}
