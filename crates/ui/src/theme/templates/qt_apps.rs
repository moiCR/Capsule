use crate::theme::templates::AppTheme;
use crate::theme::{Theme, ThemeMode};
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

impl AppTheme for QtApps {
    fn apply_current_theme(theme: &Theme) {
        let is_dark = matches!(theme.mode, ThemeMode::Dark);

        let bg = &theme.background_color.hex;
        let bg_alt = &theme.background_color_alt.hex;
        let surface = &theme.surface_color.hex;
        let fg = &theme.foreground_color.hex;
        let fg_muted = &theme.foreground_color_muted.hex;
        let accent = &theme.accent_color.hex;

        let bg_rgb = hex_to_rgb_str(bg);
        let bg_alt_rgb = hex_to_rgb_str(bg_alt);
        let surface_rgb = hex_to_rgb_str(surface);
        let fg_rgb = hex_to_rgb_str(fg);
        let fg_muted_rgb = hex_to_rgb_str(fg_muted);
        let accent_rgb = hex_to_rgb_str(accent);

        // 1. Write qt5ct and qt6ct custom.conf
        let active_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, false);
        let inactive_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, false);
        let disabled_list = make_color_list(bg, bg_alt, fg, fg_muted, accent, true);

        let qt_scheme = format!(
            "[ColorScheme]\nactive_colors={active_list}\ndisabled_colors={disabled_list}\ninactive_colors={inactive_list}\n"
        );

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

            // 2. Write KDE kdeglobals using proper R,G,B comma-separated values
            let kdeglobals_path = config_dir.join("kdeglobals");
            let scheme_name = if is_dark { "BreezeDark" } else { "BreezeLight" };

            let kde_content = format!(
                r#"[General]
ColorScheme={scheme_name}
style=Fusion

[KDE]
colorScheme={scheme_name}

[Colors:Window]
BackgroundNormal={bg_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:View]
BackgroundNormal={bg_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:Button]
BackgroundNormal={surface_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:Selection]
BackgroundNormal={accent_rgb}
ForegroundNormal={bg_rgb}

[Colors:Tooltip]
BackgroundNormal={bg_alt_rgb}
ForegroundNormal={fg_rgb}

[WM]
activeBackground={bg_rgb}
activeForeground={fg_rgb}
inactiveBackground={bg_alt_rgb}
inactiveForeground={fg_muted_rgb}
"#
            );
            let _ = fs::write(kdeglobals_path, kde_content);
        }

        // 3. Write KDE color-scheme file in ~/.local/share/color-schemes/Capsule.colors for Dolphin & KDE apps
        if let Some(data_dir) = dirs::data_dir() {
            let color_schemes_dir = data_dir.join("color-schemes");
            let _ = fs::create_dir_all(&color_schemes_dir);
            let capsule_colors = format!(
                r#"[Base]
Name=Capsule
Scheme=Capsule

[Colors:Window]
BackgroundNormal={bg_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:View]
BackgroundNormal={bg_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:Button]
BackgroundNormal={surface_rgb}
ForegroundNormal={fg_rgb}
BackgroundAlternate={bg_alt_rgb}

[Colors:Selection]
BackgroundNormal={accent_rgb}
ForegroundNormal={bg_rgb}

[Colors:Tooltip]
BackgroundNormal={bg_alt_rgb}
ForegroundNormal={fg_rgb}

[WM]
activeBackground={bg_rgb}
activeForeground={fg_rgb}
inactiveBackground={bg_alt_rgb}
inactiveForeground={fg_muted_rgb}
"#
            );
            let _ = fs::write(color_schemes_dir.join("Capsule.colors"), capsule_colors);
        }

        Self::reload_apps();
    }

    fn reload_apps() {
        let _ = Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.kde.KGlobalSettings",
                "/KGlobalSettings",
                "org.kde.KGlobalSettings.notifyChange",
                "int32:0",
                "int32:0",
            ])
            .status();

        let _ = Command::new("dbus-send")
            .args([
                "--session",
                "--type=signal",
                "/KGlobalSettings",
                "org.kde.KGlobalSettings.notifyChange",
                "int32:0",
                "int32:0",
            ])
            .status();

        let _ = Command::new("qdbus")
            .args([
                "org.kde.KGlobalSettings",
                "/KGlobalSettings",
                "notifyChange",
                "0",
                "0",
            ])
            .status();

        let _ = Command::new("kreadconfig6")
            .args([
                "--file",
                "kdeglobals",
                "--group",
                "General",
                "--key",
                "ColorScheme",
            ])
            .status();
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
