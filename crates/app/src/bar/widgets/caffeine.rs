use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let is_inhibited = if let Some(state) = cx.try_global::<AppState>() {
        state.idle.is_inhibited()
    } else {
        false
    };

    let hover_bg = if is_inhibited {
        theme.accent().opacity(0.25)
    } else {
        style::hover(theme)
    };

    let icon_color = if is_inhibited {
        theme.accent()
    } else {
        theme.foreground_muted()
    };

    let mut widget = div()
        .id("bar-widget-caffeine")
        .size(px(style::WIDGET_HEIGHT))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |_bar, _event, _window, cx| {
                if let Some(state) = cx.try_global::<AppState>() {
                    let _ = state.idle.toggle_inhibited();
                }
                cx.notify();
            }),
        )
        .child(
            svg()
                .path("coffee.svg")
                .size(px(15.0))
                .text_color(icon_color),
        );

    if is_inhibited {
        widget = widget.bg(theme.accent().opacity(0.14));
    }

    widget.into_any_element()
}
