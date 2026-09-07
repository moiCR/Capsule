use gpui::{Context, ElementId, FontWeight, IntoElement, div, prelude::*, px};
use ui::theme::{Theme, theme_manager::ThemeItem};

use crate::capsule::modules::select_theme::SelectThemeModule;

pub fn render_theme_card(
    slot_id: ElementId,
    item: &ThemeItem,
    is_selected: bool,
    target_idx: usize,
    theme: &Theme,
    cx: &mut Context<SelectThemeModule>,
) -> impl IntoElement {
    let item_theme = item.theme.clone();

    let border_color = if is_selected {
        theme.accent()
    } else {
        theme.surface().opacity(0.2)
    };

    let card_bg = if is_selected {
        theme.surface().opacity(0.55)
    } else {
        theme.surface().opacity(0.2)
    };

    let name_color = if is_selected {
        theme.foreground()
    } else {
        theme.foreground_muted()
    };

    let dots = [
        item.theme.red(),
        item.theme.green(),
        item.theme.accent(),
        item.theme.foreground(),
        item.theme.foreground_muted(),
        item.theme.surface(),
    ];

    let mut dots_row = div().flex().flex_row().items_center().gap(px(5.0));
    for dot in dots {
        dots_row = dots_row.child(div().size(px(10.0)).rounded_full().bg(dot));
    }

    div()
        .id(slot_id)
        .flex_shrink_0()
        .w(px(140.0))
        .h(px(84.0))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .rounded(px(16.0))
        .bg(card_bg)
        .border(if is_selected { px(2.0) } else { px(1.0) })
        .border_color(border_color)
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.4)))
        .on_click(cx.listener(move |this, _, _, cx| {
            if is_selected {
                this.select_theme(item_theme.clone(), cx);
            } else {
                this.set_selected_idx(target_idx, cx);
            }
        }))
        .child(dots_row)
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(if is_selected {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(name_color)
                .child(item.name.clone()),
        )
}
