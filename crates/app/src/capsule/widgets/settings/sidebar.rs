use gpui::{
    Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div,
    prelude::*, px, svg,
};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::modules::settings::{SettingsField, SettingsModule, SettingsTab};

pub fn render_sidebar(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let capsule_radius = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.capsule_round
    } else {
        24.0
    };

    let (placeholder, tab_capsule, tab_apps, tab_media, tab_lock, tab_sys) =
        if cx.has_global::<AppState>() {
            let lang = &cx.global::<AppState>().language;
            (
                lang.get("settings.search_placeholder"),
                lang.get("settings.tab_capsule"),
                lang.get("settings.tab_apps"),
                lang.get("settings.tab_media"),
                lang.get("settings.tab_lockscreen"),
                lang.get("settings.tab_system"),
            )
        } else {
            (
                "Buscar en configuración...".to_string(),
                "Cápsula".to_string(),
                "Aplicaciones".to_string(),
                "Multimedia".to_string(),
                "Bloqueo".to_string(),
                "Sistema".to_string(),
            )
        };

    let is_search_active = module.active_field == Some(SettingsField::Search);
    let search_border = if is_search_active {
        theme.accent()
    } else {
        theme.surface().opacity(0.3)
    };

    let search_text = &module.search_query;

    let tabs = [
        (SettingsTab::Capsule, "dashboard.svg", tab_capsule),
        (SettingsTab::Apps, "sparkles.svg", tab_apps),
        (SettingsTab::Media, "music.svg", tab_media),
        (SettingsTab::LockScreen, "lock.svg", tab_lock),
        (SettingsTab::System, "settings.svg", tab_sys),
    ];

    let mut list = div().flex().flex_col().gap(px(4.0)).w_full();

    for (tab, icon, label) in tabs {
        let is_active = module.active_tab == tab && module.search_query.trim().is_empty();
        list = list.child(render_category_item(tab, icon, label, is_active, theme, cx));
    }

    let clear_btn = if !search_text.is_empty() {
        Some(
            div()
                .id("settings-search-clear")
                .flex()
                .items_center()
                .justify_center()
                .w(px(18.0))
                .h(px(18.0))
                .rounded_full()
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.search_query.clear();
                    this.active_field = None;
                    this.start_section_transition(cx);
                    cx.notify();
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(9.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
    } else {
        None
    };

    div()
        .flex()
        .flex_col()
        .w(px(230.0))
        .min_w(px(230.0))
        .max_w(px(230.0))
        .flex_shrink_0()
        .h_full()
        .rounded_tl(px(capsule_radius))
        .rounded_bl(px(capsule_radius))
        .p_3()
        .bg(theme.background_alt())
        .border_r_1()
        .border_color(theme.surface().opacity(0.2))
        .child(
            div()
                .id("settings-search-box")
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .h(px(34.0))
                .px_2p5()
                .rounded(px(10.0))
                .bg(theme.surface().opacity(0.35))
                .border_1()
                .border_color(search_border)
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.set_active_field(Some(SettingsField::Search), cx);
                }))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .flex_1()
                        .min_w_0()
                        .child(
                            svg()
                                .path("search.svg")
                                .size(px(13.0))
                                .text_color(theme.foreground_muted().opacity(0.7)),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(if search_text.is_empty() && !is_search_active {
                                    theme.foreground_muted()
                                } else {
                                    theme.foreground()
                                })
                                .child(if is_search_active {
                                    format!("{search_text}|")
                                } else if search_text.is_empty() {
                                    placeholder
                                } else {
                                    search_text.clone()
                                }),
                        ),
                )
                .children(clear_btn),
        )
        .child(
            div()
                .id("settings-sidebar-scroll")
                .flex_1()
                .mt_3()
                .overflow_scroll()
                .child(list),
        )
}

fn render_category_item(
    tab: SettingsTab,
    icon: &'static str,
    label: String,
    is_active: bool,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let bg = if is_active {
        theme.surface().opacity(0.65)
    } else {
        gpui::transparent_black()
    };

    div()
        .id(ElementId::NamedInteger("settings-cat".into(), tab as u64))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .w_full()
        .px_2()
        .py_1p5()
        .rounded(px(12.0))
        .bg(bg)
        .hover(|s| {
            if !is_active {
                s.bg(theme.surface().opacity(0.3))
            } else {
                s
            }
        })
        .active(|s| s.bg(theme.surface().opacity(0.75)))
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.set_tab(tab, cx);
        }))
        .child(
            div()
                .w(px(28.0))
                .h(px(28.0))
                .rounded_full()
                .bg(if is_active {
                    theme.accent().opacity(0.2)
                } else {
                    theme.surface().opacity(0.45)
                })
                .flex()
                .items_center()
                .justify_center()
                .child(svg().path(icon).size(px(14.0)).text_color(if is_active {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                })),
        )
        .child(
            div()
                .font_weight(if is_active {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
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
