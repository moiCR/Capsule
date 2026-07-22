pub mod theme_manager;
use gpui::{Hsla, Rgba};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: ThemeMode,
    pub background_color: Color,
    pub background_color_alt: Color,
    pub surface_color: Color,
    pub foreground_color: Color,
    pub foreground_color_muted: Color,
    pub accent_color: Color,
    pub red_color: Color,
    pub green_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            mode: ThemeMode::Dark,
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

    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    }
    .into()
}

impl gpui::Global for Theme {}
