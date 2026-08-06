use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px};
use services::EmojiItem;
use ui::theme::Theme;

use crate::capsule::modules::emoji::{EmojiEvent, EmojiModule};

pub fn render_emoji_cell(
    global_idx: usize,
    item: &EmojiItem,
    is_selected: bool,
    theme: &Theme,
    cx: &mut Context<EmojiModule>,
) -> impl IntoElement {
    let item_emoji = item.emoji.clone();
    let item_name = item.name.clone();

    div()
        .id(format!("emoji-{global_idx}"))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w(px(56.0))
        .h(px(56.0))
        .rounded(px(16.0))
        .cursor_pointer()
        .bg(if is_selected {
            theme.accent().opacity(0.15)
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        })
        .border_1()
        .border_color(if is_selected {
            theme.accent().opacity(0.5)
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        })
        .hover(|s| {
            s.bg(theme.surface().opacity(0.4))
                .border_color(theme.surface().opacity(0.3))
        })
        .active(|s| s.bg(theme.surface().opacity(0.5)))
        .on_mouse_move(cx.listener(move |this, _, _, cx| {
            if !this.mouse_moved {
                this.mouse_moved = true;
            }
            if this.selected_index != global_idx {
                this.selected_index = global_idx;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.service.copy_emoji(&item_emoji);
            this.clear_cache();
            cx.emit(EmojiEvent::Close);
        }))
        .child(
            div()
                .text_size(if is_selected { px(26.0) } else { px(22.0) })
                .child(item.emoji.clone()),
        )
        .child(if is_selected {
            div()
                .text_size(px(7.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.accent())
                .max_w(px(52.0))
                .overflow_hidden()
                .text_ellipsis()
                .child(truncate_name(&item_name, 10))
                .into_any_element()
        } else {
            div().into_any_element()
        })
}

fn truncate_name(name: &str, max: usize) -> String {
    if name.len() <= max {
        name.to_string()
    } else {
        format!("{}…", &name[..max.saturating_sub(1)])
    }
}
