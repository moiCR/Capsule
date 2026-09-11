use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::icon_resolver::resolve_file_icon;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShelfItem {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size_bytes: u64,
    pub is_image: bool,
    pub is_dir: bool,
    #[serde(default)]
    pub icon_path: Option<PathBuf>,
}

impl ShelfItem {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let metadata = path.metadata().ok()?;
        let is_dir = metadata.is_dir();
        let size_bytes = metadata.len();

        let name = path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or("archivo")
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .unwrap_or_default();

        let is_image = matches!(
            extension.as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "svg"
        );

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let id = format!("{name}_{timestamp}");

        let icon_path = resolve_file_icon(&path, is_dir);

        Some(Self {
            id,
            path,
            name,
            size_bytes,
            is_image,
            is_dir,
            icon_path,
        })
    }

    pub fn get_icon_path(&self) -> Option<PathBuf> {
        if let Some(ref icon) = self.icon_path {
            if icon.exists() {
                return Some(icon.clone());
            }
        }
        resolve_file_icon(&self.path, self.is_dir)
    }

    pub fn formatted_size(&self) -> String {
        if self.is_dir {
            return "Carpeta".to_string();
        }

        let bytes = self.size_bytes;
        if bytes < 1024 {
            format!("{bytes} B")
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
