use gpui::{
    Context, Div, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px, svg,
};
use services::{AppState, CapsuleStyle};
use ui::theme::Theme;

use super::style;
use crate::capsule::modules::settings::{SettingsField, SettingsModule, SettingsTab};
use crate::capsule::widgets::settings::setting_item::{render_slider_row, render_toggle_row};

pub(crate) const WIDTH: f32 = 840.0;
pub(crate) const HEIGHT: f32 = 560.0;

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
        .w(px(380.0))
        .h(px(36.0))
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
                .text_size(px(12.0))
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
        .h(px(40.0))
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
            SettingsTab::Capsule,
            "dashboard.svg",
            "settings.tab_capsule",
        ),
        (SettingsTab::Apps, "sparkles.svg", "settings.tab_apps"),
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
        .gap(px(8.0))
        .w_full()
        .h(px(36.0))
        .flex_shrink_0()
        .children(tabs.into_iter().map(|(tab, icon, key)| {
            let selected = module.active_tab == tab && module.search_query.trim().is_empty();
            let foreground = if selected {
                style::on_accent(theme)
            } else {
                theme.foreground()
            };
            div()
                .id(("settings-tab", tab as usize))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.0))
                .flex_1()
                .min_w_0()
                .h_full()
                .rounded(px(style::INNER_RADIUS))
                .bg(if selected {
                    theme.accent()
                } else {
                    style::surface(theme)
                })
                .text_color(foreground)
                .text_size(px(12.0))
                .cursor_pointer()
                .hover(|s| {
                    s.bg(if selected {
                        theme.accent()
                    } else {
                        style::hover(theme)
                    })
                })
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.focus(window, cx);
                    this.set_tab(tab, cx);
                }))
                .child(
                    svg()
                        .path(icon)
                        .size(px(15.0))
                        .flex_shrink_0()
                        .text_color(foreground),
                )
                .child(div().min_w_0().text_ellipsis().child(text(key, cx)))
        }))
}

fn heading(title: &str, subtitle: &str, theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(3.0))
        .px(px(16.0))
        .pt(px(14.0))
        .pb(px(6.0))
        .child(
            div()
                .text_size(px(16.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title.to_owned()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(subtitle.to_owned()),
        )
}

fn group(theme: &Theme) -> Div {
    style::card(theme, style::CARD_RADIUS)
        .flex()
        .flex_col()
        .min_w_0()
        .overflow_hidden()
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

fn style_option(
    value: CapsuleStyle,
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let selected = value == module.capsule_style;
    let concave = value == CapsuleStyle::Concave;
    let radius = module.capsule_round_input.parse::<f32>().unwrap_or(24.0) * 0.5;
    div()
        .id(("settings-style", usize::from(concave)))
        .relative()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .flex_1()
        .min_w_0()
        .h(px(84.0))
        .px(px(12.0))
        .pt(px(10.0))
        .rounded(px(style::INNER_RADIUS))
        .bg(style::raised(theme))
        .border_1()
        .border_color(if selected {
            theme.accent()
        } else {
            style::raised(theme)
        })
        .cursor_pointer()
        .hover(|s| s.bg(style::hover(theme)))
        .on_click(cx.listener(move |this, _, _, cx| this.set_capsule_style(value, cx)))
        .child(
            div()
                .relative()
                .flex()
                .items_start()
                .justify_center()
                .w_full()
                .h(px(37.0))
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .w_full()
                        .h(px(1.0))
                        .bg(theme.foreground_muted()),
                )
                .child(
                    div()
                        .mt(px(if concave { 0.0 } else { 9.0 }))
                        .child(silhouette(concave, 78.0, 25.0, radius, theme)),
                ),
        )
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .child(text(
                    if concave {
                        "settings.capsule_style_concave"
                    } else {
                        "settings.capsule_style_normal"
                    },
                    cx,
                )),
        )
        .child(
            div()
                .absolute()
                .right(px(7.0))
                .bottom(px(7.0))
                .child(style::radio(selected, 13.0, theme)),
        )
}

pub(crate) fn render_ui_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
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
            48.0,
        ),
        (
            SettingsField::SatelliteRound,
            "settings.satellite_radius_short",
            &module.satellite_round_input,
            24.0,
        ),
        (
            SettingsField::CardsRound,
            "settings.cards_radius_short",
            &module.cards_round_input,
            24.0,
        ),
    ];
    let size = module.motion_preview.size();
    let width: f32 = size.width.into();
    let height: f32 = size.height.into();
    let concave = module.capsule_style == CapsuleStyle::Concave;
    let radius = module.capsule_round_input.parse::<f32>().unwrap_or(24.0) * 0.5;
    div()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .w_full()
        .child(
            group(theme)
                .h(px(156.0))
                .flex_shrink_0()
                .child(heading(
                    &text("settings.style_heading", cx),
                    &text("settings.style_description", cx),
                    theme,
                ))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .px(px(16.0))
                        .pb(px(14.0))
                        .child(style_option(CapsuleStyle::Normal, module, theme, cx))
                        .child(style_option(CapsuleStyle::Concave, module, theme, cx))
                        .child(div().w(px(246.0)).flex_shrink_0().child(render_toggle_row(
                            "new-capsule-toggle".into(),
                            &text("settings.new_capsule_short", cx),
                            Some(&text("settings.restart_hint", cx)),
                            module.use_new_capsule,
                            theme,
                            cx.listener(|this, _, _, cx| this.toggle_new_capsule(cx)),
                        ))),
                ),
        )
        .child(
            div()
                .flex()
                .items_stretch()
                .gap(px(10.0))
                .w_full()
                .h(px(254.0))
                .flex_shrink_0()
                .child(
                    group(theme)
                        .flex_1()
                        .child(heading(
                            &text("settings.dimensions_heading", cx),
                            &text("settings.dimensions_description", cx),
                            theme,
                        ))
                        .children(dimensions.into_iter().map(|(field, key, value, min, max)| {
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
                        })),
                )
                .child(
                    group(theme)
                        .flex_1()
                        .child(heading(
                            &text("settings.radii_heading", cx),
                            &text("settings.radii_description", cx),
                            theme,
                        ))
                        .children(radii.into_iter().map(|(field, key, value, max)| {
                            render_slider_row(
                                field,
                                &text(key, cx),
                                value,
                                "px",
                                0.0,
                                max,
                                1.0,
                                theme,
                                cx,
                            )
                        })),
                )
                .child(
                    group(theme)
                        .flex_1()
                        .child(heading(
                            &text("settings.motion_heading", cx),
                            &text("settings.motion_description", cx),
                            theme,
                        ))
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
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .px(px(16.0))
                                .pt(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_start()
                                        .justify_center()
                                        .relative()
                                        .w_full()
                                        .h(px(65.0))
                                        .rounded(px(style::INNER_RADIUS))
                                        .bg(style::raised(theme))
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
                                            div().mt(px(if concave { 8.0 } else { 18.0 })).child(
                                                silhouette(concave, width, height, radius, theme),
                                            ),
                                        ),
                                )
                                .child(
                                    style::primary_button(
                                        "settings-motion-preview",
                                        text("settings.preview_motion", cx),
                                        true,
                                        theme,
                                    )
                                    .on_click(cx.listener(
                                        |this, _, window, cx| this.play_motion_preview(window, cx),
                                    )),
                                ),
                        ),
                ),
        )
}
