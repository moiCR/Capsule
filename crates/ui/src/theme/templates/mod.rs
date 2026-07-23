use crate::theme::Theme;
pub mod fish_app;
pub mod ghostty_app;
pub mod gtk_apps;
pub mod qt_apps;
pub mod yazi_app;

pub use fish_app::FishApp;
pub use ghostty_app::GhosttyApp;
pub use gtk_apps::GtkApps;
pub use qt_apps::QtApps;
pub use yazi_app::YaziApp;

pub trait AppTheme {
    fn apply_current_theme(theme: &Theme);
    fn reload_apps();
}
