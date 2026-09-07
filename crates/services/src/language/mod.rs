use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use toml::Value;

const EMBEDDED_ES: &str = include_str!("./embedded_es.toml");
const EMBEDDED_EN: &str = include_str!("./embedded_en.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    pub name: String,
    pub code: String,
    pub path: PathBuf,
    pub is_current: bool,
}

#[derive(Default, Clone)]
pub struct LanguageBundle {
    pub name: String,
    pub code: String,
    pub strings: HashMap<String, String>,
    pub lists: HashMap<String, Vec<String>>,
}

impl LanguageBundle {
    pub fn parse(content: &str) -> Self {
        let mut bundle = Self::default();
        let Ok(table) = toml::from_str::<toml::Table>(content) else {
            return bundle;
        };

        if let Some(Value::String(name)) = table.get("name") {
            bundle.name = name.clone();
        }
        if let Some(Value::String(code)) = table.get("code") {
            bundle.code = code.clone();
        }

        Self::flatten_table(&table, "", &mut bundle);
        bundle
    }

    pub fn merge(&mut self, other: LanguageBundle) {
        if !other.name.is_empty() {
            self.name = other.name;
        }
        if !other.code.is_empty() {
            self.code = other.code;
        }
        for (k, v) in other.strings {
            self.strings.insert(k, v);
        }
        for (k, v) in other.lists {
            self.lists.insert(k, v);
        }
    }

    fn flatten_table(table: &toml::Table, prefix: &str, bundle: &mut LanguageBundle) {
        for (key, value) in table {
            match value {
                Value::String(text) => {
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}{key}")
                    };
                    bundle.strings.insert(full_key, text.clone());
                    if !prefix.is_empty() && !bundle.strings.contains_key(key) {
                        bundle.strings.insert(key.clone(), text.clone());
                    }
                }
                Value::Array(items) => {
                    let mut list = Vec::with_capacity(items.len());
                    for item in items {
                        if let Value::String(item_str) = item {
                            list.push(item_str.clone());
                        }
                    }
                    let full_key = if prefix.is_empty() {
                        key.clone()
                    } else {
                        format!("{prefix}{key}")
                    };
                    bundle.lists.insert(full_key, list.clone());
                    if !prefix.is_empty() && !bundle.lists.contains_key(key) {
                        bundle.lists.insert(key.clone(), list);
                    }
                }
                Value::Table(nested_table) => {
                    let next_prefix = format!("{prefix}{key}.");
                    Self::flatten_table(nested_table, &next_prefix, bundle);
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone)]
pub struct LangService {
    current: Arc<ArcSwap<LanguageBundle>>,
    current_target: Arc<ArcSwap<String>>,
    fallback_es: Arc<LanguageBundle>,
    fallback_en: Arc<LanguageBundle>,
    last_modified: Arc<ArcSwap<Option<std::time::SystemTime>>>,
    has_changed: Arc<AtomicBool>,
}

impl LangService {
    pub fn new(target: &str) -> Self {
        Self::ensure_default_files_exist();

        let fallback_es = LanguageBundle::parse(EMBEDDED_ES);
        let fallback_en = LanguageBundle::parse(EMBEDDED_EN);
        let resolved_bundle =
            Self::load_target_bundle(target).unwrap_or_else(|| fallback_es.clone());
        let path = Self::resolve_path(target);
        let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();

        Self {
            current: Arc::new(ArcSwap::from_pointee(resolved_bundle)),
            current_target: Arc::new(ArcSwap::from_pointee(target.to_string())),
            fallback_es: Arc::new(fallback_es),
            fallback_en: Arc::new(fallback_en),
            last_modified: Arc::new(ArcSwap::from_pointee(mtime)),
            has_changed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get(&self, key: &str) -> String {
        let current_bundle = self.current.load();
        if let Some(val) = current_bundle.strings.get(key) {
            return val.clone();
        }

        let suffix = format!(".{key}");
        for (candidate_key, candidate_val) in &current_bundle.strings {
            if candidate_key.ends_with(&suffix) {
                return candidate_val.clone();
            }
        }

        let is_en = current_bundle.code == "en"
            || self.current_target().starts_with("en")
            || self.current_target() == "en.toml";

        let (primary, secondary) = if is_en {
            (&self.fallback_en, &self.fallback_es)
        } else {
            (&self.fallback_es, &self.fallback_en)
        };

        if let Some(val) = primary.strings.get(key) {
            return val.clone();
        }
        for (candidate_key, candidate_val) in &primary.strings {
            if candidate_key.ends_with(&suffix) {
                return candidate_val.clone();
            }
        }

        if let Some(val) = secondary.strings.get(key) {
            return val.clone();
        }
        for (candidate_key, candidate_val) in &secondary.strings {
            if candidate_key.ends_with(&suffix) {
                return candidate_val.clone();
            }
        }

        key.to_string()
    }

    pub fn get_with(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut template = self.get(key);
        for (name, val) in args {
            let placeholder = format!("{{{name}}}");
            template = template.replace(&placeholder, val);
        }
        template
    }

    pub fn get_list(&self, key: &str) -> Vec<String> {
        let current_bundle = self.current.load();
        if let Some(list) = current_bundle.lists.get(key) {
            return list.clone();
        }

        let suffix = format!(".{key}");
        for (candidate_key, candidate_list) in &current_bundle.lists {
            if candidate_key.ends_with(&suffix) {
                return candidate_list.clone();
            }
        }

        let is_en = current_bundle.code == "en"
            || self.current_target().starts_with("en")
            || self.current_target() == "en.toml";

        let (primary, secondary) = if is_en {
            (&self.fallback_en, &self.fallback_es)
        } else {
            (&self.fallback_es, &self.fallback_en)
        };

        if let Some(list) = primary.lists.get(key) {
            return list.clone();
        }
        for (candidate_key, candidate_list) in &primary.lists {
            if candidate_key.ends_with(&suffix) {
                return candidate_list.clone();
            }
        }

        if let Some(list) = secondary.lists.get(key) {
            return list.clone();
        }
        for (candidate_key, candidate_list) in &secondary.lists {
            if candidate_key.ends_with(&suffix) {
                return candidate_list.clone();
            }
        }

        Vec::new()
    }

    pub fn current_language_code(&self) -> String {
        self.current.load().code.clone()
    }

    pub fn current_target(&self) -> String {
        (**self.current_target.load()).clone()
    }

    pub fn set_language(&self, target: &str) -> std::io::Result<()> {
        if let Some(bundle) = Self::load_target_bundle(target) {
            let path = Self::resolve_path(target);
            let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();
            self.last_modified.store(Arc::new(mtime));
            self.current.store(Arc::new(bundle));
            self.current_target.store(Arc::new(target.to_string()));
            self.has_changed.store(true, Ordering::SeqCst);
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Language file or variant not found for target: {target}"),
            ))
        }
    }

    pub fn check_and_reload(&self) -> bool {
        let changed = self.has_changed.swap(false, Ordering::SeqCst);
        let target = self.current_target();
        let path = Self::resolve_path(&target);
        if let Ok(metadata) = fs::metadata(&path)
            && let Ok(mtime) = metadata.modified()
        {
            let last = self.last_modified.load();
            if **last != Some(mtime) {
                self.last_modified.store(Arc::new(Some(mtime)));
                if let Some(bundle) = Self::load_target_bundle(&target) {
                    self.current.store(Arc::new(bundle));
                    return true;
                }
            }
        }
        changed
    }

    pub fn reload(&self) {
        let target = self.current_target();
        if let Some(bundle) = Self::load_target_bundle(&target) {
            let path = Self::resolve_path(&target);
            let mtime = fs::metadata(&path).and_then(|m| m.modified()).ok();
            self.last_modified.store(Arc::new(mtime));
            self.current.store(Arc::new(bundle));
            self.has_changed.store(true, Ordering::SeqCst);
        }
    }

    pub fn languages_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("languages")
    }

    pub fn ensure_default_files_exist() {
        let dir = Self::languages_dir();
        let _ = fs::create_dir_all(&dir);

        let legacy_path = dir.join("current_language.toml");
        if legacy_path.exists() {
            let _ = fs::remove_file(legacy_path);
        }

        let es_path = dir.join("es.toml");
        let should_update_es = match fs::read_to_string(&es_path) {
            Ok(content) => !content.contains("header_date_format"),
            Err(_) => true,
        };
        if should_update_es {
            let _ = fs::write(&es_path, EMBEDDED_ES);
        }

        let en_path = dir.join("en.toml");
        let should_update_en = match fs::read_to_string(&en_path) {
            Ok(content) => !content.contains("header_date_format"),
            Err(_) => true,
        };
        if should_update_en {
            let _ = fs::write(&en_path, EMBEDDED_EN);
        }
    }

    pub fn list_languages(&self) -> Vec<LanguageInfo> {
        Self::ensure_default_files_exist();
        let dir = Self::languages_dir();
        let current_target = self.current_target();
        let current_code = self.current_language_code();

        let mut items = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("toml")
                    && let Ok(content) = fs::read_to_string(&path)
                {
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default();

                    if file_stem == "current_language"
                        || file_name.starts_with('.')
                        || file_stem.starts_with('.')
                    {
                        continue;
                    }

                    let bundle = LanguageBundle::parse(&content);
                    let code = if bundle.code.is_empty() {
                        file_stem.clone()
                    } else {
                        bundle.code.clone()
                    };
                    let name = if bundle.name.is_empty() {
                        code.clone()
                    } else {
                        bundle.name.clone()
                    };

                    let is_current = current_target == file_name
                        || current_target == file_stem
                        || current_target == code
                        || current_code == code
                        || Path::new(&current_target) == path;

                    items.push(LanguageInfo {
                        name,
                        code,
                        path,
                        is_current,
                    });
                }
            }
        }

        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let mut unique: Vec<LanguageInfo> = Vec::new();
        for item in items {
            if let Some(existing) = unique.iter_mut().find(|x| {
                x.code.eq_ignore_ascii_case(&item.code)
                    || x.name.eq_ignore_ascii_case(&item.name)
                    || x.path == item.path
            }) {
                if item.is_current {
                    existing.is_current = true;
                }
            } else {
                unique.push(item);
            }
        }

        let mut found_current = false;
        for item in unique.iter_mut() {
            if item.is_current {
                if found_current {
                    item.is_current = false;
                } else {
                    found_current = true;
                }
            }
        }

        if !unique.is_empty() && !found_current {
            if let Some(pos) = unique.iter().position(|x| x.code == "es" || x.code == "en") {
                unique[pos].is_current = true;
            } else {
                unique[0].is_current = true;
            }
        }

        unique
    }

    fn resolve_path(target: &str) -> PathBuf {
        if target.contains('/') {
            PathBuf::from(target)
        } else {
            let filename = if target.ends_with(".toml") {
                target.to_string()
            } else {
                format!("{target}.toml")
            };
            Self::languages_dir().join(filename)
        }
    }

    fn load_target_bundle(target: &str) -> Option<LanguageBundle> {
        let path = Self::resolve_path(target);
        let disk_content = fs::read_to_string(&path).ok();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(target);

        let is_en = target == "en"
            || target == "en.toml"
            || stem == "en"
            || stem.starts_with("en_")
            || stem.starts_with("en-");
        let is_es = target == "es"
            || target == "es.toml"
            || stem == "es"
            || stem.starts_with("es_")
            || stem.starts_with("es-");

        let mut base = if is_en {
            LanguageBundle::parse(EMBEDDED_EN)
        } else if is_es {
            LanguageBundle::parse(EMBEDDED_ES)
        } else {
            LanguageBundle::parse(EMBEDDED_ES)
        };

        if let Some(content) = disk_content {
            let disk_bundle = LanguageBundle::parse(&content);
            if disk_bundle.code == "en" && !is_en {
                base = LanguageBundle::parse(EMBEDDED_EN);
            }
            base.merge(disk_bundle);
            Some(base)
        } else if is_en || is_es {
            Some(base)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_get() {
        let toml_data = r#"
            name = "Test"
            code = "tt"
            direct_key = "Valor Directo"

            [section]
            nested_key = "Valor Anidado"
            items = ["Uno", "Dos"]
        "#;

        let bundle = LanguageBundle::parse(toml_data);
        assert_eq!(
            bundle.strings.get("direct_key").map(|s| s.as_str()),
            Some("Valor Directo")
        );
        assert_eq!(
            bundle.strings.get("section.nested_key").map(|s| s.as_str()),
            Some("Valor Anidado")
        );
        assert_eq!(
            bundle.strings.get("nested_key").map(|s| s.as_str()),
            Some("Valor Anidado")
        );
        assert_eq!(
            bundle.lists.get("section.items").cloned(),
            Some(vec!["Uno".to_string(), "Dos".to_string()])
        );
    }

    #[test]
    fn test_service_lookup_and_fallback() {
        let service = LangService::new("es");
        assert_eq!(service.get("notifications"), "Notificaciones");
        assert_eq!(
            service.get("dashboard.notifications_title"),
            "Notificaciones"
        );
        assert_eq!(service.get("non_existent_key"), "non_existent_key");
    }

    #[test]
    fn test_service_args() {
        let service = LangService::new("es");
        let formatted = service.get_with("user_prefix", &[("user_name", "testuser")]);
        assert_eq!(formatted, "Usuario: ");
    }

    #[test]
    fn test_english_service() {
        let service = LangService::new("en");
        assert_eq!(service.get("settings.title"), "Settings");
        assert_eq!(service.get("dashboard.sound"), "Sound");
        assert_eq!(service.get("dashboard.dnd"), "Do Not Disturb");
        assert_eq!(service.get("settings.capsule_style_normal"), "Normal");
        assert_eq!(service.get("settings.capsule_style_concave"), "Concave");
    }

    #[test]
    fn test_list_languages() {
        let service = LangService::new("es");
        let list = service.list_languages();
        assert!(!list.is_empty());
        let es_count = list
            .iter()
            .filter(|l| l.code == "es" || l.name == "Español")
            .count();
        assert_eq!(es_count, 1);
        let active_count = list.iter().filter(|l| l.is_current).count();
        assert_eq!(active_count, 1);
    }
}
