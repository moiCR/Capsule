use crate::app::{InstallerApp, PendingAction, Step};
use crate::style;
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};
use ui::theme::Theme;

pub fn render_dependencies(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let missing_count = app.missing_dependencies_count();
    let aur_helper_name = app
        .aur_helper
        .as_ref()
        .map(|h| h.name())
        .unwrap_or("pacman");

    let dep_rows = app
        .dependencies
        .chunks(2)
        .enumerate()
        .map(|(row_idx, pair)| {
            let mut row = div()
                .id(("dep-row", row_idx as u32))
                .flex()
                .gap(px(8.0))
                .w_full();

            for (col_idx, (name, installed)) in pair.iter().enumerate() {
                let installed = *installed;
                let icon_color = if installed {
                    theme.green()
                } else {
                    theme.red()
                };
                let status_text = if installed { "Installed" } else { "Missing" };

                let item = div()
                    .id(("dep-item", (row_idx * 2 + col_idx) as u32))
                    .flex()
                    .items_center()
                    .justify_between()
                    .p(px(10.0))
                    .rounded(px(style::INNER_RADIUS))
                    .bg(style::raised(&theme))
                    .border_1()
                    .border_color(style::border(&theme))
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .size(px(24.0))
                                    .rounded_full()
                                    .bg(style::tint(style::surface(&theme), icon_color, 0.15))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        gpui::svg()
                                            .path(if installed { "check.svg" } else { "close.svg" })
                                            .size(px(14.0))
                                            .text_color(icon_color),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.foreground())
                                    .child(name.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(if installed {
                                theme.green()
                            } else {
                                theme.red()
                            })
                            .child(status_text),
                    );

                row = row.child(div().flex_1().child(item));
            }

            if pair.len() == 1 {
                row = row.child(div().flex_1());
            }

            row
        });

    div()
        .flex()
        .flex_col()
        .size_full()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(12.0))
                .flex_1()
                .min_h_0()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(18.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.foreground())
                                .child("System Dependencies"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child("Required packages for Wayland, sound, portals, and shell tools"),
                        ),
                )
                // Dependencies list with track scroll
                .child(
                    div()
                        .id("deps-scroll-view")
                        .track_scroll(&app.deps_scroll_handle)
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .p(px(2.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .children(dep_rows),
                        ),
                )
                .when_some(app.error_message.as_ref(), |this, err| {
                    this.child(
                        div()
                            .p(px(10.0))
                            .rounded(px(style::INNER_RADIUS))
                            .bg(style::tint(style::background(&theme), theme.red(), 0.15))
                            .border_1()
                            .border_color(theme.red())
                            .text_size(px(12.0))
                            .text_color(theme.red())
                            .child(format!("Error: {}", err)),
                    )
                })
                // Status banner / quick action
                .child(
                    if missing_count > 0 {
                        div()
                            .p(px(12.0))
                            .rounded(px(style::INNER_RADIUS))
                            .bg(style::tint(style::surface(&theme), theme.accent(), 0.12))
                            .border_1()
                            .border_color(style::tint(style::border(&theme), theme.accent(), 0.3))
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.foreground())
                                            .child(format!(
                                                "{} packages can be installed automatically via {}",
                                                missing_count, aur_helper_name
                                            )),
                                    ),
                            )
                            .child(
                                style::primary_button(
                                    "install-deps-btn",
                                    "Install Dependencies".to_string(),
                                    true,
                                    &theme,
                                )
                                .on_click(cx.listener(|this, _, _, cx| {
                                    if this.request_auth_or_proceed(PendingAction::InstallDeps) {
                                        this.start_deps_installation(cx);
                                    }
                                })),
                            )
                    } else {
                        div()
                            .p(px(10.0))
                            .rounded(px(style::INNER_RADIUS))
                            .bg(style::tint(style::surface(&theme), theme.green(), 0.1))
                            .border_1()
                            .border_color(style::tint(style::border(&theme), theme.green(), 0.3))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                gpui::svg()
                                    .path("check.svg")
                                    .size(px(16.0))
                                    .text_color(theme.green()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.green())
                                    .child("All required system dependencies are installed and ready!"),
                            )
                    },
                ),
        )
        // Footer navigation
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt(px(16.0))
                .border_t_1()
                .border_color(style::border(&theme))
                .child(
                    style::secondary_button("deps-back-btn", "Back".to_string(), &theme)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.step = Step::Welcome;
                            cx.notify();
                        })),
                )
                .child(
                    style::primary_button(
                        "deps-continue-btn",
                        "Install Capsule".to_string(),
                        true,
                        &theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        if this.request_auth_or_proceed(PendingAction::InstallCapsule) {
                            this.start_installation(cx);
                        }
                    })),
                ),
        )
        .into_any_element()
}
