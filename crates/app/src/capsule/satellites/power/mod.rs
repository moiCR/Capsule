use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use services::{AppState, PowerProfile};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn render_power_widget(
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let current_profile = if cx.has_global::<AppState>() {
        cx.global::<AppState>().power.get_active_profile()
    } else {
        PowerProfile::Balanced
    };

    let profiles = [
        (PowerProfile::Performance, lang.power.performance.clone(), "zap.svg", "power-profile-perf"),
        (PowerProfile::Balanced, lang.power.balanced.clone(), "scale.svg", "power-profile-bal"),
        (PowerProfile::PowerSaver, lang.power.power_saver.clone(), "leaf.svg", "power-profile-saver"),
    ];

    let mut profile_rows = div()
        .flex()
        .flex_col()
        .gap_1p5()
        .w_full();

    for (profile_val, label, icon_file, element_id) in profiles {
        let is_selected = current_profile == profile_val;
        let prof_clone = profile_val.clone();

        let row = div()
            .id(element_id)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_3()
            .py_2()
            .rounded_xl()
            .cursor_pointer()
            .bg(if is_selected {
                theme.accent().opacity(0.2)
            } else {
                theme.surface().opacity(0.4)
            })
            .border_1()
            .border_color(if is_selected {
                theme.accent()
            } else {
                theme.surface().opacity(0.0)
            })
            .hover(|s| {
                if !is_selected {
                    s.bg(theme.surface().opacity(0.7))
                } else {
                    s
                }
            })
            .on_click(cx.listener(move |_this, _, _, cx| {
                if cx.has_global::<AppState>() {
                    cx.global::<AppState>().power.set_active_profile(prof_clone.clone());
                }
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        svg()
                            .path(icon_file)
                            .size(px(14.0))
                            .text_color(if is_selected {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            }),
                    )
                    .child(
                        div()
                            .font_weight(if is_selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_size(px(12.0))
                            .text_color(if is_selected {
                                theme.foreground()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(label),
                    ),
            );

        profile_rows = profile_rows.child(row);
    }

    div()
        .flex()
        .flex_col()
        .w(px(200.0))
        .p_3()
        .gap_2()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .px_1()
                .child(lang.power.title),
        )
        .child(profile_rows)
}
