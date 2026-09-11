use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

static ICON_MAP_CACHE: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();

fn get_active_themes() -> Vec<String> {
    let mut themes = Vec::new();

    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "icon-theme"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !val.is_empty() && !themes.contains(&val) {
                themes.push(val);
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let gtk3 = home.join(".config/gtk-3.0/settings.ini");
        if let Ok(content) = std::fs::read_to_string(gtk3) {
            for line in content.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim() == "gtk-icon-theme-name" {
                        let val = v.trim().trim_matches('\'').trim_matches('"').to_string();
                        if !val.is_empty() && !themes.contains(&val) {
                            themes.push(val);
                        }
                    }
                }
            }
        }
    }

    for fallback in [
        "breeze-dark",
        "breeze",
        "Adwaita",
        "AdwaitaLegacy",
        "hicolor",
    ] {
        let s = fallback.to_string();
        if !themes.contains(&s) {
            themes.push(s);
        }
    }

    themes
}

fn score_icon_path(path_str: &str) -> i32 {
    let mut score = 0;
    let p = path_str.to_lowercase();
    if p.contains("symbolic") {
        score -= 100;
    }
    if p.ends_with(".svg") {
        score += 10;
    }
    if p.contains("scalable") {
        score += 50;
    } else if p.contains("/64") || p.contains("64x64") {
        score += 45;
    } else if p.contains("/48") || p.contains("48x48") {
        score += 40;
    } else if p.contains("/128") || p.contains("128x128") {
        score += 35;
    } else if p.contains("/256") || p.contains("256x256") {
        score += 30;
    } else if p.contains("/32") || p.contains("32x32") {
        score += 25;
    } else if p.contains("/24") || p.contains("24x24") {
        score += 20;
    } else if p.contains("/22") || p.contains("22x22") {
        score += 15;
    } else if p.contains("/16") || p.contains("16x16") {
        score += 10;
    }
    score
}

fn scan_theme_dir(dir: &Path, map: &mut HashMap<String, (i32, PathBuf)>) {
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if !name_str.starts_with('.') {
                    stack.push(path);
                }
            } else if path.is_file() {
                let path_str = path.to_string_lossy();
                if path_str.ends_with(".svg") || path_str.ends_with(".png") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let score = score_icon_path(&path_str);
                        match map.get(stem) {
                            Some((existing_score, _)) if *existing_score >= score => {}
                            _ => {
                                map.insert(stem.to_string(), (score, path));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn build_icon_map() -> HashMap<String, PathBuf> {
    let themes = get_active_themes();
    let mut base_dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        base_dirs.push(home.join(".local/share/icons"));
    }
    base_dirs.push(PathBuf::from("/usr/local/share/icons"));
    base_dirs.push(PathBuf::from("/usr/share/icons"));
    base_dirs.push(PathBuf::from("/usr/share/pixmaps"));

    let mut result = HashMap::new();

    for theme in themes.iter().rev() {
        let mut theme_map = HashMap::new();
        for base in &base_dirs {
            let theme_path = base.join(theme);
            if theme_path.is_dir() {
                scan_theme_dir(&theme_path, &mut theme_map);
            }
        }
        for (stem, (_, path)) in theme_map {
            result.insert(stem, path);
        }
    }

    let pixmaps = PathBuf::from("/usr/share/pixmaps");
    if pixmaps.is_dir() {
        let mut pixmaps_map = HashMap::new();
        scan_theme_dir(&pixmaps, &mut pixmaps_map);
        for (stem, (_, path)) in pixmaps_map {
            result.entry(stem).or_insert(path);
        }
    }

    result
}

fn get_icon_map() -> &'static HashMap<String, PathBuf> {
    ICON_MAP_CACHE.get_or_init(build_icon_map)
}

fn get_icon_names_for_path(path: &Path, is_dir: bool) -> Vec<String> {
    if is_dir {
        return vec!["folder".to_string(), "inode-directory".to_string()];
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    match file_name.as_str() {
        "makefile" => return vec!["text-x-makefile".to_string(), "text-x-generic".to_string()],
        "dockerfile" => {
            return vec![
                "text-x-dockerfile".to_string(),
                "text-x-generic".to_string(),
            ];
        }
        "cargo.toml" => return vec!["application-toml".to_string(), "text-x-generic".to_string()],
        "license" | "licence" | "copying" => {
            return vec!["text-plain".to_string(), "text-x-generic".to_string()];
        }
        _ => {}
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "md" | "markdown" => vec![
            "text-markdown".to_string(),
            "x-office-document".to_string(),
            "text-x-generic".to_string(),
        ],
        "rs" => vec![
            "text-rust".to_string(),
            "text-x-rust".to_string(),
            "text-x-generic".to_string(),
        ],
        "py" | "pyw" => vec![
            "text-x-python".to_string(),
            "text-x-python3".to_string(),
            "text-x-generic".to_string(),
        ],
        "c" | "h" => vec![
            "text-x-csrc".to_string(),
            "text-x-c".to_string(),
            "text-x-generic".to_string(),
        ],
        "cpp" | "cxx" | "cc" | "hpp" | "hxx" => vec![
            "text-x-c++src".to_string(),
            "text-x-c++".to_string(),
            "text-x-generic".to_string(),
        ],
        "js" | "mjs" | "cjs" => vec![
            "application-javascript".to_string(),
            "text-javascript".to_string(),
            "text-x-script".to_string(),
        ],
        "ts" | "mts" | "cts" => vec![
            "application-typescript".to_string(),
            "text-typescript".to_string(),
            "text-x-script".to_string(),
        ],
        "jsx" | "tsx" => vec![
            "text-jsx".to_string(),
            "text-tsx".to_string(),
            "application-javascript".to_string(),
            "text-x-script".to_string(),
        ],
        "json" => vec![
            "application-json".to_string(),
            "text-x-script".to_string(),
            "text-x-generic".to_string(),
        ],
        "toml" => vec!["application-toml".to_string(), "text-x-generic".to_string()],
        "yaml" | "yml" => vec![
            "application-yaml".to_string(),
            "application-x-yaml".to_string(),
            "text-x-generic".to_string(),
        ],
        "xml" => vec![
            "application-xml".to_string(),
            "text-xml".to_string(),
            "text-x-generic".to_string(),
        ],
        "html" | "htm" => vec!["text-html".to_string(), "text-x-generic".to_string()],
        "css" | "scss" | "sass" | "less" => {
            vec!["text-css".to_string(), "text-x-generic".to_string()]
        }
        "sh" | "bash" | "zsh" | "fish" => vec![
            "text-x-shellscript".to_string(),
            "text-x-script".to_string(),
            "application-x-executable".to_string(),
        ],
        "txt" | "log" | "conf" | "cfg" | "ini" => {
            vec!["text-plain".to_string(), "text-x-generic".to_string()]
        }
        "pdf" => vec![
            "application-pdf".to_string(),
            "x-office-document".to_string(),
        ],
        "doc" | "docx" | "odt" | "rtf" => vec![
            "application-msword".to_string(),
            "x-office-document".to_string(),
            "x-office-document-template".to_string(),
        ],
        "xls" | "xlsx" | "ods" | "csv" => vec![
            "application-vnd.ms-excel".to_string(),
            "x-office-spreadsheet".to_string(),
        ],
        "ppt" | "pptx" | "odp" => vec![
            "application-vnd.ms-powerpoint".to_string(),
            "x-office-presentation".to_string(),
        ],
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => vec![
            "application-zip".to_string(),
            "package-x-generic".to_string(),
            "application-x-archive".to_string(),
        ],
        "deb" | "rpm" | "pkg" => vec!["package-x-generic".to_string()],
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" => {
            vec!["audio-mpeg".to_string(), "audio-x-generic".to_string()]
        }
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" => {
            vec!["video-mp4".to_string(), "video-x-generic".to_string()]
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "bmp" | "ico" | "tiff" => {
            vec!["image-x-generic".to_string()]
        }
        "iso" | "img" => vec!["application-x-cd-image".to_string()],
        "exe" | "bin" | "appimage" => vec!["application-x-executable".to_string()],
        "ttf" | "otf" | "woff" | "woff2" => vec!["font-x-generic".to_string()],
        _ => {
            if path.exists() {
                if let Ok(output) = Command::new("gio")
                    .args(["info", "-a", "standard::icon"])
                    .arg(path)
                    .output()
                {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout);
                        for line in text.lines() {
                            if let Some(rest) = line.trim().strip_prefix("standard::icon:") {
                                let mut names = Vec::new();
                                for item in rest.split(',') {
                                    let trimmed = item.trim();
                                    if !trimmed.is_empty() && !trimmed.ends_with("-symbolic") {
                                        names.push(trimmed.to_string());
                                    }
                                }
                                if !names.is_empty() {
                                    names.push("text-x-generic".to_string());
                                    names.push("application-x-generic".to_string());
                                    return names;
                                }
                            }
                        }
                    }
                }
            }
            vec![
                "text-x-generic".to_string(),
                "application-x-generic".to_string(),
            ]
        }
    }
}

pub fn resolve_file_icon(path: &Path, is_dir: bool) -> Option<PathBuf> {
    let map = get_icon_map();
    let candidates = get_icon_names_for_path(path, is_dir);

    for candidate in candidates {
        if let Some(found) = map.get(&candidate) {
            return Some(found.clone());
        }
    }

    if is_dir {
        map.get("folder")
            .or_else(|| map.get("inode-directory"))
            .cloned()
    } else {
        map.get("text-x-generic")
            .or_else(|| map.get("application-x-generic"))
            .or_else(|| map.get("x-office-document"))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_folder_icon() {
        let path = PathBuf::from("/usr");
        let icon = resolve_file_icon(&path, true);
        assert!(icon.is_some());
        let icon_path = icon.unwrap();
        assert!(icon_path.exists());
    }

    #[test]
    fn test_resolve_markdown_icon() {
        let path = PathBuf::from("/home/moi/pro/Capsule/README.md");
        let icon = resolve_file_icon(&path, false);
        assert!(icon.is_some());
        let icon_path = icon.unwrap();
        assert!(icon_path.exists());
    }

    #[test]
    fn test_resolve_rust_icon() {
        let path = PathBuf::from("/home/moi/pro/Capsule/crates/app/src/main.rs");
        let icon = resolve_file_icon(&path, false);
        assert!(icon.is_some());
        let icon_path = icon.unwrap();
        assert!(icon_path.exists());
    }
}
