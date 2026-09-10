#[allow(clippy::module_inception)]
pub mod capsule;
pub mod container;
pub mod modules;
pub mod satellites;
pub mod widgets;

pub use capsule::Capsule;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleMode {
    Default,
    Dashboard,
    Notification,
    Launcher,
    Volume,
    Polkit,
    SelectTheme,
    Wallpaper,
    Clipboard,
    Emoji,
    Settings,
    Record,
}

impl CapsuleMode {
    pub fn dimensions(&self) -> (f32, f32) {
        match self {
            CapsuleMode::Default => (138.0, 42.0),
            CapsuleMode::Dashboard => (490.0, 480.0),
            CapsuleMode::Notification => (348.0, 68.0),
            CapsuleMode::Launcher => (380.0, 360.0),
            CapsuleMode::Volume => (280.0, 42.0),
            CapsuleMode::Polkit => (380.0, 190.0),
            CapsuleMode::SelectTheme => (680.0, 180.0),
            CapsuleMode::Wallpaper => (700.0, 240.0),
            CapsuleMode::Clipboard => (380.0, 360.0),
            CapsuleMode::Emoji => (430.0, 390.0),
            CapsuleMode::Settings => (840.0, 560.0),
            CapsuleMode::Record => (220.0, 42.0),
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
            CapsuleMode::Wallpaper => 42.0,
            CapsuleMode::Clipboard => 42.0,
            CapsuleMode::Emoji => 42.0,
            CapsuleMode::Settings => 24.0,
            CapsuleMode::Record => 42.0,
        }
    }
}

fn spring_eval(t: f32, zeta: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let wd = 2.0 * std::f32::consts::PI;
    let omega0 = wd / (1.0 - zeta * zeta).sqrt();
    let beta = zeta * omega0;
    let scale = 1.0 / (1.0 - (-beta).exp());
    let envelope = (-beta * t).exp();
    let osc = (wd * t).cos() + (beta / wd) * (wd * t).sin();
    (1.0 - envelope * osc) * scale
}

pub fn apple_island_spring(t: f32) -> f32 {
    spring_eval(t, 0.75)
}

pub fn apple_island_morph(t: f32, expanding: bool) -> (f32, f32) {
    if t <= 0.0 {
        return (0.0, 0.0);
    }
    if t >= 1.0 {
        return (1.0, 1.0);
    }
    let (uw, uh) = if expanding {
        (t.powf(0.92), t.powf(1.04))
    } else {
        (t.powf(1.04), t.powf(0.92))
    };
    (spring_eval(uw, 0.73), spring_eval(uh, 0.77))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_bounds() {
        assert_eq!(apple_island_spring(0.0), 0.0);
        assert_eq!(apple_island_spring(-0.5), 0.0);
        assert_eq!(apple_island_spring(1.0), 1.0);
        assert_eq!(apple_island_spring(1.5), 1.0);
    }

    #[test]
    fn test_spring_overshoot() {
        let mid = apple_island_spring(0.5);
        assert!(mid > 1.02 && mid < 1.05);
    }

    #[test]
    fn test_morph_bounds() {
        let (w_start, h_start) = apple_island_morph(0.0, true);
        assert_eq!(w_start, 0.0);
        assert_eq!(h_start, 0.0);

        let (w_end, h_end) = apple_island_morph(1.0, true);
        assert_eq!(w_end, 1.0);
        assert_eq!(h_end, 1.0);

        let (w_end_c, h_end_c) = apple_island_morph(1.0, false);
        assert_eq!(w_end_c, 1.0);
        assert_eq!(h_end_c, 1.0);
    }

    #[test]
    fn test_morph_asymmetry() {
        let (w_exp, h_exp) = apple_island_morph(0.2, true);
        assert!(w_exp > h_exp);

        let (w_col, h_col) = apple_island_morph(0.2, false);
        assert!(h_col > w_col);
    }
}
