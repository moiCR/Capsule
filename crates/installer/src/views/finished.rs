use crate::app::InstallerApp;
use crate::style;
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};
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
        .unwrap_or("2.3.0");

    let launch_on_finish = app.launch_on_finish;

    div()
        .flex()
        .flex_col()
        .size_full()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .flex_1()
                .child(style::check_badge(52.0, &theme))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(20.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Installation Complete!"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child(format!(
                                    "Capsule v{} was successfully installed on your system.",
                                    installed_ver
                                )),
                        ),
                )
                // Integration details
                .child(
                    div()
                        .w_full()
                        .max_w(px(480.0))
                        .p(px(14.0))
                        .rounded(px(style::INNER_RADIUS))
                        .bg(style::raised(&theme))
                        .border_1()
                        .border_color(style::border(&theme))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
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
                                        .text_size(px(12.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child("/usr/local/bin/capsule"),
                                ),
                        )
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
                                        .text_size(px(12.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child("/usr/share/applications/capsule.desktop"),
                                ),
                        )
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
                                        .text_size(px(12.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.accent())
                                        .child("capsule"),
                                ),
                        ),
                )
                // Launch on finish toggle
                .child(
                    div()
                        .id("launch-toggle-card")
                        .w_full()
                        .max_w(px(480.0))
                        .p(px(12.0))
                        .rounded(px(style::INNER_RADIUS))
                        .bg(style::raised(&theme))
                        .border_1()
                        .border_color(if launch_on_finish {
                            style::tint(style::border(&theme), theme.accent(), 0.4)
                        } else {
                            style::border(&theme)
                        })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_between()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.launch_on_finish = !this.launch_on_finish;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .size(px(20.0))
                                        .rounded(px(6.0))
                                        .border_1()
                                        .border_color(if launch_on_finish {
                                            theme.accent()
                                        } else {
                                            style::border(&theme)
                                        })
                                        .bg(if launch_on_finish {
                                            theme.accent()
                                        } else {
                                            style::background(&theme)
                                        })
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            gpui::svg()
                                                .path("check.svg")
                                                .size(px(12.0))
                                                .text_color(if launch_on_finish {
                                                    style::on_accent(&theme)
                                                } else {
                                                    gpui::transparent_black()
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground())
                                        .child("Start Capsule daemon now"),
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
                .border_color(style::border(&theme))
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
