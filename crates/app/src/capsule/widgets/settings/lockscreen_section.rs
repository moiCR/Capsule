use gpui::{Context, ElementId, IntoElement, ParentElement, Styled, div, prelude::*, px};
use services::AppState;
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_lockscreen_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let idle_mins = module.idle_timeout_input.parse::<u64>().unwrap_or(900) / 60;
    let cards_round = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        16.0
    };

    let (
        header_title,
        header_sub,
        idle_title,
        idle_sub,
        time_title,
        time_sub,
        date_title,
        date_sub,
        clock_title,
        clock_sub,
        media_title,
        media_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.lockscreen_header_title"),
            lang.get("settings.lockscreen_header_subtitle"),
            lang.get("settings.idle_timeout_title"),
            lang.get_with(
                "settings.idle_timeout_subtitle",
                &[("minutes", &idle_mins.to_string())],
            ),
            lang.get("settings.time_format_title"),
            lang.get("settings.time_format_subtitle"),
            lang.get("settings.date_format_title"),
            lang.get("settings.date_format_subtitle"),
            lang.get("settings.show_clock_title"),
            lang.get("settings.show_clock_subtitle"),
            lang.get("settings.show_media_title"),
            lang.get("settings.show_media_subtitle"),
        )
    } else {
        (
            "Pantalla de Bloqueo".to_string(),
            "Configura la suspensión automática, formato del reloj y controles en la pantalla de autenticación.".to_string(),
            "Tiempo de Inactividad".to_string(),
            format!("Bloquear tras {idle_mins} min sin actividad (0 = desactivado)."),
            "Formato de Hora".to_string(),
            "Sintaxis chrono (ej: %H:%M para 24h, %I:%M %p para 12h).".to_string(),
            "Formato de Fecha".to_string(),
            "Sintaxis chrono para la fecha bajo el reloj principal.".to_string(),
            "Mostrar Reloj".to_string(),
            "Muestra la hora y fecha en el centro de la pantalla.".to_string(),
            "Reproductor Multimedia".to_string(),
            "Muestra controles MPRIS y letras de canciones en el lockscreen.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(&header_title, &header_sub, theme))
        .child(render_setting_row(
            &idle_title,
            &idle_sub,
            render_timeout_stepper(module, theme, cx),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &time_title,
            &time_sub,
            render_text_input(
                SettingsField::TimeFormat,
                &module.time_format_input,
                module.active_field == Some(SettingsField::TimeFormat),
                &["%H:%M", "%I:%M %p"],
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &date_title,
            &date_sub,
            render_text_input(
                SettingsField::DateFormat,
                &module.date_format_input,
                module.active_field == Some(SettingsField::DateFormat),
                &["%A, %B %e, %Y", "%d/%m/%Y", "%Y-%m-%d"],
                theme,
                cx,
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &clock_title,
            &clock_sub,
            render_toggle(
                "toggle-show-clock",
                module.show_clock,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_show_clock(cx);
                }),
            ),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &media_title,
            &media_sub,
            render_toggle(
                "toggle-show-media",
                module.show_media_player,
                theme,
                cx.listener(|this, _, _, cx| {
                    this.toggle_show_media_player(cx);
                }),
            ),
            cards_round,
            theme,
        ))
}

fn render_timeout_stepper(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let current_secs = module.idle_timeout_input.parse::<u64>().unwrap_or(900);

    let val_up = current_secs + 300;
    let val_down = current_secs.saturating_sub(300);

    let secs_label = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get_with(
            "settings.seconds_suffix",
            &[("seconds", &current_secs.to_string())],
        )
    } else {
        format!("{current_secs} seg")
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(
            div()
                .id("lock-timeout-down")
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
                    this.set_field_int(SettingsField::IdleTimeout, val_down, cx);
                }))
                .child("-"),
        )
        .child(
            div()
                .w(px(72.0))
                .h(px(24.0))
                .rounded(px(6.0))
                .bg(theme.surface().opacity(0.25))
                .border_1()
                .border_color(theme.surface().opacity(0.2))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground())
                        .child(secs_label),
                ),
        )
        .child(
            div()
                .id("lock-timeout-up")
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
                    this.set_field_int(SettingsField::IdleTimeout, val_up, cx);
                }))
                .child("+"),
        )
}

fn render_text_input(
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
        .w(px(180.0))
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

    let mut preset_row = div().flex().flex_row().gap(px(4.0));
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
        .gap(px(6.0))
        .child(input_box)
        .child(preset_row)
}

fn render_toggle(
    id: &'static str,
    is_on: bool,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let bg = if is_on {
        theme.accent()
    } else {
        theme.surface().opacity(0.6)
    };

    div()
        .id(id)
        .w(px(34.0))
        .h(px(18.0))
        .rounded_full()
        .bg(bg)
        .p(px(2.0))
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .w(px(14.0))
                .h(px(14.0))
                .rounded_full()
                .bg(gpui::white())
                .ml(if is_on { px(16.0) } else { px(0.0) }),
        )
}
