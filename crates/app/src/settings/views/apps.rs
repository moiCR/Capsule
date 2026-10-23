use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, ElementId, IntoElement, ParentElement, Styled, Window, div, px, svg,
};
use services::{AppState, Application};
use ui::components::select::{Select, SelectItem};
use ui::theme::Theme;

use super::SettingsView;
use super::common::{card, control_row, hero_header, row_divider};
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppsField {
    Terminal,
    Browser,
    Editor,
    FileManager,
}

pub struct AppsView {
    open_dropdown: Option<AppsField>,
    terminal_cell: Rc<Cell<(f32, f32)>>,
    browser_cell: Rc<Cell<(f32, f32)>>,
    editor_cell: Rc<Cell<(f32, f32)>>,
    file_manager_cell: Rc<Cell<(f32, f32)>>,
}

impl AppsView {
    pub fn new() -> Self {
        Self {
            open_dropdown: None,
            terminal_cell: Rc::new(Cell::new((0.0, 0.0))),
            browser_cell: Rc::new(Cell::new((0.0, 0.0))),
            editor_cell: Rc::new(Cell::new((0.0, 0.0))),
            file_manager_cell: Rc::new(Cell::new((0.0, 0.0))),
        }
    }
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
    if stem_clean == "foot" || bin_name == "foot" {
        return "foot".to_string();
    }
    if stem_clean == "wezterm" || bin_name == "wezterm" {
        return "wezterm".to_string();
    }
    if stem_clean == "code" || bin_name == "code" {
        return "code".to_string();
    }
    if stem_clean == "firefox" || bin_name == "firefox" {
        return "firefox".to_string();
    }
    if stem_clean == "google-chrome" || bin_name == "google-chrome" {
        return "google-chrome".to_string();
    }
    if stem_clean == "chromium" || bin_name == "chromium" {
        return "chromium".to_string();
    }
    if stem_clean == "brave-browser" || bin_name == "brave-browser" {
        return "brave-browser".to_string();
    }
    if stem_clean == "nautilus" || bin_name == "nautilus" {
        return "nautilus".to_string();
    }
    if stem_clean == "thunar" || bin_name == "thunar" {
        return "thunar".to_string();
    }
    if stem_clean == "dolphin" || bin_name == "dolphin" {
        return "dolphin".to_string();
    }
    if stem_clean == "nemo" || bin_name == "nemo" {
        return "nemo".to_string();
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

fn filter_apps_for_field<'a>(field: AppsField, apps: &'a [Application]) -> Vec<&'a Application> {
    let mut filtered: Vec<&'a Application> = apps
        .iter()
        .filter(|app| {
            let id = app.id.to_lowercase();
            let name = app.name.to_lowercase();
            let exec = app.exec.to_lowercase();
            let generic_name = app.generic_name.as_deref().unwrap_or("").to_lowercase();
            let comment = app.comment.as_deref().unwrap_or("").to_lowercase();

            match field {
                AppsField::Terminal => {
                    generic_name.contains("terminal")
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
                AppsField::Browser => {
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
                AppsField::Editor => {
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
                AppsField::FileManager => {
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

fn render_select_for_field(
    field: AppsField,
    current_value: &str,
    is_open: bool,
    apps: &[Application],
    cell: Rc<Cell<(f32, f32)>>,
    fallback_icon: &'static str,
    theme: &Theme,
    cx: &mut Context<Settings>,
) -> AnyElement {
    let matched_app = find_matching_app(current_value, apps);
    let placeholder = text("settings.select_app", cx);
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

    let candidate_apps = filter_apps_for_field(field, apps);
    let select_id = ElementId::NamedInteger("app-select-btn".into(), field as u64);

    let mut select = Select::new(select_id)
        .placeholder(placeholder)
        .selected_label(display_name)
        .icon(display_icon)
        .is_open(is_open)
        .width(220.0)
        .track_bounds(cell)
        .on_toggle(cx.listener(move |this, _, _, cx| {
            if let Some(view) = this.view_as_mut::<AppsView>() {
                if view.open_dropdown == Some(field) {
                    view.open_dropdown = None;
                } else {
                    view.open_dropdown = Some(field);
                }
            }
            cx.notify();
        }));

    for app in candidate_apps {
        let cmd = app_to_command(app);
        let is_selected = current_value == cmd || current_value == app.name;
        let app_icon = render_app_icon(app.icon_path.as_deref(), fallback_icon, theme);
        let cmd_target = cmd.clone();

        let item = SelectItem::new(format!("app-item-{:?}-{}", field, app.id), app.name.clone())
            .selected(is_selected)
            .icon(app_icon)
            .on_click(cx.listener(move |this, _, _, cx| {
                if cx.has_global::<AppState>() {
                    let c = cmd_target.clone();
                    let _ = cx.global::<AppState>().config.update(|cfg| match field {
                        AppsField::Terminal => cfg.defaults.terminal = c,
                        AppsField::Browser => cfg.defaults.browser = c,
                        AppsField::Editor => cfg.defaults.editor = c,
                        AppsField::FileManager => cfg.defaults.file_manager = c,
                    });
                }
                if let Some(view) = this.view_as_mut::<AppsView>() {
                    view.open_dropdown = None;
                }
                cx.notify();
            }));

        select = select.item(item);
    }

    select.into_any_element()
}

impl SettingsView for AppsView {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(&mut self, _window: &mut Window, cx: &mut Context<Settings>) -> AnyElement {
        let theme = cx.global::<Theme>().clone();
        let config = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get()
        } else {
            Default::default()
        };

        let empty_apps = Vec::new();
        let installed_apps = if cx.has_global::<AppState>() {
            cx.global::<AppState>().launcher.get_apps()
        } else {
            std::sync::Arc::new(empty_apps)
        };

        let term_select = render_select_for_field(
            AppsField::Terminal,
            &config.defaults.terminal,
            self.open_dropdown == Some(AppsField::Terminal),
            &installed_apps,
            self.terminal_cell.clone(),
            "sparkles.svg",
            &theme,
            cx,
        );

        let browser_select = render_select_for_field(
            AppsField::Browser,
            &config.defaults.browser,
            self.open_dropdown == Some(AppsField::Browser),
            &installed_apps,
            self.browser_cell.clone(),
            "sparkles.svg",
            &theme,
            cx,
        );

        let editor_select = render_select_for_field(
            AppsField::Editor,
            &config.defaults.editor,
            self.open_dropdown == Some(AppsField::Editor),
            &installed_apps,
            self.editor_cell.clone(),
            "sparkles.svg",
            &theme,
            cx,
        );

        let file_manager_select = render_select_for_field(
            AppsField::FileManager,
            &config.defaults.file_manager,
            self.open_dropdown == Some(AppsField::FileManager),
            &installed_apps,
            self.file_manager_cell.clone(),
            "sparkles.svg",
            &theme,
            cx,
        );

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            .child(hero_header(
                "grid-dots.svg",
                &text("settings.apps_header_title", cx),
                &text("settings.apps_header_subtitle", cx),
                &theme,
            ))
            .child(
                card(&theme)
                    .child(control_row(
                        &text("settings.terminal_title", cx),
                        Some(&text("settings.terminal_subtitle", cx)),
                        term_select,
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.browser_title", cx),
                        Some(&text("settings.browser_subtitle", cx)),
                        browser_select,
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.editor_title", cx),
                        Some(&text("settings.editor_subtitle", cx)),
                        editor_select,
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.file_manager_title", cx),
                        Some(&text("settings.file_manager_subtitle", cx)),
                        file_manager_select,
                        &theme,
                    )),
            )
            .into_any_element()
    }
}
