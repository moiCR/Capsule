use crate::app::{InstallerApp, PendingAction, Step};
use crate::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
    svg,
};
use ui::theme::Theme;

pub fn render_dependencies(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let missing_count = app.missing_dependencies_count();
    let manager_name = app.depends_manager.manager_name();

    let dep_rows = app
        .dependencies
        .chunks(2)
        .enumerate()
        .map(|(row_idx, pair)| {
            let mut row = div()
                .id(("dep-row", row_idx as u32))
                .flex()
                .gap(px(10.0))
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
                    .rounded(px(10.0))
                    .bg(theme.surface_container())
                    .border_1()
                    .border_color(theme.outline_variant().opacity(0.30))
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
                                    .bg(icon_color.opacity(0.15))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        svg()
                                            .path(if installed { "check.svg" } else { "close.svg" })
                                            .size(px(13.0))
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
                            .px(px(8.0))
                            .py(px(2.0))
                            .rounded_full()
                            .bg(icon_color.opacity(0.12))
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(icon_color)
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
        .p(px(26.0))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .flex_1()
                .min_h_0()
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
                                .child("System Dependencies"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child(format!(
                                    "Package requirements for {} via {}.",
                                    app.depends_manager.distro_name(),
                                    manager_name
                                )),
                        ),
                )
                // Dependencies list container
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
                            .p(px(12.0))
                            .rounded(px(10.0))
                            .bg(theme.red().opacity(0.12))
                            .border_1()
                            .border_color(theme.red().opacity(0.40))
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
                            .rounded(px(12.0))
                            .bg(theme.primary().opacity(0.10))
                            .border_1()
                            .border_color(theme.primary().opacity(0.30))
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
                                                missing_count, manager_name
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
                            .p(px(12.0))
                            .rounded(px(12.0))
                            .bg(theme.green().opacity(0.10))
                            .border_1()
                            .border_color(theme.green().opacity(0.30))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                svg()
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
                .border_color(theme.outline_variant().opacity(0.30))
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
                        "Choose Compositor".to_string(),
                        true,
                        &theme,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.scan_compositors();
                        this.step = Step::Compositor;
                        this.error_message = None;
                        cx.notify();
                    })),
                ),
        )
        .into_any_element()
}
