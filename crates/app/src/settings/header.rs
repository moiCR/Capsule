use gpui::{ColorExt, Context, FontWeight, IntoElement, ParentElement, Styled, div, px, svg};
use ui::components::{IconButton, Input};
use ui::theme::Theme;

use crate::island::widgets::style;
use crate::settings::Settings;

pub fn render_header(
    search_input_state: &ui::components::InputState,
    theme: &Theme,
    cx: &mut Context<Settings>,
) -> gpui::AnyElement {
    let placeholder = if cx.has_global::<services::AppState>() {
        cx.global::<services::AppState>()
            .language
            .get("settings.search_short")
    } else {
        "Buscar...".to_string()
    };

    let is_searching = search_input_state.focused;
    let query_empty = search_input_state.is_empty();

    let suffix = if query_empty && !is_searching {
        Some(
            div()
                .px(px(6.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .bg(style::raised(theme))
                .text_size(px(11.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground_muted().opacity(0.7))
                .child("Ctrl+F")
                .into_any_element(),
        )
    } else {
        None
    };

    let mut search_input = Input::from_state("settings-search-box", search_input_state)
        .placeholder(placeholder)
        .cleanable(true)
        .w_px(420.0)
        .max_w_px(420.0)
        .h_px(44.0)
        .rounded_full()
        .bg(style::surface(theme))
        .border_color(style::border(theme))
        .focused_border_color(theme.accent())
        .prefix(
            svg()
                .path("search.svg")
                .size(px(15.0))
                .text_color(if is_searching {
                    theme.accent()
                } else {
                    theme.foreground_muted()
                }),
        )
        .on_click(cx.listener(|this, _, window, cx| {
            this.search_input.focused = true;
            window.focus(&this.focus_handle, cx);
            cx.notify();
        }))
        .on_clear(cx.listener(|this, _, window, cx| {
            this.search_input.clear();
            window.focus(&this.focus_handle, cx);
            cx.notify();
        }));

    if let Some(sfx) = suffix {
        search_input = search_input.suffix(sfx);
    }

    div()
        .flex()
        .items_center()
        .gap(px(10.0))
        .w_full()
        .min_w_0()
        .h(px(48.0))
        .flex_shrink_0()
        .child(div().w(px(30.0)).flex_shrink_0())
        .child(
            div()
                .flex()
                .flex_1()
                .min_w_0()
                .justify_center()
                .child(search_input),
        )
        .child(
            IconButton::new("settings-close-btn", "close.svg")
                .size(30.0)
                .tooltip(if cx.has_global::<services::AppState>() {
                    cx.global::<services::AppState>()
                        .language
                        .get("common.close")
                } else {
                    "Cerrar".to_string()
                })
                .on_click(cx.listener(|this, _, _, cx| this.request_close(cx))),
        )
        .into_any_element()
}
