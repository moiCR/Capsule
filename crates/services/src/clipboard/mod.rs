use std::collections::VecDeque;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardItem {
    pub id: String,
    pub preview: String,
    pub is_image: bool,
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

    /// Spawns cliphist watch daemon processes if not already active.
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
                        let is_image = rest.contains("[[ binary data")
                            || rest.contains("image/")
                            || rest.contains("PNG")
                            || rest.contains("JPEG");

                        items.push(ClipboardItem {
                            id: id.trim().to_string(),
                            preview: rest.trim().to_string(),
                            is_image,
                        });
                    }
                }

                if !items.is_empty() {
                    return items;
                }
            }
        }

        // Fallback to internal in-memory history
        if let Ok(guard) = self.fallback_history.lock() {
            return guard
                .iter()
                .enumerate()
                .map(|(idx, text)| ClipboardItem {
                    id: idx.to_string(),
                    preview: text.clone(),
                    is_image: false,
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
        true
    }
}
