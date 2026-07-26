use crate::theme::templates::AppTheme;
use crate::theme::{Theme, ThemeMode};
use std::fs;
use std::process::Command;

pub struct GtkApps;

impl AppTheme for GtkApps {
    fn apply_current_theme(theme: &Theme) {
        let is_dark = matches!(theme.mode, ThemeMode::Dark);
        let color_scheme = if is_dark {
            "prefer-dark"
        } else {
            "prefer-light"
        };
        let gtk_theme_name = if is_dark { "Adwaita-dark" } else { "Adwaita" };
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
            // GTK 3 settings.ini
            let ini_gtk3 = format!(
                "[Settings]\ngtk-theme-name={gtk_theme_name}\ngtk-application-prefer-dark-theme={prefer_dark_val}\n"
            );

            // GTK 4 settings.ini (Omit deprecation keys to prevent Libadwaita warnings)
            let ini_gtk4 = format!("[Settings]\ngtk-theme-name={gtk_theme_name}\n");

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

            // 3. Write GTK 3.0 & GTK 4.0 configurations
            let dir_gtk3 = config_dir.join("gtk-3.0");
            let _ = fs::create_dir_all(&dir_gtk3);
            let _ = fs::write(dir_gtk3.join("settings.ini"), &ini_gtk3);
            let _ = fs::write(dir_gtk3.join("gtk.css"), &gtk_css);

            let dir_gtk4 = config_dir.join("gtk-4.0");
            let _ = fs::create_dir_all(&dir_gtk4);
            let _ = fs::write(dir_gtk4.join("settings.ini"), &ini_gtk4);
            let _ = fs::write(dir_gtk4.join("gtk.css"), &gtk_css);

            // 4. Update nwg-look configuration file if present
            let nwg_gsettings_file = config_dir.join("nwg-look").join("gsettings");
            if nwg_gsettings_file.exists() {
                if let Ok(content) = fs::read_to_string(&nwg_gsettings_file) {
                    let mut updated_lines = Vec::new();
                    for line in content.lines() {
                        if line.starts_with("gtk-theme:") {
                            updated_lines.push(format!("gtk-theme: {gtk_theme_name}"));
                        } else if line.starts_with("color-scheme:") {
                            updated_lines.push(format!("color-scheme: {color_scheme}"));
                        } else {
                            updated_lines.push(line.to_string());
                        }
                    }
                    let _ = fs::write(&nwg_gsettings_file, updated_lines.join("\n"));
                }
            }
        }
        Self::reload_apps();
    }

    fn reload_apps() {
        let _ = Command::new("nwg-look").arg("-a").status();

        let _ = Command::new("systemctl")
            .args(["--user", "restart", "xdg-desktop-portal"])
            .status();

        let _ = Command::new("systemctl")
            .args(["--user", "restart", "xdg-desktop-portal-gtk"])
            .status();

        let _ = Command::new("systemctl")
            .args(["--user", "restart", "xdg-desktop-portal-hyprland"])
            .status();

        let _ = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .status();

        let current_theme = Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "'Adwaita-dark'".to_string());

        let fallback_theme = if current_theme.contains("dark") {
            "Adwaita"
        } else {
            "Adwaita-dark"
        };

        let target_theme = current_theme.trim_matches('\'');

        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "gtk-theme",
                fallback_theme,
            ])
            .status();

        let _ = Command::new("gsettings")
            .args([
                "set",
                "org.gnome.desktop.interface",
                "gtk-theme",
                target_theme,
            ])
            .status();
    }
}
