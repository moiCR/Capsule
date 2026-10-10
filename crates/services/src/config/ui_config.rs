use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CapsuleStyle {
    #[default]
    Normal,
    Concave,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UIConfig {
    #[serde(default = "default_enabled")]
    pub use_new_capsule: bool,

    #[serde(default)]
    pub dynamic_colors: bool,

    #[serde(default = "default_enabled")]
    pub dark_mode: bool,

    #[serde(default = "default_enabled")]
    pub clock_24_hour: bool,

    #[serde(default)]
    pub capsule_style: CapsuleStyle,

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
            use_new_capsule: true,
            dynamic_colors: false,
            dark_mode: true,
            clock_24_hour: true,
            capsule_style: CapsuleStyle::Normal,
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
        let fresh: UIConfig = toml::from_str("").expect("new config");
        assert!(fresh.use_new_capsule);
    }

    #[test]
    fn appearance_preferences_survive_serialization() {
        let config = UIConfig {
            dynamic_colors: true,
            dark_mode: false,
            clock_24_hour: false,
            ..UIConfig::default()
        };
        let encoded = toml::to_string(&config).expect("serialize config");
        let decoded: UIConfig = toml::from_str(&encoded).expect("deserialize config");
        assert_eq!(decoded, config);
        assert_eq!(decoded.clock_format(), "%I:%M %p");
    }
}
