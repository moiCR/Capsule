use std::{
    hash::{Hash, Hasher},
    path::PathBuf,
};
use ui::theme::theme_manager::{ThemeItem, ThemeManager};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WallpaperEntry {
    pub name: String,
    pub path: PathBuf,
    pub thumbnail: Option<PathBuf>,
}
pub(super) struct Catalog {
    pub themes: Vec<ThemeItem>,
    pub wallpapers: Vec<WallpaperEntry>,
}
impl Catalog {
    pub fn load() -> Self {
        let mut wallpapers = Vec::new();
        if let Some(home) = dirs::home_dir()
            && let Ok(entries) = std::fs::read_dir(home.join("Wallpapers"))
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file()
                    || !path.extension().and_then(|s| s.to_str()).is_some_and(|s| {
                        matches!(
                            s.to_ascii_lowercase().as_str(),
                            "png" | "jpg" | "jpeg" | "webp"
                        )
                    })
                {
                    continue;
                }
                let name = path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let thumbnail = thumbnail(&path);
                wallpapers.push(WallpaperEntry {
                    name,
                    path,
                    thumbnail,
                });
            }
        }
        wallpapers.sort_by_key(|item| item.name.to_lowercase());
        Self {
            themes: ThemeManager::read_presets(),
            wallpapers,
        }
    }
}
fn thumbnail(path: &PathBuf) -> Option<PathBuf> {
    let metadata = std::fs::metadata(path).ok()?;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hash);
    metadata.modified().ok()?.hash(&mut hash);
    metadata.len().hash(&mut hash);
    let directory = dirs::cache_dir()?.join("capsule/dashboard-wallpapers");
    std::fs::create_dir_all(&directory).ok()?;
    let output = directory.join(format!("{:x}.png", hash.finish()));
    if !output.exists() {
        image::ImageReader::open(path)
            .ok()?
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?
            .thumbnail(280, 160)
            .save(&output)
            .ok()?;
    }
    Some(output)
}
