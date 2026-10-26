use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, px};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let layout = if let Some(state) = cx.try_global::<AppState>() {
        state.compositor.get_keyboard_layout()
    } else {
        "ES".to_string()
    };

    let hover_bg = style::hover(theme);

    div()
        .id("bar-widget-keyboard")
        .h(px(style::WIDGET_HEIGHT))
        .px(px(8.0))
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
                    let compositor = state.compositor.clone();
                    services::spawn_tokio(async move {
                        compositor.switch_keyboard_layout().await;
                    });
                }
                cx.notify();
            }),
        )
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(style::text(theme))
                .child(layout),
        )
        .into_any_element()
}
