use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default = "default_browser")]
    pub browser: String,
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default = "default_file_manager", alias = "fileManager")]
    pub file_manager: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            terminal: default_terminal(),
            browser: default_browser(),
            editor: default_editor(),
            file_manager: default_file_manager(),
        }
    }
}

fn default_terminal() -> String {
    std::env::var("TERMINAL").unwrap_or_else(|_| "alacritty".to_string())
}

fn default_browser() -> String {
    std::env::var("BROWSER").unwrap_or_else(|_| "firefox".to_string())
}

fn default_editor() -> String {
    std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "nvim".to_string())
}

fn default_file_manager() -> String {
    std::env::var("FILEMANAGER").unwrap_or_else(|_| "thunar".to_string())
}
