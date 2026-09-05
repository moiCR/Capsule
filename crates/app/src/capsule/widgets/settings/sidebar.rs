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
        .border_color(theme.surface().opacity(0.5))
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
                        .pb_2()
                        .border_b_1()
                        .border_color(theme.surface().opacity(0.4))
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
                                        .child("Configuración"),
                                ),
                        )
                        .child(
                            div()
                                .id("close-settings-btn")
                                .w(px(26.0))
                                .h(px(26.0))
                                .rounded_full()
                                .bg(theme.surface().opacity(0.5))
                                .hover(|s| s.bg(theme.accent().opacity(0.2)))
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
                                        .size(px(13.0))
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
                            "General",
                            active_tab == SettingsTab::General,
                            theme,
                            cx,
                        ))
                        .child(render_tab_button(
                            SettingsTab::UI,
                            "palette_2.svg",
                            "Apariencia",
                            active_tab == SettingsTab::UI,
                            theme,
                            cx,
                        ))
                        .child(render_tab_button(
                            SettingsTab::LockScreen,
                            "moon_1.svg",
                            "Bloqueo",
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
    label: &'static str,
    is_active: bool,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let bg = if is_active {
        theme.accent().opacity(0.18)
    } else {
        gpui::transparent_black()
    };
    let fg = if is_active {
        theme.accent()
    } else {
        theme.foreground_muted()
    };

    div()
        .id(ElementId::NamedInteger("settings-tab".into(), tab as u64))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .w_full()
        .px_3()
        .py_2()
        .rounded(px(12.0))
        .bg(bg)
        .hover(|s| {
            if !is_active {
                s.bg(theme.surface().opacity(0.4))
                    .text_color(theme.foreground())
            } else {
                s
            }
        })
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_tab(tab, cx);
        }))
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
