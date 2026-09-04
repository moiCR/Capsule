use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UIConfig {
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
}

pub type UiConfig = UIConfig;

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            capsule_round: default_capsule_round(),
            satellite_round: default_satellite_round(),
            cards_round: default_cards_round(),
            margin_top: default_margin_top(),
            idle_height: default_idle_height(),
            gap: default_gap(),
            badge_round: default_badge_round(),
            animation_duration_ms: default_animation_duration_ms(),
        }
    }
}

impl UIConfig {
    pub fn exclusive_zone(&self) -> f32 {
        self.idle_height + self.margin_top
    }
}

fn default_capsule_round() -> f32 {
    42.0
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
