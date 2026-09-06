use gpui::{Context, IntoElement, div, prelude::*, px};
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

    div()
        .id(format!("emoji-{global_idx}"))
        .flex()
        .items_center()
        .justify_center()
        .w(px(54.0))
        .h(px(48.0))
        .rounded(px(12.0))
        .cursor_pointer()
        .bg(if is_selected {
            theme.surface().opacity(0.55)
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        })
        .border(if is_selected { px(2.0) } else { px(0.0) })
        .border_color(theme.accent())
        .hover(|s| s.bg(theme.surface().opacity(0.4)))
        .active(|s| s.bg(theme.surface().opacity(0.6)))
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
        .child(div().text_size(px(24.0)).child(item.emoji.clone()))
}
