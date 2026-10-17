use crate::app::InstallerApp;
use crate::style;
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};
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
            .text_size(px(11.0))
            .line_height(px(16.0))
            .text_color(if line.starts_with("==>") || line.starts_with('[') {
                theme.accent()
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
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(16.0))
                .flex_1()
                .min_h_0()
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
                                        .text_size(px(18.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground())
                                        .child("Installing Capsule"),
                                )
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme.accent())
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
                        .bg(style::raised(&theme))
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(app.progress.clamp(0.0, 1.0)))
                                .rounded_full()
                                .bg(theme.accent()),
                        ),
                )
                // Terminal Activity Log
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .p(px(12.0))
                        .rounded(px(style::INNER_RADIUS))
                        .bg(gpui::Rgba::new(0.05, 0.05, 0.07, 1.0))
                        .border_1()
                        .border_color(style::border(&theme))
                        .child(
                            div()
                                .id("logs-scroll-view")
                                .track_scroll(&app.logs_scroll_handle)
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .children(log_lines),
                        ),
                ),
        )
        .child(
            div()
                .pt(px(14.0))
                .border_t_1()
                .border_color(style::border(&theme))
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
