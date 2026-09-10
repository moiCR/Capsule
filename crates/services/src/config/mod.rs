pub mod defaults;
pub mod lock_screen_config;
pub mod mpris_config;
pub mod record_config;
pub mod ui_config;

pub use defaults::Defaults;
pub use lock_screen_config::{LockScreenConfig, LockscreenConfig};
pub use mpris_config::MprisConfig;
pub use record_config::RecordConfig;
pub use ui_config::{CapsuleStyle, UIConfig, UiConfig};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    #[serde(alias = "lock_screen")]
    pub lockscreen: LockScreenConfig,

    #[serde(default)]
    pub defaults: Defaults,

    #[serde(default)]
    pub ui: UIConfig,

    #[serde(default)]
    #[serde(alias = "music")]
    pub mpris: MprisConfig,

    #[serde(default)]
    pub record: RecordConfig,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path)
            && let Ok(cfg) = toml::from_str::<AppConfig>(&content)
        {
            return cfg;
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
        let content = toml::to_string_pretty(cfg).map_err(std::io::Error::other)?;
        fs::write(config_path, content)
    }

    pub fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule")
            .join("config.toml")
    }
}

#[derive(Clone)]
pub struct ConfigService {
    config: Arc<ArcSwap<AppConfig>>,
    tx: broadcast::Sender<Arc<AppConfig>>,
}

impl Default for ConfigService {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigService {
    pub fn new() -> Self {
        let loaded = AppConfig::load();
        let (tx, _) = broadcast::channel(16);
        Self {
            config: Arc::new(ArcSwap::from_pointee(loaded)),
            tx,
        }
    }

    pub fn get(&self) -> Arc<AppConfig> {
        self.config.load_full()
    }

    pub fn set(&self, new_config: AppConfig) -> std::io::Result<()> {
        AppConfig::save(&new_config)?;
        let arc_config = Arc::new(new_config);
        self.config.store(arc_config.clone());
        let _ = self.tx.send(arc_config);
        Ok(())
    }

    pub fn update<F>(&self, f: F) -> std::io::Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut current = (**self.config.load()).clone();
        f(&mut current);
        AppConfig::save(&current)?;
        let arc_config = Arc::new(current);
        self.config.store(arc_config.clone());
        let _ = self.tx.send(arc_config);
        Ok(())
    }

    pub fn reload(&self) {
        let reloaded = AppConfig::load();
        let arc_config = Arc::new(reloaded);
        self.config.store(arc_config.clone());
        let _ = self.tx.send(arc_config);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<AppConfig>> {
        self.tx.subscribe()
    }
}
