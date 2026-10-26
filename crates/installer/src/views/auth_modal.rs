use crate::app::InstallerApp;
use crate::style;
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Styled, Window, div, prelude::*, px, svg,
};
use ui::components::Input;
use ui::theme::Theme;

pub fn render_auth_modal(
    app: &mut InstallerApp,
    _window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let has_error = app.password_error.is_some();

    div()
        .id("auth-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("auth-modal-card")
                .occlude()
                .w(px(420.0))
                .p(px(24.0))
                .rounded(px(16.0))
                .bg(theme.surface_container_high())
                .border_1()
                .border_color(theme.outline_variant().opacity(0.40))
                .shadow_xl()
                .flex()
                .flex_col()
                .gap(px(18.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(14.0))
                        .child(
                            div()
                                .size(px(40.0))
                                .rounded_full()
                                .bg(theme.primary().opacity(0.15))
                                .border_1()
                                .border_color(theme.primary().opacity(0.35))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    svg()
                                        .path("lock.svg")
                                        .size(px(20.0))
                                        .text_color(theme.primary()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_size(px(16.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme.foreground())
                                        .child("Authentication Required"),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.foreground_muted())
                                        .child("Administrator privileges are needed to proceed."),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            Input::from_state("auth-password-field", &app.password_input)
                                .placeholder("Password")
                                .masked(true)
                                .focused(true)
                                .error(has_error)
                                .h_px(40.0)
                                .rounded_full()
                                .bg(theme.surface_container())
                                .border_color(if has_error {
                                    theme.red()
                                } else {
                                    theme.outline_variant().opacity(0.40)
                                })
                                .focused_border_color(theme.primary())
                                .on_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                                    let theme = cx.global::<Theme>().clone();
                                    this.password_input.handle_mouse_down(event, window, &theme);
                                    cx.notify();
                                }))
                                .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                                    if this.password_input.is_dragging {
                                        let theme = cx.global::<Theme>().clone();
                                        this.password_input.handle_mouse_move(event, window, &theme);
                                        cx.notify();
                                    }
                                }))
                                .on_mouse_up(cx.listener(|this, event: &MouseUpEvent, _, cx| {
                                    this.password_input.handle_mouse_up(event);
                                    cx.notify();
                                })),
                        )
                        .when_some(app.password_error.as_ref(), |this, err| {
                            this.child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(theme.red())
                                    .child(err.clone()),
                            )
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap(px(10.0))
                        .pt(px(4.0))
                        .child(
                            style::secondary_button(
                                "auth-cancel-btn",
                                "Cancel".to_string(),
                                &theme,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel_password_modal(cx);
                            })),
                        )
                        .child(
                            style::primary_button(
                                "auth-submit-btn",
                                "Authenticate".to_string(),
                                true,
                                &theme,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                if this.submit_password() {
                                    this.execute_pending_action(cx);
                                }
                                cx.notify();
                            })),
                        ),
                ),
        )
        .into_any_element()
}
