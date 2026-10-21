use std::time::Duration;

use gpui::{Context, IntoElement, Styled, px, svg};
use services::AppState;
use ui::components::{Input, InputState};
use ui::theme::Theme;

use crate::capsule::module::launcher::LauncherModule;
use crate::capsule::widgets::style;

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
        .bg(style::surface(theme))
        .border_color(style::border(theme))
        .focused_border_color(theme.accent())
        .prefix(
            svg()
                .path("search.svg")
                .size(px(17.0))
                .text_color(theme.foreground_muted()),
        )
        .on_clear(cx.listener(|this, _, _, cx| this.reset_search(cx)))
}
