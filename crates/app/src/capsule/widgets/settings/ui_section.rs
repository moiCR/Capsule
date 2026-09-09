use gpui::{Context, ElementId, IntoElement, ParentElement, Styled, div, px};
use services::{AppState, CapsuleStyle};
use ui::theme::Theme;

use super::setting_item::{
    render_card_container, render_hero_header, render_row_divider, render_slider_row,
    render_toggle_row,
};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_ui_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (
        hero_title,
        hero_sub,
        concave_title,
        concave_sub,
        idle_title,
        margin_title,
        gap_title,
        round_title,
        sat_title,
        card_title,
        anim_title,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.tab_capsule"),
            lang.get("settings.ui_header_subtitle"),
            lang.get("settings.capsule_style_title"),
            lang.get("settings.capsule_style_subtitle"),
            lang.get("settings.idle_height_title"),
            lang.get("settings.margin_top_title"),
            lang.get("settings.gap_title"),
            lang.get("settings.capsule_round_title"),
            lang.get("settings.satellite_round_title"),
            lang.get("settings.cards_round_title"),
            lang.get("settings.anim_duration_title"),
        )
    } else {
        (
            "Cápsula".to_string(),
            "Ajusta el estilo, dimensiones, radios de curvatura y velocidad de animación de la interfaz.".to_string(),
            "Modo Cóncavo".to_string(),
            "Cápsula cóncava integrada a la pantalla o barra flotante.".to_string(),
            "Altura Base (Idle)".to_string(),
            "Margen Superior".to_string(),
            "Separación entre Elementos (Gap)".to_string(),
            "Radio de la Cápsula".to_string(),
            "Radio de Satélites".to_string(),
            "Radio de Tarjetas".to_string(),
            "Duración de Animación".to_string(),
        )
    };

    let is_concave = module.capsule_style == CapsuleStyle::Concave;

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "dashboard.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_toggle_row(
                    ElementId::Name("capsule-concave-toggle".into()),
                    &concave_title,
                    Some(&concave_sub),
                    is_concave,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        let next = if this.capsule_style == CapsuleStyle::Concave {
                            CapsuleStyle::Normal
                        } else {
                            CapsuleStyle::Concave
                        };
                        this.set_capsule_style(next, cx);
                    }),
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::IdleHeight,
                    &idle_title,
                    &module.idle_height_input,
                    "px",
                    16.0,
                    60.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::MarginTop,
                    &margin_title,
                    &module.margin_top_input,
                    "px",
                    0.0,
                    40.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::Gap,
                    &gap_title,
                    &module.gap_input,
                    "px",
                    0.0,
                    32.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::CapsuleRound,
                    &round_title,
                    &module.capsule_round_input,
                    "px",
                    0.0,
                    24.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::SatelliteRound,
                    &sat_title,
                    &module.satellite_round_input,
                    "px",
                    0.0,
                    24.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::CardsRound,
                    &card_title,
                    &module.cards_round_input,
                    "px",
                    0.0,
                    24.0,
                    1.0,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_slider_row(
                    SettingsField::AnimDuration,
                    &anim_title,
                    &module.anim_duration_input,
                    "ms",
                    0.0,
                    1000.0,
                    25.0,
                    theme,
                    cx,
                )),
        )
}
