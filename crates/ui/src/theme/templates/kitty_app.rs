use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

pub struct KittyApp;

impl AppTheme for KittyApp {
    fn apply_current_theme(&self, theme: &Theme) {
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

        let mut kitty_config = String::new();

        // 1. UI & Cursor colors
        kitty_config.push_str(&format!("background {bg}\n"));
        kitty_config.push_str(&format!("foreground {fg}\n"));
        kitty_config.push_str(&format!("cursor {fg}\n"));
        kitty_config.push_str(&format!("cursor_text_color {bg}\n"));
        kitty_config.push_str(&format!("selection_foreground {fg}\n"));
        kitty_config.push_str(&format!("selection_background {surface}\n"));
        kitty_config.push_str(&format!("active_border_color {accent}\n"));
        kitty_config.push_str(&format!("inactive_border_color {bg_alt}\n"));
        kitty_config.push_str(&format!("url_color {accent}\n"));
        kitty_config.push_str(&format!("active_tab_foreground {bg}\n"));
        kitty_config.push_str(&format!("active_tab_background {accent}\n"));
        kitty_config.push_str(&format!("inactive_tab_foreground {fg_muted}\n"));
        kitty_config.push_str(&format!("inactive_tab_background {bg_alt}\n\n"));

        // 2. ANSI Palette colors (0-15)
        for (index, color) in palette_colors {
            kitty_config.push_str(&format!("color{index} {color}\n"));
        }

        if let Some(config_dir) = dirs::config_dir() {
            let kitty_dir = config_dir.join("kitty");
            let _ = fs::create_dir_all(&kitty_dir);

            let theme_file = kitty_dir.join("theme.conf");
            let _ = fs::write(&theme_file, &kitty_config);

            let main_config = kitty_dir.join("kitty.conf");
            let existing_content = if main_config.exists() {
                fs::read_to_string(&main_config).unwrap_or_default()
            } else {
                String::new()
            };

            if !existing_content.contains("include theme.conf")
                && !existing_content.contains("include ./theme.conf")
            {
                let new_content = format!("include theme.conf\n{existing_content}");
                let _ = fs::write(&main_config, new_content);
            }
        }

        self.reload_apps();
    }

    fn reload_apps(&self) {
        let _ = Command::new("pkill")
            .args(["-USR1", "-x", "kitty"])
            .status();
    }
}
