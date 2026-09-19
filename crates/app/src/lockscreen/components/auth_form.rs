use gpui::{Context, FontWeight, IntoElement, ParentElement, Styled, div, px};
use services::AppState;
use ui::theme::Theme;

pub fn render_auth_form(
    theme: &Theme,
    password_len: usize,
    auth_failed: bool,
    is_checking: bool,
    cx: &mut Context<crate::lockscreen::LockScreen>,
) -> impl IntoElement {
    let (enter_pwd, checking_text, incorrect_text) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("lockscreen.enter_password"),
            lang.get("lockscreen.checking"),
            lang.get("lockscreen.incorrect_password"),
        )
    } else {
        (
            "Contraseña".to_string(),
            "Verificando...".to_string(),
            "Contraseña incorrecta".to_string(),
        )
    };

    let border_color = if auth_failed {
        theme.red()
    } else {
        theme.surface().opacity(0.35)
    };

    let masked_password = "● ".repeat(password_len);

    div().flex().flex_col().items_center().gap(px(6.0)).child(
        div()
            .w(px(200.0))
            .h(px(34.0))
            .px(px(16.0))
            .rounded_full()
            .bg(theme.surface().opacity(0.35))
            .border_1()
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .text_center()
                    .font_family(theme.font_family())
                    .child(if is_checking {
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.accent())
                            .child(checking_text)
                    } else if auth_failed && password_len == 0 {
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.red())
                            .child(incorrect_text)
                    } else if password_len == 0 {
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted().opacity(0.5))
                            .child(enter_pwd)
                    } else {
                        div()
                            .text_size(px(14.0))
                            .text_color(theme.foreground())
                            .child(masked_password)
                    }),
            ),
    )
}
