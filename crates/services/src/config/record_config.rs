use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordConfig {
    #[serde(default = "default_output")]
    pub output: String,
    #[serde(default = "default_fps")]
    pub fps: u32,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_quality")]
    pub quality: String,
    #[serde(default = "default_container")]
    pub container: String,
    #[serde(default = "default_audio")]
    pub audio: String,
    #[serde(default = "default_include_cursor")]
    pub include_cursor: bool,
}

impl Default for RecordConfig {
    fn default() -> Self {
        Self {
            output: default_output(),
            fps: default_fps(),
            resolution: default_resolution(),
            quality: default_quality(),
            container: default_container(),
            audio: default_audio(),
            include_cursor: default_include_cursor(),
        }
    }
}

fn default_output() -> String {
    "screen".to_string()
}

fn default_fps() -> u32 {
    60
}

fn default_resolution() -> String {
    "native".to_string()
}

fn default_quality() -> String {
    "very_high".to_string()
}

fn default_container() -> String {
    "mp4".to_string()
}

fn default_audio() -> String {
    "desktop".to_string()
}

fn default_include_cursor() -> bool {
    true
}
