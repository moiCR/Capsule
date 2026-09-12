use crate::theme::Theme;
use crate::theme::templates::engine::TemplateEngine;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
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
    #[serde(default)]
    pub replace_section: Option<String>,
    #[serde(default)]
    pub start_marker: Option<String>,
    #[serde(default)]
    pub end_marker: Option<String>,
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

        let write_result = if let Some(ref section) = self.replace_section {
            let rendered_section = TemplateEngine::render(section, context);
            Self::write_replace_section(&target_path, &rendered_section, &rendered)
        } else if let (Some(start), Some(end)) = (&self.start_marker, &self.end_marker) {
            let rendered_start = TemplateEngine::render(start, context);
            let rendered_end = TemplateEngine::render(end, context);
            Self::write_replace_markers(&target_path, &rendered_start, &rendered_end, &rendered)
        } else {
            fs::write(&target_path, rendered)
        };

        if write_result.is_err() {
            return false;
        }

        if let Some(ref hook) = self.hook {
            self.execute_hook(hook);
        }

        if let Some(ref reload_cmd) = self.reload {
            let rendered_reload = TemplateEngine::render(reload_cmd, context);
            self.execute_reload(&rendered_reload);
        }

        true
    }

    fn write_replace_section(
        path: &Path,
        section_header: &str,
        rendered: &str,
    ) -> std::io::Result<()> {
        let existing = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let section_trimmed = section_header.trim();
        let normalized_header =
            if section_trimmed.starts_with('[') && section_trimmed.ends_with(']') {
                section_trimmed.to_string()
            } else {
                format!("[{section_trimmed}]")
            };

        let lines: Vec<&str> = existing.lines().collect();
        let mut intervals = Vec::new();
        let mut current_start = None;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Some(start) = current_start.take() {
                    intervals.push((start, i));
                }
                if trimmed.eq_ignore_ascii_case(&normalized_header) {
                    current_start = Some(i);
                }
            }
        }
        if let Some(start) = current_start {
            intervals.push((start, lines.len()));
        }

        let new_content = if let Some(&(first_start, first_end)) = intervals.first() {
            let mut rendered_keys = HashSet::new();
            for line in rendered.lines() {
                let trimmed = line.trim();
                if let Some((k, _)) = trimmed.split_once('=') {
                    rendered_keys.insert(k.trim().to_ascii_lowercase());
                }
            }

            let mut preserved_lines = Vec::new();
            for line in &lines[first_start..first_end] {
                let trimmed = line.trim();
                if trimmed.is_empty() || (trimmed.starts_with('[') && trimmed.ends_with(']')) {
                    continue;
                }
                if let Some((k, _)) = trimmed.split_once('=')
                    && !rendered_keys.contains(&k.trim().to_ascii_lowercase())
                {
                    preserved_lines.push(*line);
                }
            }

            let mut section_body = String::new();
            let rendered_trimmed = rendered.trim();
            let mut rendered_lines = rendered_trimmed.lines();
            let first_rendered_line = rendered_lines.next().unwrap_or("");

            if first_rendered_line.trim().starts_with('[')
                && first_rendered_line.trim().ends_with(']')
            {
                section_body.push_str(first_rendered_line);
                section_body.push('\n');
                for line in &preserved_lines {
                    section_body.push_str(line);
                    section_body.push('\n');
                }
                for line in rendered_lines {
                    section_body.push_str(line);
                    section_body.push('\n');
                }
            } else {
                section_body.push_str(&normalized_header);
                section_body.push('\n');
                for line in &preserved_lines {
                    section_body.push_str(line);
                    section_body.push('\n');
                }
                section_body.push_str(rendered_trimmed);
                section_body.push('\n');
            }

            let mut result = String::new();
            let mut idx = 0;
            while idx < lines.len() {
                if idx == first_start {
                    result.push_str(section_body.trim());
                    result.push('\n');
                    if lines
                        .get(first_end)
                        .is_some_and(|l| l.trim().starts_with('['))
                    {
                        result.push('\n');
                    }
                    idx = first_end;
                } else if let Some(&(_, e)) = intervals
                    .get(1..)
                    .unwrap_or(&[])
                    .iter()
                    .find(|&(s, e)| idx >= *s && idx < *e)
                {
                    idx = e;
                } else {
                    result.push_str(lines[idx]);
                    result.push('\n');
                    idx += 1;
                }
            }
            result
        } else if existing.trim().is_empty() {
            rendered.trim().to_string()
        } else {
            format!("{}\n\n{}\n", existing.trim_end(), rendered.trim())
        };

        fs::write(path, new_content)
    }

    fn write_replace_markers(
        path: &Path,
        start_marker: &str,
        end_marker: &str,
        rendered: &str,
    ) -> std::io::Result<()> {
        let existing = if path.exists() {
            fs::read_to_string(path)?
        } else {
            String::new()
        };

        let new_block = format!(
            "{}\n{}\n{}",
            start_marker.trim(),
            rendered.trim(),
            end_marker.trim()
        );

        let new_content = if let Some(start_pos) = existing.find(start_marker)
            && let Some(end_rel) = existing[start_pos..].find(end_marker)
        {
            let end_pos = start_pos + end_rel + end_marker.len();
            format!(
                "{}{}{}",
                &existing[..start_pos],
                new_block,
                &existing[end_pos..]
            )
        } else if existing.trim().is_empty() {
            format!("{}\n", new_block)
        } else {
            format!("{}\n\n{}\n", existing.trim_end(), new_block)
        };

        fs::write(path, new_content)
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
            replace_section = "[colors]"

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
        assert_eq!(p.replace_section.as_deref(), Some("[colors]"));
        assert_eq!(p.template.content.as_deref(), Some("bg={{bg}}"));
    }

    #[test]
    fn test_write_replace_section() {
        let temp_dir = std::env::temp_dir().join("capsule_section_test");
        let _ = fs::create_dir_all(&temp_dir);
        let config_file = temp_dir.join("test.ini");

        let initial_config =
            "font=Mono\n\n[colors]\nbg=111111\nfg=222222\n\n[cursor]\nstyle=beam\n";
        let _ = fs::write(&config_file, initial_config);

        let new_colors = "[colors]\nbg=333333\nfg=444444";
        assert!(
            TemplatePlugin::write_replace_section(&config_file, "[colors]", new_colors).is_ok()
        );

        let updated = fs::read_to_string(&config_file).unwrap_or_default();
        assert!(updated.contains("font=Mono"));
        assert!(updated.contains("[cursor]"));
        assert!(updated.contains("bg=333333"));
        assert!(!updated.contains("bg=111111"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_write_replace_section_preserves_keys_and_deduplicates() {
        let temp_dir = std::env::temp_dir().join("capsule_section_preserve_test");
        let _ = fs::create_dir_all(&temp_dir);
        let config_file = temp_dir.join("foot.ini");

        let initial_config = "font=Mono\n\n[colors-dark]\nalpha=0.78\nblur=yes\nbg=111111\n\n[cursor]\nstyle=beam\n\n[colors-dark]\nbg=999999\n";
        let _ = fs::write(&config_file, initial_config);

        let new_colors = "[colors-dark]\nbg=333333\nfg=444444";
        assert!(
            TemplatePlugin::write_replace_section(&config_file, "[colors-dark]", new_colors)
                .is_ok()
        );

        let updated = fs::read_to_string(&config_file).unwrap_or_default();
        assert!(updated.contains("font=Mono"));
        assert!(updated.contains("[cursor]"));
        assert!(updated.contains("alpha=0.78"));
        assert!(updated.contains("blur=yes"));
        assert!(updated.contains("bg=333333"));
        assert!(updated.contains("fg=444444"));
        assert!(!updated.contains("bg=111111"));
        assert!(!updated.contains("bg=999999"));
        assert_eq!(updated.matches("[colors-dark]").count(), 1);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_write_replace_markers() {
        let temp_dir = std::env::temp_dir().join("capsule_marker_test");
        let _ = fs::create_dir_all(&temp_dir);
        let config_file = temp_dir.join("test.conf");

        let initial_config = "font=Mono\n# START\nold=1\n# END\nother=2\n";
        let _ = fs::write(&config_file, initial_config);

        let new_body = "new=99";
        assert!(
            TemplatePlugin::write_replace_markers(&config_file, "# START", "# END", new_body)
                .is_ok()
        );

        let updated = fs::read_to_string(&config_file).unwrap_or_default();
        assert!(updated.contains("font=Mono"));
        assert!(updated.contains("new=99"));
        assert!(updated.contains("other=2"));
        assert!(!updated.contains("old=1"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
