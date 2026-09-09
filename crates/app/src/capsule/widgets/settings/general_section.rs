use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, canvas, div,
    prelude::*, px, svg,
};
use services::{AppState, Application, LanguageInfo, PowerProfile};
use std::cell::Cell;
use std::rc::Rc;
use ui::components::select::{Select, SelectItem};
use ui::theme::Theme;

use super::setting_item::{
    render_card_container, render_control_row, render_hero_header, render_row_divider,
    render_subsection_trigger_button, render_toggle_row,
};
use crate::capsule::modules::settings::{SettingsField, SettingsModule, SettingsSubSection};

pub fn render_apps_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let empty_apps = Vec::new();
    let installed_apps = if cx.has_global::<AppState>() {
        cx.global::<AppState>().launcher.get_apps()
    } else {
        std::sync::Arc::new(empty_apps)
    };

    let (
        hero_title,
        hero_sub,
        term_title,
        term_sub,
        browser_title,
        browser_sub,
        editor_title,
        editor_sub,
        file_manager_title,
        file_manager_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.apps_header_title"),
            lang.get("settings.apps_header_subtitle"),
            lang.get("settings.terminal_title"),
            lang.get("settings.terminal_subtitle"),
            lang.get("settings.browser_title"),
            lang.get("settings.browser_subtitle"),
            lang.get("settings.editor_title"),
            lang.get("settings.editor_subtitle"),
            lang.get("settings.file_manager_title"),
            lang.get("settings.file_manager_subtitle"),
        )
    } else {
        (
            "Aplicaciones Predeterminadas".to_string(),
            "Herramientas y binarios ejecutados por Capsule para tus tareas principales."
                .to_string(),
            "Terminal".to_string(),
            "Lanzado con 'capsule terminal' o apps en terminal.".to_string(),
            "Navegador Web".to_string(),
            "Lanzado con 'capsule browser' o links del sistema.".to_string(),
            "Editor de Código / Texto".to_string(),
            "Lanzado con 'capsule editor'. Si es CLI, se ejecuta en terminal.".to_string(),
            "Gestor de Archivos".to_string(),
            "Lanzado con 'capsule files' o explorador del sistema.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "sparkles.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_control_row(
                    &term_title,
                    Some(&term_sub),
                    render_app_select(
                        module,
                        SettingsField::Terminal,
                        &module.terminal_input,
                        module.open_dropdown == Some(SettingsField::Terminal),
                        &installed_apps,
                        "sparkles.svg",
                        theme,
                        cx,
                    ),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &browser_title,
                    Some(&browser_sub),
                    render_app_select(
                        module,
                        SettingsField::Browser,
                        &module.browser_input,
                        module.open_dropdown == Some(SettingsField::Browser),
                        &installed_apps,
                        "sparkles.svg",
                        theme,
                        cx,
                    ),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &editor_title,
                    Some(&editor_sub),
                    render_app_select(
                        module,
                        SettingsField::Editor,
                        &module.editor_input,
                        module.open_dropdown == Some(SettingsField::Editor),
                        &installed_apps,
                        "sparkles.svg",
                        theme,
                        cx,
                    ),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &file_manager_title,
                    Some(&file_manager_sub),
                    render_app_select(
                        module,
                        SettingsField::FileManager,
                        &module.file_manager_input,
                        module.open_dropdown == Some(SettingsField::FileManager),
                        &installed_apps,
                        "folder.svg",
                        theme,
                        cx,
                    ),
                    theme,
                )),
        )
}

pub fn render_media_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (
        hero_title,
        hero_sub,
        idle_lyrics_title,
        idle_lyrics_sub,
        media_title,
        media_sub,
        music_title,
        music_sub,
        manage_label,
        output_vol_title,
        output_vol_sub,
        input_vol_title,
        input_vol_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.media_header_title"),
            lang.get("settings.media_header_subtitle"),
            lang.get("settings.idle_lyrics_title"),
            lang.get("settings.idle_lyrics_subtitle"),
            lang.get("settings.show_media_title"),
            lang.get("settings.show_media_subtitle"),
            lang.get("settings.music_players_title"),
            lang.get("settings.music_players_subtitle"),
            lang.get("settings.manage_label"),
            lang.get("settings.output_volume_title"),
            lang.get("settings.output_volume_subtitle"),
            lang.get("settings.input_volume_title"),
            lang.get("settings.input_volume_subtitle"),
        )
    } else {
        (
            "Multimedia".to_string(),
            "Control de reproductores multimedia y dispositivos de audio.".to_string(),
            "Letras en Reposo".to_string(),
            "Muestra la letra sincronizada de la canción en la cápsula idle.".to_string(),
            "Reproductor en Bloqueo".to_string(),
            "Muestra controles MPRIS y letras de canciones en el lockscreen.".to_string(),
            "Reproductores Autorizados".to_string(),
            "Servicios MPRIS permitidos para control multimedia (ej. Spotify, Fastpotify)."
                .to_string(),
            "Gestionar".to_string(),
            "Salida de Audio".to_string(),
            "Control de volumen y selección del dispositivo de reproducción.".to_string(),
            "Entrada de Audio (Micrófono)".to_string(),
            "Control de volumen y selección del dispositivo de captura.".to_string(),
        )
    };

    let (out_vol, out_muted, sinks, in_vol, in_muted, sources) = if cx.has_global::<AppState>() {
        let status = cx.global::<AppState>().system.get_status();
        (
            status.volume,
            status.is_muted,
            status
                .audio_sinks
                .into_iter()
                .map(|s| (s.name, s.description, s.is_default))
                .collect::<Vec<_>>(),
            status.input_volume,
            status.is_input_muted,
            status
                .audio_sources
                .into_iter()
                .map(|s| (s.name, s.description, s.is_default))
                .collect::<Vec<_>>(),
        )
    } else {
        (50, false, Vec::new(), 50, false, Vec::new())
    };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "music.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_toggle_row(
                    ElementId::Name("media-idle-lyrics".into()),
                    &idle_lyrics_title,
                    Some(&idle_lyrics_sub),
                    module.show_lyrics,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_show_lyrics(cx);
                    }),
                ))
                .child(render_row_divider(theme))
                .child(render_toggle_row(
                    ElementId::Name("media-show-lockscreen".into()),
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
                    &music_title,
                    Some(&music_sub),
                    render_subsection_trigger_button(
                        "open-music-players-subsection",
                        &manage_label,
                        module.music_players.len(),
                        theme,
                        cx.listener(|this, _, _, cx| {
                            this.open_subsection(SettingsSubSection::MusicPlayers, cx);
                        }),
                    ),
                    theme,
                )),
        )
        .child(render_audio_device_control(
            SettingsField::VolumeOutput,
            &output_vol_title,
            &output_vol_sub,
            out_vol,
            out_muted,
            "volume-2.svg",
            "volume-x.svg",
            module.show_output_devices,
            sinks,
            module.output_slider_bounds.clone(),
            theme,
            cx,
        ))
        .child(render_audio_device_control(
            SettingsField::VolumeInput,
            &input_vol_title,
            &input_vol_sub,
            in_vol,
            in_muted,
            "mic.svg",
            "mic-off.svg",
            module.show_input_devices,
            sources,
            module.input_slider_bounds.clone(),
            theme,
            cx,
        ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_audio_device_control(
    field: SettingsField,
    title: &str,
    subtitle: &str,
    volume: u32,
    is_muted: bool,
    icon_on: &'static str,
    icon_off: &'static str,
    is_expanded: bool,
    devices: Vec<(String, String, bool)>,
    bounds_cell: Rc<Cell<(f32, f32)>>,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let vol_pct = volume.min(100);
    let is_output = field == SettingsField::VolumeOutput;
    let icon_path = if is_muted || volume == 0 {
        icon_off
    } else {
        icon_on
    };

    let slider_cell = bounds_cell.clone();

    let chevron_icon = if is_expanded {
        "chevron-down.svg"
    } else {
        "chevron-right.svg"
    };

    let mut device_items = div().flex().flex_col().w_full().gap(px(4.0));
    if devices.is_empty() {
        device_items = device_items.child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .px_1()
                .child("No devices found".to_string()),
        );
    } else {
        for (idx, (dev_name, dev_desc, is_def)) in devices.into_iter().enumerate() {
            let target_name = dev_name.clone();
            let target_field = field;
            let indicator_color = if is_def {
                theme.accent()
            } else {
                gpui::hsla(0.0, 0.0, 0.0, 0.0)
            };

            device_items = device_items.child(
                div()
                    .id(ElementId::NamedInteger(
                        if is_output {
                            "sink-item"
                        } else {
                            "source-item"
                        }
                        .into(),
                        idx as u64,
                    ))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2p5()
                    .py_1p5()
                    .rounded(px(10.0))
                    .bg(if is_def {
                        theme.surface().opacity(0.55)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| s.bg(theme.surface().opacity(0.4)))
                    .active(|s| s.bg(theme.surface().opacity(0.6)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let sys = cx.global::<AppState>().system.clone();
                            let t_name = target_name.clone();
                            let this = cx.entity().downgrade();
                            cx.spawn(async move |_this, cx| {
                                if target_field == SettingsField::VolumeOutput {
                                    let _ = sys.set_default_sink(&t_name).await;
                                } else {
                                    let _ = sys.set_default_source(&t_name).await;
                                }
                                let _ = this.update(cx, |_view, cx| cx.notify());
                            })
                            .detach();
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .min_w_0()
                            .flex_1()
                            .overflow_hidden()
                            .child(
                                div()
                                    .w(px(3.0))
                                    .h(px(14.0))
                                    .rounded_full()
                                    .bg(indicator_color),
                            )
                            .child(svg().path(icon_on).size(px(14.0)).text_color(if is_def {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            }))
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .font_weight(if is_def {
                                        FontWeight::SEMIBOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(if is_def {
                                        theme.accent()
                                    } else {
                                        theme.foreground()
                                    })
                                    .truncate()
                                    .child(dev_desc),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(10.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent().opacity(0.85))
                            .child(if is_def { "Activo" } else { "" }),
                    ),
            );
        }
    }

    render_card_container(theme)
        .p_4()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .pr_3()
                        .child(
                            div()
                                .font_weight(FontWeight::MEDIUM)
                                .text_size(px(13.0))
                                .text_color(theme.foreground())
                                .child(title.to_string()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.foreground_muted())
                                .child(subtitle.to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(12.5))
                                .text_color(if is_muted {
                                    theme.foreground_muted()
                                } else {
                                    theme.accent()
                                })
                                .child(format!("{vol_pct}%")),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger(
                                    if is_output {
                                        "mute-sink-btn"
                                    } else {
                                        "mute-source-btn"
                                    }
                                    .into(),
                                    field as u64,
                                ))
                                .w(px(26.0))
                                .h(px(26.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.6))
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .active(|s| s.bg(theme.surface()))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .on_click(cx.listener(move |_this, _, _, cx| {
                                    if cx.has_global::<AppState>() {
                                        let sys = cx.global::<AppState>().system.clone();
                                        let this = cx.entity().downgrade();
                                        cx.spawn(async move |_this, cx| {
                                            if is_output {
                                                let _ = sys.toggle_mute().await;
                                            } else {
                                                let _ = sys.toggle_input_mute().await;
                                            }
                                            let _ = this.update(cx, |_view, cx| cx.notify());
                                        })
                                        .detach();
                                    }
                                }))
                                .child(svg().path(icon_path).size(px(14.0)).text_color(
                                    if is_muted {
                                        theme.foreground_muted()
                                    } else {
                                        theme.accent()
                                    },
                                )),
                        )
                        .child(
                            div()
                                .id(ElementId::NamedInteger(
                                    if is_output {
                                        "toggle-sink-devices"
                                    } else {
                                        "toggle-source-devices"
                                    }
                                    .into(),
                                    field as u64,
                                ))
                                .w(px(26.0))
                                .h(px(26.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.6))
                                .hover(|s| s.bg(theme.surface().opacity(0.9)))
                                .active(|s| s.bg(theme.surface()))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if is_output {
                                        this.toggle_output_devices(cx);
                                    } else {
                                        this.toggle_input_devices(cx);
                                    }
                                }))
                                .child(
                                    svg()
                                        .path(chevron_icon)
                                        .size(px(14.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .id(ElementId::NamedInteger(
                    if is_output {
                        "volume-slider-output"
                    } else {
                        "volume-slider-input"
                    }
                    .into(),
                    field as u64,
                ))
                .relative()
                .flex()
                .items_center()
                .w_full()
                .h(px(34.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.55))
                .cursor_pointer()
                .overflow_hidden()
                .child(
                    canvas(
                        move |bounds, _, _| {
                            let left: f32 = bounds.origin.x.into();
                            let width: f32 = bounds.size.width.into();
                            slider_cell.set((left, width));
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                        let (left, width) = bounds_cell.get();
                        if width > 0.0 {
                            let x: f32 = event.position.x.into();
                            let rel = (x - left).clamp(0.0, width);
                            let pct = ((rel / width) * 100.0).round().clamp(0.0, 100.0) as u32;
                            if is_output {
                                if cx.has_global::<AppState>() {
                                    cx.global::<AppState>().system.set_volume_fast(pct);
                                }
                            } else if cx.has_global::<AppState>() {
                                cx.global::<AppState>().system.set_input_volume_fast(pct);
                            }
                            this.active_slider_drag =
                                Some((field, 0.0, 100.0, 1.0, bounds_cell.clone()));
                            cx.notify();
                        }
                    }),
                )
                .child(
                    div()
                        .h_full()
                        .w(gpui::DefiniteLength::Fraction(
                            (vol_pct as f32 / 100.0).clamp(0.0, 1.0),
                        ))
                        .rounded_full()
                        .bg(if is_muted {
                            theme.foreground_muted().opacity(0.4)
                        } else {
                            theme.accent()
                        }),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(10.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(18.0))
                        .h(px(18.0))
                        .child(
                            svg()
                                .path(icon_path)
                                .size(px(16.0))
                                .text_color(if is_muted {
                                    theme.foreground_muted()
                                } else {
                                    theme.background()
                                }),
                        ),
                ),
        )
        .when(is_expanded, |card| {
            card.child(render_row_divider(theme)).child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap(px(6.0))
                    .pt_1()
                    .child(device_items),
            )
        })
}

pub fn render_music_players_subsection(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (music_title, music_sub, music_placeholder, music_add, music_empty) =
        if cx.has_global::<AppState>() {
            let lang = &cx.global::<AppState>().language;
            (
                lang.get("settings.music_players_title"),
                lang.get("settings.music_players_subtitle"),
                lang.get("settings.music_players_placeholder"),
                lang.get("settings.music_players_add"),
                lang.get("settings.music_players_empty"),
            )
        } else {
            (
                "Reproductores Autorizados".to_string(),
                "Servicios MPRIS permitidos para control multimedia (ej. Spotify, Fastpotify)."
                    .to_string(),
                "Nombre de la app (ej. Fastpotify)...".to_string(),
                "Añadir".to_string(),
                "No hay reproductores configurados.".to_string(),
            )
        };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "music.svg",
            &music_title,
            &music_sub,
            theme,
        ))
        .child(
            render_card_container(theme).child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .min_w_0()
                    .p_4()
                    .child(render_music_players_card(
                        module,
                        &music_placeholder,
                        &music_add,
                        &music_empty,
                        theme,
                        cx,
                    )),
            ),
        )
}

pub fn render_system_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let current_profile = if cx.has_global::<AppState>() {
        cx.global::<AppState>().power.get_active_profile()
    } else {
        PowerProfile::Balanced
    };

    let languages = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.list_languages()
    } else {
        Vec::new()
    };

    let (hero_title, hero_sub, power_title, power_sub, lang_title, lang_sub) = if cx
        .has_global::<AppState>()
    {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.system_header_title"),
            lang.get("settings.system_header_subtitle"),
            lang.get("settings.power_plan_title"),
            lang.get("settings.power_plan_subtitle"),
            lang.get("settings.language_title"),
            lang.get("settings.language_subtitle"),
        )
    } else {
        (
            "Sistema".to_string(),
            "Plan de energía del procesador e idioma de la interfaz.".to_string(),
            "Plan de Energía".to_string(),
            "Ajusta el consumo y velocidad del procesador vía power-profiles-daemon.".to_string(),
            "Idioma de la Interfaz".to_string(),
            "Cambia el idioma activo de las etiquetas y controles del sistema.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "settings.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(
            render_card_container(theme)
                .child(render_control_row(
                    &power_title,
                    Some(&power_sub),
                    render_power_selector(current_profile, theme, cx),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &lang_title,
                    Some(&lang_sub),
                    render_language_select(module, &languages, theme, cx),
                    theme,
                )),
        )
}

pub(crate) fn render_music_players_card(
    module: &SettingsModule,
    placeholder: &str,
    add_label: &str,
    empty_label: &str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let mut chips_row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .justify_end()
        .gap(px(6.0));

    if module.music_players.is_empty() {
        chips_row = chips_row.child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(empty_label.to_string()),
        );
    } else {
        for (idx, player) in module.music_players.iter().enumerate() {
            let is_spotify = player.to_lowercase().contains("spotify");
            let icon_svg = if is_spotify {
                "spotify.svg"
            } else {
                "music.svg"
            };
            let player_name = player.clone();

            chips_row = chips_row.child(
                div()
                    .id(ElementId::NamedInteger(
                        "music-player-chip".into(),
                        idx as u64,
                    ))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px_2p5()
                    .py_1()
                    .rounded_full()
                    .bg(theme.surface().opacity(0.7))
                    .border_1()
                    .border_color(theme.surface().opacity(0.35))
                    .child(
                        svg()
                            .path(icon_svg)
                            .size(px(12.0))
                            .text_color(theme.accent()),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(player_name),
                    )
                    .child(
                        div()
                            .id(ElementId::NamedInteger(
                                "remove-music-player".into(),
                                idx as u64,
                            ))
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(14.0))
                            .h(px(14.0))
                            .rounded_full()
                            .hover(|s| s.bg(theme.surface().opacity(0.9)))
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_music_player(idx, cx);
                            }))
                            .child(
                                svg()
                                    .path("close.svg")
                                    .size(px(8.0))
                                    .text_color(theme.foreground_muted()),
                            ),
                    ),
            );
        }
    }

    let is_input_active = module.active_field == Some(SettingsField::MusicPlayer);
    let border_color = if is_input_active {
        theme.accent()
    } else {
        theme.surface().opacity(0.5)
    };

    let current_val = &module.music_player_input;
    let input_field = div()
        .id("music-player-input-box")
        .flex()
        .flex_row()
        .items_center()
        .w(px(190.0))
        .h(px(26.0))
        .px_2p5()
        .rounded(px(8.0))
        .bg(theme.background())
        .border_1()
        .border_color(border_color)
        .overflow_hidden()
        .cursor_pointer()
        .on_click(cx.listener(|this, _, _, cx| {
            this.set_active_field(Some(SettingsField::MusicPlayer), cx);
        }))
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if current_val.is_empty() && !is_input_active {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .truncate()
                .overflow_hidden()
                .max_w_full()
                .child(if is_input_active {
                    format!("{current_val}|")
                } else if current_val.is_empty() {
                    placeholder.to_string()
                } else {
                    current_val.clone()
                }),
        );

    let add_btn = div()
        .id("music-player-add-btn")
        .flex()
        .items_center()
        .justify_center()
        .px_2p5()
        .h(px(26.0))
        .rounded(px(8.0))
        .bg(theme.accent().opacity(0.2))
        .hover(|s| s.bg(theme.accent().opacity(0.35)))
        .active(|s| s.bg(theme.accent().opacity(0.5)))
        .cursor_pointer()
        .on_click(cx.listener(|this, _, _, cx| {
            let text = this.music_player_input.clone();
            this.add_music_player(text, cx);
        }))
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.accent())
                .child(add_label.to_string()),
        );

    let common_suggestions = ["Spotify", "Fastpotify"];
    let mut suggestions_row = div().flex().flex_row().items_center().gap(px(6.0));
    for sugg in common_suggestions {
        if !module
            .music_players
            .iter()
            .any(|p| p.eq_ignore_ascii_case(sugg))
        {
            let sugg_str = sugg.to_string();
            suggestions_row = suggestions_row.child(
                div()
                    .id(ElementId::Name(format!("sugg-player-{sugg}").into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .px_2()
                    .py_0p5()
                    .rounded_full()
                    .bg(theme.surface().opacity(0.3))
                    .border_1()
                    .border_color(theme.surface().opacity(0.2))
                    .hover(|s| {
                        s.bg(theme.accent().opacity(0.15))
                            .border_color(theme.accent().opacity(0.4))
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.add_music_player(sugg_str.clone(), cx);
                    }))
                    .child(
                        div()
                            .text_size(px(10.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent())
                            .child(format!("+ {sugg}")),
                    ),
            );
        }
    }

    div()
        .flex()
        .flex_col()
        .items_end()
        .min_w_0()
        .gap(px(8.0))
        .child(chips_row)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .child(input_field)
                .child(add_btn),
        )
        .child(suggestions_row)
}

pub(crate) fn render_power_selector(
    current: PowerProfile,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (perf_label, bal_label, saver_label) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("power.performance"),
            lang.get("power.balanced"),
            lang.get("power.power_saver"),
        )
    } else {
        (
            "Rendimiento".to_string(),
            "Equilibrado".to_string(),
            "Ahorro".to_string(),
        )
    };

    let profiles = [
        (PowerProfile::Performance, perf_label, "zap.svg"),
        (PowerProfile::Balanced, bal_label, "scale.svg"),
        (PowerProfile::PowerSaver, saver_label, "leaf.svg"),
    ];

    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .p_1()
        .gap(px(2.0))
        .rounded_full()
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.25));

    for (prof, label, icon) in profiles {
        let is_active = current == prof;
        let prof_val = prof.clone();

        container = container.child(
            div()
                .id(ElementId::Name(format!("sys-power-{label}").into()))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(4.0))
                .px_2p5()
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
                .on_click(cx.listener(move |_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>()
                            .power
                            .set_active_profile(prof_val.clone());
                    }
                    cx.notify();
                }))
                .child(svg().path(icon).size(px(11.0)).text_color(if is_active {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }))
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
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

fn compute_dropdown_placement(
    scroll_bounds: gpui::Bounds<gpui::Pixels>,
    trigger_bounds: (f32, f32),
    fallback_local: (f32, f32),
    item_count: usize,
) -> (bool, f32) {
    let scroll_top: f32 = scroll_bounds.top().into();
    let scroll_bottom: f32 = scroll_bounds.bottom().into();

    let (viewport_top, viewport_bottom) = if scroll_bottom > scroll_top {
        (scroll_top, scroll_bottom)
    } else {
        (0.0, 560.0)
    };

    let (trigger_top, trigger_bottom) = if trigger_bounds.1 > trigger_bounds.0 {
        (trigger_bounds.0, trigger_bounds.1)
    } else {
        (
            viewport_top + fallback_local.0,
            viewport_top + fallback_local.1,
        )
    };

    let margin = 12.0;
    let space_above = (trigger_top - viewport_top - margin).max(0.0);
    let space_below = (viewport_bottom - trigger_bottom - margin).max(0.0);

    let desired_height = (item_count as f32 * 32.0 + 14.0).clamp(40.0, 220.0);

    let (open_upwards, available_space) = if space_below >= desired_height {
        (false, space_below)
    } else if space_above >= desired_height || space_above > space_below {
        (true, space_above)
    } else {
        (false, space_below)
    };

    let max_height = available_space.clamp(48.0, 220.0);
    (open_upwards, max_height)
}

pub(crate) fn render_language_select(
    module: &SettingsModule,
    languages: &[LanguageInfo],
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let current_lang = languages.iter().find(|l| l.is_current);
    let placeholder = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .language
            .get("settings.select_language")
    } else {
        "Seleccionar idioma...".to_string()
    };
    let display_name = if let Some(lang) = current_lang {
        lang.name.clone()
    } else {
        placeholder.clone()
    };

    let is_open = module.open_dropdown == Some(SettingsField::Language);
    let dropdown_p = if is_open && module.dropdown_anim_field == Some(SettingsField::Language) {
        module.dropdown_anim_progress
    } else if is_open {
        1.0
    } else {
        0.0
    };

    let fallback = (140.0, 176.0);
    let (open_upwards, max_height) = compute_dropdown_placement(
        module.scroll_handle.bounds(),
        module.language_trigger_bounds.get(),
        fallback,
        languages.len(),
    );

    let items = languages.iter().map(|lang| {
        let is_selected = lang.is_current;
        let target_file = lang
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&lang.code)
            .to_string();
        SelectItem::new(format!("lang-{}", lang.code), lang.name.clone())
            .selected(is_selected)
            .icon(
                svg()
                    .path("languages.svg")
                    .size(px(14.0))
                    .text_color(if is_selected {
                        theme.accent()
                    } else {
                        theme.foreground_muted()
                    }),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_language(target_file.clone(), cx);
            }))
    });

    Select::new("language-select-btn")
        .placeholder(placeholder)
        .selected_label(display_name)
        .icon(
            svg()
                .path("languages.svg")
                .size(px(14.0))
                .text_color(theme.accent()),
        )
        .is_open(is_open)
        .anim_progress(dropdown_p)
        .open_upwards(open_upwards)
        .max_menu_height(max_height)
        .track_bounds(module.language_trigger_bounds.clone())
        .width(220.0)
        .on_toggle(cx.listener(move |this, _, _, cx| {
            this.toggle_dropdown(SettingsField::Language, cx);
        }))
        .items(items)
}

fn app_to_command(app: &Application) -> String {
    let raw = app.exec.split_whitespace().next().unwrap_or(&app.exec);
    let bin_name = std::path::Path::new(raw)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(raw);

    let desktop_stem = app.id.strip_suffix(".desktop").unwrap_or(&app.id);
    let stem_clean = desktop_stem
        .split('.')
        .next_back()
        .unwrap_or(desktop_stem)
        .to_lowercase();

    if stem_clean == "zed" || bin_name == "zed" {
        return "zed".to_string();
    }
    if stem_clean == "zen" || stem_clean == "zen-browser" || bin_name.contains("zen") {
        return "zen-browser".to_string();
    }
    if stem_clean == "kitty" || bin_name == "kitty" {
        return "kitty".to_string();
    }
    if stem_clean == "alacritty" || bin_name == "alacritty" {
        return "alacritty".to_string();
    }
    if stem_clean == "ghostty" || bin_name == "ghostty" {
        return "ghostty".to_string();
    }
    if stem_clean == "firefox" || bin_name == "firefox" {
        return "firefox".to_string();
    }

    bin_name.to_string()
}

fn find_matching_app<'a>(cmd: &str, apps: &'a [Application]) -> Option<&'a Application> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(app) = apps.iter().find(|a| app_to_command(a) == trimmed) {
        return Some(app);
    }
    if let Some(app) = apps.iter().find(|a| a.name.eq_ignore_ascii_case(trimmed)) {
        return Some(app);
    }
    if let Some(app) = apps.iter().find(|a| {
        a.exec
            .split_whitespace()
            .next()
            .and_then(|w| std::path::Path::new(w).file_name()?.to_str())
            .is_some_and(|f| f.eq_ignore_ascii_case(trimmed))
    }) {
        return Some(app);
    }
    apps.iter().find(|a| {
        let name = a.name.to_lowercase();
        let id = a.id.to_lowercase();
        let target = trimmed.to_lowercase();
        name == target || id.starts_with(&target)
    })
}

fn filter_apps_for_field<'a>(
    field: SettingsField,
    apps: &'a [Application],
) -> Vec<&'a Application> {
    let mut filtered: Vec<&'a Application> = apps
        .iter()
        .filter(|app| {
            let id = app.id.to_lowercase();
            let name = app.name.to_lowercase();
            let exec = app.exec.to_lowercase();
            let generic_name = app.generic_name.as_deref().unwrap_or("").to_lowercase();
            let comment = app.comment.as_deref().unwrap_or("").to_lowercase();

            match field {
                SettingsField::Terminal => {
                    app.terminal
                        || generic_name.contains("terminal")
                        || name.contains("terminal")
                        || name.contains("kitty")
                        || name.contains("alacritty")
                        || name.contains("ghostty")
                        || name.contains("foot")
                        || name.contains("wezterm")
                        || name.contains("konsole")
                        || id.contains("terminal")
                        || id.contains("kitty")
                        || id.contains("alacritty")
                        || id.contains("ghostty")
                        || id.contains("foot")
                        || id.contains("wezterm")
                        || exec.contains("kitty")
                        || exec.contains("alacritty")
                        || exec.contains("ghostty")
                }
                SettingsField::Browser => {
                    generic_name.contains("browser")
                        || generic_name.contains("navegador")
                        || name.contains("browser")
                        || name.contains("navegador")
                        || name.contains("firefox")
                        || name.contains("zen")
                        || name.contains("chrome")
                        || name.contains("chromium")
                        || name.contains("brave")
                        || name.contains("opera")
                        || id.contains("browser")
                        || id.contains("firefox")
                        || id.contains("zen")
                        || id.contains("chrome")
                        || id.contains("brave")
                        || exec.contains("firefox")
                        || exec.contains("zen")
                        || exec.contains("brave")
                        || exec.contains("chrome")
                }
                SettingsField::Editor => {
                    generic_name.contains("editor")
                        || generic_name.contains("ide")
                        || comment.contains("editor")
                        || name.contains("editor")
                        || name.contains("zed")
                        || name.contains("code")
                        || name.contains("vim")
                        || name.contains("nvim")
                        || name.contains("neovim")
                        || name.contains("helix")
                        || name.contains("sublime")
                        || name.contains("micro")
                        || id.contains("editor")
                        || id.contains("zed")
                        || id.contains("code")
                        || id.contains("vim")
                        || id.contains("micro")
                        || exec.contains("zed")
                        || exec.contains("code")
                        || exec.contains("vim")
                        || exec.contains("nvim")
                        || exec.contains("micro")
                }
                SettingsField::FileManager => {
                    generic_name.contains("file")
                        || generic_name.contains("archivos")
                        || generic_name.contains("manager")
                        || generic_name.contains("gestor")
                        || name.contains("file")
                        || name.contains("archivo")
                        || name.contains("thunar")
                        || name.contains("nautilus")
                        || name.contains("dolphin")
                        || name.contains("nemo")
                        || name.contains("pcmanfm")
                        || name.contains("yazi")
                        || name.contains("ranger")
                        || id.contains("file")
                        || id.contains("thunar")
                        || id.contains("nautilus")
                        || id.contains("dolphin")
                        || id.contains("nemo")
                        || exec.contains("thunar")
                        || exec.contains("nautilus")
                        || exec.contains("dolphin")
                        || exec.contains("nemo")
                        || exec.contains("pcmanfm")
                }
                _ => false,
            }
        })
        .collect();

    filtered.sort_by(|a, b| a.name.cmp(&b.name));
    filtered.dedup_by(|a, b| a.name == b.name);
    filtered
}

fn render_app_icon(
    icon_path: Option<&std::path::Path>,
    fallback_svg: &'static str,
    theme: &Theme,
) -> AnyElement {
    if let Some(path) = icon_path {
        gpui::img(path.to_path_buf())
            .size(px(18.0))
            .rounded(px(4.0))
            .into_any_element()
    } else {
        svg()
            .path(fallback_svg)
            .size(px(16.0))
            .text_color(theme.accent())
            .into_any_element()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_app_select(
    module: &SettingsModule,
    field: SettingsField,
    current_value: &str,
    is_open: bool,
    apps: &[Application],
    fallback_icon: &'static str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let matched_app = find_matching_app(current_value, apps);
    let placeholder = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get("settings.select_app")
    } else {
        "Seleccionar aplicación...".to_string()
    };
    let display_name = if let Some(app) = matched_app {
        app.name.clone()
    } else if current_value.is_empty() {
        placeholder.clone()
    } else {
        current_value.to_string()
    };
    let display_icon = render_app_icon(
        matched_app.and_then(|a| a.icon_path.as_deref()),
        fallback_icon,
        theme,
    );

    let is_animating = is_open && module.dropdown_anim_field == Some(field);
    let dropdown_p = if is_animating {
        module.dropdown_anim_progress
    } else if is_open {
        1.0
    } else {
        0.0
    };

    let candidate_apps = filter_apps_for_field(field, apps);
    let select_id = ElementId::NamedInteger("app-select-btn".into(), field as u64);

    let (fallback, trigger_cell) = match field {
        SettingsField::Terminal => ((260.0, 296.0), module.terminal_trigger_bounds.clone()),
        SettingsField::Browser => ((336.0, 372.0), module.browser_trigger_bounds.clone()),
        SettingsField::Editor => ((412.0, 448.0), module.editor_trigger_bounds.clone()),
        SettingsField::FileManager => ((488.0, 524.0), module.file_manager_trigger_bounds.clone()),
        _ => ((260.0, 296.0), module.terminal_trigger_bounds.clone()),
    };

    let (open_upwards, max_height) = compute_dropdown_placement(
        module.scroll_handle.bounds(),
        trigger_cell.get(),
        fallback,
        candidate_apps.len(),
    );

    let items = candidate_apps.into_iter().map(|app| {
        let cmd = app_to_command(app);
        let is_selected = current_value == cmd || current_value == app.name;
        let app_icon = render_app_icon(app.icon_path.as_deref(), fallback_icon, theme);

        SelectItem::new(format!("app-item-{field:?}-{}", app.id), app.name.clone())
            .selected(is_selected)
            .icon(app_icon)
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select_app_command(field, cmd.clone(), cx);
            }))
    });

    Select::new(select_id)
        .placeholder(placeholder)
        .selected_label(display_name)
        .icon(display_icon)
        .is_open(is_open)
        .anim_progress(dropdown_p)
        .open_upwards(open_upwards)
        .max_menu_height(max_height)
        .track_bounds(trigger_cell)
        .width(220.0)
        .on_toggle(cx.listener(move |this, _, _, cx| {
            this.toggle_dropdown(field, cx);
        }))
        .items(items)
}
