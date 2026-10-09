use super::style;
use crate::new_capsule::module::polkit::PolkitModule;
use gpui::{AnyElement, Context, FocusHandle, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

pub const WIDTH: f32 = 400.0;
pub const HEIGHT: f32 = 212.0;

pub(crate) fn height(has_error: bool) -> f32 {
    HEIGHT + if has_error { 40.0 } else { 0.0 }
}

pub(crate) fn render(
    module: &PolkitModule,
    focus: &FocusHandle,
    cx: &mut Context<PolkitModule>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let config = cx.global::<AppState>().config.get();
    let Some(request) = &module.request else {
        return div().into_any_element();
    };
    let enabled = !module.authenticating && !module.password.is_empty();
    let placeholder = module.text(
        if module.authenticating {
            "polkit.verifying"
        } else {
            "polkit.password_placeholder"
        },
        cx,
    );
    let field = div()
        .id("polkit-password")
        .w_full()
        .h(px(40.0))
        .flex_shrink_0()
        .px(px(14.0))
        .rounded(px(style::INNER_RADIUS))
        .bg(style::surface(&theme))
        .border_1()
        .border_color(if module.error.is_some() {
            theme.red()
        } else {
            theme.accent()
        })
        .flex()
        .items_center()
        .gap(px(10.0))
        .on_click(cx.listener(|this, _, window, cx| window.focus(&this.focus_handle(), cx)))
        .child(
            svg()
                .path("lock.svg")
                .size(px(18.0))
                .flex_shrink_0()
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_size(px(14.0))
                .text_color(if module.password.is_empty() {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .child(if module.password.is_empty() {
                    placeholder
                } else {
                    "•".repeat(module.password.chars().count().min(32))
                }),
        );
    let mut feedback = div()
        .id("polkit-feedback")
        .w_full()
        .h(px(34.0))
        .flex_shrink_0()
        .overflow_y_scroll()
        .text_size(px(12.0))
        .line_height(px(18.0));
    if let Some(error) = &module.error {
        feedback = feedback.text_color(theme.red()).child(error.clone());
    }
    div()
        .id("polkit-module")
        .track_focus(focus)
        .on_key_down(cx.listener(PolkitModule::key_down))
        .w(px(WIDTH))
        .h(px(height(module.error.is_some())))
        .p(px(14.0))
        .rounded(px(config.ui.capsule_round))
        .overflow_hidden()
        .bg(style::background(&theme))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .flex()
        .flex_col()
        .gap(px(6.0))
        .child(
            div()
                .w_full()
                .h(px(40.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(40.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(style::selected(&theme))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .path("lock.svg")
                                .size(px(20.0))
                                .text_color(theme.accent()),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(16.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(module.text("polkit.auth_required", cx)),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.foreground_muted())
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(format!(
                                    "{}{}",
                                    module.text("polkit.user_prefix", cx),
                                    request.user_name
                                )),
                        ),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(54.0))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .id("polkit-message")
                        .w_full()
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .text_size(px(13.0))
                        .line_height(px(17.0))
                        .child(request.message.clone()),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(14.0))
                        .flex_shrink_0()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(request.action_id.clone()),
                ),
        )
        .child(field)
        .when(module.error.is_some(), |s| s.child(feedback))
        .child(
            div()
                .w_full()
                .h(px(32.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .id("polkit-cancel")
                        .h(px(32.0))
                        .px(px(18.0))
                        .rounded_full()
                        .bg(style::surface(&theme))
                        .text_size(px(13.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(style::hover(&theme)))
                        .on_click(cx.listener(|this, _, _, cx| this.cancel(cx)))
                        .child(module.text("common.cancel", cx)),
                )
                .child(
                    style::primary_button(
                        "polkit-authenticate",
                        module.text(
                            if module.authenticating {
                                "polkit.verifying"
                            } else {
                                "common.authenticate"
                            },
                            cx,
                        ),
                        enabled,
                        &theme,
                    )
                    .h(px(32.0))
                    .text_size(px(13.0))
                    .when(enabled, |s| {
                        s.on_click(cx.listener(|this, _, _, cx| this.submit(cx)))
                    }),
                ),
        )
        .into_any_element()
}
