#[allow(clippy::module_inception)]
pub mod capsule;
pub mod modules;
pub mod widgets;

pub use capsule::Capsule;

pub const MARGIN_TOP: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleMode {
    Default,
    Dashboard,
    Notification,
    Launcher,
    Volume,
    Polkit,
    SelectTheme,
    CreateTheme,
    Wallpaper,
}

impl CapsuleMode {
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            CapsuleMode::Default => (138.0, 42.0),
            CapsuleMode::Dashboard => (440.0, 500.0),
            CapsuleMode::Notification => (348.0, 68.0),
            CapsuleMode::Launcher => (348.0, 480.0),
            CapsuleMode::Volume => (280.0, 48.0),
            CapsuleMode::Polkit => (348.0, 240.0),
            CapsuleMode::SelectTheme => (348.0, 500.0),
            CapsuleMode::CreateTheme => (348.0, 500.0),
            CapsuleMode::Wallpaper => (700.0, 240.0),
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            CapsuleMode::Default => 42.0,
            CapsuleMode::Dashboard => 42.0,
            CapsuleMode::Notification => 42.0,
            CapsuleMode::Launcher => 42.0,
            CapsuleMode::Volume => 42.0,
            CapsuleMode::Polkit => 42.0,
            CapsuleMode::SelectTheme => 42.0,
            CapsuleMode::CreateTheme => 42.0,
            CapsuleMode::Wallpaper => 32.0,
        }
    }
}

pub fn apple_island_ease(t: f32) -> f32 {
    if t >= 1.0 {
        return 1.0;
    }
    let p = 1.0 - t;
    1.0 - p * p * p
}
