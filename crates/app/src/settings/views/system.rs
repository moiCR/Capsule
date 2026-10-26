use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    AnyElement, App, ColorExt, Context, DurationWithEasing, ElementId, FontWeight,
    InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
    div, ease_in_out, millis, px, svg,
};
use services::{AppState, PowerProfile};
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

pub struct SystemView {
    is_lang_open: bool,
    lang_cell: Rc<Cell<(f32, f32)>>,
}

impl SystemView {
    pub fn new() -> Self {
        Self {
            is_lang_open: false,
            lang_cell: Rc::new(Cell::new((0.0, 0.0))),
        }
    }
}

impl SettingsView for SystemView {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(&mut self, _window: &mut Window, cx: &mut Context<Settings>) -> AnyElement {
        let theme = cx.global::<Theme>().clone();

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

        // 1. Power Profile Selector
        let perf_label = text("power.performance", cx);
        let bal_label = text("power.balanced", cx);
        let saver_label = text("power.power_saver", cx);

        let profiles = [
            (PowerProfile::Performance, perf_label, "zap.svg"),
            (PowerProfile::Balanced, bal_label, "scale.svg"),
            (PowerProfile::PowerSaver, saver_label, "leaf.svg"),
        ];

        let mut power_selector = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_shrink_0()
            .p(px(3.0))
            .gap(px(4.0))
            .rounded_full()
            .bg(theme.surface_container_high())
            .border_1()
            .border_color(theme.outline_variant().opacity(0.35));

        for (prof, label, icon) in profiles {
            let is_active = current_profile == prof;
            let prof_val = prof.clone();

            power_selector = power_selector.child(
                div()
                    .id(ElementId::Name(format!("sys-power-{label}").into()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px(px(14.0))
                    .py(px(5.0))
                    .rounded_full()
                    .bg(if is_active {
                        theme.primary_container()
                    } else {
                        gpui::transparent_black()
                    })
                    .transitions(|t| t.bg(millis(160).with_easing(ease_in_out)))
                    .hover(|s| {
                        if !is_active {
                            s.bg(theme.surface_container_highest())
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
                    .child(
                        svg()
                            .path(icon)
                            .size(px(13.0))
                            .text_color(if is_active {
                                theme.on_primary_container()
                            } else {
                                theme.on_surface_variant()
                            })
                            .transitions(|t| t.text_color(millis(160).with_easing(ease_in_out))),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(if is_active {
                                theme.on_primary_container()
                            } else {
                                theme.foreground()
                            })
                            .transitions(|t| t.text_color(millis(160).with_easing(ease_in_out)))
                            .child(label),
                    ),
            );
        }

        // 2. Language Selector
        let current_lang = languages.iter().find(|l| l.is_current);
        let placeholder = text("settings.select_language", cx);
        let display_name = if let Some(lang) = current_lang {
            lang.name.clone()
        } else {
            placeholder.clone()
        };

        let mut lang_select = Select::new("system-lang-select")
            .placeholder(placeholder)
            .selected_label(display_name)
            .icon(
                svg()
                    .path("languages.svg")
                    .size(px(16.0))
                    .text_color(theme.accent()),
            )
            .is_open(self.is_lang_open)
            .width(220.0)
            .track_bounds(self.lang_cell.clone())
            .on_toggle(cx.listener(|this, _, _, cx| {
                if let Some(view) = this.view_as_mut::<SystemView>() {
                    view.is_lang_open = !view.is_lang_open;
                }
                cx.notify();
            }));

        for lang in &languages {
            let is_selected = lang.is_current;
            let target_file = lang
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&lang.code)
                .to_string();

            let item = SelectItem::new(format!("lang-{}", lang.code), lang.name.clone())
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
                    if cx.has_global::<AppState>() {
                        let app_state = cx.global::<AppState>();
                        let _ = app_state.language.set_language(&target_file);
                        let _ = app_state
                            .config
                            .update(|cfg| cfg.ui.language = target_file.clone());
                    }
                    if let Some(view) = this.view_as_mut::<SystemView>() {
                        view.is_lang_open = false;
                    }
                    cx.notify();
                }));

            lang_select = lang_select.item(item);
        }

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            // Hero Header
            .child(hero_header(
                "settings.svg",
                &text("settings.system_header_title", cx),
                &text("settings.system_header_subtitle", cx),
                &theme,
            ))
            // System Options Card
            .child(
                card(&theme)
                    .child(control_row(
                        &text("settings.power_plan_title", cx),
                        Some(&text("settings.power_plan_subtitle", cx)),
                        power_selector,
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.language_title", cx),
                        Some(&text("settings.language_subtitle", cx)),
                        lang_select.into_any_element(),
                        &theme,
                    )),
            )
            .into_any_element()
    }
}
