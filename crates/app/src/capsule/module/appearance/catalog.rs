use ui::theme::theme_manager::{ThemeItem, ThemeManager};

pub(super) struct Catalog {
    pub themes: Vec<ThemeItem>,
}

impl Catalog {
    pub fn load() -> Self {
        Self {
            themes: ThemeManager::read_presets(),
        }
    }
}
