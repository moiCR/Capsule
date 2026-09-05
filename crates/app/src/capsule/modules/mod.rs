use gpui::{AnyElement, AppContext, Context, Entity, IntoElement};

use crate::capsule::Capsule;
use crate::capsule::CapsuleMode;
use crate::capsule::modules::{
    clipboard::ClipboardModule, create_theme::CreateThemeModule, dashboard::DashboardModule,
    emoji::EmojiModule, idle::IdleModule, launcher::LauncherModule,
    notification::NotificationModule, polkit::PolkitModule, select_theme::SelectThemeModule,
    settings::SettingsModule, volume::VolumeModule, wallpaper::WallpaperModule,
};

pub mod clipboard;
pub mod create_theme;
pub mod dashboard;
pub mod emoji;
pub mod idle;
pub mod launcher;
pub mod notification;
pub mod polkit;
pub mod select_theme;
pub mod settings;
pub mod volume;
pub mod wallpaper;

pub struct CapsuleModules {
    pub idle_view: Entity<IdleModule>,
    pub dashboard_view: Entity<DashboardModule>,
    pub notification_view: Entity<NotificationModule>,
    pub launcher_view: Entity<LauncherModule>,
    pub volume_view: Entity<VolumeModule>,
    pub polkit_view: Entity<PolkitModule>,
    pub select_theme_view: Entity<SelectThemeModule>,
    pub create_theme_view: Entity<CreateThemeModule>,
    pub wallpaper_view: Entity<WallpaperModule>,
    pub clipboard_view: Entity<ClipboardModule>,
    pub emoji_view: Entity<EmojiModule>,
    pub settings_view: Entity<SettingsModule>,
}

impl CapsuleModules {
    pub fn new(cx: &mut Context<Capsule>) -> Self {
        Self {
            idle_view: cx.new(IdleModule::new),
            dashboard_view: cx.new(DashboardModule::new),
            notification_view: cx.new(NotificationModule::new),
            launcher_view: cx.new(LauncherModule::new),
            volume_view: cx.new(VolumeModule::new),
            polkit_view: cx.new(PolkitModule::new),
            select_theme_view: cx.new(SelectThemeModule::new),
            create_theme_view: cx.new(CreateThemeModule::new),
            wallpaper_view: cx.new(WallpaperModule::new),
            clipboard_view: cx.new(ClipboardModule::new),
            emoji_view: cx.new(EmojiModule::new),
            settings_view: cx.new(SettingsModule::new),
        }
    }

    pub fn render_active_view(&self, mode: CapsuleMode) -> AnyElement {
        match mode {
            CapsuleMode::Default => self.idle_view.clone().into_any_element(),
            CapsuleMode::Dashboard => self.dashboard_view.clone().into_any_element(),
            CapsuleMode::Notification => self.notification_view.clone().into_any_element(),
            CapsuleMode::Launcher => self.launcher_view.clone().into_any_element(),
            CapsuleMode::Volume => self.volume_view.clone().into_any_element(),
            CapsuleMode::Polkit => self.polkit_view.clone().into_any_element(),
            CapsuleMode::SelectTheme => self.select_theme_view.clone().into_any_element(),
            CapsuleMode::CreateTheme => self.create_theme_view.clone().into_any_element(),
            CapsuleMode::Wallpaper => self.wallpaper_view.clone().into_any_element(),
            CapsuleMode::Clipboard => self.clipboard_view.clone().into_any_element(),
            CapsuleMode::Emoji => self.emoji_view.clone().into_any_element(),
            CapsuleMode::Settings => self.settings_view.clone().into_any_element(),
        }
    }
}
