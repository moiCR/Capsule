use crate::theme::Theme;
pub mod fish_app;
pub mod ghostty_app;
pub mod gtk_apps;
pub mod qt_apps;
pub mod yazi_app;
pub mod kitty_app;

pub use fish_app::FishApp;
pub use ghostty_app::GhosttyApp;
pub use gtk_apps::GtkApps;
pub use qt_apps::QtApps;
pub use yazi_app::YaziApp;
pub use kitty_app::KittyApp;

pub trait AppTheme: Send + Sync + 'static {
    fn apply_current_theme(&self, theme: &Theme);
    fn reload_apps(&self);
}
