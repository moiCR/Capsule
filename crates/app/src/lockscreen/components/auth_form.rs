use gpui::{Context, Element, ParentElement, Styled, div, px};
use services::AppState;
use ui::theme::Theme;

pub fn render_auth_form(
    theme: &Theme,
    password_len: usize,
    auth_failed: bool,
    is_checking: bool,
    cx: &Context<crate::lockscreen::LockScreen>,
) -> impl Element {
    let (enter_pwd, checking_text, incorrect_text) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("lockscreen.enter_password"),
            lang.get("lockscreen.checking"),
            lang.get("lockscreen.incorrect_password"),
        )
    } else {
        (
            "Enter Password".to_string(),
            "Checking...".to_string(),
            "Incorrect password".to_string(),
        )
    };

    let masked_password = if password_len == 0 {
        enter_pwd
    } else {
        "●".repeat(password_len)
    };

    let border_color = if auth_failed {
        theme.red()
    } else {
        theme.surface()
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .w(px(280.0))
        // Password Input Container
        .child(
            div()
                .w_full()
                .h(px(46.0))
                .px(px(16.0))
                .rounded(px(16.0))
                .bg(theme.surface().opacity(0.5))
                .border_1()
                .border_color(border_color)
                .shadow_md()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .font_family(theme.font_family())
                        .text_size(px(15.0))
                        .text_center()
                        .text_color(if password_len == 0 {
                            theme.foreground_muted()
                        } else {
                            theme.foreground()
                        })
                        .child(masked_password),
                )
                .children(if is_checking {
                    Some(
                        div()
                            .absolute()
                            .right(px(16.0))
                            .font_family(theme.font_family())
                            .text_size(px(12.5))
                            .text_color(theme.accent())
                            .child(checking_text),
                    )
                } else {
                    None
                }),
        )
        // Error Message
        .children(if auth_failed {
            Some(
                div()
                    .font_family(theme.font_family())
                    .text_size(px(12.0))
                    .text_color(theme.red())
                    .child(incorrect_text),
            )
        } else {
            None
        })
}
