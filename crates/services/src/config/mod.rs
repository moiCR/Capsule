use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockscreenConfig {
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64, // In seconds. Default 900 = 15 minutes. 0 = disabled.
}

impl Default for LockscreenConfig {
    fn default() -> Self {
        Self {
            idle_timeout: default_idle_timeout(),
        }
    }
}

fn default_idle_timeout() -> u64 {
    900
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub lockscreen: LockscreenConfig,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(cfg) = toml::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }

        let default_cfg = AppConfig::default();
        let _ = Self::save(&default_cfg);
        default_cfg
    }

    pub fn save(cfg: &AppConfig) -> std::io::Result<()> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(cfg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(config_path, content)
    }

    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("config.toml")
    }
}
