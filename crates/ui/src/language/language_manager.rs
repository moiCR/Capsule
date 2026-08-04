use crate::language::Language;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct LanguageItem {
    pub name: String,
    pub code: String,
    pub path: PathBuf,
    pub language: Language,
    pub is_current: bool,
}

pub struct LanguageManager {
    pub current_language: Language,
    pub last_modified: Option<std::time::SystemTime>,
}

impl gpui::Global for LanguageManager {}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageManager {
    pub fn new() -> Self {
        Self::ensure_default_variants_exist();

        let path = Self::language_path();
        let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();

        let language = if path.exists() {
            Self::load(&path).unwrap_or_default()
        } else {
            let default_lang = Language::default();
            if let Err(error) = Self::save(&path, &default_lang) {
                eprintln!("Failed to save current_language.toml: {error}");
            }
            default_lang
        };

        let manager = Self {
            current_language: language,
            last_modified: mtime,
        };
        manager
    }

    pub fn language_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("capsule")
            .join("languages")
            .join("current_language.toml")
    }

    pub fn variants_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("capsule")
            .join("languages")
            .join("variants")
    }

    pub fn ensure_default_variants_exist() {
        let variants_dir = Self::variants_path();
        let _ = fs::create_dir_all(&variants_dir);

        let es_path = variants_dir.join("es.toml");
        if !es_path.exists() {
            let es_lang = Language::spanish();
            if let Err(err) = Self::save(&es_path, &es_lang) {
                eprintln!("Failed to save es.toml variant: {err}");
            }
        }

        let en_path = variants_dir.join("en.toml");
        if !en_path.exists() {
            let en_lang = Language::english();
            if let Err(err) = Self::save(&en_path, &en_lang) {
                eprintln!("Failed to save en.toml variant: {err}");
            }
        }
    }

    pub fn list_languages(&self) -> Vec<LanguageItem> {
        Self::ensure_default_variants_exist();
        let variants_dir = Self::variants_path();
        let mut items = Vec::new();

        if let Ok(entries) = fs::read_dir(&variants_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Ok(lang) = Self::load(&path) {
                        let is_current = lang.code == self.current_language.code;
                        items.push(LanguageItem {
                            name: lang.name.clone(),
                            code: lang.code.clone(),
                            path,
                            language: lang,
                            is_current,
                        });
                    }
                }
            }
        }

        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        items
    }

    pub fn check_and_reload(&mut self) -> bool {
        let path = Self::language_path();
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                if self.last_modified != Some(mtime) {
                    self.last_modified = Some(mtime);
                    if let Ok(lang) = Self::load(&path) {
                        self.current_language = lang;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn set_language(&mut self, language: Language) {
        self.current_language = language.clone();
        let path = Self::language_path();
        if let Err(error) = Self::save(&path, &self.current_language) {
            eprintln!("Failed to save current_language.toml: {error}");
        }
    }

    fn load(path: &PathBuf) -> Result<Language, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let lang: Language = toml::from_str(&content)?;
        Ok(lang)
    }

    fn save(path: &PathBuf, language: &Language) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(language)?;
        fs::write(path, content)?;
        Ok(())
    }
}
