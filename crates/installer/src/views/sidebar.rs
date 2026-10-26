use crate::app::{InstallerApp, Step};
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, div, prelude::*, px, svg,
};
use ui::theme::Theme;

pub fn render_sidebar(app: &InstallerApp, cx: &mut Context<InstallerApp>) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let current_step = app.step;

    let steps = [
        (Step::Welcome, "Overview", "System check"),
        (Step::Dependencies, "Dependencies", "Required tools"),
        (Step::Compositor, "Compositor", "Desktop setup"),
        (Step::Installing, "Installation", "Extract & install"),
        (Step::Finished, "Finished", "Ready to use"),
    ];

    let step_items = steps.into_iter().enumerate().map(|(i, (step, title, subtitle))| {
        let is_active = step == current_step;
        let is_completed = matches!(
            (current_step, step),
            (Step::Dependencies, Step::Welcome)
                | (Step::Compositor, Step::Welcome | Step::Dependencies)
                | (
                    Step::Installing,
                    Step::Welcome | Step::Dependencies | Step::Compositor
                )
                | (Step::Finished, _)
        );

        let can_navigate = is_completed
            && current_step != Step::Installing
            && current_step != Step::Finished;

        let mut item = div()
            .id(("sidebar-step", i as u32))
            .w_full()
            .p(px(10.0))
            .rounded(px(10.0))
            .flex()
            .items_center()
            .gap(px(12.0))
            .transitions(|t| t.bg(gpui::millis(150)));

        if is_active {
            item = item
                .bg(theme.surface_container_high())
                .border_1()
                .border_color(theme.outline_variant().opacity(0.40));
        } else if can_navigate {
            item = item
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface_container().opacity(0.60)))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.step = step;
                    cx.notify();
                }));
        }

        // Left indicator circle
        let indicator = div()
            .size(px(24.0))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(11.0))
            .font_weight(gpui::FontWeight::BOLD)
            .when(is_completed, |s| {
                s.bg(theme.accent().opacity(0.20))
                    .text_color(theme.accent())
                    .child("✓")
            })
            .when(is_active && !is_completed, |s| {
                s.bg(theme.primary())
                    .text_color(theme.on_primary())
                    .child(format!("{}", i + 1))
            })
            .when(!is_active && !is_completed, |s| {
                s.bg(theme.surface_container_highest())
                    .text_color(theme.foreground_muted())
                    .child(format!("{}", i + 1))
            });

        // Label and description
        let text_block = div()
            .flex()
            .flex_col()
            .gap(px(1.0))
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(if is_active {
                        gpui::FontWeight::SEMIBOLD
                    } else if is_completed {
                        gpui::FontWeight::MEDIUM
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(if is_active || is_completed {
                        theme.foreground()
                    } else {
                        theme.foreground_muted()
                    })
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(if is_active {
                        theme.primary()
                    } else if is_completed {
                        theme.accent().opacity(0.85)
                    } else {
                        theme.foreground_muted().opacity(0.65)
                    })
                    .child(if is_completed { "Completed" } else { subtitle }),
            );

        item.child(indicator).child(text_block)
    });

    let distro = app.depends_manager.distro_name();
    let version = app.target_version.as_deref().unwrap_or("2.4.0");

    div()
        .w(px(230.0))
        .h_full()
        .flex_shrink_0()
        .bg(theme.surface_container_low())
        .border_r_1()
        .border_color(theme.outline_variant().opacity(0.35))
        .p(px(20.0))
        .flex()
        .flex_col()
        .justify_between()
        // Top branding
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(22.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .child(
                            div()
                                .size(px(38.0))
                                .rounded(px(10.0))
                                .bg(theme.primary().opacity(0.12))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .path("package.svg")
                                        .size(px(20.0))
                                        .text_color(theme.primary()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(15.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.foreground())
                                        .child("Capsule"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(theme.foreground_muted())
                                        .child("Installation Setup"),
                                ),
                        ),
                )
                // Step items list
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .children(step_items),
                ),
        )
        // Bottom System status info
        .child(
            div()
                .p(px(12.0))
                .rounded(px(10.0))
                .bg(theme.surface_container())
                .border_1()
                .border_color(theme.outline_variant().opacity(0.30))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground_muted())
                                .child("SYSTEM"),
                        )
                        .child(
                            div()
                                .px(px(6.0))
                                .py(px(1.0))
                                .rounded_full()
                                .bg(if app.beta_mode {
                                    theme.yellow().opacity(0.20)
                                } else {
                                    theme.green().opacity(0.20)
                                })
                                .text_size(px(9.5))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(if app.beta_mode {
                                    theme.yellow()
                                } else {
                                    theme.green()
                                })
                                .child(if app.beta_mode { "Beta" } else { "Stable" }),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            svg()
                                .path("monitor.svg")
                                .size(px(13.0))
                                .text_color(theme.foreground_muted()),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(distro.to_string()),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(theme.foreground_muted())
                        .child(format!("Target: v{}", version)),
                ),
        )
        .into_any_element()
}
