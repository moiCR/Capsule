use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

pub struct GhosttyApp;

impl AppTheme for GhosttyApp {
    fn apply_current_theme(theme: &Theme) {
        let bg = &theme.background_color.hex;
        let bg_alt = &theme.background_color_alt.hex;
        let surface = &theme.surface_color.hex;
        let fg = &theme.foreground_color.hex;
        let fg_muted = &theme.foreground_color_muted.hex;
        let accent = &theme.accent_color.hex;
        let red = &theme.red_color.hex;
        let green = &theme.green_color.hex;

        let mut palette_colors = BTreeMap::new();
        palette_colors.insert(0, bg_alt.as_str());
        palette_colors.insert(1, red.as_str());
        palette_colors.insert(2, green.as_str());
        palette_colors.insert(3, "#f9e2af");
        palette_colors.insert(4, accent.as_str());
        palette_colors.insert(5, "#cba6f7");
        palette_colors.insert(6, "#89dceb");
        palette_colors.insert(7, fg_muted.as_str());
        palette_colors.insert(8, surface.as_str());
        palette_colors.insert(9, red.as_str());
        palette_colors.insert(10, green.as_str());
        palette_colors.insert(11, "#f9e2af");
        palette_colors.insert(12, accent.as_str());
        palette_colors.insert(13, "#cba6f7");
        palette_colors.insert(14, "#89dceb");
        palette_colors.insert(15, fg.as_str());

        let mut ghostty_config = format!("background = {bg}\nforeground = {fg}\n");
        for (index, color) in palette_colors {
            ghostty_config.push_str(&format!("palette = {index}={color}\n"));
        }

        if let Some(config_dir) = dirs::config_dir() {
            let ghostty_dir = config_dir.join("ghostty");
            let _ = fs::create_dir_all(&ghostty_dir);

            // Write theme file
            let theme_file = ghostty_dir.join("theme");
            let _ = fs::write(&theme_file, &ghostty_config);

            // Ensure config file includes config-file = theme
            let main_config = ghostty_dir.join("config");
            let existing_content = if main_config.exists() {
                fs::read_to_string(&main_config).unwrap_or_default()
            } else {
                String::new()
            };

            if !existing_content.contains("config-file = theme")
                && !existing_content.contains("config-file=theme")
            {
                let new_content = format!("config-file = theme\n{existing_content}");
                let _ = fs::write(&main_config, new_content);
            } else {
                // Touch main_config to trigger Ghostty's file watcher on config
                let _ = fs::write(&main_config, &existing_content);
            }
        }

        Self::reload_apps();
    }

    fn reload_apps() {
        if let Some(config_dir) = dirs::config_dir() {
            let main_config = config_dir.join("ghostty").join("config");
            if main_config.exists() {
                let _ = Command::new("touch").arg(&main_config).status();
            }
        }
        let _ = Command::new("pkill")
            .args(["-USR1", "-x", "ghostty"])
            .status();
        let _ = Command::new("pkill")
            .args(["-SIGUSR1", "-x", "ghostty"])
            .status();
    }
}
