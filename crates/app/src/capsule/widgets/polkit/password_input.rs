use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::polkit::PolkitModule;

pub fn render_password_input(
    password: &str,
    is_error: bool,
    is_authenticating: bool,
    placeholder: &str,
    err_text: &str,
    theme: &Theme,
    cx: &mut Context<PolkitModule>,
) -> impl IntoElement {
    let has_password = !password.is_empty();
    let masked_password = "•".repeat(password.len());

    let border_color = if is_error {
        theme.red()
    } else if is_authenticating {
        theme.accent().opacity(0.6)
    } else if has_password {
        theme.accent().opacity(0.4)
    } else {
        theme.surface().opacity(0.3)
    };

    let icon_color = if is_error {
        theme.red()
    } else if is_authenticating || has_password {
        theme.accent()
    } else {
        theme.foreground_muted().opacity(0.7)
    };

    div()
        .flex()
        .items_center()
        .gap_3()
        .w_full()
        .px_3()
        .py_2()
        .rounded_xl()
        .bg(theme.surface().opacity(0.55))
        .border_1()
        .border_color(border_color)
        .child(svg().path("lock.svg").size(px(14.0)).text_color(icon_color))
        .child(div().flex_1().text_sm().child(if is_authenticating {
            div()
                .text_size(px(12.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.accent())
                .child(placeholder.to_string())
        } else if !has_password {
            div()
                .text_size(px(12.5))
                .text_color(if is_error {
                    theme.red().opacity(0.9)
                } else {
                    theme.foreground_muted().opacity(0.5)
                })
                .child(if is_error {
                    err_text.to_string()
                } else {
                    placeholder.to_string()
                })
        } else {
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(13.0))
                .text_color(theme.foreground())
                .child(masked_password)
        }))
        .children(if is_authenticating {
            Some(
                div()
                    .id("polkit-auth-indicator")
                    .flex()
                    .items_center()
                    .gap(px(3.0))
                    .child(div().size(px(4.0)).rounded_full().bg(theme.accent()))
                    .child(
                        div()
                            .size(px(4.0))
                            .rounded_full()
                            .bg(theme.accent().opacity(0.6)),
                    )
                    .child(
                        div()
                            .size(px(4.0))
                            .rounded_full()
                            .bg(theme.accent().opacity(0.3)),
                    ),
            )
        } else if has_password {
            Some(
                div()
                    .id("polkit-submit-arrow")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(22.0))
                    .rounded_full()
                    .bg(theme.accent())
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.85))
                    .active(|s| s.opacity(0.7))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.submit_auth(cx);
                    }))
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(12.0))
                            .text_color(theme.background()),
                    ),
            )
        } else {
            None
        })
}
