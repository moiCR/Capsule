use gpui::{Hsla, Styled};
pub use ui::theme::{SHAPE_LG, SHAPE_MD, SHAPE_SM, SHAPE_XL, SHAPE_XS, Theme, opaque, tint};

pub const BAR_HEIGHT: f32 = 40.0;
pub const BAR_PADDING_H: f32 = 12.0;
pub const BAR_PADDING_V: f32 = 4.0;
pub const GAP: f32 = 8.0;
pub const WIDGET_HEIGHT: f32 = 32.0;
pub const WIDGET_RADIUS: f32 = 16.0;

pub fn background(theme: &Theme) -> Hsla {
    opaque(theme.background())
}

pub fn surface(theme: &Theme) -> Hsla {
    theme.surface_container()
}

pub fn surface_high(theme: &Theme) -> Hsla {
    theme.surface_container_high()
}

pub fn surface_highest(theme: &Theme) -> Hsla {
    theme.surface_container_highest()
}

pub fn border(theme: &Theme) -> Hsla {
    theme.outline_variant()
}

pub fn outline(theme: &Theme) -> Hsla {
    theme.outline()
}

pub fn hover(theme: &Theme) -> Hsla {
    theme.hover_layer(surface(theme))
}

pub fn active(theme: &Theme) -> Hsla {
    theme.primary_container()
}

pub fn on_active(theme: &Theme) -> Hsla {
    theme.on_primary_container()
}

pub fn text(theme: &Theme) -> Hsla {
    opaque(theme.foreground())
}

pub fn text_muted(theme: &Theme) -> Hsla {
    opaque(theme.foreground_muted())
}

pub fn accent(theme: &Theme) -> Hsla {
    opaque(theme.primary())
}

pub fn on_accent(theme: &Theme) -> Hsla {
    opaque(theme.on_primary())
}

pub fn card(theme: &Theme, radius: f32) -> gpui::Div {
    gpui::div().rounded(gpui::px(radius)).bg(surface(theme))
}
