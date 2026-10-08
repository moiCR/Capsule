pub mod snippets;
pub use snippets::Snippet;

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

pub enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
}

fn decode_content(bytes: Vec<u8>, is_image: bool) -> Result<ClipboardContent, String> {
    if bytes.is_empty() {
        return Err("clipboard.copy_error".into());
    }
    if is_image {
        let image =
            image::load_from_memory(&bytes).map_err(|_| "clipboard.copy_error".to_string())?;
        let mut png = std::io::Cursor::new(Vec::new());
        image
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|_| "clipboard.copy_error".to_string())?;
        Ok(ClipboardContent::Image(png.into_inner()))
    } else {
        String::from_utf8(bytes)
            .map(ClipboardContent::Text)
            .map_err(|_| "clipboard.copy_error".to_string())
    }
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
    snippets: Arc<Mutex<snippets::SnippetsConfig>>,
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
            snippets: Arc::new(Mutex::new(snippets::SnippetsConfig::load())),
        };

        let has_cliphist = Command::new("cliphist")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if has_cliphist {
            service.ensure_watch_daemon();
        } else {
            let svc = service.clone();
            crate::spawn_tokio(async move {
                svc.start_watcher().await;
            });
        }

        service
    }

    fn ensure_watch_daemon(&self) {
        crate::spawn_tokio(async {
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
            tokio::time::sleep(Duration::from_secs(2)).await;

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
                            let file_path = cache_dir.join(format!("{id_trimmed}-preview.png"));

                            let is_cached = std::fs::read(&file_path)
                                .is_ok_and(|bytes| image::load_from_memory(&bytes).is_ok());

                            if !is_cached {
                                if let Ok(out) = Command::new("cliphist")
                                    .args(["decode", id_trimmed])
                                    .output()
                                {
                                    if out.status.success()
                                        && let Ok(ClipboardContent::Image(bytes)) =
                                            decode_content(out.stdout, true)
                                    {
                                        let _ = std::fs::write(&file_path, bytes);
                                    }
                                }
                            }

                            let resolved_path = if std::fs::read(&file_path)
                                .is_ok_and(|bytes| image::load_from_memory(&bytes).is_ok())
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
                .map(|text| {
                    use std::hash::{Hash, Hasher};
                    let mut hash = std::collections::hash_map::DefaultHasher::new();
                    text.hash(&mut hash);
                    ClipboardItem {
                        id: format!("fallback-{:x}", hash.finish()),
                        preview: text.clone(),
                        is_image: false,
                        image_path: None,
                    }
                })
                .collect();
        }

        Vec::new()
    }

    pub fn read_item(&self, item: &ClipboardItem) -> Result<ClipboardContent, String> {
        if !item.is_image
            && self
                .fallback_history
                .lock()
                .is_ok_and(|history| history.iter().any(|text| text == &item.preview))
        {
            return Ok(ClipboardContent::Text(item.preview.clone()));
        }
        if item.is_image
            && let Some(path) = &item.image_path
            && let Ok(bytes) = std::fs::read(path)
            && let Ok(content) = decode_content(bytes, true)
        {
            return Ok(content);
        }
        let output = Command::new("cliphist")
            .args(["decode", &item.id])
            .output()
            .map_err(|_| "clipboard.copy_error".to_string())?;
        if !output.status.success() {
            return Err("clipboard.copy_error".into());
        }
        decode_content(output.stdout, item.is_image)
    }

    pub fn get_item_text(&self, item: &ClipboardItem) -> String {
        if !item.is_image {
            if let Ok(out) = Command::new("cliphist").args(["decode", &item.id]).output() {
                if out.status.success() && !out.stdout.is_empty() {
                    return String::from_utf8_lossy(&out.stdout).to_string();
                }
            }
        }
        item.preview.clone()
    }

    pub fn copy_binary(&self, bytes: &[u8], mime: &str) -> bool {
        let Ok(mut child) = Command::new("wl-copy")
            .args(["--type", mime])
            .stdin(Stdio::piped())
            .spawn()
        else {
            return false;
        };
        let written = child.stdin.take().is_some_and(|mut stdin| {
            use std::io::Write;
            stdin.write_all(bytes).is_ok()
        });
        let success = child.wait().is_ok_and(|status| status.success());
        written && success
    }

    pub fn copy_item(&self, item: &ClipboardItem) -> bool {
        if item.is_image {
            if let Some(ref path) = item.image_path {
                if path.exists() {
                    if let Ok(bytes) = std::fs::read(path) {
                        if self.copy_binary(&bytes, "image/png") {
                            return true;
                        }
                    }
                }
            }
            if let Ok(out) = Command::new("cliphist").args(["decode", &item.id]).output() {
                if out.status.success() && !out.stdout.is_empty() {
                    return self.copy_binary(&out.stdout, "image/png");
                }
            }
            return false;
        }

        if let Ok(out) = Command::new("cliphist").args(["decode", &item.id]).output() {
            if out.status.success() && !out.stdout.is_empty() {
                let text = String::from_utf8_lossy(&out.stdout);
                return self.copy_text(&text);
            }
        }

        self.copy_text(&item.preview)
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

    pub fn get_snippets(&self) -> Vec<Snippet> {
        self.snippets
            .lock()
            .map(|guard| guard.snippets.clone())
            .unwrap_or_default()
    }

    pub fn add_snippet(&self, title: &str, content: &str) -> Option<Snippet> {
        let mut guard = self.snippets.lock().ok()?;
        Some(guard.add(title.to_string(), content.to_string()))
    }

    pub fn remove_snippet(&self, id: &str) -> bool {
        let mut guard = match self.snippets.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        guard.remove(id)
    }

    pub fn pin_from_history(&self, item: &ClipboardItem) -> bool {
        if item.is_image {
            return false;
        }

        let content = item.preview.trim();
        if content.is_empty() {
            return false;
        }

        let first_line = content.lines().next().unwrap_or(content).trim();
        let title = if first_line.chars().count() > 30 {
            let truncated: String = first_line.chars().take(27).collect();
            format!("{truncated}...")
        } else {
            first_line.to_string()
        };

        self.add_snippet(&title, content).is_some()
    }

    pub fn is_pinned(&self, content: &str) -> bool {
        self.snippets
            .lock()
            .map(|guard| {
                guard
                    .snippets
                    .iter()
                    .any(|snippet| snippet.content == content)
            })
            .unwrap_or(false)
    }

    pub fn copy_text(&self, text: &str) -> bool {
        let Ok(mut child) = Command::new("wl-copy")
            .args(["--type", "text/plain;charset=utf-8"])
            .stdin(Stdio::piped())
            .spawn()
        else {
            return false;
        };
        let written = child.stdin.take().is_some_and(|mut stdin| {
            use std::io::Write;
            stdin.write_all(text.as_bytes()).is_ok()
        });
        let success = child.wait().is_ok_and(|status| status.success());
        written && success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_payloads_are_validated_and_normalized_to_png() {
        let mut source = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 1)
            .write_to(&mut source, image::ImageFormat::Jpeg)
            .expect("JPEG fixture");
        let ClipboardContent::Image(bytes) =
            decode_content(source.into_inner(), true).expect("image payload")
        else {
            panic!("image payload");
        };
        assert_eq!(
            image::guess_format(&bytes).expect("image format"),
            image::ImageFormat::Png
        );
        let decoded = image::load_from_memory(&bytes).expect("PNG payload");
        assert_eq!((decoded.width(), decoded.height()), (2, 1));
    }

    #[test]
    fn invalid_clipboard_data_never_becomes_a_copy_payload() {
        assert!(decode_content(Vec::new(), false).is_err());
        assert!(decode_content(vec![0xff], false).is_err());
        assert!(decode_content(b"not an image".to_vec(), true).is_err());
        let ClipboardContent::Text(text) =
            decode_content("primera\nsegunda 🚀".as_bytes().to_vec(), false)
                .expect("UTF-8 payload")
        else {
            panic!("text payload");
        };
        assert_eq!(text, "primera\nsegunda 🚀");
    }

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
            snippets: Arc::new(Mutex::new(snippets::SnippetsConfig::default())),
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
