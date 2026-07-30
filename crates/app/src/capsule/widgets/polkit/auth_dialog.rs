use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::polkit::PolkitModule;

pub fn render_auth_dialog(
    user_name: &str,
    req_message: &str,
    password: &str,
    is_error: bool,
    error_msg: Option<String>,
    is_authenticating: bool,
    theme: &Theme,
    cx: &mut Context<PolkitModule>,
) -> impl IntoElement {
    let masked_password = "•".repeat(password.len());
    let err_text = error_msg.unwrap_or_else(|| "Contraseña incorrecta. Reintenta...".to_string());

    div()
        .flex()
        .flex_col()
        .w(px(348.0))
        .p_4()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w_7()
                        .h_7()
                        .rounded_lg()
                        .bg(theme.accent())
                        .child(
                            svg()
                                .path("sparkles.svg")
                                .w_4()
                                .h_4()
                                .text_color(theme.background()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child("Autenticación Requerida"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme.foreground_muted())
                                .child(format!("Usuario: {user_name}")),
                        ),
                ),
        )
        .child(
            div()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(theme.surface())
                .text_xs()
                .text_color(theme.foreground())
                .child(req_message.to_string()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .w_full()
                        .px_3p5()
                        .py_2()
                        .rounded_xl()
                        .bg(theme.surface())
                        .border_1()
                        .border_color(if is_error {
                            theme.red_color.to_hsla()
                        } else {
                            theme.background_alt()
                        })
                        .child(div().flex_1().text_sm().child(if is_authenticating {
                            div()
                                .text_color(theme.foreground_muted())
                                .child("Verificando contraseña...")
                        } else if password.is_empty() {
                            div()
                                .text_color(if is_error {
                                    theme.red_color.to_hsla()
                                } else {
                                    theme.foreground_muted()
                                })
                                .child(if is_error {
                                    err_text.clone()
                                } else {
                                    "Escribe tu contraseña...".to_string()
                                })
                        } else {
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child(masked_password)
                        })),
                )
                .when(is_error && !password.is_empty(), |el| {
                    el.child(
                        div()
                            .px_1()
                            .text_xs()
                            .text_color(theme.red_color.to_hsla())
                            .child(err_text.clone()),
                    )
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_2()
                .pt_1()
                .child(
                    div()
                        .id("polkit-cancel-btn")
                        .px_3()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme.background_alt())
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.cancel(cx);
                        }))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground_muted())
                                .child("Cancelar"),
                        ),
                )
                .child(
                    div()
                        .id("polkit-submit-btn")
                        .px_3p5()
                        .py_1p5()
                        .rounded_lg()
                        .bg(if is_authenticating {
                            theme.background_alt()
                        } else {
                            theme.accent()
                        })
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.submit_auth(cx);
                        }))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_authenticating {
                                    theme.foreground_muted()
                                } else {
                                    theme.background()
                                })
                                .child(if is_authenticating {
                                    "Verificando..."
                                } else {
                                    "Autenticar"
                                }),
                        ),
                ),
        )
}
