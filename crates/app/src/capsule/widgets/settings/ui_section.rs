use gpui::{
    Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px,
};
use services::{AppState, CapsuleStyle};
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_ui_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let cards_round = module.cards_round_input.parse::<f32>().unwrap_or(16.0);

    let (
        header_title,
        header_sub,
        style_title,
        style_sub,
        round_title,
        round_sub,
        sat_title,
        sat_sub,
        card_title,
        card_sub,
        margin_title,
        margin_sub,
        idle_title,
        idle_sub,
        gap_title,
        gap_sub,
        anim_title,
        anim_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.ui_header_title"),
            lang.get("settings.ui_header_subtitle"),
            lang.get("settings.capsule_style_title"),
            lang.get("settings.capsule_style_subtitle"),
            lang.get("settings.capsule_round_title"),
            lang.get("settings.capsule_round_subtitle"),
            lang.get("settings.satellite_round_title"),
            lang.get("settings.satellite_round_subtitle"),
            lang.get("settings.cards_round_title"),
            lang.get("settings.cards_round_subtitle"),
            lang.get("settings.margin_top_title"),
            lang.get("settings.margin_top_subtitle"),
            lang.get("settings.idle_height_title"),
            lang.get("settings.idle_height_subtitle"),
            lang.get("settings.gap_title"),
            lang.get("settings.gap_subtitle"),
            lang.get("settings.anim_duration_title"),
            lang.get("settings.anim_duration_subtitle"),
        )
    } else {
        (
            "Apariencia & Dimensiones".to_string(),
            "Ajusta los radios, márgenes, separación de elementos y la velocidad de animación de la interfaz.".to_string(),
            "Estilo de Cápsula".to_string(),
            "Elige entre la cápsula flotante normal o la cápsula cóncava integrada a la pantalla.".to_string(),
            "Radio de la Cápsula".to_string(),
            "Curvatura de las esquinas de la barra principal y modales (px).".to_string(),
            "Radio de Satélites".to_string(),
            "Curvatura de esquinas para paneles orbitales (wifi, bluetooth, etc).".to_string(),
            "Radio de Tarjetas".to_string(),
            "Curvatura de esquinas para tarjetas del dashboard y widgets.".to_string(),
            "Margen Superior".to_string(),
            "Distancia desde el borde superior de la pantalla hasta la barra.".to_string(),
            "Altura Base (Idle)".to_string(),
            "Altura en reposo de la cápsula en la barra superior.".to_string(),
            "Separación entre Elementos (Gap)".to_string(),
            "Espaciado entre paneles orbitales y módulos.".to_string(),
            "Duración de Animación (ms)".to_string(),
            "Tiempo total de transiciones. Establece 0 para desactivar.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(&header_title, &header_sub, theme))
        .child(render_setting_row(
            &style_title,
            &style_sub,
            render_capsule_style_selector(module, theme, cx),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &round_title,
            &round_sub,
            render_number_stepper(
                SettingsField::CapsuleRound,
                &module.capsule_round_input,
                module.active_field == Some(SettingsField::CapsuleRound),
                2.0,
                0.0,
                60.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &sat_title,
            &sat_sub,
            render_number_stepper(
                SettingsField::SatelliteRound,
                &module.satellite_round_input,
                module.active_field == Some(SettingsField::SatelliteRound),
                2.0,
                0.0,
                40.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &card_title,
            &card_sub,
            render_number_stepper(
                SettingsField::CardsRound,
                &module.cards_round_input,
                module.active_field == Some(SettingsField::CardsRound),
                2.0,
                0.0,
                40.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &margin_title,
            &margin_sub,
            render_number_stepper(
                SettingsField::MarginTop,
                &module.margin_top_input,
                module.active_field == Some(SettingsField::MarginTop),
                2.0,
                0.0,
                40.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &idle_title,
            &idle_sub,
            render_number_stepper(
                SettingsField::IdleHeight,
                &module.idle_height_input,
                module.active_field == Some(SettingsField::IdleHeight),
                1.0,
                16.0,
                60.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &gap_title,
            &gap_sub,
            render_number_stepper(
                SettingsField::Gap,
                &module.gap_input,
                module.active_field == Some(SettingsField::Gap),
                1.0,
                0.0,
                32.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &anim_title,
            &anim_sub,
            render_number_stepper(
                SettingsField::AnimDuration,
                &module.anim_duration_input,
                module.active_field == Some(SettingsField::AnimDuration),
                25.0,
                0.0,
                1000.0,
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
}

#[allow(clippy::too_many_arguments)]
fn render_number_stepper(
    field: SettingsField,
    val_str: &str,
    is_active: bool,
    step: f32,
    min: f32,
    max: f32,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let current_val: f32 = val_str.parse().unwrap_or(0.0);

    let border_color = if is_active {
        theme.accent()
    } else {
        theme.surface().opacity(0.6)
    };

    let val_up = (current_val + step).min(max);
    let val_down = (current_val - step).max(min);

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(
            div()
                .id(ElementId::NamedInteger("step-down".into(), field as u64))
                .w(px(24.0))
                .h(px(24.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.5))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_field_float(field, val_down, cx);
                }))
                .child("-"),
        )
        .child(
            div()
                .id(ElementId::NamedInteger("step-input".into(), field as u64))
                .w(px(72.0))
                .h(px(24.0))
                .rounded(px(6.0))
                .bg(theme.surface().opacity(0.25))
                .border_1()
                .border_color(border_color)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_field(Some(field), cx);
                }))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground())
                        .child(if is_active {
                            format!("{val_str}|")
                        } else {
                            val_str.to_string()
                        }),
                ),
        )
        .child(
            div()
                .id(ElementId::NamedInteger("step-up".into(), field as u64))
                .w(px(24.0))
                .h(px(24.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.5))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_field_float(field, val_up, cx);
                }))
                .child("+"),
        )
}

fn render_capsule_style_selector(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (normal_label, concave_label) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.capsule_style_normal"),
            lang.get("settings.capsule_style_concave"),
        )
    } else {
        ("Normal".to_string(), "Cóncava".to_string())
    };

    let options = [
        (CapsuleStyle::Normal, normal_label),
        (CapsuleStyle::Concave, concave_label),
    ];

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .p_1()
        .gap(px(2.0))
        .rounded_full()
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.25));

    for (style, label) in options {
        let is_active = module.capsule_style == style;

        container = container.child(
            div()
                .id(ElementId::Name(format!("ui-capsule-style-{label}").into()))
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .px_3()
                .py_1()
                .rounded_full()
                .bg(if is_active {
                    theme.surface().opacity(0.85)
                } else {
                    gpui::transparent_black()
                })
                .hover(|s| {
                    if !is_active {
                        s.bg(theme.surface().opacity(0.35))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_capsule_style(style, cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if is_active {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(label),
                ),
        );
    }

    container
}
