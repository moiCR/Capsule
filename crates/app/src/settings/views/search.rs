use gpui::{
    AnyElement, App, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div,
    prelude::*, px, svg,
};
use services::{AppState, CapsuleStyle};
use ui::theme::Theme;

use super::common::{card, control_row, hero_header, row_divider, toggle_row};
use crate::island::widgets::style;
use crate::settings::sidebar::SettingsTab;
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

fn tab_name(tab: SettingsTab, cx: &App) -> String {
    match tab {
        SettingsTab::Appearance => text("settings.tab_appearance", cx),
        SettingsTab::Apps => text("settings.tab_apps", cx),
        SettingsTab::Media => text("settings.tab_media", cx),
        SettingsTab::LockScreen => text("settings.tab_lockscreen", cx),
        SettingsTab::System => text("settings.tab_system", cx),
        SettingsTab::Record => text("settings.tab_record", cx),
    }
}

#[derive(Clone, Copy)]
pub enum SearchControl {
    ToggleDynamicColors,
    ToggleDarkMode,
    ToggleConcaveStyle,
    ToggleClock24,
    ToggleLegacyCapsule,
    ToggleLockscreenClock,
    ToggleLockscreenMedia,
    ToggleRecordCursor,
    Navigate(SettingsTab),
}

struct SearchDefinition {
    id: &'static str,
    title_key: &'static str,
    desc_key: &'static str,
    tab: SettingsTab,
    keywords: &'static [&'static str],
    control: SearchControl,
}

const SEARCH_DEFINITIONS: &[SearchDefinition] = &[
    // --- Appearance ---
    SearchDefinition {
        id: "dynamic-colors",
        title_key: "settings.dynamic_colors",
        desc_key: "settings.dynamic_colors_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "colores",
            "dinamicos",
            "fondo",
            "wallpaper",
            "dynamic",
            "colors",
            "material",
            "tema",
        ],
        control: SearchControl::ToggleDynamicColors,
    },
    SearchDefinition {
        id: "dark-mode",
        title_key: "settings.dark_mode",
        desc_key: "settings.color_mode_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "oscuro", "claro", "dark", "light", "modo", "mode", "color", "tema", "theme",
        ],
        control: SearchControl::ToggleDarkMode,
    },
    SearchDefinition {
        id: "concave-style",
        title_key: "settings.concave_style",
        desc_key: "settings.style_description",
        tab: SettingsTab::Appearance,
        keywords: &[
            "concavo",
            "modo concavo",
            "concave",
            "curvo",
            "estilo",
            "style",
            "flotante",
            "floating",
            "capsula",
        ],
        control: SearchControl::ToggleConcaveStyle,
    },
    SearchDefinition {
        id: "clock-24",
        title_key: "settings.clock_24",
        desc_key: "settings.clock_format_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "reloj", "clock", "24", "12", "hora", "time", "formato", "format",
        ],
        control: SearchControl::ToggleClock24,
    },
    SearchDefinition {
        id: "capsule-location",
        title_key: "settings.location_title",
        desc_key: "settings.location_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "ubicacion",
            "posicion",
            "arriba",
            "abajo",
            "top",
            "bottom",
            "location",
            "position",
            "capsula",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "dimensions-radii",
        title_key: "settings.radii_heading",
        desc_key: "settings.appearance_subtitle",
        tab: SettingsTab::Appearance,
        keywords: &[
            "radio",
            "radius",
            "round",
            "esquinas",
            "satelites",
            "tarjetas",
            "altura",
            "height",
            "idle",
            "margen",
            "margin",
            "gap",
            "separacion",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "motion-duration",
        title_key: "settings.anim_duration_title",
        desc_key: "settings.motion_heading",
        tab: SettingsTab::Appearance,
        keywords: &[
            "animacion",
            "animaciones",
            "duracion",
            "velocidad",
            "speed",
            "animation",
            "motion",
            "ms",
            "fluid",
            "fluido",
            "cubic",
            "cubico",
            "expo",
            "back",
            "elastico",
            "spring",
            "rebote",
            "rapido",
            "snappy",
            "suave",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "content-transition",
        title_key: "settings.content_transition_title",
        desc_key: "settings.content_transition_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "transicion",
            "contenido",
            "content",
            "fade",
            "desvanecer",
            "escala",
            "scale",
            "crossfade",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "spring-bounciness",
        title_key: "settings.spring_bounciness_title",
        desc_key: "settings.spring_bounciness_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "rebote",
            "bounciness",
            "intensidad",
            "elasticidad",
            "spring",
            "resorte",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "squash-stretch",
        title_key: "settings.squash_stretch_title",
        desc_key: "settings.squash_stretch_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "squash",
            "stretch",
            "fisica",
            "fluidos",
            "physics",
            "elastico",
            "compresion",
            "inercial",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "stagger-animation",
        title_key: "settings.stagger_animation_title",
        desc_key: "settings.stagger_animation_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "cascada",
            "stagger",
            "escalonado",
            "satelites",
            "orbitas",
            "paneles",
            "desfase",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "reduced-motion",
        title_key: "settings.reduced_motion_title",
        desc_key: "settings.reduced_motion_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "reducir",
            "movimiento",
            "reduced",
            "motion",
            "accesibilidad",
            "accessibility",
            "bateria",
            "ahorro",
        ],
        control: SearchControl::Navigate(SettingsTab::Appearance),
    },
    SearchDefinition {
        id: "legacy-capsule",
        title_key: "settings.old_capsule",
        desc_key: "settings.restart_hint",
        tab: SettingsTab::Appearance,
        keywords: &[
            "antigua", "vieja", "old", "legacy", "clasica", "capsula", "obsoleta",
        ],
        control: SearchControl::ToggleLegacyCapsule,
    },
    // --- Apps ---
    SearchDefinition {
        id: "apps-defaults",
        title_key: "settings.apps_title",
        desc_key: "settings.apps_subtitle",
        tab: SettingsTab::Apps,
        keywords: &[
            "aplicaciones",
            "apps",
            "terminal",
            "consola",
            "browser",
            "navegador",
            "web",
            "editor",
            "codigo",
            "code",
            "archivos",
            "files",
            "explorador",
        ],
        control: SearchControl::Navigate(SettingsTab::Apps),
    },
    // --- Media ---
    SearchDefinition {
        id: "audio-output",
        title_key: "settings.output_volume_title",
        desc_key: "settings.output_volume_subtitle",
        tab: SettingsTab::Media,
        keywords: &[
            "audio",
            "volumen",
            "volume",
            "salida",
            "output",
            "altavoces",
            "speakers",
            "parlantes",
            "sonido",
            "sound",
        ],
        control: SearchControl::Navigate(SettingsTab::Media),
    },
    SearchDefinition {
        id: "audio-input",
        title_key: "settings.input_volume_title",
        desc_key: "settings.input_volume_subtitle",
        tab: SettingsTab::Media,
        keywords: &[
            "microfono",
            "mic",
            "input",
            "entrada",
            "audio",
            "grabacion",
            "voz",
            "sonido",
        ],
        control: SearchControl::Navigate(SettingsTab::Media),
    },
    SearchDefinition {
        id: "music-players",
        title_key: "settings.music_players_title",
        desc_key: "settings.music_players_subtitle",
        tab: SettingsTab::Media,
        keywords: &[
            "reproductores",
            "players",
            "musica",
            "music",
            "spotify",
            "mpris",
            "autorizados",
            "whitelist",
        ],
        control: SearchControl::Navigate(SettingsTab::Media),
    },
    // --- Lock Screen ---
    SearchDefinition {
        id: "lock-clock",
        title_key: "settings.show_clock_title",
        desc_key: "settings.show_clock_subtitle",
        tab: SettingsTab::LockScreen,
        keywords: &["reloj", "bloqueo", "clock", "lockscreen", "hora", "fecha"],
        control: SearchControl::ToggleLockscreenClock,
    },
    SearchDefinition {
        id: "lock-media",
        title_key: "settings.show_media_title",
        desc_key: "settings.show_media_subtitle",
        tab: SettingsTab::LockScreen,
        keywords: &[
            "reproductor",
            "bloqueo",
            "lockscreen",
            "mpris",
            "musica",
            "music",
            "spotify",
            "cancion",
            "media",
        ],
        control: SearchControl::ToggleLockscreenMedia,
    },
    SearchDefinition {
        id: "lock-idle-timeout",
        title_key: "settings.idle_timeout_title",
        desc_key: "settings.lockscreen_header_subtitle",
        tab: SettingsTab::LockScreen,
        keywords: &[
            "inactividad",
            "bloqueo",
            "timeout",
            "idle",
            "suspension",
            "lock",
            "pantalla de bloqueo",
            "tiempo",
        ],
        control: SearchControl::Navigate(SettingsTab::LockScreen),
    },
    SearchDefinition {
        id: "lock-date-time-format",
        title_key: "settings.time_format_title",
        desc_key: "settings.date_format_title",
        tab: SettingsTab::LockScreen,
        keywords: &[
            "formato", "hora", "fecha", "format", "time", "date", "chrono", "reloj",
        ],
        control: SearchControl::Navigate(SettingsTab::LockScreen),
    },
    // --- System ---
    SearchDefinition {
        id: "system-language",
        title_key: "settings.language_title",
        desc_key: "settings.language_subtitle",
        tab: SettingsTab::System,
        keywords: &[
            "idioma", "lenguaje", "language", "español", "espanol", "spanish", "ingles", "english",
            "interfaz",
        ],
        control: SearchControl::Navigate(SettingsTab::System),
    },
    SearchDefinition {
        id: "system-power",
        title_key: "settings.power_plan_title",
        desc_key: "settings.power_plan_subtitle",
        tab: SettingsTab::System,
        keywords: &[
            "energia",
            "power",
            "bateria",
            "battery",
            "rendimiento",
            "performance",
            "perfil",
            "plan",
            "balanceado",
            "ahorro",
        ],
        control: SearchControl::Navigate(SettingsTab::System),
    },
    // --- Record ---
    SearchDefinition {
        id: "record-output",
        title_key: "settings.record_output_title",
        desc_key: "settings.record_output_subtitle",
        tab: SettingsTab::Record,
        keywords: &[
            "grabacion",
            "record",
            "pantalla",
            "screen",
            "monitor",
            "video",
            "salida",
        ],
        control: SearchControl::Navigate(SettingsTab::Record),
    },
    SearchDefinition {
        id: "record-quality",
        title_key: "settings.record_quality_title",
        desc_key: "settings.record_fps_title",
        tab: SettingsTab::Record,
        keywords: &[
            "fps",
            "resolucion",
            "calidad",
            "quality",
            "frame rate",
            "video",
            "1080p",
            "720p",
            "mkv",
            "mp4",
            "contenedor",
            "container",
        ],
        control: SearchControl::Navigate(SettingsTab::Record),
    },
    SearchDefinition {
        id: "record-cursor",
        title_key: "settings.record_cursor_title",
        desc_key: "settings.record_cursor_subtitle",
        tab: SettingsTab::Record,
        keywords: &[
            "cursor",
            "puntero",
            "raton",
            "mouse",
            "pointer",
            "grabacion",
            "record",
        ],
        control: SearchControl::ToggleRecordCursor,
    },
];

pub fn render_search_results(query: &str, theme: &Theme, cx: &mut Context<Settings>) -> AnyElement {
    let query_clean = query.trim().to_lowercase();
    let query_words: Vec<&str> = query_clean.split_whitespace().collect();

    let config = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get()
    } else {
        Default::default()
    };

    let mut matched_rows = Vec::new();

    for def in SEARCH_DEFINITIONS {
        let title = text(def.title_key, cx);
        let desc = text(def.desc_key, cx);
        let tab_str = tab_name(def.tab, cx);

        let matches = query_words.iter().all(|qw| {
            title.to_lowercase().contains(qw)
                || desc.to_lowercase().contains(qw)
                || tab_str.to_lowercase().contains(qw)
                || def
                    .keywords
                    .iter()
                    .any(|k| k.to_lowercase().contains(qw) || qw.contains(&k.to_lowercase()))
        });

        if !matches {
            continue;
        }

        let subtitle = format!("{} • {}", tab_str, desc);

        let row: AnyElement = match def.control {
            SearchControl::ToggleDynamicColors => {
                let is_on = config.ui.dynamic_colors;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.dynamic_colors = !cfg.ui.dynamic_colors;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleDarkMode => {
                let is_on = config.ui.dark_mode;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.dark_mode = !cfg.ui.dark_mode;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleConcaveStyle => {
                let is_on = config.ui.capsule_style == CapsuleStyle::Concave;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.capsule_style =
                                    if cfg.ui.capsule_style == CapsuleStyle::Normal {
                                        CapsuleStyle::Concave
                                    } else {
                                        CapsuleStyle::Normal
                                    };
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleClock24 => {
                let is_on = config.ui.clock_24_hour;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.clock_24_hour = !cfg.ui.clock_24_hour;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleLegacyCapsule => {
                let is_on = !config.ui.use_new_capsule;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.use_new_capsule = !cfg.ui.use_new_capsule;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleLockscreenClock => {
                let is_on = config.lockscreen.show_clock;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.lockscreen.show_clock = !cfg.lockscreen.show_clock;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleLockscreenMedia => {
                let is_on = config.lockscreen.show_media_player;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.lockscreen.show_media_player =
                                    !cfg.lockscreen.show_media_player;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::ToggleRecordCursor => {
                let is_on = config.record.include_cursor;
                toggle_row(
                    ElementId::Name(format!("search-toggle-{}", def.id).into()),
                    &title,
                    Some(&subtitle),
                    is_on,
                    theme,
                    cx.listener(|_this, _, _, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.record.include_cursor = !cfg.record.include_cursor;
                            });
                        }
                        cx.notify();
                    }),
                )
            }
            SearchControl::Navigate(tab) => {
                let tab_label = tab_str.clone();
                let nav_btn = div()
                    .id(format!("search-nav-{}", def.id))
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .h(px(28.0))
                    .px(px(12.0))
                    .rounded_full()
                    .bg(style::surface(theme))
                    .border_1()
                    .border_color(style::border(theme))
                    .hover(|s| s.bg(style::hover(theme)).border_color(theme.accent()))
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.set_tab(tab, cx);
                    }))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(tab_label),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(13.0))
                            .text_color(theme.foreground_muted()),
                    );

                control_row(&title, Some(&subtitle), nav_btn, theme)
            }
        };

        matched_rows.push(row);
    }

    let display_query = if query.chars().count() > 30 {
        format!("{}…", query.chars().take(30).collect::<String>())
    } else {
        query.to_string()
    };

    if matched_rows.is_empty() {
        let no_sub =
            text("settings.search_no_results_subtitle", cx).replace("{query}", &display_query);

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w_full()
            .min_w_0()
            .py(px(48.0))
            .gap(px(12.0))
            .child(
                div()
                    .size(px(52.0))
                    .rounded_full()
                    .bg(style::surface(theme))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("search.svg")
                            .size(px(24.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(16.0))
                    .text_color(theme.foreground())
                    .child(text("settings.search_no_results_title", cx)),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.foreground_muted())
                    .text_center()
                    .max_w(px(380.0))
                    .overflow_hidden()
                    .text_ellipsis()
                    .child(no_sub),
            )
            .into_any_element()
    } else {
        let count_str = matched_rows.len().to_string();
        let sub_text = text("settings.search_results_subtitle", cx)
            .replace("{count}", &count_str)
            .replace("{query}", &display_query);

        let mut card_el = card(theme);
        for (idx, row) in matched_rows.into_iter().enumerate() {
            if idx > 0 {
                card_el = card_el.child(row_divider(theme));
            }
            card_el = card_el.child(row);
        }

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            .child(hero_header(
                "search.svg",
                &text("settings.search_results_title", cx),
                &sub_text,
                theme,
            ))
            .child(card_el)
            .into_any_element()
    }
}
