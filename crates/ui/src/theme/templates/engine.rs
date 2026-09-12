use crate::theme::{Theme, ThemeMode};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorFormats {
    pub hex: String,
    pub strip: String,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub rgb: String,
    pub rgba: String,
    pub qt: String,
}

impl ColorFormats {
    pub fn from_hex(raw_hex: &str) -> Self {
        let clean = raw_hex.trim().trim_start_matches('#');
        let (r, g, b) = match clean.len() {
            3 => {
                let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).unwrap_or(0);
                (r, g, b)
            }
            6 | 8 => {
                let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(0);
                (r, g, b)
            }
            _ => (0, 0, 0),
        };

        let hex = format!("#{clean}");
        let strip = clean.to_string();
        let rgb = format!("{r}, {g}, {b}");
        let rgba = format!("rgba({r}, {g}, {b}, 1.0)");
        let lower_clean = clean.to_ascii_lowercase();
        let qt = if lower_clean.len() == 6 {
            format!("#ff{lower_clean}")
        } else if lower_clean.len() == 8 {
            format!("#{lower_clean}")
        } else {
            format!("#ff{lower_clean:0<6}")
        };

        Self {
            hex,
            strip,
            r,
            g,
            b,
            rgb,
            rgba,
            qt,
        }
    }
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn build_context(theme: &Theme) -> HashMap<String, String> {
        let mut ctx = HashMap::new();

        let is_dark = matches!(theme.mode, ThemeMode::Dark);
        let mode_str = if is_dark { "dark" } else { "light" };

        ctx.insert("theme.mode".to_string(), mode_str.to_string());
        ctx.insert("mode".to_string(), mode_str.to_string());
        ctx.insert("theme.is_dark".to_string(), is_dark.to_string());
        ctx.insert("is_dark".to_string(), is_dark.to_string());
        ctx.insert("theme.is_light".to_string(), (!is_dark).to_string());
        ctx.insert("is_light".to_string(), (!is_dark).to_string());

        let font = theme.font_family.clone();
        ctx.insert("theme.font_family".to_string(), font.clone());
        ctx.insert("font_family".to_string(), font.clone());
        ctx.insert("theme.font".to_string(), font.clone());
        ctx.insert("font".to_string(), font);

        let colors = [
            ("background", &theme.background_color.hex, Some("bg")),
            (
                "background_alt",
                &theme.background_color_alt.hex,
                Some("bg_alt"),
            ),
            ("surface", &theme.surface_color.hex, None),
            ("foreground", &theme.foreground_color.hex, Some("fg")),
            (
                "foreground_muted",
                &theme.foreground_color_muted.hex,
                Some("fg_muted"),
            ),
            ("accent", &theme.accent_color.hex, None),
            ("red", &theme.red_color.hex, None),
            ("green", &theme.green_color.hex, None),
        ];

        for (name, raw_hex, alias) in colors {
            let formats = ColorFormats::from_hex(raw_hex);
            Self::insert_color_formats(&mut ctx, name, &formats);
            if let Some(alias_name) = alias {
                Self::insert_color_formats(&mut ctx, alias_name, &formats);
            }
        }

        let bg_alt = &theme.background_color_alt.hex;
        let red = &theme.red_color.hex;
        let green = &theme.green_color.hex;
        let accent = &theme.accent_color.hex;
        let fg_muted = &theme.foreground_color_muted.hex;
        let surface = &theme.surface_color.hex;
        let fg = &theme.foreground_color.hex;

        let palette = [
            bg_alt.as_str(),
            red.as_str(),
            green.as_str(),
            "#f9e2af",
            accent.as_str(),
            "#cba6f7",
            "#89dceb",
            fg_muted.as_str(),
            surface.as_str(),
            red.as_str(),
            green.as_str(),
            "#f9e2af",
            accent.as_str(),
            "#cba6f7",
            "#89dceb",
            fg.as_str(),
        ];

        for (idx, color_hex) in palette.iter().enumerate() {
            let formats = ColorFormats::from_hex(color_hex);
            let key = format!("palette.{idx}");
            Self::insert_color_formats(&mut ctx, &key, &formats);
        }

        ctx
    }

    fn insert_color_formats(ctx: &mut HashMap<String, String>, name: &str, formats: &ColorFormats) {
        ctx.insert(name.to_string(), formats.hex.clone());
        ctx.insert(format!("{name}.hex"), formats.hex.clone());
        ctx.insert(format!("{name}.strip"), formats.strip.clone());
        ctx.insert(format!("{name}.raw"), formats.strip.clone());
        ctx.insert(format!("{name}.rgb"), formats.rgb.clone());
        ctx.insert(format!("{name}.r"), formats.r.to_string());
        ctx.insert(format!("{name}.g"), formats.g.to_string());
        ctx.insert(format!("{name}.b"), formats.b.to_string());
        ctx.insert(format!("{name}.rgba"), formats.rgba.clone());
        ctx.insert(format!("{name}.qt"), formats.qt.clone());

        ctx.insert(format!("theme.{name}"), formats.hex.clone());
        ctx.insert(format!("theme.{name}.hex"), formats.hex.clone());
        ctx.insert(format!("theme.{name}.strip"), formats.strip.clone());
        ctx.insert(format!("theme.{name}.raw"), formats.strip.clone());
        ctx.insert(format!("theme.{name}.rgb"), formats.rgb.clone());
        ctx.insert(format!("theme.{name}.r"), formats.r.to_string());
        ctx.insert(format!("theme.{name}.g"), formats.g.to_string());
        ctx.insert(format!("theme.{name}.b"), formats.b.to_string());
        ctx.insert(format!("theme.{name}.rgba"), formats.rgba.clone());
        ctx.insert(format!("theme.{name}.qt"), formats.qt.clone());
    }

    pub fn render(template: &str, context: &HashMap<String, String>) -> String {
        let after_conditionals = Self::evaluate_conditionals(template, context);
        Self::evaluate_variables(&after_conditionals, context)
    }

    fn evaluate_conditionals(template: &str, context: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(template.len());
        let mut cursor = 0;

        while let Some(start_tag) = template[cursor..].find("{% if ") {
            let actual_start = cursor + start_tag;
            result.push_str(&template[cursor..actual_start]);

            let cond_end = match template[actual_start..].find("%}") {
                Some(idx) => actual_start + idx,
                None => {
                    result.push_str(&template[actual_start..]);
                    return result;
                }
            };

            let cond_expr = template[actual_start + 6..cond_end].trim();
            let after_if = cond_end + 2;

            let end_tag = match template[after_if..].find("{% endif %}") {
                Some(idx) => after_if + idx,
                None => {
                    result.push_str(&template[actual_start..]);
                    return result;
                }
            };

            let block_content = &template[after_if..end_tag];
            let is_true = match context.get(cond_expr) {
                Some(val) => val == "true" || val == "1" || (!val.is_empty() && val != "false"),
                None => false,
            };

            if let Some(else_idx) = block_content.find("{% else %}") {
                let if_branch = &block_content[..else_idx];
                let else_branch = &block_content[else_idx + 10..];
                if is_true {
                    result.push_str(&Self::evaluate_conditionals(if_branch, context));
                } else {
                    result.push_str(&Self::evaluate_conditionals(else_branch, context));
                }
            } else if is_true {
                result.push_str(&Self::evaluate_conditionals(block_content, context));
            }

            cursor = end_tag + 11;
        }

        result.push_str(&template[cursor..]);
        result
    }

    fn evaluate_variables(template: &str, context: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(template.len());
        let mut cursor = 0;

        while let Some(start) = template[cursor..].find("{{") {
            let actual_start = cursor + start;
            result.push_str(&template[cursor..actual_start]);

            let end = match template[actual_start..].find("}}") {
                Some(idx) => actual_start + idx,
                None => {
                    result.push_str(&template[actual_start..]);
                    return result;
                }
            };

            let key = template[actual_start + 2..end].trim();
            if let Some(value) = context.get(key) {
                result.push_str(value);
            } else {
                result.push_str(&template[actual_start..end + 2]);
            }

            cursor = end + 2;
        }

        result.push_str(&template[cursor..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Color;

    #[test]
    fn test_color_formats() {
        let fmt = ColorFormats::from_hex("#007BFF");
        assert_eq!(fmt.hex, "#007BFF");
        assert_eq!(fmt.strip, "007BFF");
        assert_eq!(fmt.r, 0);
        assert_eq!(fmt.g, 123);
        assert_eq!(fmt.b, 255);
        assert_eq!(fmt.rgb, "0, 123, 255");
        assert_eq!(fmt.rgba, "rgba(0, 123, 255, 1.0)");
        assert_eq!(fmt.qt, "#ff007bff");
    }

    #[test]
    fn test_render_variables() {
        let theme = Theme {
            accent_color: Color::from("#FF5500"),
            ..Default::default()
        };
        let ctx = TemplateEngine::build_context(&theme);

        let tpl = "accent={{accent}} raw={{accent.strip}} qt={{accent.qt}}";
        let rendered = TemplateEngine::render(tpl, &ctx);
        assert_eq!(rendered, "accent=#FF5500 raw=FF5500 qt=#ffff5500");
    }

    #[test]
    fn test_render_conditionals() {
        let theme = Theme {
            mode: ThemeMode::Dark,
            ..Default::default()
        };
        let ctx = TemplateEngine::build_context(&theme);

        let tpl = "{% if is_dark %}theme-dark{% else %}theme-light{% endif %}";
        let rendered = TemplateEngine::render(tpl, &ctx);
        assert_eq!(rendered, "theme-dark");

        let light_theme = Theme {
            mode: ThemeMode::Light,
            ..Default::default()
        };
        let light_ctx = TemplateEngine::build_context(&light_theme);
        let light_rendered = TemplateEngine::render(tpl, &light_ctx);
        assert_eq!(light_rendered, "theme-light");
    }

    #[test]
    fn test_palette_rendering() {
        let theme = Theme::default();
        let ctx = TemplateEngine::build_context(&theme);

        let tpl = "c0={{palette.0.strip}} c4={{palette.4}}";
        let rendered = TemplateEngine::render(tpl, &ctx);
        assert!(rendered.contains("c0="));
        assert!(rendered.contains("c4=#"));
    }
}
