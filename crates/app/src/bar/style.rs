use gpui::Hsla;
use ui::theme::Theme;

pub const BAR_HEIGHT: f32 = 40.0;
pub const BAR_PADDING_H: f32 = 12.0;
pub const BAR_PADDING_V: f32 = 4.0;
pub const GAP: f32 = 8.0;
pub const WIDGET_HEIGHT: f32 = 32.0;
pub const WIDGET_RADIUS: f32 = 16.0;

fn opaque(mut color: Hsla) -> Hsla {
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

pub fn border(theme: &Theme) -> Hsla {
    tint(
        tint(background(theme), theme.background_alt(), 0.5),
        opaque(theme.background_alt()),
        theme.light_mix(),
    )
}

pub fn hover(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.08)
}

pub fn active(theme: &Theme) -> Hsla {
    tint(surface(theme), theme.accent(), 0.16)
}

pub fn text(theme: &Theme) -> Hsla {
    opaque(theme.foreground())
}

pub fn text_muted(theme: &Theme) -> Hsla {
    opaque(theme.foreground_muted())
}

pub fn accent(theme: &Theme) -> Hsla {
    opaque(theme.accent())
}
