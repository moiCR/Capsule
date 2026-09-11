use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snippet {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SnippetsConfig {
    #[serde(default)]
    pub snippets: Vec<Snippet>,
}

impl SnippetsConfig {
    pub fn get_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("snippets.toml")
    }

    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(cfg) = toml::from_str::<SnippetsConfig>(&content)
        {
            return cfg;
        }

        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, toml_str)
    }

    pub fn add(&mut self, title: String, content: String) -> Snippet {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let id = format!("{timestamp}");
        let snippet = Snippet { id, title, content };
        self.snippets.insert(0, snippet.clone());
        let _ = self.save();
        snippet
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let previous_count = self.snippets.len();
        self.snippets.retain(|snippet| snippet.id != id);
        if self.snippets.len() != previous_count {
            let _ = self.save();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippets_config_add_and_remove() {
        let mut config = SnippetsConfig::default();
        let snippet = config.add("Git Push".to_string(), "git push --force".to_string());
        assert_eq!(config.snippets.len(), 1);
        assert_eq!(config.snippets[0].title, "Git Push");

        let removed = config.remove(&snippet.id);
        assert!(removed);
        assert_eq!(config.snippets.len(), 0);
    }
}
