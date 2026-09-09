use gpui::{Context, ElementId, IntoElement, ParentElement, Styled, div, prelude::*, px};
use services::AppState;
use ui::theme::Theme;

use super::setting_item::{
    render_card_container, render_control_row, render_hero_header, render_int_slider_row,
    render_row_divider, render_subsection_trigger_button, render_toggle_row,
};
use crate::capsule::modules::settings::{SettingsField, SettingsModule, SettingsSubSection};

pub fn render_lockscreen_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let idle_secs = module.idle_timeout_input.parse::<u64>().unwrap_or(900);

    let (
        hero_title,
        hero_sub,
        idle_title,
        _idle_sub,
        clock_title,
        clock_sub,
        media_title,
        media_sub,
        formats_title,
        formats_sub,
        configure_label,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.tab_lockscreen"),
            lang.get("settings.lockscreen_header_subtitle"),
            lang.get("settings.idle_timeout_title"),
            lang.get("settings.idle_timeout_subtitle"),
            lang.get("settings.show_clock_title"),
            lang.get("settings.show_clock_subtitle"),
            lang.get("settings.show_media_title"),
            lang.get("settings.show_media_subtitle"),
            lang.get("settings.lockscreen_formats_title"),
            lang.get("settings.lockscreen_formats_subtitle"),
            lang.get("settings.configure_label"),
        )
    } else {
        (
            "Pantalla de Bloqueo".to_string(),
            "Configura la suspensión automática, formato del reloj y controles en la pantalla de autenticación.".to_string(),
            "Tiempo de Inactividad".to_string(),
            "Bloquear tras tiempo de inactividad (0 = desactivado).".to_string(),
            "Mostrar Reloj".to_string(),
            "Muestra la hora y fecha en el centro de la pantalla.".to_string(),
            "Reproductor Multimedia".to_string(),
            "Muestra controles MPRIS y letras de canciones en el lockscreen.".to_string(),
            "Formatos de Hora y Fecha".to_string(),
            "Personaliza la sintaxis de hora y fecha para la pantalla de bloqueo.".to_string(),
            "Configurar".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "lock.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_int_slider_row(
                    SettingsField::IdleTimeout,
                    &idle_title,
                    idle_secs,
                    theme,
                    cx,
                ))
                .child(render_row_divider(theme))
                .child(render_toggle_row(
                    ElementId::Name("lock-show-clock-toggle".into()),
                    &clock_title,
                    Some(&clock_sub),
                    module.show_clock,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_show_clock(cx);
                    }),
                ))
                .child(render_row_divider(theme))
                .child(render_toggle_row(
                    ElementId::Name("lock-show-media-toggle".into()),
                    &media_title,
                    Some(&media_sub),
                    module.show_media_player,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_show_media_player(cx);
                    }),
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &formats_title,
                    Some(&formats_sub),
                    render_subsection_trigger_button(
                        "open-lockscreen-formats-subsection",
                        &configure_label,
                        0,
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.open_subsection(SettingsSubSection::LockScreenFormats, cx);
                        }),
                    ),
                    theme,
                )),
        )
}

pub fn render_lockscreen_formats_subsection(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (hero_title, hero_sub, time_title, time_sub, date_title, date_sub) =
        if cx.has_global::<AppState>() {
            let lang = &cx.global::<AppState>().language;
            (
                lang.get("settings.lockscreen_formats_title"),
                lang.get("settings.lockscreen_formats_subtitle"),
                lang.get("settings.time_format_title"),
                lang.get("settings.time_format_subtitle"),
                lang.get("settings.date_format_title"),
                lang.get("settings.date_format_subtitle"),
            )
        } else {
            (
                "Formatos de Hora y Fecha".to_string(),
                "Personaliza la sintaxis de hora y fecha para la pantalla de bloqueo.".to_string(),
                "Formato de Hora".to_string(),
                "Sintaxis chrono (ej: %H:%M para 24h, %I:%M %p para 12h).".to_string(),
                "Formato de Fecha".to_string(),
                "Sintaxis chrono para la fecha bajo el reloj principal.".to_string(),
            )
        };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "lock.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_control_row(
                    &time_title,
                    Some(&time_sub),
                    render_text_input(
                        SettingsField::TimeFormat,
                        &module.time_format_input,
                        module.active_field == Some(SettingsField::TimeFormat),
                        &["%H:%M", "%I:%M %p"],
                        theme,
                        cx,
                    ),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &date_title,
                    Some(&date_sub),
                    render_text_input(
                        SettingsField::DateFormat,
                        &module.date_format_input,
                        module.active_field == Some(SettingsField::DateFormat),
                        &["%A, %B %e, %Y", "%d/%m/%Y", "%Y-%m-%d"],
                        theme,
                        cx,
                    ),
                    theme,
                )),
        )
}

pub(crate) fn render_text_input(
    field: SettingsField,
    current_value: &str,
    is_active: bool,
    presets: &[&'static str],
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let border_color = if is_active {
        theme.accent()
    } else {
        theme.surface().opacity(0.6)
    };

    let input_box = div()
        .id(ElementId::NamedInteger("ls-input".into(), field as u64))
        .w(px(170.0))
        .px_3()
        .py_1p5()
        .rounded(px(10.0))
        .bg(theme.background())
        .border_1()
        .border_color(border_color)
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_active_field(Some(field), cx);
        }))
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme.foreground())
                .child(if is_active {
                    format!("{current_value}|")
                } else {
                    current_value.to_string()
                }),
        );

    let mut preset_row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .justify_end()
        .gap(px(4.0));
    for preset in presets {
        let is_selected = current_value == *preset;
        let p_str = (*preset).to_string();
        preset_row = preset_row.child(
            div()
                .id(ElementId::Name(
                    format!("ls-preset-{field:?}-{preset}").into(),
                ))
                .px_2()
                .py_1()
                .rounded(px(8.0))
                .text_size(px(10.5))
                .bg(if is_selected {
                    theme.accent().opacity(0.2)
                } else {
                    theme.surface().opacity(0.35)
                })
                .text_color(if is_selected {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                })
                .hover(|s| s.bg(theme.surface().opacity(0.6)))
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.apply_preset(field, p_str.clone(), cx);
                }))
                .child(*preset),
        );
    }

    div()
        .flex()
        .flex_col()
        .items_end()
        .min_w_0()
        .gap(px(6.0))
        .child(input_box)
        .child(preset_row)
}
