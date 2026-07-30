use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use std::fs;
use std::process::Command;

pub struct FishApp;

fn clean_hex(hex: &str) -> String {
    hex.trim_start_matches('#').to_string()
}

impl AppTheme for FishApp {
    fn apply_current_theme(&self, theme: &Theme) {
        let fg = clean_hex(&theme.foreground_color.hex);
        let fg_muted = clean_hex(&theme.foreground_color_muted.hex);
        let accent = clean_hex(&theme.accent_color.hex);
        let red = clean_hex(&theme.red_color.hex);
        let green = clean_hex(&theme.green_color.hex);

        let fish_theme = format!(
            r#"# Dynamic Fish shell colors from Capsule

set -U fish_color_normal {fg}
set -U fish_color_command {accent}
set -U fish_color_quote {green}
set -U fish_color_redirection {fg}
set -U fish_color_end {fg}
set -U fish_color_error {red}
set -U fish_color_param {fg}
set -U fish_color_comment {fg_muted}
set -U fish_color_match {accent}
set -U fish_color_selection {fg_muted}
set -U fish_color_search_match {accent}
set -U fish_color_operator {accent}
set -U fish_color_escape {accent}
set -U fish_color_autosuggestion {fg_muted}
set -U fish_color_cwd {accent}
set -U fish_color_accent {accent}
"#
        );

        if let Some(config_dir) = dirs::config_dir() {
            let conf_d = config_dir.join("fish").join("conf.d");
            let _ = fs::create_dir_all(&conf_d);
            let _ = fs::write(conf_d.join("theme.fish"), &fish_theme);
        }

        let fish_cmd = format!(
            "set -U fish_color_normal {fg}; \
             set -U fish_color_command {accent}; \
             set -U fish_color_quote {green}; \
             set -U fish_color_redirection {fg}; \
             set -U fish_color_end {fg}; \
             set -U fish_color_error {red}; \
             set -U fish_color_param {fg}; \
             set -U fish_color_comment {fg_muted}; \
             set -U fish_color_match {accent}; \
             set -U fish_color_selection {fg_muted}; \
             set -U fish_color_search_match {accent}; \
             set -U fish_color_operator {accent}; \
             set -U fish_color_escape {accent}; \
             set -U fish_color_autosuggestion {fg_muted}; \
             set -U fish_color_cwd {accent}; \
             set -U fish_color_accent {accent}"
        );

        let _ = Command::new("fish").args(["-c", &fish_cmd]).status();
    }

    fn reload_apps(&self) {}
}
