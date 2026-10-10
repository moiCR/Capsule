pub mod templates;
pub mod theme_manager;
use gpui::{Hsla, Rgba, SharedString, rgb_to_hsla};
use serde::{Deserialize, Serialize};
pub use templates::{
    AppTheme, GtkApps, QtApps, TemplateEngine, TemplatePlugin, TemplatePluginManager,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    pub background_color: Color,
    pub background_color_alt: Color,
    pub surface_color: Color,
    pub foreground_color: Color,
    pub foreground_color_muted: Color,
    pub accent_color: Color,
    pub red_color: Color,
    pub green_color: Color,
}

fn default_font_family() -> String {
    "Geist".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            mode: ThemeMode::Dark,
            font_family: "Geist".to_string(),
            background_color: Color::from("#000000"),
            background_color_alt: Color::from("#1A1A1A"),
            surface_color: Color::from("#2D2D2D"),
            foreground_color: Color::from("#FFFFFF"),
            foreground_color_muted: Color::from("#AAAAAA"),
            accent_color: Color::from("#007BFF"),
            red_color: Color::from("#FF0000"),
            green_color: Color::from("#00FF00"),
        }
    }
}

impl Theme {
    pub fn with_mode(mut self, dark: bool) -> Self {
        let mode = if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        if self.mode == mode {
            return self;
        }
        self.mode = mode;
        let colors = if dark {
            ["#09090B", "#18181B", "#27272A", "#FAFAFA", "#A1A1AA"]
        } else {
            ["#FAFAFA", "#F0F0F2", "#E4E4E7", "#18181B", "#71717A"]
        };
        self.background_color = Color::from(colors[0]);
        self.background_color_alt = Color::from(colors[1]);
        self.surface_color = Color::from(colors[2]);
        self.foreground_color = Color::from(colors[3]);
        self.foreground_color_muted = Color::from(colors[4]);
        self
    }

    pub fn from_wallpaper(rgb: [u8; 3], dark: bool, font: String) -> Self {
        let color = rgb_to_hsla(Rgba::new(
            rgb[0] as f32 / 255.0,
            rgb[1] as f32 / 255.0,
            rgb[2] as f32 / 255.0,
            1.0,
        ));
        let to_color = |saturation: f32, lightness: f32| {
            let rgba = gpui::hsla_to_rgba(gpui::hsla(
                color.hue.into_positive_degrees() / 360.0,
                saturation,
                lightness,
                1.0,
            ));
            Color::new(format!(
                "#{:02X}{:02X}{:02X}",
                (rgba.red * 255.0).round() as u8,
                (rgba.green * 255.0).round() as u8,
                (rgba.blue * 255.0).round() as u8
            ))
        };
        let mut theme = Self::default().with_mode(dark);
        theme.font_family = font;
        theme.accent_color = to_color(
            color.saturation.clamp(0.4, 0.8),
            if dark { 0.65 } else { 0.4 },
        );
        theme.background_color = to_color(0.12, if dark { 0.055 } else { 0.98 });
        theme.background_color_alt = to_color(0.1, if dark { 0.09 } else { 0.94 });
        theme.surface_color = to_color(0.12, if dark { 0.16 } else { 0.89 });
        theme
    }

    pub fn font_family(&self) -> SharedString {
        if self.font_family.is_empty() {
            SharedString::from("Geist")
        } else {
            SharedString::from(self.font_family.clone())
        }
    }
    pub fn background(&self) -> Hsla {
        self.background_color.to_hsla()
    }

    pub fn background_alt(&self) -> Hsla {
        self.background_color_alt.to_hsla()
    }

    pub fn surface(&self) -> Hsla {
        self.surface_color.to_hsla()
    }

    pub fn foreground(&self) -> Hsla {
        self.foreground_color.to_hsla()
    }

    pub fn foreground_muted(&self) -> Hsla {
        self.foreground_color_muted.to_hsla()
    }

    pub fn accent(&self) -> Hsla {
        self.accent_color.to_hsla()
    }

    pub fn red(&self) -> Hsla {
        self.red_color.to_hsla()
    }

    pub fn green(&self) -> Hsla {
        self.green_color.to_hsla()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Color {
    pub hex: String,
}

impl Color {
    pub fn new(hex: impl Into<String>) -> Self {
        Self { hex: hex.into() }
    }

    pub fn to_hsla(&self) -> Hsla {
        parse_hex_to_hsla(&self.hex)
    }
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Color::new(s)
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Color::new(s)
    }
}

pub fn parse_hex_to_hsla(hex: &str) -> Hsla {
    let hex = hex.trim_start_matches('#');
    let (r, g, b, a) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            (r, g, b, 255)
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(255);
            (r, g, b, a)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            (r, g, b, 255)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            (r, g, b, a)
        }
        _ => (0, 0, 0, 255),
    };

    rgb_to_hsla(Rgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

impl gpui::Global for Theme {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_mode_is_independent_of_the_selected_palette() {
        let theme = Theme {
            accent_color: Color::from("#B23388"),
            ..Theme::default()
        };
        let light = theme.clone().with_mode(false);
        assert_eq!(light.mode, ThemeMode::Light);
        assert_eq!(light.accent_color, theme.accent_color);
        assert_ne!(light.background_color, theme.background_color);
        let dark = light.with_mode(true);
        assert_eq!(dark.mode, ThemeMode::Dark);
        assert_eq!(dark.accent_color, theme.accent_color);
    }

    #[test]
    fn dynamic_palette_changes_with_wallpaper_and_respects_mode() {
        let warm = Theme::from_wallpaper([210, 70, 40], true, "Geist".into());
        let cool = Theme::from_wallpaper([40, 70, 210], true, "Geist".into());
        let light = Theme::from_wallpaper([210, 70, 40], false, "Geist".into());
        assert_ne!(warm.accent_color, cool.accent_color);
        assert_eq!(warm.mode, ThemeMode::Dark);
        assert_eq!(light.mode, ThemeMode::Light);
        assert!(warm.background().lightness < warm.foreground().lightness);
        assert!(light.background().lightness > light.foreground().lightness);
    }
}
