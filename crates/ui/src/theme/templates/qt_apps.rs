use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct QtApps;

fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    } else {
        (0, 0, 0)
    }
}

fn hex_to_rgb_str(hex: &str) -> String {
    let (r, g, b) = hex_to_rgb(hex);
    format!("{r},{g},{b}")
}

fn to_qt_hex(hex: &str) -> String {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        format!("#ff{clean}")
    } else if clean.len() == 8 {
        format!("#{clean}")
    } else {
        "#ffffffff".to_string()
    }
}

fn make_color_list(
    bg: &str,
    bg_alt: &str,
    fg: &str,
    fg_muted: &str,
    accent: &str,
    disabled: bool,
) -> String {
    let fg_eff = if disabled { fg_muted } else { fg };
    let accent_eff = if disabled { fg_muted } else { accent };

    let items = vec![
        fg_eff,      // 0: WindowText
        bg,          // 1: Button
        bg_alt,      // 2: Light
        bg_alt,      // 3: Midlight
        bg,          // 4: Dark
        bg_alt,      // 5: Mid
        fg_eff,      // 6: Text
        fg_eff,      // 7: BrightText
        fg_eff,      // 8: ButtonText
        bg,          // 9: Base
        bg,          // 10: Window
        "#ff000000", // 11: Shadow
        accent_eff,  // 12: Highlight
        bg,          // 13: HighlightedText
        accent_eff,  // 14: Link
        accent_eff,  // 15: LinkVisited
        bg_alt,      // 16: AlternateBase
        "#ff000000", // 17: NoRole
        bg,          // 18: ToolTipBase
        fg_eff,      // 19: ToolTipText
        fg_muted,    // 20: PlaceholderText
        accent_eff,  // 21: Accent
    ];

    items
        .into_iter()
        .map(to_qt_hex)
        .collect::<Vec<_>>()
        .join(", ")
}

fn upsert_ini_section(content: &str, section_header: &str, keys: &[(&str, String)]) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let header_trimmed = section_header.trim();

    let mut intervals = Vec::new();
    let mut current_start = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some(start) = current_start.take() {
                intervals.push((start, idx));
            }
            if trimmed == header_trimmed {
                current_start = Some(idx);
            }
        }
    }
    if let Some(start) = current_start {
        intervals.push((start, lines.len()));
    }

    if let Some(&(first_start, first_end)) = intervals.first() {
        let mut preserved = Vec::new();
        for line in &lines[first_start + 1..first_end] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some((k, _)) = trimmed.split_once('=') {
                let k_trim = k.trim();
                if keys.iter().any(|(uk, _)| *uk == k_trim) {
                    continue;
                }
            }
            preserved.push(*line);
        }

        let mut new_lines: Vec<String> = Vec::new();
        let mut idx = 0;
        while idx < lines.len() {
            if idx == first_start {
                new_lines.push(header_trimmed.to_string());
                for line in &preserved {
                    new_lines.push((*line).to_string());
                }
                for (k, v) in keys {
                    new_lines.push(format!("{k}={v}"));
                }
                if lines
                    .get(first_end)
                    .is_some_and(|l| l.trim().starts_with('['))
                {
                    new_lines.push(String::new());
                }
                idx = first_end;
            } else if let Some(&(_, end)) = intervals
                .get(1..)
                .unwrap_or(&[])
                .iter()
                .find(|&&(s, e)| idx >= s && idx < e)
            {
                idx = end;
            } else {
                new_lines.push(lines[idx].to_string());
                idx += 1;
            }
        }
        new_lines.join("\n") + "\n"
    } else {
        let mut out = content.trim_end().to_string();
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(header_trimmed);
        out.push('\n');
        for (k, v) in keys {
            out.push_str(&format!("{k}={v}\n"));
        }
        out
    }
}

fn upsert_ini_sections(path: &Path, sections: &[(&str, Vec<(&str, String)>)]) {
    let mut content = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    for (header, keys) in sections {
        content = upsert_ini_section(&content, header, keys);
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, content);
}

fn build_kde_color_sections(theme: &Theme) -> Vec<(&'static str, Vec<(&'static str, String)>)> {
    let bg_rgb = hex_to_rgb_str(&theme.background_color.hex);
    let bg_alt_rgb = hex_to_rgb_str(&theme.background_color_alt.hex);
    let surface_rgb = hex_to_rgb_str(&theme.surface_color.hex);
    let fg_rgb = hex_to_rgb_str(&theme.foreground_color.hex);
    let fg_muted_rgb = hex_to_rgb_str(&theme.foreground_color_muted.hex);
    let accent_rgb = hex_to_rgb_str(&theme.accent_color.hex);
    let red_rgb = hex_to_rgb_str(&theme.red_color.hex);
    let green_rgb = hex_to_rgb_str(&theme.green_color.hex);

    let common_colors = |normal_bg: &str, alt_bg: &str| {
        vec![
            ("BackgroundNormal", normal_bg.to_string()),
            ("BackgroundAlternate", alt_bg.to_string()),
            ("ForegroundNormal", fg_rgb.clone()),
            ("ForegroundInactive", fg_muted_rgb.clone()),
            ("ForegroundActive", accent_rgb.clone()),
            ("ForegroundLink", accent_rgb.clone()),
            ("ForegroundVisited", accent_rgb.clone()),
            ("ForegroundNegative", red_rgb.clone()),
            ("ForegroundNeutral", accent_rgb.clone()),
            ("ForegroundPositive", green_rgb.clone()),
            ("DecorationFocus", accent_rgb.clone()),
            ("DecorationHover", accent_rgb.clone()),
        ]
    };

    vec![
        ("[Colors:Window]", common_colors(&bg_rgb, &bg_alt_rgb)),
        ("[Colors:View]", common_colors(&bg_rgb, &bg_alt_rgb)),
        ("[Colors:Button]", common_colors(&surface_rgb, &bg_alt_rgb)),
        (
            "[Colors:Selection]",
            vec![
                ("BackgroundNormal", accent_rgb.clone()),
                ("BackgroundAlternate", accent_rgb.clone()),
                ("ForegroundNormal", bg_rgb.clone()),
                ("ForegroundInactive", fg_muted_rgb.clone()),
                ("ForegroundActive", bg_rgb.clone()),
                ("ForegroundLink", bg_rgb.clone()),
                ("ForegroundVisited", bg_rgb.clone()),
                ("ForegroundNegative", red_rgb.clone()),
                ("ForegroundNeutral", bg_rgb.clone()),
                ("ForegroundPositive", green_rgb.clone()),
                ("DecorationFocus", accent_rgb.clone()),
                ("DecorationHover", accent_rgb.clone()),
            ],
        ),
        ("[Colors:Tooltip]", common_colors(&bg_alt_rgb, &bg_rgb)),
        ("[Colors:Header]", common_colors(&bg_rgb, &bg_alt_rgb)),
        (
            "[Colors:Header][Inactive]",
            common_colors(&bg_alt_rgb, &bg_rgb),
        ),
        (
            "[Colors:Complementary]",
            common_colors(&bg_alt_rgb, &bg_rgb),
        ),
        (
            "[WM]",
            vec![
                ("activeBackground", bg_rgb.clone()),
                ("activeForeground", fg_rgb.clone()),
                ("inactiveBackground", bg_alt_rgb.clone()),
                ("inactiveForeground", fg_muted_rgb.clone()),
                ("activeBlend", accent_rgb),
                ("inactiveBlend", bg_alt_rgb),
            ],
        ),
    ]
}

fn format_capsule_colors_file(theme: &Theme) -> String {
    let mut sections = vec![(
        "[General]",
        vec![
            ("ColorScheme", "Capsule".to_string()),
            ("Name", "Capsule".to_string()),
            ("shadeSortColumn", "true".to_string()),
        ],
    )];
    sections.extend(build_kde_color_sections(theme));

    let mut content = String::new();
    for (header, keys) in &sections {
        content = upsert_ini_section(&content, header, keys);
    }
    content
}

impl AppTheme for QtApps {
    fn apply_current_theme(&self, theme: &Theme) {
        let bg = &theme.background_color.hex;
        let bg_alt = &theme.background_color_alt.hex;
        let fg = &theme.foreground_color.hex;
        let fg_muted = &theme.foreground_color_muted.hex;
        let accent = &theme.accent_color.hex;

        // 1. Write qt5ct and qt6ct custom.conf
        let active_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, false);
        let inactive_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, false);
        let disabled_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, true);

        let qt_scheme = format!(
            "[ColorScheme]\nactive_colors={active_list}\ndisabled_colors={disabled_list}\ninactive_colors={inactive_list}\n"
        );

        let capsule_colors = format_capsule_colors_file(theme);

        // 2. Write KDE color-scheme file in ~/.local/share/color-schemes/Capsule.colors for Dolphin & KDE apps
        if let Some(data_dir) = dirs::data_dir() {
            let color_schemes_dir = data_dir.join("color-schemes");
            let _ = fs::create_dir_all(&color_schemes_dir);
            let _ = fs::write(color_schemes_dir.join("Capsule.colors"), &capsule_colors);
        }

        if let Some(config_dir) = dirs::config_dir() {
            for version in ["qt5ct", "qt6ct"] {
                let colors_dir = config_dir.join(version).join("colors");
                let _ = fs::create_dir_all(&colors_dir);
                let _ = fs::write(colors_dir.join("custom.conf"), &qt_scheme);

                let conf_path = config_dir.join(version).join(format!("{version}.conf"));
                let custom_conf_file = colors_dir.join("custom.conf");
                let custom_conf_str = custom_conf_file.to_string_lossy();

                update_qtct_conf(&conf_path, &custom_conf_str);
            }

            // 3. Non-destructively update ~/.config/kdeglobals with [General], [UiSettings], [KDE], and color groups
            let kdeglobals_path = config_dir.join("kdeglobals");
            let mut kde_sections = vec![
                (
                    "[General]",
                    vec![
                        ("ColorScheme", "Capsule".to_string()),
                        ("Name", "Capsule".to_string()),
                        ("style", "Fusion".to_string()),
                        ("shadeSortColumn", "true".to_string()),
                    ],
                ),
                ("[UiSettings]", vec![("ColorScheme", "Capsule".to_string())]),
                (
                    "[KDE]",
                    vec![
                        ("colorScheme", "Capsule".to_string()),
                        ("widgetStyle", "Fusion".to_string()),
                    ],
                ),
            ];
            kde_sections.extend(build_kde_color_sections(theme));
            upsert_ini_sections(&kdeglobals_path, &kde_sections);

            // 4. Non-destructively update ~/.config/dolphinrc so Dolphin uses Capsule instead of falling back to Breeze
            let dolphinrc_path = config_dir.join("dolphinrc");
            upsert_ini_sections(
                &dolphinrc_path,
                &[(
                    "[UiSettings]",
                    vec![("ColorScheme", "Capsule".to_string())],
                )],
            );
        }

        self.reload_apps();
    }

    fn reload_apps(&self) {
        for (file, group) in [
            ("kdeglobals", "General"),
            ("kdeglobals", "UiSettings"),
            ("dolphinrc", "UiSettings"),
        ] {
            let _ = Command::new("kwriteconfig6")
                .args([
                    "--file",
                    file,
                    "--group",
                    group,
                    "--key",
                    "ColorScheme",
                    "--notify",
                    "Capsule",
                ])
                .status();
        }
    }
}

fn update_qtct_conf(conf_path: &Path, color_scheme_path: &str) {
    let lines: Vec<String> = if conf_path.exists() {
        fs::read_to_string(conf_path)
            .unwrap_or_default()
            .lines()
            .map(|s| s.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let mut appearance_idx = None;
    for (idx, line) in lines.iter().enumerate() {
        if line.trim() == "[Appearance]" {
            appearance_idx = Some(idx);
            break;
        }
    }

    let keys = vec![
        format!("color_scheme_path={color_scheme_path}"),
        "custom_palette=true".to_string(),
        "style=Fusion".to_string(),
    ];

    let mut new_lines = Vec::new();

    if let Some(app_idx) = appearance_idx {
        let mut in_appearance = false;
        for (idx, line) in lines.into_iter().enumerate() {
            if idx == app_idx {
                in_appearance = true;
                new_lines.push(line);
                continue;
            }
            if in_appearance {
                if line.trim().starts_with('[') {
                    in_appearance = false;
                } else if line.starts_with("color_scheme_path=")
                    || line.starts_with("custom_palette=")
                    || line.starts_with("style=")
                {
                    continue;
                }
            }
            new_lines.push(line);
        }

        let new_app_idx = new_lines
            .iter()
            .position(|l| l.trim() == "[Appearance]")
            .unwrap_or(0);

        for (offset, key_line) in keys.into_iter().enumerate() {
            new_lines.insert(new_app_idx + 1 + offset, key_line);
        }
    } else {
        new_lines.push("[Appearance]".to_string());
        for key_line in keys {
            new_lines.push(key_line);
        }
    }

    if let Some(parent) = conf_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(conf_path, new_lines.join("\n") + "\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capsule_colors_uses_general_header_and_valid_keys() {
        let theme = Theme::default();
        let content = format_capsule_colors_file(&theme);

        assert!(content.starts_with("[General]\n"));
        assert!(!content.contains("[Base]"));
        assert!(content.contains("ColorScheme=Capsule"));
        assert!(content.contains("Name=Capsule"));
        assert!(content.contains("[Colors:Window]"));
        assert!(content.contains("[Colors:View]"));
        assert!(content.contains("[Colors:Header]"));
        assert!(content.contains("[WM]"));
    }

    #[test]
    fn test_upsert_ini_sections_preserves_existing_config_and_injects_uisettings() {
        let temp_dir = std::env::temp_dir().join("capsule_kde_ini_test");
        let _ = fs::create_dir_all(&temp_dir);
        let dolphinrc = temp_dir.join("dolphinrc");

        let initial = "[General]\nVersion=202\n\n[MainWindow]\nMenuBar=Disabled\n";
        let _ = fs::write(&dolphinrc, initial);

        upsert_ini_sections(
            &dolphinrc,
            &[(
                "[UiSettings]",
                vec![("ColorScheme", "Capsule".to_string())],
            )],
        );

        let updated = fs::read_to_string(&dolphinrc).unwrap_or_default();
        assert!(updated.contains("[General]"));
        assert!(updated.contains("Version=202"));
        assert!(updated.contains("[MainWindow]"));
        assert!(updated.contains("MenuBar=Disabled"));
        assert!(updated.contains("[UiSettings]"));
        assert!(updated.contains("ColorScheme=Capsule"));

        // Second pass updates existing UiSettings without duplicating the section
        upsert_ini_sections(
            &dolphinrc,
            &[(
                "[UiSettings]",
                vec![("ColorScheme", "Capsule".to_string())],
            )],
        );
        let updated_twice = fs::read_to_string(&dolphinrc).unwrap_or_default();
        assert_eq!(updated_twice.matches("[UiSettings]").count(), 1);
        assert_eq!(updated_twice.matches("ColorScheme=Capsule").count(), 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
