use gpui::{
    Context, Div, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px, svg,
};
use services::{AppState, CapsuleStyle};
use ui::theme::Theme;

use super::style;
use crate::capsule::modules::settings::{SettingsField, SettingsModule, SettingsTab};
use crate::capsule::widgets::settings::setting_item::{render_slider_row, render_toggle_row};

pub(crate) const WIDTH: f32 = 1120.0;
pub(crate) const HEIGHT: f32 = 780.0;

fn text(key: &str, cx: &gpui::App) -> String {
    cx.global::<AppState>().language.get(key)
}

pub(crate) fn render_header(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let searching = module.active_field == Some(SettingsField::Search);
    let label = if module.search_query.is_empty() && !searching {
        text("settings.search_short", cx)
    } else if searching {
        format!("{}│", module.search_query)
    } else {
        module.search_query.clone()
    };
    let search = div()
        .id("settings-search-box")
        .flex()
        .items_center()
        .gap(px(10.0))
        .w(px(420.0))
        .h(px(44.0))
        .px(px(14.0))
        .rounded_full()
        .bg(style::surface(theme))
        .border_1()
        .border_color(if searching {
            theme.accent()
        } else {
            style::surface(theme)
        })
        .cursor_text()
        .on_click(cx.listener(|this, _, window, cx| {
            this.focus(window, cx);
            this.set_active_field(Some(SettingsField::Search), cx);
        }))
        .child(
            svg()
                .path("search.svg")
                .size(px(15.0))
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_ellipsis()
                .text_size(px(14.0))
                .text_color(if searching {
                    theme.foreground()
                } else {
                    theme.foreground_muted()
                })
                .child(label),
        )
        .when(!module.search_query.is_empty(), |s| {
            s.child(
                style::circle_button("settings-search-clear", "close.svg", 22.0, theme).on_click(
                    cx.listener(|this, _, _, cx| {
                        cx.stop_propagation();
                        this.search_query.clear();
                        this.start_section_transition(cx);
                        cx.notify();
                    }),
                ),
            )
        });
    div()
        .relative()
        .flex()
        .items_center()
        .justify_center()
        .w_full()
        .h(px(48.0))
        .flex_shrink_0()
        .child(search)
        .when(module.can_navigate_back(), |s| {
            s.child(
                div().absolute().left_0().child(
                    style::circle_button("settings-nav-back", "chevron-left.svg", 30.0, theme)
                        .on_click(cx.listener(|this, _, _, cx| this.navigate_back(cx))),
                ),
            )
        })
        .child(
            div().absolute().right_0().child(
                style::circle_button("settings-close-btn", "close.svg", 30.0, theme)
                    .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
            ),
        )
}

pub(crate) fn render_tabs(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let tabs = [
        (
            SettingsTab::Appearance,
            "palette.svg",
            "settings.tab_appearance",
        ),
        (SettingsTab::Apps, "grid-dots.svg", "settings.tab_apps"),
        (SettingsTab::Media, "music.svg", "settings.tab_media"),
        (
            SettingsTab::LockScreen,
            "lock.svg",
            "settings.tab_lockscreen",
        ),
        (SettingsTab::System, "settings.svg", "settings.tab_system"),
        (SettingsTab::Record, "play.svg", "settings.tab_record"),
    ];
    div()
        .flex()
        .flex_col()
        .gap(px(6.0))
        .w(px(176.0))
        .h_full()
        .flex_shrink_0()
        .children(tabs.into_iter().map(|(tab, icon, key)| {
            let selected = module.active_tab == tab && module.search_query.trim().is_empty();
            let foreground = if selected {
                theme.foreground()
            } else {
                theme.foreground_muted()
            };
            div()
                .id(("settings-tab", tab as usize))
                .flex()
                .items_center()
                .gap(px(12.0))
                .w_full()
                .h(px(48.0))
                .px(px(14.0))
                .flex_shrink_0()
                .rounded(px(10.0))
                .when(selected, |s| s.bg(style::surface(theme)))
                .text_color(foreground)
                .text_size(px(14.0))
                .font_weight(if selected {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .cursor_pointer()
                .hover(|s| s.bg(style::hover(theme)))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.focus(window, cx);
                    this.set_tab(tab, cx);
                }))
                .child(
                    svg()
                        .path(icon)
                        .size(px(18.0))
                        .flex_shrink_0()
                        .text_color(foreground),
                )
                .child(div().min_w_0().text_ellipsis().child(text(key, cx)))
        }))
}

fn heading(title: &str, theme: &Theme) -> Div {
    div()
        .px(px(18.0))
        .pt(px(8.0))
        .pb(px(0.0))
        .text_size(px(18.0))
        .line_height(px(22.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.foreground())
        .child(title.to_owned())
}

fn group(theme: &Theme) -> Div {
    style::card(theme, style::CARD_RADIUS)
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
}

pub(crate) fn render_switch_row(
    field: SettingsField,
    title: &str,
    description: &str,
    enabled: bool,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    render_toggle_row(
        ("settings-switch", field as usize).into(),
        title,
        Some(description),
        enabled,
        theme,
        cx.listener(move |this, _, window, cx| {
            this.focus(window, cx);
            this.toggle_setting(field, cx);
        }),
    )
}

fn silhouette(concave: bool, width: f32, height: f32, radius: f32, theme: &Theme) -> Div {
    div()
        .relative()
        .w(px(width))
        .h(px(height))
        .flex_shrink_0()
        .when(!concave, |s| {
            s.child(div().size_full().bg(theme.accent()).rounded(px(radius)))
        })
        .when(concave, |s| {
            s.child(style::concave_background(
                gpui::size(px(width), px(height)),
                radius,
                false,
                theme.accent(),
            ))
        })
}

pub(crate) fn render_ui_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let config = cx.global::<AppState>().config.get();
    let dimensions = [
        (
            SettingsField::IdleHeight,
            "settings.height_short",
            &module.idle_height_input,
            16.0,
            60.0,
        ),
        (
            SettingsField::MarginTop,
            "settings.margin_short",
            &module.margin_top_input,
            0.0,
            40.0,
        ),
        (
            SettingsField::Gap,
            "settings.gap_short",
            &module.gap_input,
            0.0,
            32.0,
        ),
    ];
    let radii = [
        (
            SettingsField::CapsuleRound,
            "settings.capsule_radius_short",
            &module.capsule_round_input,
        ),
        (
            SettingsField::SatelliteRound,
            "settings.satellite_radius_short",
            &module.satellite_round_input,
        ),
        (
            SettingsField::CardsRound,
            "settings.cards_radius_short",
            &module.cards_round_input,
        ),
    ];
    let size = module.motion_preview.size();
    let width = 250.0 + (f32::from(size.width) - 72.0) * 1.875;
    let base_height = module
        .idle_height_input
        .parse::<f32>()
        .unwrap_or(40.0)
        .max(40.0);
    let height = base_height + (f32::from(size.height) - 22.0);
    let concave = module.capsule_style == CapsuleStyle::Concave;
    let radius = module.capsule_round_input.parse::<f32>().unwrap_or(24.0);
    let margin = if concave {
        0.0
    } else {
        module.margin_top_input.parse::<f32>().unwrap_or(8.0)
    };
    let gap = module.gap_input.parse::<f32>().unwrap_or(8.0);
    let error = module.appearance_error.clone().or_else(|| {
        cx.global::<crate::settings::appearance::AppearanceStatus>()
            .error
            .clone()
    });
    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .w_full()
        .child(
            div()
                .text_size(px(26.0))
                .line_height(px(32.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child(text("settings.tab_appearance", cx)),
        )
        .child(
            group(theme)
                .child(heading(&text("settings.general_heading", cx), theme))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .child(render_switch_row(
                            SettingsField::DynamicColors,
                            &text("settings.dynamic_colors", cx),
                            &text("settings.dynamic_colors_hint", cx),
                            config.ui.dynamic_colors,
                            theme,
                            cx,
                        ))
                        .child(render_switch_row(
                            SettingsField::DarkMode,
                            &text("settings.dark_mode", cx),
                            &text("settings.color_mode_hint", cx),
                            config.ui.dark_mode,
                            theme,
                            cx,
                        )),
                )
                .when_some(error, |s, error| {
                    s.child(
                        div()
                            .px(px(18.0))
                            .pb(px(12.0))
                            .text_size(px(12.0))
                            .text_color(theme.accent())
                            .child(error),
                    )
                }),
        )
        .child(
            group(theme)
                .child(heading("Capsule", theme))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .child(render_switch_row(
                            SettingsField::CapsuleStyle,
                            &text("settings.concave_style", cx),
                            &text("settings.style_description", cx),
                            concave,
                            theme,
                            cx,
                        ))
                        .child(render_switch_row(
                            SettingsField::ClockFormat,
                            &text("settings.clock_24", cx),
                            &text("settings.clock_format_hint", cx),
                            config.ui.clock_24_hour,
                            theme,
                            cx,
                        ))
                        .child(render_switch_row(
                            SettingsField::LegacyCapsule,
                            &text("settings.old_capsule", cx),
                            &text("settings.restart_hint", cx),
                            !config.ui.use_new_capsule,
                            theme,
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .gap(px(8.0))
                        .px(px(4.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .flex_shrink_0()
                                .min_w_0()
                                .child(heading(&text("settings.dimensions_heading", cx), theme))
                                .children(dimensions.into_iter().map(
                                    |(field, key, value, min, max)| {
                                        render_slider_row(
                                            field,
                                            &text(key, cx),
                                            value,
                                            "px",
                                            min,
                                            max,
                                            1.0,
                                            theme,
                                            cx,
                                        )
                                    },
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .flex_shrink_0()
                                .min_w_0()
                                .child(heading(&text("settings.radii_heading", cx), theme))
                                .children(radii.into_iter().map(|(field, key, value)| {
                                    render_slider_row(
                                        field,
                                        &text(key, cx),
                                        value,
                                        "px",
                                        0.0,
                                        24.0,
                                        1.0,
                                        theme,
                                        cx,
                                    )
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .flex_shrink_0()
                                .min_w_0()
                                .child(heading(&text("settings.motion_heading", cx), theme))
                                .child(render_slider_row(
                                    SettingsField::AnimDuration,
                                    &text("settings.animation_short", cx),
                                    &module.anim_duration_input,
                                    "ms",
                                    0.0,
                                    1000.0,
                                    25.0,
                                    theme,
                                    cx,
                                ))
                                .child(
                                    div()
                                        .px(px(18.0))
                                        .pt(px(8.0))
                                        .text_size(px(12.0))
                                        .text_color(theme.foreground_muted())
                                        .child(text("settings.motion_hint", cx)),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(8.0))
                        .px(px(18.0))
                        .py(px(8.0))
                        .child(
                            div()
                                .relative()
                                .flex()
                                .items_start()
                                .justify_center()
                                .w_full()
                                .h(px((base_height + 20.0 + margin + 16.0).max(82.0)))
                                .rounded(px(10.0))
                                .bg(style::background(theme))
                                .overflow_hidden()
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(8.0))
                                        .w_full()
                                        .h(px(1.0))
                                        .bg(style::border(theme)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(gap))
                                        .mt(px(8.0 + margin))
                                        .child(
                                            div()
                                                .size(px(12.0))
                                                .rounded_full()
                                                .bg(style::raised(theme)),
                                        )
                                        .child(silhouette(concave, width, height, radius, theme))
                                        .child(
                                            div()
                                                .size(px(12.0))
                                                .rounded_full()
                                                .bg(style::raised(theme)),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .id("settings-motion-preview")
                                .flex()
                                .items_center()
                                .gap(px(7.0))
                                .px(px(12.0))
                                .py(px(5.0))
                                .rounded(px(8.0))
                                .text_size(px(12.0))
                                .text_color(theme.foreground_muted())
                                .cursor_pointer()
                                .hover(|s| s.bg(style::hover(theme)))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.play_motion_preview(window, cx)
                                }))
                                .child(svg().path("play.svg").size(px(12.0)))
                                .child(text("settings.preview_motion", cx)),
                        ),
                ),
        )
}
