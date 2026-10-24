use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let window_title = if let Some(state) = cx.try_global::<AppState>() {
        let active = state.compositor.get_active_window();
        if !active.title.trim().is_empty() {
            active.title
        } else if !active.class.trim().is_empty() {
            active.class
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if window_title.is_empty() {
        return div()
            .id("bar-widget-active-window-empty")
            .into_any_element();
    }

    let display_title = if window_title.chars().count() > 36 {
        let truncated: String = window_title.chars().take(35).collect();
        format!("{truncated}…")
    } else {
        window_title
    };

    div()
        .id("bar-widget-active-window")
        .h(px(style::WIDGET_HEIGHT))
        .px(px(8.0))
        .flex()
        .items_center()
        .gap(px(6.0))
        .child(
            svg()
                .path("monitor.svg")
                .size(px(13.5))
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(style::text(theme))
                .child(display_title),
        )
        .into_any_element()
}
