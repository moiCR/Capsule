use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, div, px, svg};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let hover_bg = style::hover(theme);
    let (anchor, tracker) = crate::bar::widgets::track_anchor();
    let anchor_click = anchor.clone();

    div()
        .id("bar-widget-clipboard")
        .relative()
        .child(tracker)
        .size(px(style::WIDGET_HEIGHT))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .on_mouse_down(
            gpui::MouseButton::Left,
            cx.listener(move |bar, event: &gpui::MouseDownEvent, window, cx| {
                let center_x = anchor_click.get();
                let target = if center_x > px(0.0) {
                    center_x
                } else {
                    event.position.x
                };
                bar.toggle_shared_panel(
                    crate::bar::shared_panel::SharedPanelModule::Clipboard,
                    target,
                    window,
                    cx,
                );
            }),
        )
        .child(
            svg()
                .path("clipboard-list.svg")
                .size(px(15.0))
                .text_color(theme.foreground()),
        )
        .into_any_element()
}
