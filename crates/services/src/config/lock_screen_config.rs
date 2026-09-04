use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockScreenConfig {
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default = "default_show_media_player")]
    pub show_media_player: bool,
    #[serde(default = "default_show_clock")]
    pub show_clock: bool,
    #[serde(default = "default_time_format")]
    pub time_format: String,
    #[serde(default = "default_date_format")]
    pub date_format: String,
}

impl Default for LockScreenConfig {
    fn default() -> Self {
        Self {
            idle_timeout: default_idle_timeout(),
            show_media_player: default_show_media_player(),
            show_clock: default_show_clock(),
            time_format: default_time_format(),
            date_format: default_date_format(),
        }
    }
}

pub type LockscreenConfig = LockScreenConfig;

fn default_idle_timeout() -> u64 {
    900
}

fn default_show_media_player() -> bool {
    true
}

fn default_show_clock() -> bool {
    true
}

fn default_time_format() -> String {
    "%H:%M".to_string()
}

fn default_date_format() -> String {
    "%A, %B %e, %Y".to_string()
}
