#![allow(dead_code)]

use gpui::{Bounds, Hsla, PathBuilder, Pixels, Size, div, point, prelude::*, px};
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

pub(crate) fn concave_radii(size: Size<Pixels>, radius: f32) -> (f32, f32) {
    let width = f32::from(size.width).max(0.0);
    let height = f32::from(size.height).max(0.0);
    let radius = radius.max(0.0).min(width / 2.0).min(height / 2.0);
    let shoulder = 12.0_f32.min(height / 4.0).min(width / 4.0);
    (radius, shoulder)
}

pub(crate) fn concave_background(
    size: Size<Pixels>,
    radius: f32,
    bottom: bool,
    color: Hsla,
) -> impl IntoElement {
    let (radius, shoulder) = concave_radii(size, radius);
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let width = f32::from(size.width);
            let height = f32::from(size.height);
            let p = |x, y| {
                bounds.origin + point(px(x + shoulder), px(if bottom { height - y } else { y }))
            };
            let k = 0.552_284_8;
            let mut path = PathBuilder::fill();
            path.move_to(p(-shoulder, 0.0));
            path.line_to(p(width + shoulder, 0.0));
            path.cubic_bezier_to(
                p(width, shoulder),
                p(width + shoulder * (1.0 - k), 0.0),
                p(width, shoulder * (1.0 - k)),
            );
            path.line_to(p(width, height - radius));
            path.cubic_bezier_to(
                p(width - radius, height),
                p(width, height - radius * (1.0 - k)),
                p(width - radius * (1.0 - k), height),
            );
            path.line_to(p(radius, height));
            path.cubic_bezier_to(
                p(0.0, height - radius),
                p(radius * (1.0 - k), height),
                p(0.0, height - radius * (1.0 - k)),
            );
            path.line_to(p(0.0, shoulder));
            path.cubic_bezier_to(
                p(-shoulder, 0.0),
                p(0.0, shoulder * (1.0 - k)),
                p(-shoulder * (1.0 - k), 0.0),
            );
            path.close();
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .absolute()
    .top_0()
    .left(px(-shoulder))
    .w(size.width + px(shoulder * 2.0))
    .h(size.height)
}

pub(crate) fn concave_input_regions(
    bounds: Bounds<Pixels>,
    radius: f32,
    bottom: bool,
) -> Vec<Bounds<Pixels>> {
    let (radius, shoulder) = concave_radii(bounds.size, radius);
    let height = f32::from(bounds.size.height);
    let mut regions = Vec::new();
    let mut y = 0.0;
    while y < height {
        let band_height = if y >= shoulder && y < height - radius {
            height - radius - y
        } else {
            (height - y).min(1.0)
        };
        let sample = y + band_height / 2.0;
        let inset = if sample < shoulder {
            -shoulder
                + (shoulder * shoulder - (sample - shoulder).powi(2))
                    .max(0.0)
                    .sqrt()
        } else if sample > height - radius {
            radius
                - (radius * radius - (sample - height + radius).powi(2))
                    .max(0.0)
                    .sqrt()
        } else {
            0.0
        };
        regions.push(Bounds {
            origin: bounds.origin
                + point(
                    px(inset),
                    px(if bottom { height - y - band_height } else { y }),
                ),
            size: gpui::size(bounds.size.width - px(inset * 2.0), px(band_height)),
        });
        y += band_height;
    }
    regions
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
    fn concave_curves_fit_compact_and_expanded_capsules() {
        for width in [1.0, 40.0, 250.0, 560.0] {
            for height in [1.0, 25.0, 40.0, 212.0, 562.0] {
                for configured_radius in [0.0, 12.0, 24.0, 100.0, 1000.0] {
                    let size = gpui::size(px(width), px(height));
                    let (radius, shoulder) = concave_radii(size, configured_radius);
                    assert!(radius + shoulder <= height);
                    assert!(radius * 2.0 <= width);
                    assert!(shoulder <= 12.0);
                    assert_eq!(shoulder, concave_radii(size, 0.0).1);
                    assert_eq!(shoulder, concave_radii(size, 1000.0).1);
                }
            }
        }
    }

    #[test]
    fn concave_input_regions_follow_the_outline_on_both_edges() {
        let bounds = Bounds {
            origin: point(px(100.0), px(50.0)),
            size: gpui::size(px(250.0), px(40.0)),
        };
        let top = concave_input_regions(bounds, 1000.0, false);
        let bottom = concave_input_regions(bounds, 1000.0, true);
        assert_eq!(top.len(), bottom.len());
        assert!(top[0].origin.x < bounds.origin.x);
        assert!(top[top.len() - 1].origin.x > bounds.origin.x);
        assert!(top.len() < 40);
        let mut covered_height = px(0.0);
        for (top, bottom) in top.iter().zip(&bottom) {
            assert!(top.size.width > px(0.0));
            assert!(top.size.height > px(0.0));
            assert_eq!(top.size, bottom.size);
            assert_eq!(top.origin.x, bottom.origin.x);
            assert_eq!(
                bottom.origin.y,
                bounds.origin.y + bounds.size.height
                    - (top.origin.y - bounds.origin.y)
                    - top.size.height,
            );
            assert_eq!(top.origin.y, bounds.origin.y + covered_height);
            covered_height += top.size.height;
        }
        assert_eq!(covered_height, bounds.size.height);
    }

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
