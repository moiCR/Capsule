use crate::theme::Theme;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct ThemeItem {
    pub name: String,
    pub path: PathBuf,
    pub theme: Theme,
    pub is_default: bool,
}

pub struct ThemeManager {
    pub current_theme: Theme,
    pub last_modified: Option<std::time::SystemTime>,
}

impl gpui::Global for ThemeManager {}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        Self::ensure_default_theme_exists();

        let path = Self::theme_path();
        let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();

        let theme = if path.exists() {
            Self::load(&path).unwrap_or_default()
        } else {
            let theme = Theme::default();

            if let Err(error) = Self::save(&path, &theme) {
                eprintln!("Failed to save default theme: {error}");
            }

            theme
        };

        let manager = Self {
            current_theme: theme,
            last_modified: mtime,
        };
        manager.apply_theme_to_apps();
        manager
    }

    pub fn ensure_default_theme_exists() {
        let presets = Self::themes_path();
        let default_path = presets.join("default.toml");
        if !default_path.exists() {
            let default_theme = Theme::default();
            if let Err(error) = Self::save(&default_path, &default_theme) {
                eprintln!("Failed to save default theme preset: {error}");
            }
        }
    }

    pub fn list_themes(&self) -> Vec<ThemeItem> {
        Self::ensure_default_theme_exists();
        let presets_dir = Self::themes_path();
        let mut items = Vec::new();

        if let Ok(entries) = fs::read_dir(&presets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Theme")
                        .to_string();

                    if let Ok(theme) = Self::load(&path) {
                        let is_default = stem == "default";
                        items.push(ThemeItem {
                            name: stem,
                            path,
                            theme,
                            is_default,
                        });
                    }
                }
            }
        }

        items.sort_by(|a, b| {
            if a.is_default {
                std::cmp::Ordering::Less
            } else if b.is_default {
                std::cmp::Ordering::Greater
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        items
    }

    pub fn check_and_reload(&mut self) -> bool {
        let path = Self::theme_path();
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if self.last_modified != Some(mtime) {
                    self.last_modified = Some(mtime);
                    if let Ok(theme) = Self::load(&path) {
                        self.current_theme = theme;
                        self.apply_theme_to_apps();
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn apply_theme_to_apps(&self) {
        use crate::theme::{AppTheme, FishApp, GhosttyApp, GtkApps, QtApps, YaziApp};

        GtkApps::apply_current_theme(&self.current_theme);
        GtkApps::reload_apps();

        QtApps::apply_current_theme(&self.current_theme);
        QtApps::reload_apps();

        GhosttyApp::apply_current_theme(&self.current_theme);
        GhosttyApp::reload_apps();

        FishApp::apply_current_theme(&self.current_theme);
        FishApp::reload_apps();

        YaziApp::apply_current_theme(&self.current_theme);
        YaziApp::reload_apps();
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
        if let Err(error) = Self::save(&Self::theme_path(), &self.current_theme) {
            eprintln!("Failed to save theme: {error}");
        }
        self.apply_theme_to_apps();
    }

    pub fn theme_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("capsule")
            .join("themes")
            .join("current.toml")
    }

    pub fn themes_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("capsule")
            .join("themes")
            .join("presets")
    }

    fn load(path: &PathBuf) -> Result<Theme, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let theme = toml::from_str(&content)?;

        Ok(theme)
    }

    pub fn create_theme(name: &str, theme: &Theme) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let file_name = if name.ends_with(".toml") {
            name.to_string()
        } else {
            format!("{name}.toml")
        };

        let path = Self::themes_path().join(file_name);
        Self::save(&path, theme)?;
        Ok(path)
    }

    fn save(path: &PathBuf, theme: &Theme) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(theme)?;

        fs::write(path, content)?;

        Ok(())
    }
}
