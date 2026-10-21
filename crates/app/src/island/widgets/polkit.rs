use super::style;
use crate::capsule::module::polkit::PolkitModule;
use gpui::{AnyElement, Context, FocusHandle, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::components::{Button, ButtonVariant, Input};
use ui::theme::Theme;

pub const WIDTH: f32 = 432.0;
pub const HEIGHT: f32 = 258.0;

pub(crate) fn message_lines(message: &str) -> f32 {
    let mut lines = 1.0_f32;
    let mut line_length = 0;
    for word in message.split_whitespace() {
        let word_length = word.chars().count();
        let wraps = line_length > 0 && line_length + word_length + 1 > 58;
        if wraps {
            lines += 1.0;
        }
        lines += (word_length.saturating_sub(1) / 58) as f32;
        line_length = if wraps {
            word_length % 58
        } else {
            line_length + word_length + usize::from(line_length > 0)
        };
    }
    lines.clamp(1.0, 3.0)
}

pub(crate) fn height(has_error: bool, message: Option<&str>) -> f32 {
    let lines = message.map_or(2.0, message_lines);
    HEIGHT + (lines - 2.0) * 17.0 + if has_error { 44.0 } else { 0.0 }
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
    let message_height = message_lines(&request.message) * 17.0;
    let message_card_height = 34.0 + message_height;
    let enabled = !module.authenticating && !module.password_input.is_empty();
    let placeholder = module.text(
        if module.authenticating {
            "polkit.verifying"
        } else {
            "polkit.password_placeholder"
        },
        cx,
    );
    let field = Input::from_state("polkit-password", &module.password_input)
        .placeholder(placeholder)
        .masked(true)
        .focused(true)
        .disabled(module.authenticating)
        .error(module.error.is_some())
        .prefix(
            svg()
                .path("lock.svg")
                .size(px(18.0))
                .flex_shrink_0()
                .text_color(theme.foreground_muted())
                .into_any_element(),
        )
        .h_px(42.0)
        .rounded_full()
        .bg(style::background(&theme))
        .border_color(if module.error.is_some() {
            theme.red()
        } else {
            style::tint(style::border(&theme), theme.accent(), 0.35)
        })
        .on_click(cx.listener(|this, _, window, cx| window.focus(&this.focus_handle(), cx)));
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
        .h(px(height(module.error.is_some(), Some(&request.message))))
        .p(px(14.0))
        .rounded(px(config.ui.capsule_round))
        .overflow_hidden()
        .bg(style::background(&theme))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .w_full()
                .h(px(36.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(30.0))
                        .flex_shrink_0()
                        .rounded_full()
                        .bg(theme.accent())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            svg()
                                .path("lock.svg")
                                .size(px(15.0))
                                .text_color(style::on_accent(&theme)),
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
                                .text_size(px(15.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .whitespace_nowrap()
                                .text_ellipsis()
                                .child(module.text("polkit.auth_required", cx)),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
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
                .h(px(message_card_height))
                .p(px(8.0))
                .rounded(px(style::CARD_RADIUS))
                .bg(style::surface(&theme))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .id("polkit-message")
                        .w_full()
                        .h(px(message_height))
                        .flex_shrink_0()
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
        .child(
            div()
                .w_full()
                .h(px(64.0))
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .h(px(14.0))
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child(module.text("polkit.password_placeholder", cx)),
                )
                .child(field),
        )
        .when(module.error.is_some(), |s| s.child(feedback))
        .child(
            div()
                .w_full()
                .h(px(32.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_end()
                .gap(px(8.0))
                .child(
                    Button::new("polkit-cancel")
                        .label(module.text("common.cancel", cx))
                        .variant(ButtonVariant::Secondary)
                        .padding_x(18.0)
                        .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                )
                .child(
                    Button::new("polkit-authenticate")
                        .label(module.text(
                            if module.authenticating {
                                "polkit.verifying"
                            } else {
                                "common.authenticate"
                            },
                            cx,
                        ))
                        .variant(ButtonVariant::Primary)
                        .disabled(!enabled)
                        .padding_x(18.0)
                        .on_click(cx.listener(|this, _, _, cx| this.submit(cx))),
                ),
        )
        .into_any_element()
}
