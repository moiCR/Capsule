use gpui::{
    Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px, svg,
};
use services::{AppState, PowerProfile};
use ui::language::language_manager::LanguageManager;
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::SettingsModule;

pub fn render_system_section(
    _module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let current_profile = if cx.has_global::<AppState>() {
        cx.global::<AppState>().power.get_active_profile()
    } else {
        PowerProfile::Balanced
    };

    let languages = if cx.has_global::<LanguageManager>() {
        cx.global::<LanguageManager>().list_languages()
    } else {
        Vec::new()
    };

    let cards_round = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        16.0
    };

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(
            "Sistema & Energía",
            "Administra los perfiles de rendimiento de energía y el idioma principal de Capsule.",
            theme,
        ))
        .child(render_setting_row(
            "Plan de Energía",
            "Ajusta el consumo y velocidad del procesador vía power-profiles-daemon.",
            render_power_selector(current_profile, theme, cx),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            "Idioma de la Interfaz",
            "Cambia el idioma activo de las etiquetas, controles y textos del sistema.",
            render_language_selector(languages, theme, cx),
            cards_round,
            theme,
        ))
}

fn render_power_selector(
    current: PowerProfile,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let profiles = [
        (PowerProfile::Performance, "Rendimiento", "zap.svg"),
        (PowerProfile::Balanced, "Equilibrado", "scale.svg"),
        (PowerProfile::PowerSaver, "Ahorro", "leaf.svg"),
    ];

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .p_1()
        .gap(px(2.0))
        .rounded(px(12.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6));

    for (prof, label, icon) in profiles {
        let is_active = current == prof;
        let prof_val = prof.clone();

        container = container.child(
            div()
                .id(ElementId::Name(format!("sys-power-{label}").into()))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(5.0))
                .px_2p5()
                .py_1()
                .rounded(px(8.0))
                .bg(if is_active {
                    theme.accent().opacity(0.2)
                } else {
                    gpui::transparent_black()
                })
                .border_1()
                .border_color(if is_active {
                    theme.accent().opacity(0.5)
                } else {
                    gpui::transparent_black()
                })
                .hover(|s| {
                    if !is_active {
                        s.bg(theme.surface().opacity(0.4))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>()
                            .power
                            .set_active_profile(prof_val.clone());
                    }
                    cx.notify();
                }))
                .child(svg().path(icon).size(px(12.0)).text_color(if is_active {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(if is_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_active {
                            theme.foreground()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(label),
                ),
        );
    }

    container
}

fn render_language_selector(
    languages: Vec<ui::language::language_manager::LanguageItem>,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .p_1()
        .gap(px(2.0))
        .rounded(px(12.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6));

    for item in languages {
        let is_active = item.is_current;
        let lang_code = item.code.clone();
        let item_lang = item.language.clone();

        container = container.child(
            div()
                .id(ElementId::Name(format!("sys-lang-{lang_code}").into()))
                .flex()
                .flex_row()
                .items_center()
                .px_3()
                .py_1()
                .rounded(px(8.0))
                .bg(if is_active {
                    theme.accent().opacity(0.2)
                } else {
                    gpui::transparent_black()
                })
                .border_1()
                .border_color(if is_active {
                    theme.accent().opacity(0.5)
                } else {
                    gpui::transparent_black()
                })
                .hover(|s| {
                    if !is_active {
                        s.bg(theme.surface().opacity(0.4))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |_this, _, _, cx| {
                    if cx.has_global::<LanguageManager>() {
                        cx.global_mut::<LanguageManager>()
                            .set_language(item_lang.clone());
                        cx.set_global(cx.global::<LanguageManager>().current_language.clone());
                    }
                    cx.notify();
                }))
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(if is_active {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_active {
                            theme.foreground()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(item.name),
                ),
        );
    }

    container
}
