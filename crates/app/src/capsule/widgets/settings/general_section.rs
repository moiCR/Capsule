use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div,
    prelude::*, px, svg,
};
use services::{AppState, Application, LanguageInfo, PowerProfile};
use ui::components::select::{Select, SelectItem};
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_general_section(
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

    let empty_apps = Vec::new();
    let installed_apps = if cx.has_global::<AppState>() {
        cx.global::<AppState>().launcher.get_apps()
    } else {
        std::sync::Arc::new(empty_apps)
    };

    let cards_round = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        16.0
    };

    let (
        header_title,
        header_sub,
        power_title,
        power_sub,
        lang_title,
        lang_sub,
        apps_title,
        apps_sub,
        term_title,
        term_sub,
        browser_title,
        browser_sub,
        editor_title,
        editor_sub,
        music_title,
        music_sub,
        music_placeholder,
        music_add,
        music_empty,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.general_header_title"),
            lang.get("settings.general_header_subtitle"),
            lang.get("settings.power_plan_title"),
            lang.get("settings.power_plan_subtitle"),
            lang.get("settings.language_title"),
            lang.get("settings.language_subtitle"),
            lang.get("settings.default_apps_title"),
            lang.get("settings.default_apps_subtitle"),
            lang.get("settings.terminal_title"),
            lang.get("settings.terminal_subtitle"),
            lang.get("settings.browser_title"),
            lang.get("settings.browser_subtitle"),
            lang.get("settings.editor_title"),
            lang.get("settings.editor_subtitle"),
            lang.get("settings.music_players_title"),
            lang.get("settings.music_players_subtitle"),
            lang.get("settings.music_players_placeholder"),
            lang.get("settings.music_players_add"),
            lang.get("settings.music_players_empty"),
        )
    } else {
        (
            "General".to_string(),
            "Configuración general del sistema, perfiles de energía, idioma y aplicaciones predeterminadas.".to_string(),
            "Plan de Energía".to_string(),
            "Ajusta el consumo y velocidad del procesador vía power-profiles-daemon.".to_string(),
            "Idioma de la Interfaz".to_string(),
            "Cambia el idioma activo de las etiquetas, controles y textos del sistema.".to_string(),
            "Aplicaciones Predeterminadas".to_string(),
            "Configura los binarios o comandos ejecutados por Capsule para tus herramientas principales.".to_string(),
            "Terminal".to_string(),
            "Lanzado con 'capsule terminal' o apps en terminal.".to_string(),
            "Navegador Web".to_string(),
            "Lanzado con 'capsule browser' o links del sistema.".to_string(),
            "Editor de Código / Texto".to_string(),
            "Lanzado con 'capsule editor'. Si es CLI, se ejecuta en terminal.".to_string(),
            "Reproductores de Música (MPRIS)".to_string(),
            "Servicios MPRIS permitidos para control multimedia (ej. Spotify, Fastpotify).".to_string(),
            "Nombre de la app (ej. Fastpotify)...".to_string(),
            "Añadir".to_string(),
            "No hay reproductores configurados.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(&header_title, &header_sub, theme))
        .child(render_setting_row(
            &power_title,
            &power_sub,
            render_power_selector(current_profile, theme, cx),
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &lang_title,
            &lang_sub,
            render_language_select(module, &languages, theme, cx),
            cards_round,
            theme,
        ))
        .child(
            div()
                .pt_3()
                .child(render_section_header(&apps_title, &apps_sub, theme)),
        )
        .child(render_setting_row(
            &term_title,
            &term_sub,
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
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &browser_title,
            &browser_sub,
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
            cards_round,
            theme,
        ))
        .child(render_setting_row(
            &editor_title,
            &editor_sub,
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
            cards_round,
            theme,
        ))
        .child(render_music_players_card(
            module,
            &music_title,
            &music_sub,
            &music_placeholder,
            &music_add,
            &music_empty,
            cards_round,
            theme,
            cx,
        ))
}

#[allow(clippy::too_many_arguments)]
fn render_music_players_card(
    module: &SettingsModule,
    title: &str,
    subtitle: &str,
    placeholder: &str,
    add_label: &str,
    empty_label: &str,
    cards_round: f32,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let mut chips_row = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
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
        .w(px(220.0))
        .px_3()
        .py_1p5()
        .rounded(px(24.0))
        .bg(theme.background())
        .border_1()
        .border_color(border_color)
        .cursor_text()
        .on_click(cx.listener(|this, _, _, cx| {
            this.set_active_field(Some(SettingsField::MusicPlayer), cx);
        }))
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(if current_val.is_empty() {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .truncate()
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
        .px_3()
        .py_1p5()
        .rounded(px(24.0))
        .bg(theme.accent())
        .hover(|s| s.opacity(0.9))
        .active(|s| s.opacity(0.75))
        .cursor_pointer()
        .on_click(cx.listener(|this, _, _, cx| {
            let input = this.music_player_input.clone();
            this.add_music_player(input, cx);
            this.set_active_field(None, cx);
        }))
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(theme.background())
                .child(format!("+ {add_label}")),
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
                    .px_2p5()
                    .py_1()
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

    let input_actions = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(px(8.0))
        .child(input_field)
        .child(add_btn)
        .child(suggestions_row);

    div()
        .id("music-players-card")
        .flex()
        .flex_col()
        .w_full()
        .px_4()
        .py_3p5()
        .rounded(px(cards_round))
        .bg(theme.surface().opacity(0.4))
        .border_1()
        .border_color(theme.surface().opacity(0.25))
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
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
        .child(chips_row)
        .child(input_actions)
}

fn render_power_selector(
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
                .gap(px(5.0))
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

fn render_language_select(
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
        .width(250.0)
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
fn render_app_select(
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
        .width(250.0)
        .on_toggle(cx.listener(move |this, _, _, cx| {
            this.toggle_dropdown(field, cx);
        }))
        .items(items)
}
