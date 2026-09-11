use gpui::{AnyElement, AppContext, Context, Entity, IntoElement};

use crate::capsule::Capsule;
use crate::capsule::CapsuleMode;
use crate::capsule::modules::{
    clipboard::ClipboardModule, dashboard::DashboardModule, emoji::EmojiModule, idle::IdleModule,
    launcher::LauncherModule, notification::NotificationModule, polkit::PolkitModule,
    record::RecordModule, select_theme::SelectThemeModule, settings::SettingsModule,
    shelf::ShelfModule, volume::VolumeModule, wallpaper::WallpaperModule,
};

pub mod clipboard;
pub mod dashboard;
pub mod emoji;
pub mod idle;
pub mod launcher;
pub mod notification;
pub mod polkit;
pub mod record;
pub mod select_theme;
pub mod settings;
pub mod shelf;
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
    pub wallpaper_view: Entity<WallpaperModule>,
    pub clipboard_view: Entity<ClipboardModule>,
    pub emoji_view: Entity<EmojiModule>,
    pub settings_view: Entity<SettingsModule>,
    pub record_view: Entity<RecordModule>,
    pub shelf_view: Entity<ShelfModule>,
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
            wallpaper_view: cx.new(WallpaperModule::new),
            clipboard_view: cx.new(ClipboardModule::new),
            emoji_view: cx.new(EmojiModule::new),
            settings_view: cx.new(SettingsModule::new),
            record_view: cx.new(RecordModule::new),
            shelf_view: cx.new(ShelfModule::new),
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
            CapsuleMode::Wallpaper => self.wallpaper_view.clone().into_any_element(),
            CapsuleMode::Clipboard => self.clipboard_view.clone().into_any_element(),
            CapsuleMode::Emoji => self.emoji_view.clone().into_any_element(),
            CapsuleMode::Settings => self.settings_view.clone().into_any_element(),
            CapsuleMode::Record => self.record_view.clone().into_any_element(),
            CapsuleMode::Shelf => self.shelf_view.clone().into_any_element(),
        }
    }

    pub fn notify_all(&self, cx: &mut Context<Capsule>) {
        self.idle_view.update(cx, |_, cx| cx.notify());
        self.dashboard_view.update(cx, |_, cx| cx.notify());
        self.notification_view.update(cx, |_, cx| cx.notify());
        self.launcher_view.update(cx, |_, cx| cx.notify());
        self.volume_view.update(cx, |_, cx| cx.notify());
        self.polkit_view.update(cx, |_, cx| cx.notify());
        self.select_theme_view.update(cx, |_, cx| cx.notify());
        self.wallpaper_view.update(cx, |_, cx| cx.notify());
        self.clipboard_view.update(cx, |_, cx| cx.notify());
        self.emoji_view.update(cx, |_, cx| cx.notify());
        self.settings_view.update(cx, |_, cx| cx.notify());
        self.record_view.update(cx, |_, cx| cx.notify());
        self.shelf_view.update(cx, |_, cx| cx.notify());
    }
}
