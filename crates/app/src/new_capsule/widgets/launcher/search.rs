use crate::new_capsule::widgets::style;
use std::time::Duration;

use gpui::{Context, IntoElement, MotionDurationExt, div, ease_in_out, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::new_capsule::module::launcher::LauncherModule;

pub fn render_search(
    query: &str,
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let text = if query.is_empty() {
        cx.global::<AppState>()
            .language
            .get("launcher.search_placeholder")
    } else {
        query.to_string()
    };
    let surface = style::surface(theme);
    let hover = style::hover(theme);
    div()
        .id("launcher-search")
        .h(px(44.0))
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(10.0))
        .px(px(12.0))
        .rounded_full()
        .bg(surface)
        .transitions(|transitions| transitions.bg(duration.with_easing(ease_in_out)))
        .hover(move |style| style.bg(hover))
        .child(
            svg()
                .path("search.svg")
                .size(px(17.0))
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(14.0))
                .text_color(if query.is_empty() {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .child(text),
        )
        .child(
            div()
                .id("launcher-search-clear")
                .size(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .rounded_full()
                .on_click(cx.listener(|this, _, _, cx| this.reset_search(cx)))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted()),
                )
                .opacity(if query.is_empty() { 0.0 } else { 1.0 })
                .transitions(|transitions| transitions.opacity(duration.with_easing(ease_in_out))),
        )
}
