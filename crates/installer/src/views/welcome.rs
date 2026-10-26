use crate::app::{InstallerApp, Step};
use crate::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
    svg,
};
use ui::theme::Theme;

pub fn render_welcome(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();

    let target_ver = app
        .target_version
        .as_deref()
        .unwrap_or("Checking...")
        .to_string();

    let current_ver = app
        .installed_version
        .as_deref()
        .map(|v| format!("v{}", v))
        .unwrap_or_else(|| "Not installed".to_string());

    let missing_count = app.missing_dependencies_count();
    let deps_status = if missing_count == 0 {
        "All ready".to_string()
    } else {
        format!("{} missing", missing_count)
    };

    let beta_mode = app.beta_mode;

    let card_target_ver = div()
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(theme.surface_container())
        .border_1()
        .border_color(theme.outline_variant().opacity(0.35))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme.foreground_muted())
                        .child("TARGET VERSION"),
                )
                .child(
                    style::chip(
                        "toggle-beta",
                        if beta_mode { "Beta" } else { "Stable" }.to_string(),
                        beta_mode,
                        &theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_beta(cx);
                    })),
                ),
        )
        .child(
            div()
                .text_size(px(17.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .child(target_ver),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child("From GitHub Releases"),
        );

    let card_architecture = div()
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(theme.surface_container())
        .border_1()
        .border_color(theme.outline_variant().opacity(0.35))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child("SYSTEM ARCHITECTURE"),
        )
        .child(
            div()
                .text_size(px(15.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .child(app.target.to_string()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child("Linux GNU Desktop"),
        );

    let card_current_status = div()
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(theme.surface_container())
        .border_1()
        .border_color(theme.outline_variant().opacity(0.35))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child("CURRENT STATUS"),
        )
        .child(
            div()
                .text_size(px(17.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(if app.installed_version.is_some() {
                    theme.accent()
                } else {
                    theme.foreground()
                })
                .child(current_ver),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(if app.capsule_was_running {
                    "Daemon active (will be restarted)"
                } else {
                    "Fresh installation"
                }),
        );

    let card_dependencies = div()
        .p(px(14.0))
        .rounded(px(12.0))
        .bg(theme.surface_container())
        .border_1()
        .border_color(theme.outline_variant().opacity(0.35))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.foreground_muted())
                .child("DEPENDENCIES"),
        )
        .child(
            div()
                .text_size(px(17.0))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(if missing_count == 0 {
                    theme.green()
                } else {
                    theme.red()
                })
                .child(deps_status),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(format!(
                    "{} ({})",
                    app.depends_manager.distro_name(),
                    app.depends_manager.manager_name()
                )),
        );

    div()
        .flex()
        .flex_col()
        .size_full()
        .justify_between()
        .p(px(26.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(18.0))
                // View Header
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(22.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Welcome to Capsule"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child("Modern Dynamic Island & Desktop Shell for Linux Wayland"),
                        ),
                )
                // 2x2 Grid
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .gap(px(12.0))
                                .w_full()
                                .child(div().flex_1().child(card_target_ver))
                                .child(div().flex_1().child(card_architecture)),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(12.0))
                                .w_full()
                                .child(div().flex_1().child(card_current_status))
                                .child(div().flex_1().child(card_dependencies)),
                        ),
                )
                // Highlights Banner Card (eliminates the empty void!)
                .child(
                    div()
                        .p(px(14.0))
                        .rounded(px(12.0))
                        .bg(theme.surface_container_low())
                        .border_1()
                        .border_color(theme.outline_variant().opacity(0.30))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.foreground_muted())
                                .child("INCLUDED FEATURES"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .child(
                                            div()
                                                .size(px(32.0))
                                                .rounded(px(8.0))
                                                .bg(theme.accent().opacity(0.15))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    svg()
                                                        .path("sparkles.svg")
                                                        .size(px(16.0))
                                                        .text_color(theme.accent()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_size(px(12.5))
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(theme.foreground())
                                                        .child("Dynamic Island"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(theme.foreground_muted())
                                                        .child("Media, notifications, shelf"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .child(
                                            div()
                                                .size(px(32.0))
                                                .rounded(px(8.0))
                                                .bg(theme.primary().opacity(0.15))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    svg()
                                                        .path("monitor.svg")
                                                        .size(px(16.0))
                                                        .text_color(theme.primary()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_size(px(12.5))
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(theme.foreground())
                                                        .child("Desktop Bar"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(theme.foreground_muted())
                                                        .child("Dockless tray & calendar"),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.0))
                                        .child(
                                            div()
                                                .size(px(32.0))
                                                .rounded(px(8.0))
                                                .bg(theme.yellow().opacity(0.15))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    svg()
                                                        .path("palette.svg")
                                                        .size(px(16.0))
                                                        .text_color(theme.yellow()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_size(px(12.5))
                                                        .font_weight(gpui::FontWeight::MEDIUM)
                                                        .text_color(theme.foreground())
                                                        .child("Adaptive Theme"),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.5))
                                                        .text_color(theme.foreground_muted())
                                                        .child("Material You GTK & Qt sync"),
                                                ),
                                        ),
                                ),
                        ),
                )
                .when_some(app.error_message.as_ref(), |this, err| {
                    this.child(
                        div()
                            .p(px(12.0))
                            .rounded(px(10.0))
                            .bg(theme.red().opacity(0.12))
                            .border_1()
                            .border_color(theme.red().opacity(0.40))
                            .text_size(px(12.0))
                            .text_color(theme.red())
                            .child(format!("Error: {}", err)),
                    )
                }),
        )
        // Bottom Action buttons
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt(px(16.0))
                .border_t_1()
                .border_color(theme.outline_variant().opacity(0.30))
                .child(
                    style::secondary_button("welcome-cancel-btn", "Cancel".to_string(), &theme)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.cancel_and_exit(cx);
                        })),
                )
                .child(
                    style::primary_button(
                        "welcome-continue-btn",
                        "Check Dependencies & Continue".to_string(),
                        true,
                        &theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.step = Step::Dependencies;
                        cx.notify();
                    })),
                ),
        )
        .into_any_element()
}
