use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::polkit::PolkitModule;

pub fn render_auth_header(
    user_name: &str,
    title: &str,
    theme: &Theme,
    cx: &mut Context<PolkitModule>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    svg()
                        .path("lock.svg")
                        .size(px(14.0))
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.foreground_muted().opacity(0.85))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .px_2()
                        .py(px(1.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(0.55))
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground().opacity(0.8))
                                .child(user_name.to_string()),
                        ),
                ),
        )
        .child(
            div()
                .id("polkit-close-btn")
                .flex()
                .items_center()
                .justify_center()
                .w(px(20.0))
                .h(px(20.0))
                .rounded_full()
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.opacity(0.6))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.cancel(cx);
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
}

pub fn render_auth_message(message: &str, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .px_3()
        .py_2()
        .rounded_xl()
        .bg(theme.surface().opacity(0.35))
        .max_h(px(52.0))
        .overflow_hidden()
        .child(
            div()
                .text_size(px(11.5))
                .line_height(px(16.0))
                .text_color(theme.foreground_muted())
                .child(message.to_string()),
        )
}

pub fn render_auth_footer(
    has_password: bool,
    is_authenticating: bool,
    err_msg: Option<&str>,
    theme: &Theme,
    cx: &mut Context<PolkitModule>,
) -> impl IntoElement {
    let (cancel_text, auth_text) = if cx.has_global::<services::AppState>() {
        let lang = &cx.global::<services::AppState>().language;
        (lang.get("common.cancel"), lang.get("common.authenticate"))
    } else {
        ("Cancelar".to_string(), "Autenticar".to_string())
    };

    let error_display = err_msg.map(|e| {
        div()
            .text_size(px(11.0))
            .text_color(theme.red())
            .truncate()
            .child(e.to_string())
    });

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .px_1()
        .child(div().flex_1().mr_2().children(error_display))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .id("polkit-cancel-hint")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground_muted().opacity(0.6))
                        .cursor_pointer()
                        .hover(|s| s.text_color(theme.foreground()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.cancel(cx);
                        }))
                        .child(format!("esc {cancel_text}")),
                )
                .child(
                    div()
                        .id("polkit-auth-hint")
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if is_authenticating || !has_password {
                            theme.foreground_muted().opacity(0.4)
                        } else {
                            theme.accent()
                        })
                        .cursor_pointer()
                        .when(has_password && !is_authenticating, |s| {
                            s.hover(|h| h.opacity(0.85))
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.submit_auth(cx);
                        }))
                        .child(format!("↵ {auth_text}")),
                ),
        )
}
