use gpui::{Context, ElementId, FontWeight, IntoElement, div, prelude::*, px};
use ui::theme::{Theme, theme_manager::ThemeItem};

use crate::capsule::modules::select_theme::SelectThemeModule;

pub struct CardProps {
    pub card_w: f32,
    pub card_h: f32,
    pub y_offset: f32,
    pub opacity: f32,
    pub border_w: f32,
    pub border_alpha: f32,
    pub card_bg_alpha: f32,
    pub dot_size: f32,
    pub dot_gap: f32,
    pub font_size: f32,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn get_theme_card_props(abs_pos: f32) -> CardProps {
    if abs_pos <= 1.0 {
        let t = abs_pos;
        CardProps {
            card_w: lerp(154.0, 120.0, t),
            card_h: lerp(86.0, 70.0, t),
            y_offset: lerp(-12.0, 0.0, t),
            opacity: lerp(1.0, 0.55, t),
            border_w: lerp(2.0, 1.0, t),
            border_alpha: lerp(1.0, 0.25, t),
            card_bg_alpha: lerp(0.6, 0.25, t),
            dot_size: lerp(10.0, 8.0, t),
            dot_gap: lerp(5.0, 4.0, t),
            font_size: lerp(12.5, 11.0, t),
        }
    } else if abs_pos <= 2.0 {
        let t = abs_pos - 1.0;
        CardProps {
            card_w: lerp(120.0, 94.0, t),
            card_h: lerp(70.0, 56.0, t),
            y_offset: lerp(0.0, 3.0, t),
            opacity: lerp(0.55, 0.25, t),
            border_w: lerp(1.0, 0.5, t),
            border_alpha: lerp(0.25, 0.1, t),
            card_bg_alpha: lerp(0.25, 0.15, t),
            dot_size: lerp(8.0, 6.0, t),
            dot_gap: lerp(4.0, 3.0, t),
            font_size: lerp(11.0, 9.5, t),
        }
    } else {
        let t = (abs_pos - 2.0).min(1.0);
        CardProps {
            card_w: lerp(94.0, 70.0, t),
            card_h: lerp(56.0, 44.0, t),
            y_offset: lerp(3.0, 5.0, t),
            opacity: lerp(0.25, 0.0, t),
            border_w: lerp(0.5, 0.0, t),
            border_alpha: lerp(0.1, 0.0, t),
            card_bg_alpha: lerp(0.15, 0.0, t),
            dot_size: lerp(6.0, 5.0, t),
            dot_gap: lerp(3.0, 2.0, t),
            font_size: lerp(9.5, 8.0, t),
        }
    }
}

pub fn render_theme_card(
    slot_id: ElementId,
    item: &ThemeItem,
    target_idx: usize,
    offset: i32,
    props: &CardProps,
    theme: &Theme,
    cx: &mut Context<SelectThemeModule>,
) -> impl IntoElement {
    let is_center = offset == 0;
    let item_theme = item.theme.clone();

    let border_color = if is_center {
        theme.accent().opacity(props.border_alpha)
    } else {
        theme.surface().opacity(props.border_alpha.max(0.15))
    };

    let name_color = if is_center {
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

    let mut dots_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(props.dot_gap));
    for dot in dots {
        dots_row = dots_row.child(div().size(px(props.dot_size)).rounded_full().bg(dot));
    }

    div()
        .id(slot_id)
        .relative()
        .top(px(props.y_offset))
        .flex_shrink_0()
        .w(px(props.card_w))
        .h(px(props.card_h))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(props.dot_gap + 2.0))
        .rounded(px(16.0))
        .bg(theme.surface().opacity(props.card_bg_alpha))
        .border(px(props.border_w))
        .border_color(border_color)
        .opacity(props.opacity)
        .when(is_center, |s| s.shadow_lg())
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.45)))
        .on_click(cx.listener(move |this, _, _, cx| {
            if is_center {
                this.select_theme(item_theme.clone(), cx);
            } else {
                let dir = if offset > 0 { 1.0 } else { -1.0 };
                this.navigate(dir, target_idx, cx);
            }
        }))
        .child(dots_row)
        .child(
            div()
                .text_size(px(props.font_size))
                .font_weight(if is_center {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_color(name_color)
                .truncate()
                .child(item.name.clone()),
        )
}
