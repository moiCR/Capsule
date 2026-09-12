use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::Snippet;
use ui::theme::Theme;

use crate::capsule::modules::clipboard::{ClipboardEvent, ClipboardModule};

pub fn render_snippet_item(
    idx: usize,
    snippet: &Snippet,
    is_selected: bool,
    theme: &Theme,
    cx: &mut Context<ClipboardModule>,
) -> impl IntoElement {
    let snippet_id = snippet.id.clone();
    let content_to_copy = snippet.content.clone();

    let indicator_color = if is_selected {
        theme.accent()
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    let item_bg = if is_selected {
        theme.surface().opacity(0.55)
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    div()
        .id(format!("snippet-item-{idx}"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap_2p5()
        .px_2()
        .py(px(6.0))
        .rounded(px(12.0))
        .cursor_pointer()
        .bg(item_bg)
        .hover(|s| s.bg(theme.surface().opacity(0.4)))
        .active(|s| s.bg(theme.surface().opacity(0.6)))
        .on_mouse_move(cx.listener(move |this, _, _, cx| {
            if !this.mouse_moved {
                this.mouse_moved = true;
            }
            if this.selected_index != idx {
                this.selected_index = idx;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.service.copy_text(&content_to_copy);
            cx.emit(ClipboardEvent::Close);
        }))
        .child(
            div()
                .w(px(3.0))
                .h(px(18.0))
                .rounded_full()
                .bg(indicator_color),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(28.0))
                .h(px(28.0))
                .rounded(px(8.0))
                .bg(theme.surface().opacity(0.6))
                .child(
                    svg()
                        .path("pin.svg")
                        .size(px(13.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .gap_0p5()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(13.0))
                        .text_color(if is_selected {
                            theme.foreground()
                        } else {
                            theme.foreground_muted()
                        })
                        .truncate()
                        .child(snippet.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted().opacity(0.75))
                        .truncate()
                        .child(snippet.content.clone()),
                ),
        )
        .child(
            div()
                .id(format!("remove-snippet-{idx}"))
                .flex()
                .items_center()
                .justify_center()
                .w(px(22.0))
                .h(px(22.0))
                .rounded_full()
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.opacity(0.6))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.remove_snippet(&snippet_id, cx);
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted().opacity(0.6)),
                ),
        )
}
