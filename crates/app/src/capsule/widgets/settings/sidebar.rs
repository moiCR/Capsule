use gpui::{Context, IntoElement, ParentElement, Styled, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::modules::settings::{SettingsModule, SettingsTab};

pub fn render_sidebar(
    active_tab: SettingsTab,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let capsule_radius = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.capsule_round
    } else {
        36.0
    };

    let (title, tab_gen, tab_ui, tab_lock) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.title"),
            lang.get("settings.tab_general"),
            lang.get("settings.tab_appearance"),
            lang.get("settings.tab_lockscreen"),
        )
    } else {
        (
            "Configuración".to_string(),
            "General".to_string(),
            "Apariencia".to_string(),
            "Bloqueo".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .justify_between()
        .w(px(220.0))
        .h_full()
        .rounded_tl(px(capsule_radius))
        .rounded_bl(px(capsule_radius))
        .p_4()
        .bg(theme.background_alt())
        .border_r_1()
        .border_color(theme.surface().opacity(0.25))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .pb_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .w(px(24.0))
                                        .h(px(24.0))
                                        .rounded_full()
                                        .bg(theme.accent().opacity(0.15))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            svg()
                                                .path("dashboard.svg")
                                                .size(px(14.0))
                                                .text_color(theme.accent()),
                                        ),
                                )
                                .child(
                                    div()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_size(px(14.0))
                                        .text_color(theme.foreground())
                                        .child(title),
                                ),
                        )
                        .child(
                            div()
                                .id("close-settings-btn")
                                .w(px(24.0))
                                .h(px(24.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.45))
                                .hover(|s| s.bg(theme.surface().opacity(0.75)))
                                .active(|s| s.bg(theme.surface()))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.close(cx);
                                }))
                                .child(
                                    svg()
                                        .path("close.svg")
                                        .size(px(12.0))
                                        .text_color(theme.foreground_muted()),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(render_tab_button(
                            SettingsTab::General,
                            "settings.svg",
                            tab_gen,
                            active_tab == SettingsTab::General,
                            theme,
                            cx,
                        ))
                        .child(render_tab_button(
                            SettingsTab::UI,
                            "palette_2.svg",
                            tab_ui,
                            active_tab == SettingsTab::UI,
                            theme,
                            cx,
                        ))
                        .child(render_tab_button(
                            SettingsTab::LockScreen,
                            "moon_1.svg",
                            tab_lock,
                            active_tab == SettingsTab::LockScreen,
                            theme,
                            cx,
                        )),
                ),
        )
}

fn render_tab_button(
    tab: SettingsTab,
    icon: &'static str,
    label: String,
    is_active: bool,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let bg = if is_active {
        theme.surface().opacity(0.55)
    } else {
        gpui::transparent_black()
    };
    let fg = if is_active {
        theme.accent()
    } else {
        theme.foreground_muted()
    };

    let indicator_color = if is_active {
        theme.accent()
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    div()
        .id(ElementId::NamedInteger("settings-tab".into(), tab as u64))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .w_full()
        .px_2p5()
        .py_2()
        .rounded(px(10.0))
        .bg(bg)
        .hover(|s| {
            if !is_active {
                s.bg(theme.surface().opacity(0.35))
                    .text_color(theme.foreground())
            } else {
                s
            }
        })
        .active(|s| s.bg(theme.surface().opacity(0.65)))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_tab(tab, cx);
        }))
        .child(
            div()
                .w(px(3.0))
                .h(px(16.0))
                .rounded_full()
                .bg(indicator_color),
        )
        .child(svg().path(icon).size(px(15.0)).text_color(fg))
        .child(
            div()
                .font_weight(if is_active {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::NORMAL
                })
                .text_size(px(12.5))
                .text_color(if is_active {
                    theme.foreground()
                } else {
                    theme.foreground_muted()
                })
                .child(label),
        )
}

use gpui::ElementId;
