use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, div, px};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let (track_title, track_artist) = if let Some(state) = cx.try_global::<AppState>() {
        let track = state.mpris.get_current_track();
        (track.title, track.artist)
    } else {
        (String::new(), String::new())
    };

    if track_title.is_empty() {
        return div().id("bar-widget-media-empty").into_any_element();
    }

    let max_w = if let Some(state) = cx.try_global::<AppState>() {
        let left_len = state.config.get().ui.bar.left.len();
        if left_len >= 5 {
            px(110.0)
        } else if left_len >= 4 {
            px(150.0)
        } else {
            px(220.0)
        }
    } else {
        px(220.0)
    };

    let mut text_container = div()
        .flex()
        .items_center()
        .gap(px(5.0))
        .max_w(max_w)
        .overflow_hidden();

    text_container = text_container.child(
        div()
            .text_size(px(12.5))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme.foreground())
            .line_clamp(1)
            .child(track_title),
    );

    if !track_artist.is_empty() {
        text_container = text_container.child(
            div()
                .text_size(px(11.5))
                .text_color(theme.foreground_muted())
                .line_clamp(1)
                .child(format!("• {track_artist}")),
        );
    }

    let hover_bg = style::hover(theme);
    let (anchor, tracker) = crate::bar::widgets::track_anchor();
    let anchor_click = anchor.clone();

    div()
        .id("bar-widget-media")
        .relative()
        .child(tracker)
        .flex()
        .items_center()
        .h(px(style::WIDGET_HEIGHT))
        .px(px(10.0))
        .rounded_full()
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
                    crate::bar::shared_panel::SharedPanelModule::Media,
                    target,
                    window,
                    cx,
                );
            }),
        )
        .child(text_container)
        .into_any_element()
}
