use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CapsuleStyle {
    #[default]
    Normal,
    Concave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ShellMode {
    #[default]
    Island,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarWidgetId {
    Workspaces,
    Clock,
    Media,
    Status,
    Tray,
    Power,
}

impl BarWidgetId {
    pub const ALL: [BarWidgetId; 6] = [
        BarWidgetId::Workspaces,
        BarWidgetId::Clock,
        BarWidgetId::Media,
        BarWidgetId::Status,
        BarWidgetId::Tray,
        BarWidgetId::Power,
    ];

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Workspaces => "grid-dots.svg",
            Self::Clock => "clock.svg",
            Self::Media => "music.svg",
            Self::Status => "wifi.svg",
            Self::Tray => "package.svg",
            Self::Power => "power.svg",
        }
    }

    pub fn translation_key(&self) -> &'static str {
        match self {
            Self::Workspaces => "settings.widget_workspaces",
            Self::Clock => "settings.widget_clock",
            Self::Media => "settings.widget_media",
            Self::Status => "settings.widget_status",
            Self::Tray => "settings.widget_tray",
            Self::Power => "settings.widget_power",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarZone {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BarConfig {
    #[serde(default = "default_bar_left")]
    pub left: Vec<BarWidgetId>,
    #[serde(default = "default_bar_center")]
    pub center: Vec<BarWidgetId>,
    #[serde(default = "default_bar_right")]
    pub right: Vec<BarWidgetId>,
    #[serde(default)]
    pub convex: bool,
}

fn default_bar_left() -> Vec<BarWidgetId> {
    vec![BarWidgetId::Workspaces]
}

fn default_bar_center() -> Vec<BarWidgetId> {
    vec![BarWidgetId::Clock]
}

fn default_bar_right() -> Vec<BarWidgetId> {
    vec![
        BarWidgetId::Media,
        BarWidgetId::Tray,
        BarWidgetId::Status,
        BarWidgetId::Power,
    ]
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            left: default_bar_left(),
            center: default_bar_center(),
            right: default_bar_right(),
            convex: false,
        }
    }
}

impl BarConfig {
    pub fn is_widget_present(&self, widget: BarWidgetId) -> bool {
        self.left.contains(&widget) || self.center.contains(&widget) || self.right.contains(&widget)
    }

    pub fn zone_of(&self, widget: BarWidgetId) -> Option<BarZone> {
        if self.left.contains(&widget) {
            Some(BarZone::Left)
        } else if self.center.contains(&widget) {
            Some(BarZone::Center)
        } else if self.right.contains(&widget) {
            Some(BarZone::Right)
        } else {
            None
        }
    }

    pub fn remove_widget(&mut self, widget: BarWidgetId) {
        self.left.retain(|w| *w != widget);
        self.center.retain(|w| *w != widget);
        self.right.retain(|w| *w != widget);
    }

    pub fn add_widget_to_zone(&mut self, widget: BarWidgetId, zone: BarZone) {
        self.remove_widget(widget);
        match zone {
            BarZone::Left => self.left.push(widget),
            BarZone::Center => self.center.push(widget),
            BarZone::Right => self.right.push(widget),
        }
    }

    pub fn move_widget(
        &mut self,
        widget: BarWidgetId,
        target_zone: BarZone,
        target_index: Option<usize>,
    ) {
        self.remove_widget(widget);
        let list = match target_zone {
            BarZone::Left => &mut self.left,
            BarZone::Center => &mut self.center,
            BarZone::Right => &mut self.right,
        };
        if let Some(idx) = target_index {
            let clamped = idx.min(list.len());
            list.insert(clamped, widget);
        } else {
            list.push(widget);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CapsuleLocation {
    #[default]
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CapsuleAnimation {
    #[default]
    Fluid,
    Cubic,
    Expo,
    Back,
    Spring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentTransition {
    #[default]
    FadeScale,
    Fade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperTransition {
    #[default]
    Fade,
    Slide,
    Zoom,
    Wipe,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrbitConfig {
    pub power: bool,
    pub usage: bool,
    pub record: bool,
    pub shelf: bool,
}

impl Default for OrbitConfig {
    fn default() -> Self {
        Self {
            power: true,
            usage: true,
            record: true,
            shelf: true,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UIConfig {
    #[serde(default)]
    pub orbits: OrbitConfig,
    #[serde(default = "default_enabled")]
    pub use_new_capsule: bool,

    #[serde(default, alias = "mode")]
    pub shell_mode: ShellMode,

    #[serde(default)]
    pub bar: BarConfig,

    #[serde(default)]
    pub dynamic_colors: bool,

    #[serde(default = "default_enabled")]
    pub dark_mode: bool,

    #[serde(default = "default_enabled")]
    pub clock_24_hour: bool,

    #[serde(default)]
    pub capsule_style: CapsuleStyle,

    #[serde(default, alias = "location")]
    pub capsule_location: CapsuleLocation,

    #[serde(default, alias = "animation_style", alias = "animation_curve")]
    pub capsule_animation: CapsuleAnimation,

    #[serde(default)]
    pub content_transition: ContentTransition,

    #[serde(default)]
    pub wallpaper_path: Option<std::path::PathBuf>,

    #[serde(default)]
    pub wallpaper_transition: WallpaperTransition,

    #[serde(default = "default_spring_bounciness")]
    pub spring_bounciness: f32,

    #[serde(default = "default_enabled")]
    pub squash_stretch: bool,

    #[serde(default = "default_enabled")]
    pub stagger_animation: bool,

    #[serde(default)]
    pub reduced_motion: bool,

    #[serde(default = "default_capsule_round", alias = "capsule_radius")]
    pub capsule_round: f32,

    #[serde(
        default = "default_satellite_round",
        alias = "satellite_radius",
        alias = "sattelite_round"
    )]
    pub satellite_round: f32,

    #[serde(
        default = "default_cards_round",
        alias = "card_round",
        alias = "card_radius"
    )]
    pub cards_round: f32,

    #[serde(default = "default_margin_top", alias = "top_margin")]
    pub margin_top: f32,

    #[serde(default = "default_idle_height")]
    pub idle_height: f32,

    #[serde(default = "default_gap")]
    pub gap: f32,

    #[serde(default = "default_badge_round")]
    pub badge_round: f32,

    #[serde(default = "default_animation_duration_ms")]
    pub animation_duration_ms: u32,

    #[serde(default = "default_language")]
    pub language: String,
}

pub type UiConfig = UIConfig;

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            orbits: OrbitConfig::default(),
            use_new_capsule: true,
            shell_mode: ShellMode::default(),
            bar: BarConfig::default(),
            dynamic_colors: false,
            dark_mode: true,
            clock_24_hour: true,
            capsule_style: CapsuleStyle::Normal,
            capsule_location: CapsuleLocation::Top,
            capsule_animation: CapsuleAnimation::Fluid,
            content_transition: ContentTransition::FadeScale,
            wallpaper_path: None,
            wallpaper_transition: WallpaperTransition::Fade,
            spring_bounciness: default_spring_bounciness(),
            squash_stretch: true,
            stagger_animation: true,
            reduced_motion: false,
            capsule_round: default_capsule_round(),
            satellite_round: default_satellite_round(),
            cards_round: default_cards_round(),
            margin_top: default_margin_top(),
            idle_height: default_idle_height(),
            gap: default_gap(),
            badge_round: default_badge_round(),
            animation_duration_ms: default_animation_duration_ms(),
            language: default_language(),
        }
    }
}

impl UIConfig {
    pub fn clock_format(&self) -> &'static str {
        if self.clock_24_hour {
            "%H:%M"
        } else {
            "%I:%M %p"
        }
    }

    pub fn exclusive_zone(&self) -> f32 {
        if self.shell_mode == ShellMode::Bar {
            return 40.0;
        }
        if self.capsule_style == CapsuleStyle::Concave {
            self.idle_height
        } else {
            self.idle_height + self.margin_top
        }
    }
}

fn default_capsule_round() -> f32 {
    24.0
}

fn default_satellite_round() -> f32 {
    20.0
}

fn default_cards_round() -> f32 {
    24.0
}

fn default_margin_top() -> f32 {
    8.0
}

fn default_idle_height() -> f32 {
    25.0
}

fn default_gap() -> f32 {
    8.0
}

fn default_badge_round() -> f32 {
    24.0
}

fn default_spring_bounciness() -> f32 {
    0.75
}

fn default_animation_duration_ms() -> u32 {
    250
}

fn default_language() -> String {
    "es.toml".to_string()
}

fn default_enabled() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_config_preserves_legacy_capsule_and_gets_appearance_defaults() {
        let config: UIConfig = toml::from_str("use_new_capsule = false").expect("legacy config");
        assert!(!config.use_new_capsule);
        assert!(!config.dynamic_colors);
        assert!(config.dark_mode);
        assert_eq!(config.clock_format(), "%H:%M");
        assert_eq!(config.capsule_animation, CapsuleAnimation::Fluid);
        assert_eq!(config.content_transition, ContentTransition::FadeScale);
        assert_eq!(config.spring_bounciness, 0.75);
        assert!(config.squash_stretch);
        assert!(config.stagger_animation);
        assert!(!config.reduced_motion);
        let fresh: UIConfig = toml::from_str("").expect("new config");
        assert!(fresh.use_new_capsule);
        assert_eq!(fresh.capsule_location, CapsuleLocation::Top);
        assert_eq!(fresh.capsule_animation, CapsuleAnimation::Fluid);
        assert_eq!(fresh.content_transition, ContentTransition::FadeScale);
        assert_eq!(fresh.spring_bounciness, 0.75);
        assert!(fresh.squash_stretch);
        assert!(fresh.stagger_animation);
        assert!(!fresh.reduced_motion);
    }

    #[test]
    fn appearance_preferences_survive_serialization() {
        let config = UIConfig {
            dynamic_colors: true,
            dark_mode: false,
            clock_24_hour: false,
            capsule_location: CapsuleLocation::Bottom,
            capsule_animation: CapsuleAnimation::Expo,
            content_transition: ContentTransition::Fade,
            spring_bounciness: 0.9,
            squash_stretch: false,
            stagger_animation: false,
            reduced_motion: true,
            ..UIConfig::default()
        };
        let encoded = toml::to_string(&config).expect("serialize config");
        let decoded: UIConfig = toml::from_str(&encoded).expect("deserialize config");
        assert_eq!(decoded, config);
        assert_eq!(decoded.clock_format(), "%I:%M %p");
        assert_eq!(decoded.capsule_animation, CapsuleAnimation::Expo);
        assert_eq!(decoded.content_transition, ContentTransition::Fade);
        assert_eq!(decoded.spring_bounciness, 0.9);
        assert!(!decoded.squash_stretch);
        assert!(!decoded.stagger_animation);
        assert!(decoded.reduced_motion);
    }

    #[test]
    fn animation_curve_accepts_all_curves() {
        let fluid: UIConfig = toml::from_str("capsule_animation = 'fluid'").expect("fluid");
        assert_eq!(fluid.capsule_animation, CapsuleAnimation::Fluid);
        let cubic: UIConfig = toml::from_str("capsule_animation = 'cubic'").expect("cubic");
        assert_eq!(cubic.capsule_animation, CapsuleAnimation::Cubic);
        let expo: UIConfig = toml::from_str("capsule_animation = 'expo'").expect("expo");
        assert_eq!(expo.capsule_animation, CapsuleAnimation::Expo);
        let back: UIConfig = toml::from_str("capsule_animation = 'back'").expect("back");
        assert_eq!(back.capsule_animation, CapsuleAnimation::Back);
        let spring: UIConfig = toml::from_str("capsule_animation = 'spring'").expect("spring");
        assert_eq!(spring.capsule_animation, CapsuleAnimation::Spring);
    }

    #[test]
    fn orbit_switches_default_to_enabled_and_round_trip_independently() {
        let fresh: UIConfig = toml::from_str("").expect("existing config");
        assert_eq!(fresh.orbits, OrbitConfig::default());
        let config: UIConfig =
            toml::from_str("[orbits]\nusage = false\nshelf = false\n").expect("orbit settings");
        assert!(config.orbits.power);
        assert!(config.orbits.record);
        assert!(!config.orbits.usage);
        assert!(!config.orbits.shelf);
        let encoded = toml::to_string(&config).expect("serialize orbit settings");
        let restored: UIConfig = toml::from_str(&encoded).expect("restore orbit settings");
        assert_eq!(restored.orbits, config.orbits);
    }

    #[test]
    fn location_accepts_bottom_and_rejects_unknown_edges() {
        let config: UIConfig = toml::from_str("location = 'bottom'").expect("bottom config");
        assert_eq!(config.capsule_location, CapsuleLocation::Bottom);
        assert!(toml::from_str::<UIConfig>("capsule_location = 'left'").is_err());
    }

    #[test]
    fn wallpaper_transition_accepts_all_variants() {
        for (str_val, expected) in [
            ("fade", WallpaperTransition::Fade),
            ("slide", WallpaperTransition::Slide),
            ("zoom", WallpaperTransition::Zoom),
            ("wipe", WallpaperTransition::Wipe),
            ("none", WallpaperTransition::None),
        ] {
            let config: UIConfig =
                toml::from_str(&format!("wallpaper_transition = '{str_val}'")).expect(str_val);
            assert_eq!(config.wallpaper_transition, expected);
        }
    }

    #[test]
    fn bar_config_enforces_widget_uniqueness() {
        let mut bar = BarConfig::default();
        assert_eq!(bar.zone_of(BarWidgetId::Workspaces), Some(BarZone::Left));

        bar.add_widget_to_zone(BarWidgetId::Workspaces, BarZone::Center);
        assert_eq!(bar.zone_of(BarWidgetId::Workspaces), Some(BarZone::Center));
        assert!(!bar.left.contains(&BarWidgetId::Workspaces));
        assert!(bar.center.contains(&BarWidgetId::Workspaces));

        bar.move_widget(BarWidgetId::Workspaces, BarZone::Right, Some(0));
        assert_eq!(bar.zone_of(BarWidgetId::Workspaces), Some(BarZone::Right));
        assert_eq!(bar.right.first(), Some(&BarWidgetId::Workspaces));
        assert!(!bar.center.contains(&BarWidgetId::Workspaces));

        bar.remove_widget(BarWidgetId::Workspaces);
        assert_eq!(bar.zone_of(BarWidgetId::Workspaces), None);
        assert!(!bar.is_widget_present(BarWidgetId::Workspaces));
    }

    #[test]
    fn bar_config_serializes_and_deserializes() {
        let default_config = UIConfig::default();
        let encoded = toml::to_string(&default_config).expect("serialize");
        let decoded: UIConfig = toml::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded.bar, default_config.bar);
    }
}
