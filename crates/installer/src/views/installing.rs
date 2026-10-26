use crate::app::InstallerApp;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px,
};
use ui::theme::Theme;

pub fn render_installing(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let pct = (app.progress.clamp(0.0, 1.0) * 100.0).round() as u32;

    let log_lines = app.logs.iter().enumerate().map(|(idx, line)| {
        div()
            .id(("log-line", idx as u32))
            .text_size(px(11.5))
            .line_height(px(17.0))
            .font_family(theme.font_family())
            .text_color(if line.starts_with("==>") || line.starts_with('[') {
                theme.primary()
            } else if line.contains("Error") || line.contains("error") {
                theme.red()
            } else {
                theme.foreground_muted()
            })
            .child(line.clone())
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
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(22.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.foreground())
                                        .child("Installing Capsule"),
                                )
                                .child(
                                    div()
                                        .text_size(px(18.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.primary())
                                        .child(format!("{}%", pct)),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.foreground_muted())
                                .child(app.status_message.clone()),
                        ),
                )
                // Progress Bar
                .child(
                    div()
                        .h(px(8.0))
                        .w_full()
                        .rounded_full()
                        .bg(theme.surface_container_highest())
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(app.progress.clamp(0.0, 1.0)))
                                .rounded_full()
                                .bg(theme.primary()),
                        ),
                )
                // Terminal Activity Log
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .rounded(px(12.0))
                        .bg(theme.surface_container_lowest())
                        .border_1()
                        .border_color(theme.outline_variant().opacity(0.35))
                        .overflow_hidden()
                        // Terminal titlebar
                        .child(
                            div()
                                .px(px(12.0))
                                .py(px(8.0))
                                .bg(theme.surface_container_low())
                                .border_b_1()
                                .border_color(theme.outline_variant().opacity(0.25))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(div().size(px(8.0)).rounded_full().bg(theme.red().opacity(0.70)))
                                        .child(div().size(px(8.0)).rounded_full().bg(theme.yellow().opacity(0.70)))
                                        .child(div().size(px(8.0)).rounded_full().bg(theme.green().opacity(0.70))),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.foreground_muted())
                                        .child("Terminal Output"),
                                )
                                .child(div().w(px(24.0))),
                        )
                        // Terminal body
                        .child(
                            div()
                                .id("logs-scroll-view")
                                .track_scroll(&app.logs_scroll_handle)
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p(px(12.0))
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .children(log_lines),
                        ),
                ),
        )
        // Footer message
        .child(
            div()
                .pt(px(16.0))
                .border_t_1()
                .border_color(theme.outline_variant().opacity(0.30))
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child("Installation in progress, please do not close this window."),
                ),
        )
        .into_any_element()
}
