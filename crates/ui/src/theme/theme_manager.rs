use crate::theme::Theme;
use std::{fs, path::PathBuf};

pub struct ThemeManager {
    pub current_theme: Theme,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        let path = Self::theme_path();

        let theme = if path.exists() {
            Self::load(&path).unwrap_or_default()
        } else {
            let theme = Theme::default();

            if let Err(error) = Self::save(&path, &theme) {
                eprintln!("Failed to save default theme: {error}");
            }

            theme
        };

        Self {
            current_theme: theme,
        }
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
