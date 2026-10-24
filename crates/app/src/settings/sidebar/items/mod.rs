use gpui::App;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsTab {
    Appearance = 0,
    Island = 1,
    Bar = 2,
    Apps = 3,
    Media = 4,
    LockScreen = 5,
    System = 6,
    Record = 7,
}

pub trait SidebarItem {
    fn tab(&self) -> SettingsTab;
    fn title(&self, cx: &App) -> String;
    fn icon(&self) -> &'static str;
}

pub struct TabSidebarItem {
    pub tab: SettingsTab,
    pub title_key: &'static str,
    pub icon: &'static str,
}

impl TabSidebarItem {
    pub const fn new(tab: SettingsTab, title_key: &'static str, icon: &'static str) -> Self {
        Self {
            tab,
            title_key,
            icon,
        }
    }
}

impl SidebarItem for TabSidebarItem {
    fn tab(&self) -> SettingsTab {
        self.tab
    }

    fn title(&self, cx: &App) -> String {
        if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .language
                .get(self.title_key)
        } else {
            self.title_key.to_string()
        }
    }

    fn icon(&self) -> &'static str {
        self.icon
    }
}
