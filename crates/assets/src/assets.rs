use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;
use std::borrow::Cow;

#[derive(RustEmbed)]
#[folder = "icons"]
pub struct Assets;

#[derive(RustEmbed)]
#[folder = "fonts"]
pub struct FontAssets;

pub fn load_fonts() -> Vec<Cow<'static, [u8]>> {
    let mut fonts = Vec::new();
    for file in FontAssets::iter() {
        if let Some(f) = FontAssets::get(file.as_ref()) {
            fonts.push(f.data);
        }
    }
    fonts
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let key = path.strip_prefix("icons/").unwrap_or(path);
        let key = key.strip_prefix("./").unwrap_or(key);
        let key = key.strip_prefix("/").unwrap_or(key);

        Ok(Self::get(key).or_else(|| Self::get(path)).map(|f| f.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}
