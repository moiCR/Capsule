use gpui::{Context, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px, svg};
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
            "Ingresa la contraseña".to_string(),
            "Verificando...".to_string(),
            "Contraseña incorrecta".to_string(),
        )
    };

    let border_color = if auth_failed {
        theme.red()
    } else {
        theme.surface().opacity(0.6)
    };

    let icon_color = if auth_failed {
        theme.red()
    } else {
        theme.foreground_muted().opacity(0.7)
    };

    let masked_password = "●".repeat(password_len);

    let right_element = if is_checking {
        div()
            .font_family(theme.font_family())
            .text_size(px(11.5))
            .font_weight(FontWeight::MEDIUM)
            .text_color(theme.accent())
            .flex_shrink_0()
            .child(checking_text)
            .into_any_element()
    } else if password_len > 0 {
        div()
            .id("lockscreen-submit-btn")
            .w(px(26.0))
            .h(px(26.0))
            .rounded_full()
            .bg(theme.accent())
            .hover(|s| s.opacity(0.85))
            .active(|s| s.opacity(0.65))
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .on_click(cx.listener(|this, _, _, cx| {
                this.submit_password(cx);
            }))
            .child(
                svg()
                    .path("chevron-right.svg")
                    .size(px(14.0))
                    .text_color(theme.background()),
            )
            .into_any_element()
    } else {
        div().w(px(14.0)).flex_shrink_0().into_any_element()
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .w_full()
        .child(
            div()
                .w_full()
                .h(px(42.0))
                .px(px(14.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(border_color)
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(
                    svg()
                        .path("lock.svg")
                        .size(px(14.0))
                        .text_color(icon_color)
                        .flex_shrink_0(),
                )
                .child(
                    div()
                        .flex_1()
                        .text_center()
                        .font_family(theme.font_family())
                        .child(if password_len == 0 {
                            div()
                                .text_size(px(12.5))
                                .text_color(theme.foreground_muted().opacity(0.6))
                                .child(enter_pwd)
                        } else {
                            div()
                                .text_size(px(14.0))
                                .text_color(theme.foreground())
                                .child(masked_password)
                        }),
                )
                .child(right_element),
        )
        .children(if auth_failed {
            Some(
                div()
                    .font_family(theme.font_family())
                    .text_size(px(11.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.red())
                    .child(incorrect_text),
            )
        } else {
            None
        })
}
