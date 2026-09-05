use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, canvas, div,
    prelude::*, px, svg,
};
use services::{AppState, Application, PowerProfile};
use ui::language::language_manager::LanguageManager;
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

    let languages = if cx.has_global::<LanguageManager>() {
        cx.global::<LanguageManager>().list_languages()
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

    div()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .w_full()
        .child(render_section_header(
            "General",
            "Configuración general del sistema, perfiles de energía, idioma y aplicaciones predeterminadas.",
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
        .child(
            div()
                .pt_3()
                .child(render_section_header(
                    "Aplicaciones Predeterminadas",
                    "Configura los binarios o comandos ejecutados por Capsule para tus herramientas principales.",
                    theme,
                )),
        )
        .child(render_setting_row(
            "Terminal",
            "Lanzado con 'capsule terminal' o apps en terminal.",
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
            "Navegador Web",
            "Lanzado con 'capsule browser' o links del sistema.",
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
            "Editor de Código / Texto",
            "Lanzado con 'capsule editor'. Si es CLI, se ejecuta en terminal.",
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

fn splatoon_overshoot(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.95;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
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
    let display_name = if let Some(app) = matched_app {
        app.name.clone()
    } else if current_value.is_empty() {
        "Seleccionar aplicación...".to_string()
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

    let is_splat = module.splat_anim_field == Some(field);
    let splat_p = if is_splat {
        module.splat_anim_progress
    } else {
        1.0
    };
    let splat_wobble = (splat_p * std::f32::consts::PI * 3.0).sin() * (1.0 - splat_p);
    let trigger_w = if is_splat {
        250.0 + splat_wobble * 10.0
    } else if is_open {
        252.0
    } else {
        250.0
    };

    let flash_alpha = if is_splat {
        (1.0 - splat_p) * 0.35
    } else {
        0.0
    };

    let trigger_bg = if is_splat {
        theme.accent().opacity(0.12 + flash_alpha)
    } else if is_open {
        theme.accent().opacity(0.14)
    } else {
        theme.background()
    };

    let trigger_border_color = if is_splat || is_open {
        theme.accent()
    } else {
        theme.surface().opacity(0.6)
    };

    let chevron_bounce = if is_open {
        (dropdown_p * std::f32::consts::PI * 2.0).sin() * 2.5 * (1.0 - dropdown_p)
    } else {
        0.0
    };

    let trigger_cell = match field {
        SettingsField::Terminal => Some(module.terminal_trigger_bounds.clone()),
        SettingsField::Browser => Some(module.browser_trigger_bounds.clone()),
        SettingsField::Editor => Some(module.editor_trigger_bounds.clone()),
        _ => None,
    };

    div()
        .id(ElementId::NamedInteger(
            "app-select-btn".into(),
            field as u64,
        ))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w(px(trigger_w))
        .px_3()
        .py_2()
        .rounded_tl(px(if is_open { 16.0 } else { 13.0 }))
        .rounded_tr(px(if is_open { 8.0 } else { 10.0 }))
        .rounded_br(px(if is_open { 4.0 } else { 13.0 }))
        .rounded_bl(px(if is_open { 14.0 } else { 10.0 }))
        .bg(trigger_bg)
        .border_2()
        .border_color(trigger_border_color)
        .hover(|s| {
            if !is_open {
                s.border_color(theme.accent())
                    .bg(theme.accent().opacity(0.08))
            } else {
                s
            }
        })
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.toggle_dropdown(field, cx);
        }))
        .children(trigger_cell.map(|cell| {
            canvas(
                move |bounds, _, _| {
                    cell.set((
                        bounds.origin.y.into(),
                        (bounds.origin.y + bounds.size.height).into(),
                    ));
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_0()
        }))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .min_w_0()
                .child(display_icon)
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(if is_open || is_splat {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if current_value.is_empty() {
                            theme.foreground_muted()
                        } else if is_open || is_splat {
                            theme.accent()
                        } else {
                            theme.foreground()
                        })
                        .truncate()
                        .child(display_name),
                )
                .children(if is_splat {
                    Some(
                        div()
                            .w(px(6.0))
                            .h(px(6.0))
                            .rounded_full()
                            .bg(theme.accent()),
                    )
                } else {
                    None
                }),
        )
        .child(
            svg()
                .path("chevron-down.svg")
                .size(px(14.0))
                .flex_shrink_0()
                .text_color(if is_open {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                })
                .mt(px(chevron_bounce)),
        )
}

pub fn render_dropdown_overlay(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> Option<AnyElement> {
    let field = module.open_dropdown?;
    let current_value = match field {
        SettingsField::Terminal => module.terminal_input.as_str(),
        SettingsField::Browser => module.browser_input.as_str(),
        SettingsField::Editor => module.editor_input.as_str(),
        _ => return None,
    };

    let empty_apps = Vec::new();
    let installed_apps = if cx.has_global::<AppState>() {
        cx.global::<AppState>().launcher.get_apps()
    } else {
        std::sync::Arc::new(empty_apps)
    };

    let is_animating = module.dropdown_anim_field == Some(field);
    let dropdown_p = if is_animating {
        module.dropdown_anim_progress
    } else {
        1.0
    };
    let dropdown_eased = splatoon_overshoot(dropdown_p);

    let (measured_top, measured_bottom) = match field {
        SettingsField::Terminal => module.terminal_trigger_bounds.get(),
        SettingsField::Browser => module.browser_trigger_bounds.get(),
        SettingsField::Editor => module.editor_trigger_bounds.get(),
        _ => (0.0, 0.0),
    };
    let settings_top = module.settings_top_y.get();
    let (local_top, local_bottom) = if measured_bottom > settings_top && settings_top > 0.0 {
        (measured_top - settings_top, measured_bottom - settings_top)
    } else {
        match field {
            SettingsField::Terminal => (340.0, 378.0),
            SettingsField::Browser => (416.0, 454.0),
            SettingsField::Editor => (492.0, 530.0),
            _ => (340.0, 378.0),
        }
    };

    let fallback_icon = "sparkles.svg";
    let candidate_apps = filter_apps_for_field(field, &installed_apps);
    let estimated_height = (candidate_apps.len() as f32 * 34.0 + 58.0).min(220.0);

    let space_above = local_top;
    let space_below = 560.0 - local_bottom;
    let open_upwards = space_below < estimated_height && space_above >= space_below;

    let available_space = if open_upwards {
        (space_above - 12.0).max(100.0)
    } else {
        (space_below - 12.0).max(100.0)
    };
    let target_max_h = 220.0f32.min(available_space);
    let menu_max_h = (36.0 + (target_max_h - 36.0) * dropdown_eased).clamp(36.0, target_max_h);

    let squash_w = (1.0 - dropdown_p) * 16.0
        - (dropdown_p * std::f32::consts::PI).sin() * (1.0 - dropdown_p) * 10.0;
    let menu_w = 250.0 + squash_w;

    let (tl, tr, br, bl) = if open_upwards {
        (
            22.0 - 4.0 * (1.0 - dropdown_p),
            16.0 + 4.0 * (1.0 - dropdown_p),
            8.0 + 8.0 * (1.0 - dropdown_p),
            14.0 + 4.0 * (1.0 - dropdown_p),
        )
    } else {
        (
            16.0 + 4.0 * (1.0 - dropdown_p),
            8.0 + 8.0 * (1.0 - dropdown_p),
            22.0 - 4.0 * (1.0 - dropdown_p),
            16.0 + 4.0 * (1.0 - dropdown_p),
        )
    };

    let mut menu = div()
        .id(ElementId::NamedInteger(
            "app-select-dropdown".into(),
            field as u64,
        ))
        .absolute()
        .right(px(40.0))
        .flex()
        .flex_col()
        .w(px(menu_w))
        .max_h(px(menu_max_h))
        .p_1p5()
        .gap(px(2.0))
        .rounded_tl(px(tl))
        .rounded_tr(px(tr))
        .rounded_br(px(br))
        .rounded_bl(px(bl))
        .bg(theme.background())
        .border_2()
        .border_color(theme.accent())
        .shadow_xl()
        .overflow_scroll()
        .on_click(cx.listener(|_this, _, _, cx| {
            cx.stop_propagation();
        }));

    if open_upwards {
        let menu_bottom = (560.0 - local_top) + 4.0 + 4.0 * dropdown_eased.max(0.0);
        menu = menu.bottom(px(menu_bottom));
    } else {
        let menu_top = local_bottom + 4.0 + 4.0 * dropdown_eased.max(0.0);
        menu = menu.top(px(menu_top));
    }

    for (i, app) in candidate_apps.into_iter().enumerate() {
        let app_cmd = app_to_command(app);
        let is_selected = current_value == app_cmd || current_value == app.name;
        let cmd_to_save = app_cmd.clone();
        let app_name = app.name.clone();
        let icon_el = render_app_icon(app.icon_path.as_deref(), fallback_icon, theme);

        let item_delay = (i as f32) * 0.03;
        let item_t = ((dropdown_p - item_delay) / (1.0 - item_delay).max(0.01)).clamp(0.0, 1.0);
        let item_eased = splatoon_overshoot(item_t);
        let slide_x = (1.0 - item_eased) * -12.0;

        menu = menu.child(
            div()
                .id(ElementId::Name(
                    format!("app-item-{field:?}-{app_name}").into(),
                ))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .ml(px(slide_x.max(-12.0)))
                .px_2p5()
                .py_1p5()
                .rounded_tl(px(12.0))
                .rounded_tr(px(6.0))
                .rounded_br(px(14.0))
                .rounded_bl(px(8.0))
                .bg(if is_selected {
                    theme.accent().opacity(0.22)
                } else {
                    gpui::transparent_black()
                })
                .border_1()
                .border_color(if is_selected {
                    theme.accent().opacity(0.7)
                } else {
                    gpui::transparent_black()
                })
                .hover(|s| {
                    if !is_selected {
                        s.bg(theme.accent().opacity(0.24))
                            .border_1()
                            .border_color(theme.accent().opacity(0.5))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_app_command(field, cmd_to_save.clone(), cx);
                }))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .min_w_0()
                        .children(if is_selected {
                            Some(
                                div()
                                    .w(px(3.5))
                                    .h(px(12.0))
                                    .rounded_full()
                                    .bg(theme.accent())
                                    .flex_shrink_0(),
                            )
                        } else {
                            None
                        })
                        .child(icon_el)
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if is_selected {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::MEDIUM
                                })
                                .text_color(if is_selected {
                                    theme.accent()
                                } else {
                                    theme.foreground()
                                })
                                .truncate()
                                .child(app_name),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .child(app_cmd),
                ),
        );
    }

    let is_active_input = module.active_field == Some(field);
    let custom_input_border = if is_active_input {
        theme.accent()
    } else {
        theme.surface().opacity(0.5)
    };

    let custom_box = div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .pt_2()
        .mt_1()
        .border_t_1()
        .border_color(theme.accent().opacity(0.3))
        .child(
            div()
                .text_size(px(10.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child("O comando personalizado:"),
        )
        .child(
            div()
                .id(ElementId::NamedInteger(
                    "app-select-custom-input".into(),
                    field as u64,
                ))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .px_2()
                .py_1()
                .rounded_tl(px(8.0))
                .rounded_tr(px(4.0))
                .rounded_br(px(10.0))
                .rounded_bl(px(6.0))
                .bg(theme.surface().opacity(0.3))
                .border_1()
                .border_color(custom_input_border)
                .cursor_text()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_active_field(Some(field), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(if current_value.is_empty() {
                            theme.foreground_muted()
                        } else {
                            theme.foreground()
                        })
                        .truncate()
                        .child(if current_value.is_empty() {
                            "Escribir comando...".to_string()
                        } else {
                            current_value.to_string()
                        }),
                )
                .child(if is_active_input {
                    div().w(px(2.0)).h(px(12.0)).bg(theme.accent())
                } else {
                    div()
                }),
        );

    menu = menu.child(custom_box);

    let backdrop = div()
        .id("app-select-dropdown-backdrop")
        .absolute()
        .inset_0()
        .on_click(cx.listener(|this, _, _, cx| {
            cx.stop_propagation();
            this.open_dropdown = None;
            this.dropdown_anim_field = None;
            cx.notify();
        }));

    Some(
        div()
            .id("app-dropdown-overlay-root")
            .absolute()
            .inset_0()
            .child(backdrop)
            .child(menu)
            .into_any_element(),
    )
}
