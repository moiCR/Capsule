use gpui::{Context, ElementId, IntoElement, ParentElement, Styled, div, prelude::*, px};
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_ui_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let cards_round = module.cards_round_input.parse::<f32>().unwrap_or(16.0);

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(
            "Apariencia & Dimensiones",
            "Ajusta los radios, márgenes, separación de elementos y la velocidad de animación de la interfaz.",
            theme,
        ))
        .child(render_setting_row(
            "Radio de la Cápsula",
            "Curvatura de las esquinas de la barra principal y modales (px).",
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
            "Radio de Satélites",
            "Curvatura de esquinas para paneles orbitales (wifi, bluetooth, etc).",
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
            "Radio de Tarjetas",
            "Curvatura de esquinas para tarjetas del dashboard y widgets.",
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
            "Margen Superior",
            "Distancia desde el borde superior de la pantalla hasta la barra.",
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
            "Altura Base (Idle)",
            "Altura en reposo de la cápsula en la barra superior.",
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
            "Separación entre Elementos (Gap)",
            "Espaciado entre paneles orbitales y módulos.",
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
            "Duración de Animación (ms)",
            "Tiempo total de transiciones. Establece 0 para desactivar.",
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
                .w(px(26.0))
                .h(px(26.0))
                .rounded(px(8.0))
                .bg(theme.surface().opacity(0.4))
                .hover(|s| s.bg(theme.surface().opacity(0.7)))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(14.0))
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
                .w(px(90.0))
                .h(px(26.0))
                .rounded(px(8.0))
                .bg(theme.background())
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
                .w(px(26.0))
                .h(px(26.0))
                .rounded(px(8.0))
                .bg(theme.surface().opacity(0.4))
                .hover(|s| s.bg(theme.surface().opacity(0.7)))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(14.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_field_float(field, val_up, cx);
                }))
                .child("+"),
        )
}
