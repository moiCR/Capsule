use std::time::Duration;

use gpui::{
    ColorExt, Context, IntoElement, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Styled, px, svg,
};
use services::AppState;
use ui::components::{Input, InputState};
use ui::theme::Theme;

use crate::island::module::launcher::LauncherModule;

pub fn render_search(
    search_input: &InputState,
    _duration: Duration,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let placeholder = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .language
            .get("launcher.search_placeholder")
    } else {
        "Buscar aplicaciones...".to_string()
    };

    Input::from_state("launcher-search", search_input)
        .placeholder(placeholder)
        .cleanable(true)
        .w_full()
        .h_px(44.0)
        .rounded_full()
        .bg(theme.surface_container_high())
        .border_color(theme.outline_variant().opacity(0.40))
        .focused_border_color(theme.primary())
        .prefix(
            svg()
                .path("search.svg")
                .size(px(17.0))
                .text_color(theme.on_surface_variant()),
        )
        .on_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
            let theme = cx.global::<Theme>().clone();
            this.search_input.handle_mouse_down(event, window, &theme);
            cx.notify();
        }))
        .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
            if this.search_input.is_dragging {
                let theme = cx.global::<Theme>().clone();
                this.search_input.handle_mouse_move(event, window, &theme);
                cx.notify();
            }
        }))
        .on_mouse_up(cx.listener(|this, event: &MouseUpEvent, _, cx| {
            this.search_input.handle_mouse_up(event);
            cx.notify();
        }))
        .on_clear(cx.listener(|this, _, _, cx| this.reset_search(cx)))
}
