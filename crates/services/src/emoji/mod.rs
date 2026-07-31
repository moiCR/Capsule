use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmojiItem {
    pub emoji: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub category: String,
}

#[derive(Clone, Default)]
pub struct EmojiService;

impl EmojiService {
    pub fn new() -> Self {
        Self
    }

    pub fn load_emojis(&self) -> Vec<EmojiItem> {
        let json_str = include_str!("../../../assets/emoji.json");
        serde_json::from_str(json_str).unwrap_or_default()
    }

    pub fn copy_emoji(&self, emoji: &str) -> bool {
        use std::io::Write;
        use std::process::{Command, Stdio};

        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(emoji.as_bytes());
            }
            let _ = child.wait();
            return true;
        }

        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(emoji.as_bytes());
            }
            let _ = child.wait();
            return true;
        }

        false
    }
}
