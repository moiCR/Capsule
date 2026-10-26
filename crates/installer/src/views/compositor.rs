use crate::app::{InstallerApp, PendingAction, Step};
use crate::compositor::Compositor;
use crate::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
    svg,
};
use ui::theme::Theme;

pub fn render_compositor(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let selected = app.selected_compositor;
    let selected_installed = app.is_compositor_installed(selected);
    let distro_name = app.depends_manager.distro_name().to_string();

    let cards = app
        .compositors
        .iter()
        .enumerate()
        .map(|(idx, (comp, installed))| {
            let comp = *comp;
            let installed = *installed;
            let is_selected = comp == selected;

            let border_color = if is_selected {
                theme.primary()
            } else {
                theme.outline_variant().opacity(0.35)
            };

            let card_bg = if is_selected {
                theme.surface_container_high()
            } else {
                theme.surface_container()
            };

            let icon_path = match comp {
                Compositor::Hyprland => "sparkles.svg",
                Compositor::Niri => "monitor.svg",
                Compositor::MangoWm => "grid-dots.svg",
                Compositor::KWin => "monitor.svg",
            };

            let status_color = if installed {
                theme.green()
            } else if is_selected {
                theme.accent()
            } else {
                theme.foreground_muted()
            };

            let status_label = if installed {
                "Installed"
            } else {
                "Not installed"
            };

            div()
                .id(("comp-card", idx as u32))
                .cursor_pointer()
                .p(px(14.0))
                .rounded(px(12.0))
                .bg(card_bg)
                .border_1()
                .border_color(border_color)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(12.0))
                .transitions(|t| t.bg(gpui::millis(150)))
                .hover(|s| s.bg(theme.surface_container_highest()))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected_compositor = comp;
                    this.error_message = None;
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .flex_1()
                        .child(
                            div()
                                .size(px(38.0))
                                .rounded(px(10.0))
                                .bg(if is_selected {
                                    theme.primary().opacity(0.15)
                                } else {
                                    theme.surface_container_highest()
                                })
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg().path(icon_path).size(px(20.0)).text_color(
                                        if is_selected {
                                            theme.primary()
                                        } else {
                                            theme.foreground_muted()
                                        },
                                    ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .flex_1()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .text_size(px(14.0))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme.foreground())
                                                .child(comp.name()),
                                        )
                                        .when(is_selected, |this| {
                                            this.child(
                                                div()
                                                    .px(px(6.0))
                                                    .py(px(1.0))
                                                    .rounded_full()
                                                    .bg(theme.primary().opacity(0.18))
                                                    .text_size(px(10.0))
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(theme.primary())
                                                    .child("Selected"),
                                            )
                                        }),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(theme.foreground_muted())
                                        .child(comp.subtitle()),
                                ),
                        ),
                )
                .child(
                    div()
                        .px(px(10.0))
                        .py(px(3.0))
                        .rounded_full()
                        .bg(status_color.opacity(0.12))
                        .text_size(px(11.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(status_color)
                        .child(status_label),
                )
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
                .gap(px(14.0))
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
                                .child("Wayland Compositor"),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child("Choose your preferred Wayland desktop environment for Capsule integration."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .children(cards),
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
                .child(if !selected_installed {
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
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(format!(
                                    "{} is not installed yet on {}",
                                    selected.name(),
                                    distro_name
                                )),
                        )
                        .child(
                            style::primary_button(
                                "install-compositor-btn",
                                format!("Install {}", selected.name()),
                                true,
                                &theme,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.request_auth_or_proceed(PendingAction::InstallCompositor) {
                                    this.start_compositor_installation(cx);
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
                                .child(format!(
                                    "{} is installed and ready on this system!",
                                    selected.name()
                                )),
                        )
                }),
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
                    style::secondary_button("comp-back-btn", "Back".to_string(), &theme).on_click(
                        cx.listener(|this, _, _, cx| {
                            this.step = Step::Dependencies;
                            this.error_message = None;
                            cx.notify();
                        }),
                    ),
                )
                .child(
                    style::primary_button(
                        "comp-continue-btn",
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
