use crate::theme::Theme;
use crate::theme::templates::AppTheme;
use crate::theme::templates::engine::TemplateEngine;
use crate::theme::templates::plugin::TemplatePlugin;
use std::fs;
use std::path::PathBuf;

pub struct TemplatePluginManager;

impl Default for TemplatePluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplatePluginManager {
    pub fn new() -> Self {
        Self::ensure_default_templates_exist();
        Self
    }

    pub fn templates_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("templates")
    }

    pub fn ensure_default_templates_exist() {
        let dir = Self::templates_dir();
        let _ = fs::create_dir_all(&dir);

        let defaults = [
            ("kitty.toml", DEFAULT_KITTY_TEMPLATE),
            ("ghostty.toml", DEFAULT_GHOSTTY_TEMPLATE),
            ("fish.toml", DEFAULT_FISH_TEMPLATE),
            ("yazi.toml", DEFAULT_YAZI_TEMPLATE),
            ("alacritty.toml", DEFAULT_ALACRITTY_TEMPLATE),
            ("foot.toml", DEFAULT_FOOT_TEMPLATE),
            ("hyprland.toml", DEFAULT_HYPRLAND_TEMPLATE),
        ];

        for (filename, content) in defaults {
            let path = dir.join(filename);
            if !path.exists() {
                let _ = fs::write(&path, content);
            }
        }
    }

    pub fn load_plugins(&self) -> Vec<TemplatePlugin> {
        let dir = Self::templates_dir();
        let mut plugins = Vec::new();

        let Ok(entries) = fs::read_dir(&dir) else {
            return plugins;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(plugin) = TemplatePlugin::from_file(&path) {
                    plugins.push(plugin);
                }
            } else if path.is_dir()
                && let sub_toml = path.join("template.toml")
                && sub_toml.is_file()
                && let Some(plugin) = TemplatePlugin::from_file(&sub_toml)
            {
                plugins.push(plugin);
            }
        }

        plugins.sort_by_key(|a| a.name.to_lowercase());
        plugins
    }
}

impl AppTheme for TemplatePluginManager {
    fn apply_current_theme(&self, theme: &Theme) {
        Self::ensure_default_templates_exist();
        let context = TemplateEngine::build_context(theme);
        let plugins = self.load_plugins();

        for plugin in plugins {
            plugin.apply_theme(theme, &context);
        }
    }

    fn reload_apps(&self) {}
}

const DEFAULT_KITTY_TEMPLATE: &str = r##"name = "kitty"
description = "Kitty terminal emulator theme"
enabled = true
target = "~/.config/kitty/theme.conf"
reload = "pkill -USR1 -x kitty"

[hook]
file = "~/.config/kitty/kitty.conf"
contains = "include theme.conf"
inject = "include theme.conf\n"

[template]
content = """
background {{theme.background}}
foreground {{theme.foreground}}
cursor {{theme.foreground}}
cursor_text_color {{theme.background}}
selection_foreground {{theme.foreground}}
selection_background {{theme.surface}}
active_border_color {{theme.accent}}
inactive_border_color {{theme.background_alt}}
url_color {{theme.accent}}
active_tab_foreground {{theme.background}}
active_tab_background {{theme.accent}}
inactive_tab_foreground {{theme.foreground_muted}}
inactive_tab_background {{theme.background_alt}}

color0 {{palette.0}}
color1 {{palette.1}}
color2 {{palette.2}}
color3 {{palette.3}}
color4 {{palette.4}}
color5 {{palette.5}}
color6 {{palette.6}}
color7 {{palette.7}}
color8 {{palette.8}}
color9 {{palette.9}}
color10 {{palette.10}}
color11 {{palette.11}}
color12 {{palette.12}}
color13 {{palette.13}}
color14 {{palette.14}}
color15 {{palette.15}}
"""
"##;

const DEFAULT_GHOSTTY_TEMPLATE: &str = r##"name = "ghostty"
description = "Ghostty terminal emulator theme"
enabled = true
target = "~/.config/ghostty/theme"
reload = "pkill -USR2 -x ghostty"

[hook]
file = "~/.config/ghostty/config"
contains = "config-file = theme"
inject = "config-file = theme\n"

[template]
content = """
background = {{theme.background}}
foreground = {{theme.foreground}}
palette = 0={{palette.0}}
palette = 1={{palette.1}}
palette = 2={{palette.2}}
palette = 3={{palette.3}}
palette = 4={{palette.4}}
palette = 5={{palette.5}}
palette = 6={{palette.6}}
palette = 7={{palette.7}}
palette = 8={{palette.8}}
palette = 9={{palette.9}}
palette = 10={{palette.10}}
palette = 11={{palette.11}}
palette = 12={{palette.12}}
palette = 13={{palette.13}}
palette = 14={{palette.14}}
palette = 15={{palette.15}}
"""
"##;

const DEFAULT_FISH_TEMPLATE: &str = r##"name = "fish"
description = "Fish shell syntax highlighting theme"
enabled = true
target = "~/.config/fish/conf.d/capsule_theme.fish"
reload = "fish -c 'source ~/.config/fish/conf.d/capsule_theme.fish'"

[template]
content = """
set -U fish_color_normal {{foreground.strip}}
set -U fish_color_command {{accent.strip}}
set -U fish_color_quote {{green.strip}}
set -U fish_color_redirection {{foreground.strip}}
set -U fish_color_end {{foreground.strip}}
set -U fish_color_error {{red.strip}}
set -U fish_color_param {{foreground.strip}}
set -U fish_color_comment {{foreground_muted.strip}}
set -U fish_color_match {{accent.strip}}
set -U fish_color_selection {{foreground_muted.strip}}
set -U fish_color_search_match {{accent.strip}}
set -U fish_color_operator {{accent.strip}}
set -U fish_color_escape {{accent.strip}}
set -U fish_color_autosuggestion {{foreground_muted.strip}}
set -U fish_color_cwd {{accent.strip}}
set -U fish_color_accent {{accent.strip}}
"""
"##;

const DEFAULT_YAZI_TEMPLATE: &str = r##"name = "yazi"
description = "Yazi file manager theme"
enabled = true
target = "~/.config/yazi/theme.toml"
reload = "touch ~/.config/yazi/theme.toml"

[template]
content = """
[manager]
cwd = { fg = "{{theme.accent}}" }

hovered = { fg = "{{theme.foreground}}", bg = "{{theme.surface}}", bold = true }
preview_hovered = { underline = true }

find_keyword = { fg = "{{theme.accent}}", bold = true }
find_position = { fg = "{{theme.foreground_muted}}", bg = "reset" }

marker_copied = { fg = "{{theme.green}}", bg = "{{theme.green}}" }
marker_cut = { fg = "{{theme.red}}", bg = "{{theme.red}}" }
marker_selected = { fg = "{{theme.accent}}", bg = "{{theme.accent}}" }

tab_active = { fg = "{{theme.foreground}}", bg = "{{theme.accent}}", bold = true }
tab_inactive = { fg = "{{theme.foreground_muted}}", bg = "{{theme.background_alt}}" }

border_symbol = "│"
border_style = { fg = "{{theme.surface}}" }

[mode]
normal_main = { fg = "{{theme.background}}", bg = "{{theme.accent}}", bold = true }
normal_alt = { fg = "{{theme.accent}}", bg = "{{theme.background_alt}}" }

select_main = { fg = "{{theme.background}}", bg = "{{theme.green}}", bold = true }
select_alt = { fg = "{{theme.green}}", bg = "{{theme.background_alt}}" }

unset_main = { fg = "{{theme.background}}", bg = "{{theme.red}}", bold = true }
unset_alt = { fg = "{{theme.red}}", bg = "{{theme.background_alt}}" }

[status]
separator_open = ""
separator_close = ""
separator_style = { fg = "{{theme.surface}}", bg = "{{theme.surface}}" }

mode_normal = { fg = "{{theme.background}}", bg = "{{theme.accent}}", bold = true }
mode_select = { fg = "{{theme.background}}", bg = "{{theme.green}}", bold = true }
mode_unset = { fg = "{{theme.background}}", bg = "{{theme.red}}", bold = true }

permissions_t = { fg = "{{theme.accent}}" }
permissions_r = { fg = "#f9e2af" }
permissions_w = { fg = "{{theme.red}}" }
permissions_x = { fg = "{{theme.green}}" }
permissions_s = { fg = "{{theme.foreground_muted}}" }

[input]
border = { fg = "{{theme.accent}}" }
title = {}
value = {}
selected = { reversed = true }

[select]
border = { fg = "{{theme.accent}}" }
active = { fg = "{{theme.accent}}", bold = true }
inactive = {}

[tasks]
border = { fg = "{{theme.accent}}" }
title = {}
hovered = { fg = "{{theme.accent}}", underline = true }

[which]
mask = { bg = "{{theme.background_alt}}" }
cand = { fg = "{{theme.accent}}" }
rest = { fg = "{{theme.foreground_muted}}" }
desc = { fg = "{{theme.foreground}}" }
separator = "  "
separator_style = { fg = "{{theme.surface}}" }

[help]
on = { fg = "{{theme.accent}}" }
exec = { fg = "{{theme.green}}" }
desc = { fg = "{{theme.foreground}}" }
hovered = { bg = "{{theme.surface}}", bold = true }
footer = { fg = "{{theme.foreground_muted}}", bg = "{{theme.background_alt}}" }
"""
"##;

const DEFAULT_ALACRITTY_TEMPLATE: &str = r##"name = "alacritty"
description = "Alacritty terminal emulator theme"
enabled = true
target = "~/.config/alacritty/theme.toml"

[hook]
file = "~/.config/alacritty/alacritty.toml"
contains = "theme.toml"
inject = 'import = ["theme.toml"]\n'

[template]
content = """
[colors.primary]
background = "{{theme.background}}"
foreground = "{{theme.foreground}}"

[colors.cursor]
text = "{{theme.background}}"
cursor = "{{theme.foreground}}"

[colors.selection]
text = "{{theme.foreground}}"
background = "{{theme.surface}}"

[colors.normal]
black = "{{palette.0}}"
red = "{{palette.1}}"
green = "{{palette.2}}"
yellow = "{{palette.3}}"
blue = "{{palette.4}}"
magenta = "{{palette.5}}"
cyan = "{{palette.6}}"
white = "{{palette.7}}"

[colors.bright]
black = "{{palette.8}}"
red = "{{palette.9}}"
green = "{{palette.10}}"
yellow = "{{palette.11}}"
blue = "{{palette.12}}"
magenta = "{{palette.13}}"
cyan = "{{palette.14}}"
white = "{{palette.15}}"
"""
"##;

const DEFAULT_FOOT_TEMPLATE: &str = r##"name = "foot"
description = "Foot Wayland terminal emulator theme"
enabled = true
target = "~/.config/foot/foot.ini"
replace_section = "[colors-{{mode}}]"

[template]
content = """
[colors-{{mode}}]
background={{background.strip}}
foreground={{foreground.strip}}
regular0={{palette.0.strip}}
regular1={{palette.1.strip}}
regular2={{palette.2.strip}}
regular3={{palette.3.strip}}
regular4={{palette.4.strip}}
regular5={{palette.5.strip}}
regular6={{palette.6.strip}}
regular7={{palette.7.strip}}
bright0={{palette.8.strip}}
bright1={{palette.9.strip}}
bright2={{palette.10.strip}}
bright3={{palette.11.strip}}
bright4={{palette.12.strip}}
bright5={{palette.13.strip}}
bright6={{palette.14.strip}}
bright7={{palette.15.strip}}
"""
"##;

const DEFAULT_HYPRLAND_TEMPLATE: &str = r##"name = "hyprland"
description = "Hyprland borders and colors"
enabled = true
target = "~/.config/hypr/capsule_colors.lua"
reload = "hyprctl reload"

[hook]
file = "~/.config/hypr/hyprland.lua"
contains = "require(\"capsule_colors\")"
inject = "colors = require(\"capsule_colors\")\n"

[template]
content = """
return {
    active_border = "rgb({{accent.strip}})",
    inactive_border = "rgb({{background_alt.strip}})",
    surface = "rgb({{surface.strip}})"
}
"""
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_defaults_and_load() {
        TemplatePluginManager::ensure_default_templates_exist();
        let manager = TemplatePluginManager::new();
        let plugins = manager.load_plugins();
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.name == "kitty"));
        assert!(plugins.iter().any(|p| p.name == "ghostty"));
        assert!(plugins.iter().any(|p| p.name == "fish"));
        assert!(plugins.iter().any(|p| p.name == "yazi"));
        assert!(plugins.iter().any(|p| p.name == "alacritty"));
        assert!(plugins.iter().any(|p| p.name == "foot"));
        assert!(plugins.iter().any(|p| p.name == "hyprland"));
    }

    #[test]
    fn test_apply_theme_renders_to_temp() {
        let temp_dir = std::env::temp_dir().join("capsule_tpl_test");
        let _ = fs::create_dir_all(&temp_dir);
        let target_file = temp_dir.join("theme.conf");

        let plugin = TemplatePlugin {
            name: "test_temp".to_string(),
            description: None,
            enabled: true,
            target: target_file.to_string_lossy().to_string(),
            reload: None,
            hook: None,
            replace_section: None,
            start_marker: None,
            end_marker: None,
            template: crate::theme::templates::plugin::TemplateConfig {
                content: Some("accent={{accent}} bg={{bg.strip}}".to_string()),
                file: None,
            },
            base_dir: temp_dir.clone(),
        };

        let mut theme = Theme::default();
        theme.accent_color = crate::theme::Color::from("#FF0055");
        theme.background_color = crate::theme::Color::from("#101010");

        let ctx = TemplateEngine::build_context(&theme);
        assert!(plugin.apply_theme(&theme, &ctx));

        let rendered = fs::read_to_string(&target_file).unwrap_or_default();
        assert_eq!(rendered, "accent=#FF0055 bg=101010");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_hyprland_lua_template_renders() {
        let plugin = TemplatePlugin::from_file(
            &TemplatePluginManager::templates_dir().join("hyprland.toml"),
        );
        assert!(plugin.is_some());
        let plugin = plugin.unwrap_or_else(|| unreachable!());
        assert_eq!(plugin.name, "hyprland");
        assert!(plugin.target.ends_with("capsule_colors.lua"));
    }

    #[test]
    fn test_hyprland_lua_render_execution() {
        let manager = TemplatePluginManager::new();
        let theme = Theme::default();
        let ctx = TemplateEngine::build_context(&theme);
        let plugins = manager.load_plugins();
        let hypr = plugins.into_iter().find(|p| p.name == "hyprland");
        assert!(hypr.is_some());
        let hypr = hypr.unwrap_or_else(|| unreachable!());
        assert!(hypr.apply_theme(&theme, &ctx));
        let path = TemplatePlugin::expand_path(&hypr.target);
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap_or_default();
        assert!(content.contains("return {"));
        assert!(content.contains("active_border = \"rgb("));
    }

    #[test]
    fn test_foot_template_replace_section() {
        let manager = TemplatePluginManager::new();
        let theme = Theme::default();
        let ctx = TemplateEngine::build_context(&theme);
        let plugins = manager.load_plugins();
        let foot = plugins.into_iter().find(|p| p.name == "foot");
        assert!(foot.is_some());
        let foot = foot.unwrap_or_else(|| unreachable!());
        assert_eq!(foot.replace_section.as_deref(), Some("[colors-{{mode}}]"));
        assert!(foot.apply_theme(&theme, &ctx));
        let path = TemplatePlugin::expand_path(&foot.target);
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap_or_default();
        assert!(content.contains("[colors-dark]"));
        assert!(content.contains("background="));
    }
}
