use crate::app::InstallerApp;
use crate::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
    svg,
};
use ui::theme::Theme;

pub fn render_finished(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let installed_ver = app
        .installed_version
        .as_deref()
        .or(app.target_version.as_deref())
        .unwrap_or("2.4.0");

    let launch_on_finish = app.launch_on_finish;

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
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .flex_1()
                // Radiant celebration badge
                .child(
                    div()
                        .size(px(64.0))
                        .rounded_full()
                        .bg(theme.green().opacity(0.15))
                        .border_2()
                        .border_color(theme.green().opacity(0.40))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .path("check.svg")
                                .size(px(32.0))
                                .text_color(theme.green()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(22.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Installation Complete!"),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .text_color(theme.foreground_muted())
                                .child(format!(
                                    "Capsule v{} was successfully installed on your system.",
                                    installed_ver
                                )),
                        ),
                )
                // Integration details card
                .child(
                    div()
                        .w_full()
                        .max_w(px(500.0))
                        .p(px(16.0))
                        .rounded(px(12.0))
                        .bg(theme.surface_container())
                        .border_1()
                        .border_color(theme.outline_variant().opacity(0.35))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.foreground_muted())
                                        .child("Binary Path"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child("/usr/local/bin/capsule"),
                                ),
                        )
                        .child(div().h(px(1.0)).bg(theme.outline_variant().opacity(0.20)))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.foreground_muted())
                                        .child("Desktop Entry"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child("/usr/share/applications/capsule.desktop"),
                                ),
                        )
                        .child(div().h(px(1.0)).bg(theme.outline_variant().opacity(0.20)))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.foreground_muted())
                                        .child("Run Command"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.primary())
                                        .child("capsule"),
                                ),
                        ),
                )
                // Launch on finish toggle
                .child(
                    div()
                        .id("launch-toggle-card")
                        .w_full()
                        .max_w(px(500.0))
                        .p(px(14.0))
                        .rounded(px(12.0))
                        .bg(if launch_on_finish {
                            theme.surface_container_high()
                        } else {
                            theme.surface_container()
                        })
                        .border_1()
                        .border_color(if launch_on_finish {
                            theme.primary().opacity(0.50)
                        } else {
                            theme.outline_variant().opacity(0.35)
                        })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .transitions(|t| t.bg(gpui::millis(150)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.launch_on_finish = !this.launch_on_finish;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .size(px(22.0))
                                        .rounded(px(6.0))
                                        .border_1()
                                        .border_color(if launch_on_finish {
                                            theme.primary()
                                        } else {
                                            theme.outline_variant()
                                        })
                                        .bg(if launch_on_finish {
                                            theme.primary()
                                        } else {
                                            theme.surface_container_lowest()
                                        })
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            svg()
                                                .path("check.svg")
                                                .size(px(13.0))
                                                .text_color(if launch_on_finish {
                                                    theme.on_primary()
                                                } else {
                                                    gpui::transparent_black()
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(13.5))
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(theme.foreground())
                                                .child("Start Capsule daemon now"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .text_color(theme.foreground_muted())
                                                .child("Launches the dynamic island and status bar on exit"),
                                        ),
                                ),
                        ),
                ),
        )
        // Footer action
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .pt(px(16.0))
                .border_t_1()
                .border_color(theme.outline_variant().opacity(0.30))
                .child(
                    style::primary_button(
                        "finish-exit-btn",
                        if launch_on_finish {
                            "Launch & Exit".to_string()
                        } else {
                            "Finish".to_string()
                        },
                        true,
                        &theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.finish_and_exit(cx);
                    })),
                ),
        )
        .into_any_element()
}
