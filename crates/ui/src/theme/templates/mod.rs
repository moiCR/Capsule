use crate::theme::Theme;
pub mod engine;
pub mod gtk_apps;
pub mod manager;
pub mod plugin;
pub mod qt_apps;

pub use engine::{ColorFormats, TemplateEngine};
pub use gtk_apps::GtkApps;
pub use manager::TemplatePluginManager;
pub use plugin::{HookConfig, TemplateConfig, TemplatePlugin};
pub use qt_apps::QtApps;

pub trait AppTheme: Send + Sync + 'static {
    fn apply_current_theme(&self, theme: &Theme);
    fn reload_apps(&self);
}
