use crate::CompositorService;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WallpaperService {
    current: Arc<Mutex<Option<PathBuf>>>,
    config_file: PathBuf,
    changed: tokio::sync::broadcast::Sender<PathBuf>,
}

impl WallpaperService {
    pub fn new(_compositor: CompositorService) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule");
        let _ = fs::create_dir_all(&config_dir);
        let config_file = config_dir.join("current_wallpaper");

        let (changed, _) = tokio::sync::broadcast::channel(16);
        let service = Self {
            current: Arc::new(Mutex::new(None)),
            config_file,
            changed,
        };

        service.restore_saved_wallpaper();
        service
    }

    pub fn get_current_wallpaper(&self) -> Option<PathBuf> {
        if let Ok(guard) = self.current.lock() {
            if let Some(ref path) = *guard {
                return Some(path.clone());
            }
        }
        if self.config_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_file) {
                let path = PathBuf::from(content.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }
        Self::detect_system_wallpaper()
    }

    pub fn detect_system_wallpaper() -> Option<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            let plasma_cfg = home.join(".config/plasma-org.kde.plasma.desktop-appletsrc");
            if let Ok(content) = fs::read_to_string(&plasma_cfg) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(raw) = trimmed.strip_prefix("Image=") {
                        let clean = raw.trim().trim_start_matches("file://");
                        let p = PathBuf::from(clean);
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        let search_dirs = [
            Path::new("/usr/share/wallpapers"),
            Path::new("/usr/share/backgrounds"),
        ];

        for dir in search_dirs {
            if !dir.is_dir() {
                continue;
            }
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        if ext.eq_ignore_ascii_case("jpg")
                            || ext.eq_ignore_ascii_case("jpeg")
                            || ext.eq_ignore_ascii_case("png")
                            || ext.eq_ignore_ascii_case("webp")
                        {
                            return Some(path);
                        }
                    } else if path.is_dir() {
                        let candidate_4k = path.join("contents/images/3840x2160.png");
                        if candidate_4k.is_file() {
                            return Some(candidate_4k);
                        }
                        let candidate_4k_jpg = path.join("contents/images/3840x2160.jpg");
                        if candidate_4k_jpg.is_file() {
                            return Some(candidate_4k_jpg);
                        }
                        let candidate_fhd = path.join("contents/images/1920x1080.png");
                        if candidate_fhd.is_file() {
                            return Some(candidate_fhd);
                        }
                    }
                }
            }
        }

        None
    }

    pub fn get_blurred_wallpaper(&self) -> Option<PathBuf> {
        let current = self.get_current_wallpaper()?;
        Self::create_or_get_blur(&current)
    }

    pub fn create_or_get_blur(src: &Path) -> Option<PathBuf> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let metadata = std::fs::metadata(src).ok()?;
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut hasher = DefaultHasher::new();
        src.hash(&mut hasher);
        modified.hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = PathBuf::from("/tmp/capsule_blur");
        let _ = std::fs::create_dir_all(&cache_dir);
        let cached_path = cache_dir.join(format!("{:x}.jpg", hash));

        if cached_path.is_file() && cached_path.metadata().map(|m| m.len() > 0).unwrap_or(false) {
            return Some(cached_path);
        }

        let img = image::open(src).ok()?;
        let orig_w = img.width();
        let orig_h = img.height();
        if orig_w == 0 || orig_h == 0 {
            return None;
        }

        let target_w = 960.min(orig_w).max(1);
        let target_h = ((target_w as f32 * (orig_h as f32 / orig_w as f32)) as u32).max(1);

        let downscaled = img
            .resize_exact(target_w, target_h, image::imageops::FilterType::Triangle)
            .to_rgba8();
        let blurred = image::imageops::blur(&downscaled, 4.5);
        let rgb_image = image::DynamicImage::ImageRgba8(blurred).to_rgb8();
        rgb_image.save(&cached_path).ok()?;

        Some(cached_path)
    }

    pub fn create_or_get_screen_wallpaper(src: &Path, max_w: u32, max_h: u32) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let Ok(metadata) = std::fs::metadata(src) else {
            return src.to_path_buf();
        };

        if let Ok((w, h)) = image::image_dimensions(src) {
            if w <= max_w && h <= max_h {
                return src.to_path_buf();
            }
        }

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut hasher = DefaultHasher::new();
        src.hash(&mut hasher);
        modified.hash(&mut hasher);
        max_w.hash(&mut hasher);
        max_h.hash(&mut hasher);
        let hash = hasher.finish();

        let cache_dir = PathBuf::from("/tmp/capsule_screen");
        let _ = std::fs::create_dir_all(&cache_dir);
        let cached_path = cache_dir.join(format!("{:x}.jpg", hash));

        if cached_path.is_file() && cached_path.metadata().map(|m| m.len() > 0).unwrap_or(false) {
            return cached_path;
        }

        let Ok(img) = image::open(src) else {
            return src.to_path_buf();
        };

        let orig_w = img.width();
        let orig_h = img.height();
        if orig_w == 0 || orig_h == 0 {
            return src.to_path_buf();
        }

        let ratio = (max_w as f32 / orig_w as f32).min(max_h as f32 / orig_h as f32);
        if ratio >= 1.0 {
            return src.to_path_buf();
        }

        let target_w = ((orig_w as f32 * ratio).round() as u32).max(1);
        let target_h = ((orig_h as f32 * ratio).round() as u32).max(1);

        let downscaled = img.resize(target_w, target_h, image::imageops::FilterType::Triangle);
        let rgb_image = downscaled.to_rgb8();

        if rgb_image.save(&cached_path).is_ok() {
            cached_path
        } else {
            src.to_path_buf()
        }
    }

    fn restore_saved_wallpaper(&self) {
        if self.config_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_file) {
                let path = PathBuf::from(content.trim());
                if path.exists() {
                    if let Ok(mut current_guard) = self.current.lock() {
                        *current_guard = Some(path.clone());
                    }
                    let _ = self.changed.send(path);
                    return;
                }
            }
        }

        if let Some(detected) = Self::detect_system_wallpaper() {
            if let Ok(mut current_guard) = self.current.lock() {
                *current_guard = Some(detected.clone());
            }
            let _ = self.changed.send(detected);
        }
    }

    pub fn set_wallpaper(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();
        if !path.exists() {
            eprintln!("[WallpaperService] File does not exist: {:?}", path);
            return false;
        }

        let path_buf = path.to_path_buf();
        let path_str = path_buf.to_string_lossy().to_string();

        if let Ok(mut current_guard) = self.current.lock() {
            *current_guard = Some(path_buf.clone());
        }

        let _ = self.changed.send(path_buf.clone());

        let blur_target = path.to_path_buf();
        crate::spawn_tokio(async move {
            let _ = crate::spawn_blocking(move || {
                Self::create_or_get_blur(&blur_target);
            })
            .await;
        });

        let _ = fs::write(&self.config_file, &path_str);
        true
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<PathBuf> {
        self.changed.subscribe()
    }

    pub fn dominant_color(path: &Path) -> Result<[u8; 3], String> {
        let source_path = Self::create_or_get_blur(path).unwrap_or_else(|| path.to_path_buf());
        let image = image::open(&source_path)
            .or_else(|_| image::open(path))
            .map_err(|error| error.to_string())?
            .thumbnail(96, 96)
            .to_rgba8();
        Self::palette_color(image.pixels().map(|pixel| pixel.0))
            .ok_or_else(|| "Wallpaper has no visible pixels".to_owned())
    }

    fn palette_color(pixels: impl Iterator<Item = [u8; 4]>) -> Option<[u8; 3]> {
        let mut bins = vec![(0u32, [0u32; 3]); 4096];
        for [r, g, b, alpha] in pixels {
            if alpha < 128 {
                continue;
            }
            let index = ((r as usize >> 4) << 8) | ((g as usize >> 4) << 4) | (b as usize >> 4);
            let bin = &mut bins[index];
            bin.0 += 1;
            for (sum, value) in bin.1.iter_mut().zip([r, g, b]) {
                *sum += value as u32;
            }
        }
        let bin = bins
            .into_iter()
            .filter(|(count, _)| *count > 0)
            .max_by_key(|(count, channels)| {
                let max = channels.iter().copied().max().unwrap_or(0) / count;
                let min = channels.iter().copied().min().unwrap_or(0) / count;
                *count * (64 + max - min)
            })?;
        Some(bin.1.map(|sum| (sum / bin.0) as u8))
    }

    pub fn get_current(&self) -> Option<PathBuf> {
        self.current.lock().ok().and_then(|guard| guard.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_uses_wallpaper_pixels_and_ignores_transparency() {
        let pixels = [[210, 70, 40, 255], [211, 71, 41, 255], [0, 0, 255, 0]];
        assert_eq!(
            WallpaperService::palette_color(pixels.into_iter()),
            Some([210, 70, 40])
        );
        assert_eq!(
            WallpaperService::palette_color([[0, 0, 0, 0]].into_iter()),
            None
        );
    }
}
