use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled, div,
    prelude::*, px, svg,
};
use services::{AppState, Application};
use ui::theme::Theme;

use super::setting_item::{render_section_header, render_setting_row};
use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_defaults_section(
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
            "Aplicaciones Predeterminadas",
            "Configura los binarios o comandos ejecutados por Capsule para tus herramientas principales.",
            theme,
        ))
        .child(render_setting_row(
            "Terminal",
            "Lanzado con 'capsule terminal' o apps en terminal.",
            render_app_select(
                SettingsField::Terminal,
                &module.terminal_input,
                module.active_field == Some(SettingsField::Terminal),
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
                SettingsField::Browser,
                &module.browser_input,
                module.active_field == Some(SettingsField::Browser),
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
                SettingsField::Editor,
                &module.editor_input,
                module.active_field == Some(SettingsField::Editor),
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
    field: SettingsField,
    current_value: &str,
    is_active_input: bool,
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

    let select_trigger = div()
        .id(ElementId::NamedInteger(
            "app-select-btn".into(),
            field as u64,
        ))
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w(px(250.0))
        .px_3()
        .py_2()
        .rounded(px(12.0))
        .bg(theme.background())
        .border_1()
        .border_color(if is_open {
            theme.accent()
        } else {
            theme.surface().opacity(0.6)
        })
        .hover(|s| s.border_color(theme.accent().opacity(0.8)))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.toggle_dropdown(field, cx);
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
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if current_value.is_empty() {
                            theme.foreground_muted()
                        } else {
                            theme.foreground()
                        })
                        .truncate()
                        .child(display_name),
                ),
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
                }),
        );

    let dropdown_menu = if is_open {
        let candidate_apps = filter_apps_for_field(field, apps);
        let mut menu = div()
            .id(ElementId::NamedInteger(
                "app-select-dropdown".into(),
                field as u64,
            ))
            .absolute()
            .top(px(40.0))
            .right_0()
            .flex()
            .flex_col()
            .w(px(250.0))
            .max_h(px(220.0))
            .p_1p5()
            .gap(px(2.0))
            .rounded(px(12.0))
            .bg(theme.background())
            .border_1()
            .border_color(theme.accent().opacity(0.4))
            .shadow_xl()
            .overflow_scroll();

        for app in candidate_apps {
            let app_cmd = app_to_command(app);
            let is_selected = current_value == app_cmd || current_value == app.name;
            let cmd_to_save = app_cmd.clone();
            let app_name = app.name.clone();
            let icon_el = render_app_icon(app.icon_path.as_deref(), fallback_icon, theme);

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
                    .px_2p5()
                    .py_1p5()
                    .rounded(px(8.0))
                    .bg(if is_selected {
                        theme.accent().opacity(0.18)
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|s| {
                        if !is_selected {
                            s.bg(theme.surface().opacity(0.4))
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
            .border_color(theme.surface().opacity(0.4))
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(theme.foreground_muted())
                    .child("Comando personalizado:"),
            )
            .child(
                div()
                    .id(ElementId::NamedInteger(
                        "custom-cmd-input".into(),
                        field as u64,
                    ))
                    .flex()
                    .items_center()
                    .w_full()
                    .px_2p5()
                    .py_1()
                    .rounded(px(6.0))
                    .bg(theme.surface().opacity(0.3))
                    .border_1()
                    .border_color(custom_input_border)
                    .cursor_pointer()
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
                            } else if is_active_input {
                                format!("{current_value}|")
                            } else {
                                current_value.to_string()
                            }),
                    ),
            );

        menu = menu.child(custom_box);
        Some(menu)
    } else {
        None
    };

    div()
        .relative()
        .child(select_trigger)
        .children(dropdown_menu.map(|m| gpui::deferred(m).with_priority(100)))
}
