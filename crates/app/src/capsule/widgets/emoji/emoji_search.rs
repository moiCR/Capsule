use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::emoji::EmojiModule;

pub fn render_emoji_search(
    query: &str,
    theme: &Theme,
    cx: &mut Context<EmojiModule>,
) -> impl IntoElement {
    let has_query = !query.is_empty();

    div()
        .flex()
        .items_center()
        .gap_3()
        .w_full()
        .px_3()
        .py_2()
        .child(
            svg()
                .path("search.svg")
                .w_4()
                .h_4()
                .text_color(theme.foreground_muted().opacity(0.8)),
        )
        .child(div().flex_1().text_sm().child(if has_query {
            div()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground())
                .child(query.to_string())
        } else {
            div()
                .text_color(theme.foreground_muted().opacity(0.7))
                .child("Buscar emoji...")
        }))
        .children(if has_query {
            Some(
                div()
                    .id("emoji-clear-search")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.6))
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.update_search(String::new(), cx);
                    }))
                    .child(
                        svg()
                            .path("close.svg")
                            .w_3()
                            .h_3()
                            .text_color(theme.foreground_muted()),
                    ),
            )
        } else {
            None
        })
}
