use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardItem {
    pub id: String,
    pub preview: String,
    pub is_image: bool,
    pub image_path: Option<PathBuf>,
}

fn parse_image_preview(raw: &str) -> String {
    let inner = raw
        .strip_prefix("[[ binary data")
        .unwrap_or(raw)
        .trim_end_matches(']')
        .trim();

    let tokens: Vec<&str> = inner.split_whitespace().collect();
    let mut size_part = String::new();
    let mut format_part = String::new();
    let mut dimensions_part = String::new();

    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index];
        if (token.chars().all(|c| c.is_ascii_digit() || c == '.')
            || token.chars().any(|c| c.is_ascii_digit()))
            && index + 1 < tokens.len()
        {
            let next = tokens[index + 1];
            if next.ends_with('B') || next == "bytes" {
                size_part = format!("{token} {next}");
                index += 2;
                continue;
            }
        }
        if token.contains('x')
            && token.chars().all(|c| c.is_ascii_digit() || c == 'x')
            && !token.starts_with('x')
            && !token.ends_with('x')
        {
            dimensions_part = token.replace('x', "×");
            index += 1;
            continue;
        }
        if matches!(
            token.to_lowercase().as_str(),
            "png" | "jpeg" | "jpg" | "webp" | "gif" | "bmp" | "svg"
        ) {
            format_part = token.to_uppercase();
            index += 1;
            continue;
        }
        index += 1;
    }

    let mut parts = Vec::new();
    if !format_part.is_empty() {
        parts.push(format_part);
    }
    if !dimensions_part.is_empty() {
        parts.push(dimensions_part);
    }
    if !size_part.is_empty() {
        parts.push(size_part);
    }

    if parts.is_empty() {
        if !inner.is_empty() {
            inner.to_string()
        } else {
            String::new()
        }
    } else {
        parts.join(" • ")
    }
}

#[derive(Clone)]
pub struct ClipboardService {
    fallback_history: Arc<Mutex<VecDeque<String>>>,
}

impl Default for ClipboardService {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardService {
    pub fn new() -> Self {
        let service = Self {
            fallback_history: Arc::new(Mutex::new(VecDeque::new())),
        };

        service.ensure_watch_daemon();

        let svc = service.clone();
        tokio::spawn(async move {
            svc.start_watcher().await;
        });

        service
    }

    fn ensure_watch_daemon(&self) {
        tokio::spawn(async {
            let is_running = Command::new("pgrep")
                .args(["-f", "cliphist store"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if !is_running {
                let _ = Command::new("sh")
                    .args([
                        "-c",
                        "wl-paste --type text --watch cliphist store >/dev/null 2>&1 &",
                    ])
                    .spawn();

                let _ = Command::new("sh")
                    .args([
                        "-c",
                        "wl-paste --type image --watch cliphist store >/dev/null 2>&1 &",
                    ])
                    .spawn();
            }
        });
    }

    async fn start_watcher(&self) {
        let mut last_text = String::new();
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            if let Ok(out) = tokio::process::Command::new("wl-paste")
                .args(["-t", "text"])
                .output()
                .await
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !text.is_empty() && text != last_text {
                        last_text = text.clone();

                        if let Ok(mut guard) = self.fallback_history.lock() {
                            guard.retain(|x| x != &text);
                            guard.push_front(text);
                            if guard.len() > 50 {
                                guard.pop_back();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn fetch_history(&self) -> Vec<ClipboardItem> {
        let output = Command::new("cliphist").arg("list").output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut items = Vec::new();

                for line in stdout.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    if let Some((id, rest)) = line.split_once('\t') {
                        let id_trimmed = id.trim();
                        let rest_trimmed = rest.trim();
                        let is_image = rest_trimmed.starts_with("[[ binary data")
                            || rest_trimmed.contains("[[ binary data");

                        let (preview, image_path) = if is_image {
                            let clean_preview = parse_image_preview(rest_trimmed);
                            let cache_dir = PathBuf::from("/tmp/capsule_clipboard");
                            let _ = std::fs::create_dir_all(&cache_dir);
                            let file_path = cache_dir.join(format!("{id_trimmed}.png"));

                            let is_cached = file_path.exists()
                                && file_path.metadata().map(|m| m.len() > 0).unwrap_or(false);

                            if !is_cached {
                                if let Ok(out) = Command::new("cliphist")
                                    .args(["decode", id_trimmed])
                                    .output()
                                {
                                    if out.status.success() && !out.stdout.is_empty() {
                                        let _ = std::fs::write(&file_path, &out.stdout);
                                    }
                                }
                            }

                            let resolved_path = if file_path.exists()
                                && file_path.metadata().map(|m| m.len() > 0).unwrap_or(false)
                            {
                                Some(file_path)
                            } else {
                                None
                            };

                            (clean_preview, resolved_path)
                        } else {
                            (rest_trimmed.to_string(), None)
                        };

                        items.push(ClipboardItem {
                            id: id_trimmed.to_string(),
                            preview,
                            is_image,
                            image_path,
                        });
                    }
                }

                if !items.is_empty() {
                    return items;
                }
            }
        }

        if let Ok(guard) = self.fallback_history.lock() {
            return guard
                .iter()
                .enumerate()
                .map(|(idx, text)| ClipboardItem {
                    id: idx.to_string(),
                    preview: text.clone(),
                    is_image: false,
                    image_path: None,
                })
                .collect();
        }

        Vec::new()
    }

    pub fn copy_item(&self, item: &ClipboardItem) -> bool {
        let sh_cmd = format!("cliphist decode '{}' | wl-copy", item.id);
        if let Ok(st) = Command::new("sh").args(["-c", &sh_cmd]).status() {
            if st.success() {
                return true;
            }
        }

        if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(item.preview.as_bytes());
            }
            let _ = child.wait();
            return true;
        }

        false
    }

    pub fn clear_history(&self) -> bool {
        if let Ok(mut guard) = self.fallback_history.lock() {
            guard.clear();
        }
        let _ = Command::new("cliphist").arg("wipe").status();
        let _ = Command::new("wl-copy").arg("-c").status();
        let _ = std::fs::remove_dir_all("/tmp/capsule_clipboard");
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image_preview() {
        assert_eq!(
            parse_image_preview("[[ binary data 37 KiB png 563x773 ]]"),
            "PNG • 563×773 • 37 KiB"
        );
        assert_eq!(
            parse_image_preview("[[ binary data 1.2 MiB jpeg 1920x1080 ]]"),
            "JPEG • 1920×1080 • 1.2 MiB"
        );
        assert_eq!(parse_image_preview("[[ binary data 500 B ]]"), "500 B");
        assert_eq!(parse_image_preview("[[ binary data ]]"), "");
    }

    #[test]
    fn test_fetch_history_with_real_cliphist() {
        let service = ClipboardService {
            fallback_history: Arc::new(Mutex::new(VecDeque::new())),
        };
        let items = service.fetch_history();
        for item in &items {
            if item.is_image {
                assert!(!item.preview.contains("[[ binary data"));
                if let Some(ref path) = item.image_path {
                    assert!(path.exists());
                }
            }
        }
    }
}
