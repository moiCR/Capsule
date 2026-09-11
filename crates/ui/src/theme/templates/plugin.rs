use crate::theme::Theme;
use crate::theme::templates::engine::TemplateEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub file: String,
    pub contains: String,
    pub inject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePlugin {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub target: String,
    #[serde(default)]
    pub reload: Option<String>,
    #[serde(default)]
    pub hook: Option<HookConfig>,
    pub template: TemplateConfig,
    #[serde(skip)]
    pub base_dir: PathBuf,
}

fn default_enabled() -> bool {
    true
}

impl TemplatePlugin {
    pub fn from_file(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        let mut plugin: Self = toml::from_str(&content).ok()?;
        plugin.base_dir = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();
        Some(plugin)
    }

    pub fn expand_path(raw: &str) -> PathBuf {
        let trimmed = raw.trim();
        if let Some(stripped) = trimmed.strip_prefix("~/")
            && let Some(home) = dirs::home_dir()
        {
            return home.join(stripped);
        } else if trimmed == "~"
            && let Some(home) = dirs::home_dir()
        {
            return home;
        } else if let Some(stripped) = trimmed.strip_prefix("$XDG_CONFIG_HOME/")
            && let Some(cfg) = dirs::config_dir()
        {
            return cfg.join(stripped);
        }
        PathBuf::from(trimmed)
    }

    pub fn get_template_content(&self) -> Option<String> {
        if let Some(ref inline) = self.template.content {
            return Some(inline.clone());
        }

        if let Some(ref rel_file) = self.template.file {
            let path = self.base_dir.join(rel_file);
            return fs::read_to_string(path).ok();
        }

        None
    }

    pub fn apply_theme(&self, _theme: &Theme, context: &HashMap<String, String>) -> bool {
        if !self.enabled {
            return false;
        }

        let Some(raw_template) = self.get_template_content() else {
            return false;
        };

        let rendered = TemplateEngine::render(&raw_template, context);
        let target_path = Self::expand_path(&self.target);

        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if fs::write(&target_path, rendered).is_err() {
            return false;
        }

        if let Some(ref hook) = self.hook {
            self.execute_hook(hook);
        }

        if let Some(ref reload_cmd) = self.reload {
            self.execute_reload(reload_cmd);
        }

        true
    }

    fn execute_hook(&self, hook: &HookConfig) {
        let hook_path = Self::expand_path(&hook.file);
        let existing = if hook_path.exists() {
            fs::read_to_string(&hook_path).unwrap_or_default()
        } else {
            String::new()
        };

        if !existing.contains(&hook.contains) {
            if let Some(parent) = hook_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let new_content = format!("{}\n{}", hook.inject.trim_end(), existing);
            let _ = fs::write(&hook_path, new_content);
        }
    }

    fn execute_reload(&self, cmd: &str) {
        let _ = Command::new("sh").args(["-c", cmd]).status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_path() {
        let p = TemplatePlugin::expand_path("~/.config/test.conf");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(p, home.join(".config/test.conf"));
        }
    }

    #[test]
    fn test_plugin_deserialization() {
        let toml_str = r#"
            name = "test"
            enabled = true
            target = "~/.config/test/theme.conf"
            reload = "echo reload"

            [hook]
            file = "~/.config/test/config"
            contains = "include theme"
            inject = "include theme\n"

            [template]
            content = "bg={{bg}}"
        "#;

        let plugin: Result<TemplatePlugin, _> = toml::from_str(toml_str);
        assert!(plugin.is_ok());
        let p = plugin.unwrap_or_else(|_| unreachable!());
        assert_eq!(p.name, "test");
        assert!(p.enabled);
        assert_eq!(p.template.content.as_deref(), Some("bg={{bg}}"));
    }
}
