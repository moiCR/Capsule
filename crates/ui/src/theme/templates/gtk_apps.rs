use crate::theme::templates::AppTheme;
use crate::theme::{Theme, ThemeMode};
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct GtkApps;

fn update_settings_ini(path: &Path, updates: &[(&str, String)]) {
    let existing = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    let lines: Vec<&str> = existing.lines().collect();
    let settings_idx = lines.iter().position(|l| l.trim() == "[Settings]");

    let mut new_lines: Vec<String> = Vec::new();

    if let Some(sec_idx) = settings_idx {
        let mut in_settings = false;
        let mut inserted = false;

        for (idx, line) in lines.iter().enumerate() {
            if idx == sec_idx {
                in_settings = true;
                new_lines.push((*line).to_string());
                for (k, v) in updates {
                    new_lines.push(format!("{k}={v}"));
                }
                inserted = true;
                continue;
            }
            if in_settings {
                let trimmed = line.trim();
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_settings = false;
                } else if let Some((key, _)) = trimmed.split_once('=') {
                    let key_trimmed = key.trim();
                    if updates.iter().any(|(uk, _)| *uk == key_trimmed) {
                        continue;
                    }
                }
            }
            new_lines.push((*line).to_string());
        }

        if !inserted {
            for (k, v) in updates {
                new_lines.push(format!("{k}={v}"));
            }
        }
    } else {
        if !lines.is_empty() {
            for line in lines {
                new_lines.push(line.to_string());
            }
            if new_lines.last().is_some_and(|l| !l.trim().is_empty()) {
                new_lines.push(String::new());
            }
        }
        new_lines.push("[Settings]".to_string());
        for (k, v) in updates {
            new_lines.push(format!("{k}={v}"));
        }
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, new_lines.join("\n") + "\n");
}

fn update_xsettingsd_conf(
    path: &Path,
    gtk_theme_name: &str,
    color_scheme: &str,
    prefer_dark: bool,
) {
    let existing = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };

    let prefer_dark_num = if prefer_dark { "1" } else { "0" };
    let updates = [
        ("Net/ThemeName", format!("\"{gtk_theme_name}\"")),
        ("Gtk/ColorScheme", format!("\"{color_scheme}\"")),
        (
            "Gtk/ApplicationPreferDarkTheme",
            prefer_dark_num.to_string(),
        ),
    ];

    let mut new_lines: Vec<String> = existing
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !updates.iter().any(|(k, _)| {
                trimmed.starts_with(k)
                    && trimmed[k.len()..]
                        .chars()
                        .next()
                        .is_some_and(char::is_whitespace)
            })
        })
        .map(str::to_string)
        .collect();

    for (k, v) in updates {
        new_lines.push(format!("{k} {v}"));
    }

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, new_lines.join("\n") + "\n");
}

impl AppTheme for GtkApps {
    fn apply_current_theme(&self, theme: &Theme) {
        let is_dark = matches!(theme.mode, ThemeMode::Dark);
        let color_scheme = if is_dark {
            "prefer-dark"
        } else {
            "prefer-light"
        };
        let gtk_theme_name = if is_dark { "adw-gtk3-dark" } else { "adw-gtk3" };
        let prefer_dark_val = if is_dark { "1" } else { "0" };

        let bg = &theme.background_color.hex;
        let bg_alt = &theme.background_color_alt.hex;
        let fg = &theme.foreground_color.hex;
        let accent = &theme.accent_color.hex;

        // 1. Update gsettings for GNOME/GTK desktop & XDG Desktop Portal
        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                color_scheme,
            ])
            .status();

        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "gtk-theme",
                gtk_theme_name,
            ])
            .status();

        // 2. Update dconf directly for portal & GTK live sync
        let _ = Command::new("dconf")
            .args([
                "write",
                "/org/gnome/desktop/interface/color-scheme",
                &format!("'{color_scheme}'"),
            ])
            .status();

        let _ = Command::new("dconf")
            .args([
                "write",
                "/org/gnome/desktop/interface/gtk-theme",
                &format!("'{gtk_theme_name}'"),
            ])
            .status();

        if let Some(config_dir) = dirs::config_dir() {
            let gtk_css = format!(
                r#"@define-color theme_bg_color {bg};
@define-color theme_fg_color {fg};
@define-color theme_text_color {fg};
@define-color theme_selected_bg_color {accent};
@define-color theme_selected_fg_color {bg};
@define-color accent_color {accent};
@define-color accent_bg_color {accent};
@define-color accent_fg_color {bg};
@define-color window_bg_color {bg};
@define-color window_fg_color {fg};
@define-color view_bg_color {bg};
@define-color view_fg_color {fg};
@define-color headerbar_bg_color {bg};
@define-color headerbar_fg_color {fg};
@define-color card_bg_color {bg_alt};
@define-color card_fg_color {fg};
@define-color dialog_bg_color {bg};
@define-color dialog_fg_color {fg};
@define-color popover_bg_color {bg_alt};
@define-color popover_fg_color {fg};
@define-color sidebar_bg_color {bg};
@define-color sidebar_fg_color {fg};
"#
            );

            // GTK 3 settings.ini & gtk.css (preserves existing icon/font/cursor settings)
            let dir_gtk3 = config_dir.join("gtk-3.0");
            let _ = fs::create_dir_all(&dir_gtk3);
            update_settings_ini(
                &dir_gtk3.join("settings.ini"),
                &[
                    ("gtk-theme-name", gtk_theme_name.to_string()),
                    (
                        "gtk-application-prefer-dark-theme",
                        prefer_dark_val.to_string(),
                    ),
                ],
            );
            let _ = fs::write(dir_gtk3.join("gtk.css"), &gtk_css);

            // GTK 4 settings.ini & gtk.css (preserves existing settings, omits deprecated keys)
            let dir_gtk4 = config_dir.join("gtk-4.0");
            let _ = fs::create_dir_all(&dir_gtk4);
            update_settings_ini(
                &dir_gtk4.join("settings.ini"),
                &[("gtk-theme-name", gtk_theme_name.to_string())],
            );
            let _ = fs::write(dir_gtk4.join("gtk.css"), &gtk_css);

            // xsettingsd live sync support for GTK3 apps
            let xsettingsd_conf = config_dir.join("xsettingsd").join("xsettingsd.conf");
            update_xsettingsd_conf(&xsettingsd_conf, gtk_theme_name, color_scheme, is_dark);
        }

        self.reload_apps();
    }

    fn reload_apps(&self) {
        // Reload xsettingsd if running so GTK3 apps update live without nwg-look
        let _ = Command::new("pkill")
            .args(["-HUP", "-x", "xsettingsd"])
            .status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_settings_ini_preserves_existing_keys() {
        let temp_dir = std::env::temp_dir().join("capsule_gtk_ini_test");
        let _ = fs::create_dir_all(&temp_dir);
        let ini_path = temp_dir.join("settings.ini");

        let initial = "[Settings]\ngtk-icon-theme-name=Papirus-Dark\ngtk-font-name=Inter 10\ngtk-theme-name=OldTheme\n";
        let _ = fs::write(&ini_path, initial);

        update_settings_ini(
            &ini_path,
            &[
                ("gtk-theme-name", "adw-gtk3-dark".to_string()),
                ("gtk-application-prefer-dark-theme", "1".to_string()),
            ],
        );

        let updated = fs::read_to_string(&ini_path).unwrap_or_default();
        assert!(updated.contains("[Settings]"));
        assert!(updated.contains("gtk-icon-theme-name=Papirus-Dark"));
        assert!(updated.contains("gtk-font-name=Inter 10"));
        assert!(updated.contains("gtk-theme-name=adw-gtk3-dark"));
        assert!(updated.contains("gtk-application-prefer-dark-theme=1"));
        assert!(!updated.contains("OldTheme"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_xsettingsd_conf_preserves_custom_entries() {
        let temp_dir = std::env::temp_dir().join("capsule_xsettingsd_test");
        let _ = fs::create_dir_all(&temp_dir);
        let conf_path = temp_dir.join("xsettingsd.conf");

        let initial = "Net/IconThemeName \"Papirus-Dark\"\nNet/ThemeName \"Old\"\n";
        let _ = fs::write(&conf_path, initial);

        update_xsettingsd_conf(&conf_path, "adw-gtk3-dark", "prefer-dark", true);

        let updated = fs::read_to_string(&conf_path).unwrap_or_default();
        assert!(updated.contains("Net/IconThemeName \"Papirus-Dark\""));
        assert!(updated.contains("Net/ThemeName \"adw-gtk3-dark\""));
        assert!(updated.contains("Gtk/ColorScheme \"prefer-dark\""));
        assert!(updated.contains("Gtk/ApplicationPreferDarkTheme 1"));
        assert!(!updated.contains("\"Old\""));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
