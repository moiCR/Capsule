#![allow(dead_code)]

use gpui::{Hsla, div, prelude::*, px};
use ui::theme::Theme;

fn opaque(mut color: Hsla) -> Hsla {
    color.alpha = 1.0;
    color
}

pub(crate) fn tint(base: Hsla, color: Hsla, amount: f32) -> Hsla {
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

pub(crate) fn background(theme: &Theme) -> Hsla {
    opaque(theme.background())
}

pub(crate) fn surface(theme: &Theme) -> Hsla {
    tint(background(theme), theme.surface(), 0.5)
}

pub(crate) fn hover(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.08)
}

pub(crate) fn selected(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.12)
}

pub(crate) fn border(theme: &Theme) -> Hsla {
    tint(background(theme), theme.background_alt(), 0.5)
}

pub(crate) fn card(theme: &Theme, radius: f32) -> gpui::Div {
    div().rounded(px(radius)).bg(surface(theme))
}

pub(crate) fn on_accent(theme: &Theme) -> Hsla {
    fn luminance(color: Hsla) -> f32 {
        let color = gpui::hsla_to_rgba(color);
        let linear = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.red) + 0.7152 * linear(color.green) + 0.0722 * linear(color.blue)
    }
    let accent = luminance(theme.accent());
    let foreground = theme.foreground();
    let background = theme.background();
    let contrast = |color| {
        let value = luminance(color);
        (accent.max(value) + 0.05) / (accent.min(value) + 0.05)
    };
    if contrast(background) > contrast(foreground) {
        opaque(background)
    } else {
        opaque(foreground)
    }
}

pub(crate) const CARD_RADIUS: f32 = 16.0;
pub(crate) const INNER_RADIUS: f32 = 12.0;

pub(crate) fn raised(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.foreground(), 0.06)
}

pub(crate) fn accent_card(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.2)
}

pub(crate) fn breadcrumb(section: String, theme: &Theme) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(10.0))
        .letter_spacing(px(2.0))
        .text_transform(gpui::TextTransform::Uppercase)
        .text_color(theme.foreground_muted())
        .child(
            div()
                .text_color(theme.foreground())
                .font_weight(gpui::FontWeight::MEDIUM)
                .child("Capsule"),
        )
        .child("/")
        .child(section)
}

pub(crate) fn panel_header(icon: &'static str, title: String, theme: &Theme) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .min_w_0()
        .child(
            gpui::svg()
                .path(icon)
                .size(px(18.0))
                .flex_shrink_0()
                .text_color(theme.foreground()),
        )
        .child(
            div()
                .min_w_0()
                .text_size(px(14.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_ellipsis()
                .child(title),
        )
}

pub(crate) fn circle_button(
    id: impl Into<gpui::ElementId>,
    icon: &'static str,
    diameter: f32,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover = hover(theme);
    div()
        .id(id)
        .size(px(diameter))
        .flex_shrink_0()
        .rounded_full()
        .bg(raised(theme))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover))
        .child(
            gpui::svg()
                .path(icon)
                .size(px((diameter * 0.45).round()))
                .text_color(theme.foreground()),
        )
}

pub(crate) fn primary_button(
    id: impl Into<gpui::ElementId>,
    label: String,
    enabled: bool,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let accent = opaque(theme.accent());
    let pressed = tint(accent, theme.foreground(), 0.12);
    div()
        .id(id)
        .h(px(32.0))
        .px(px(18.0))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(12.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .bg(accent)
        .text_color(on_accent(theme))
        .opacity(if enabled { 1.0 } else { 0.45 })
        .when(enabled, |s| {
            s.cursor_pointer().hover(move |s| s.bg(pressed))
        })
        .child(label)
}

pub(crate) fn chip(
    id: impl Into<gpui::ElementId>,
    label: String,
    selected: bool,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let hover = hover(theme);
    div()
        .id(id)
        .h(px(28.0))
        .px(px(14.0))
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
                .text_color(theme.foreground())
                .hover(move |s| s.bg(hover))
        })
        .child(label)
}

pub(crate) fn check_badge(diameter: f32, theme: &Theme) -> gpui::Div {
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

pub(crate) fn radio(selected: bool, diameter: f32, theme: &Theme) -> gpui::Div {
    if selected {
        check_badge(diameter, theme)
    } else {
        div()
            .size(px(diameter))
            .flex_shrink_0()
            .rounded_full()
            .border_2()
            .border_color(theme.foreground_muted())
    }
}

pub(crate) fn search_field(theme: &Theme) -> gpui::Div {
    div()
        .w_full()
        .h(px(40.0))
        .flex_shrink_0()
        .px(px(14.0))
        .rounded_full()
        .bg(surface(theme))
        .border_1()
        .border_color(border(theme))
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            gpui::svg()
                .path("search.svg")
                .size(px(16.0))
                .flex_shrink_0()
                .text_color(theme.foreground_muted()),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui::theme::{Color, ThemeMode};

    #[test]
    fn tint_preserves_base_at_low_strength() {
        let color = gpui::hsla_to_rgba(tint(gpui::black(), gpui::white(), 0.06));
        for channel in [color.red, color.green, color.blue] {
            assert!((channel - 0.06).abs() < 0.0001);
        }
        assert_eq!(color.alpha, 1.0);
    }

    #[test]
    fn accent_text_uses_contrast_instead_of_hsl_lightness() {
        let mut theme = Theme {
            background_color: Color::from("#162c4f"),
            foreground_color: Color::from("#e4efff"),
            accent_color: Color::from("#38d5e8"),
            ..Theme::default()
        };
        assert_eq!(on_accent(&theme), theme.background());
        theme.accent_color = Color::from("#1545a0");
        assert_eq!(on_accent(&theme), theme.foreground());
    }

    #[test]
    fn material_surfaces_are_opaque_in_light_and_dark_themes() {
        for theme in [
            Theme::default(),
            Theme {
                mode: ThemeMode::Light,
                background_color: Color::from("#f1f4f9"),
                surface_color: Color::from("#dce2eb"),
                foreground_color: Color::from("#111827"),
                ..Theme::default()
            },
        ] {
            for color in [
                background(&theme),
                surface(&theme),
                hover(&theme),
                selected(&theme),
                border(&theme),
            ] {
                assert_eq!(color.alpha, 1.0);
            }
            assert_ne!(hover(&theme), surface(&theme));
            assert_ne!(selected(&theme), surface(&theme));
        }
    }
}
