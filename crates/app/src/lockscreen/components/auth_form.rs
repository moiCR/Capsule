use gpui::{ColorExt, Context, IntoElement};
use services::AppState;
use ui::components::{Input, InputState};
use ui::theme::Theme;

pub fn render_auth_form(
    theme: &Theme,
    password_input: &InputState,
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

    let placeholder = if is_checking {
        checking_text
    } else if auth_failed && password_input.is_empty() {
        incorrect_text
    } else {
        enter_pwd
    };

    Input::from_state("lockscreen-password-input", password_input)
        .placeholder(placeholder)
        .masked(true)
        .focused(true)
        .cursor_visible(false)
        .error(auth_failed)
        .centered(true)
        .w_px(240.0)
        .max_w_px(240.0)
        .h_px(40.0)
        .rounded_full()
        .bg(theme.surface_container_high().opacity(0.75))
        .border_color(if auth_failed {
            theme.red()
        } else {
            theme.outline_variant().opacity(0.4)
        })
        .focused_border_color(if auth_failed {
            theme.red()
        } else {
            theme.primary()
        })
}
