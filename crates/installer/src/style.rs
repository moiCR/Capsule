#![allow(dead_code)]

use gpui::{Hsla, div, prelude::*, px};
use ui::theme::Theme;

pub fn opaque(mut color: Hsla) -> Hsla {
    color.alpha = 1.0;
    color
}

pub fn tint(base: Hsla, color: Hsla, amount: f32) -> Hsla {
    let base = gpui::hsla_to_rgba(base);
    let color = gpui::hsla_to_rgba(color);
    let amount = (amount * color.alpha).clamp(0.0, 1.0);
    gpui::rgb_to_hsla(gpui::Rgba::new(
        base.red + (color.red - base.red) * amount,
        base.green + (color.green - base.green) * amount,
        base.blue + (color.blue - base.blue) * amount,
        1.0,
    ))
}

pub fn background(theme: &Theme) -> Hsla {
    opaque(theme.background())
}

pub fn surface(theme: &Theme) -> Hsla {
    tint(
        tint(background(theme), theme.surface(), 0.5),
        opaque(theme.surface()),
        theme.light_mix(),
    )
}

pub fn hover(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.08)
}

pub fn selected(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.12)
}

pub fn border(theme: &Theme) -> Hsla {
    tint(
        tint(background(theme), theme.background_alt(), 0.5),
        opaque(theme.background_alt()),
        theme.light_mix(),
    )
}

pub fn raised(theme: &Theme) -> Hsla {
    tint(
        tint(surface(theme), theme.foreground(), 0.06),
        tint(surface(theme), border(theme), 0.5),
        theme.light_mix(),
    )
}

pub fn on_accent(theme: &Theme) -> Hsla {
    theme.on_accent()
}

pub const CARD_RADIUS: f32 = 16.0;
pub const INNER_RADIUS: f32 = 12.0;
pub const CAPSULE_ROUND: f32 = 24.0;

pub fn card(theme: &Theme, radius: f32) -> gpui::Div {
    div().rounded(px(radius)).bg(surface(theme))
}

pub fn primary_button(
    id: impl Into<gpui::ElementId>,
    label: String,
    enabled: bool,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let accent = opaque(theme.accent());
    let pressed = tint(accent, theme.foreground(), 0.12);
    div()
        .id(id)
        .h(px(36.0))
        .px(px(20.0))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .bg(accent)
        .text_color(on_accent(theme))
        .opacity(if enabled { 1.0 } else { 0.45 })
        .when(enabled, |s| {
            s.cursor_pointer().hover(move |s| s.bg(pressed))
        })
        .child(label)
}

pub fn secondary_button(
    id: impl Into<gpui::ElementId>,
    label: String,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover_bg = hover(theme);
    div()
        .id(id)
        .h(px(36.0))
        .px(px(18.0))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.0))
        .font_weight(gpui::FontWeight::NORMAL)
        .bg(raised(theme))
        .border_1()
        .border_color(border(theme))
        .text_color(theme.foreground())
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .child(label)
}

pub fn circle_button(
    id: impl Into<gpui::ElementId>,
    icon: &'static str,
    diameter: f32,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover_bg = hover(theme);
    div()
        .id(id)
        .size(px(diameter))
        .flex_shrink_0()
        .rounded_full()
        .bg(raised(theme))
        .border_1()
        .border_color(border(theme))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .child(
            gpui::svg()
                .path(icon)
                .size(px((diameter * 0.45).round()))
                .text_color(theme.foreground()),
        )
}

pub fn chip(
    id: impl Into<gpui::ElementId>,
    label: String,
    selected: bool,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover_bg = hover(theme);
    div()
        .id(id)
        .h(px(28.0))
        .px(px(12.0))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .text_size(px(11.0))
        .cursor_pointer()
        .when(selected, |s| {
            s.bg(opaque(theme.accent())).text_color(on_accent(theme))
        })
        .when(!selected, |s| {
            s.border_1()
                .border_color(border(theme))
                .bg(raised(theme))
                .text_color(theme.foreground_muted())
                .hover(move |s| s.bg(hover_bg).text_color(theme.foreground()))
        })
        .child(label)
}

pub fn check_badge(diameter: f32, theme: &Theme) -> gpui::Div {
    div()
        .size(px(diameter))
        .flex_shrink_0()
        .rounded_full()
        .bg(opaque(theme.accent()))
        .flex()
        .items_center()
        .justify_center()
        .child(
            gpui::svg()
                .path("check.svg")
                .size(px((diameter * 0.6).round()))
                .text_color(on_accent(theme)),
        )
}
