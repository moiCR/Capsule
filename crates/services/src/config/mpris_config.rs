use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MprisConfig {
    #[serde(default = "default_music_players", alias = "music_players")]
    pub players: Vec<String>,
}

impl Default for MprisConfig {
    fn default() -> Self {
        Self {
            players: default_music_players(),
        }
    }
}

pub fn default_music_players() -> Vec<String> {
    vec!["spotify".to_string(), "fastpotify".to_string()]
}
