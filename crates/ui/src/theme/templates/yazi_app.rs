use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use std::fs;
use std::process::Command;

pub struct YaziApp;

impl AppTheme for YaziApp {
    fn apply_current_theme(&self, theme: &Theme) {
        let bg = &theme.background_color.hex;
        let bg_alt = &theme.background_color_alt.hex;
        let surface = &theme.surface_color.hex;
        let fg = &theme.foreground_color.hex;
        let fg_muted = &theme.foreground_color_muted.hex;
        let accent = &theme.accent_color.hex;
        let red = &theme.red_color.hex;
        let green = &theme.green_color.hex;

        let yazi_theme = format!(
            "[manager]\n\
             cwd = {{ fg = \"{accent}\" }}\n\n\
             hovered = {{ fg = \"{fg}\", bg = \"{surface}\", bold = true }}\n\
             preview_hovered = {{ underline = true }}\n\n\
             find_keyword = {{ fg = \"{accent}\", bold = true }}\n\
             find_position = {{ fg = \"{fg_muted}\", bg = \"reset\" }}\n\n\
             marker_copied = {{ fg = \"{green}\", bg = \"{green}\" }}\n\
             marker_cut = {{ fg = \"{red}\", bg = \"{red}\" }}\n\
             marker_selected = {{ fg = \"{accent}\", bg = \"{accent}\" }}\n\n\
             tab_active = {{ fg = \"{fg}\", bg = \"{accent}\", bold = true }}\n\
             tab_inactive = {{ fg = \"{fg_muted}\", bg = \"{bg_alt}\" }}\n\n\
             border_symbol = \"│\"\n\
             border_style = {{ fg = \"{surface}\" }}\n\n\
             [mode]\n\
             normal_main = {{ fg = \"{bg}\", bg = \"{accent}\", bold = true }}\n\
             normal_alt = {{ fg = \"{accent}\", bg = \"{bg_alt}\" }}\n\n\
             select_main = {{ fg = \"{bg}\", bg = \"{green}\", bold = true }}\n\
             select_alt = {{ fg = \"{green}\", bg = \"{bg_alt}\" }}\n\n\
             unset_main = {{ fg = \"{bg}\", bg = \"{red}\", bold = true }}\n\
             unset_alt = {{ fg = \"{red}\", bg = \"{bg_alt}\" }}\n\n\
             [status]\n\
             separator_open = \"\u{e0b6}\"\n\
             separator_close = \"\u{e0b4}\"\n\
             separator_style = {{ fg = \"{surface}\", bg = \"{surface}\" }}\n\n\
             mode_normal = {{ fg = \"{bg}\", bg = \"{accent}\", bold = true }}\n\
             mode_select = {{ fg = \"{bg}\", bg = \"{green}\", bold = true }}\n\
             mode_unset = {{ fg = \"{bg}\", bg = \"{red}\", bold = true }}\n\n\
             permissions_t = {{ fg = \"{accent}\" }}\n\
             permissions_r = {{ fg = \"#f9e2af\" }}\n\
             permissions_w = {{ fg = \"{red}\" }}\n\
             permissions_x = {{ fg = \"{green}\" }}\n\
             permissions_s = {{ fg = \"{fg_muted}\" }}\n\n\
             [input]\n\
             border = {{ fg = \"{accent}\" }}\n\
             title = {{}}\n\
             value = {{}}\n\
             selected = {{ reversed = true }}\n\n\
             [select]\n\
             border = {{ fg = \"{accent}\" }}\n\
             active = {{ fg = \"{accent}\", bold = true }}\n\
             inactive = {{}}\n\n\
             [tasks]\n\
             border = {{ fg = \"{accent}\" }}\n\
             title = {{}}\n\
             hovered = {{ fg = \"{accent}\", underline = true }}\n\n\
             [which]\n\
             mask = {{ bg = \"{bg_alt}\" }}\n\
             cand = {{ fg = \"{accent}\" }}\n\
             rest = {{ fg = \"{fg_muted}\" }}\n\
             desc = {{ fg = \"{fg}\" }}\n\
             separator = \" \u{ea9c} \"\n\
             separator_style = {{ fg = \"{surface}\" }}\n\n\
             [help]\n\
             on = {{ fg = \"{accent}\" }}\n\
             exec = {{ fg = \"{green}\" }}\n\
             desc = {{ fg = \"{fg}\" }}\n\
             hovered = {{ bg = \"{surface}\", bold = true }}\n\
             footer = {{ fg = \"{fg_muted}\", bg = \"{bg_alt}\" }}\n"
        );

        if let Some(config_dir) = dirs::config_dir() {
            let yazi_dir = config_dir.join("yazi");
            let _ = fs::create_dir_all(&yazi_dir);
            let theme_path = yazi_dir.join("theme.toml");
            let _ = fs::write(&theme_path, &yazi_theme);
        }

        &self.reload_apps();
        
    }

    fn reload_apps(&self) {
        if let Some(config_dir) = dirs::config_dir() {
            let theme_path = config_dir.join("yazi").join("theme.toml");
            if theme_path.exists() {
                let _ = Command::new("touch").arg(&theme_path).status();
            }
        }
    }

}
