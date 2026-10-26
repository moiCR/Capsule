use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, FontWeight, IntoElement, div, px, svg};
use services::{AppState, RecordStatus};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let (status, duration_secs) = if let Some(state) = cx.try_global::<AppState>() {
        (state.record.get_status(), state.record.get_duration_secs())
    } else {
        (RecordStatus::Stopped, 0)
    };

    let (anchor, tracker) = crate::bar::widgets::track_anchor();
    let anchor_click = anchor.clone();

    let is_active = status == RecordStatus::Recording || status == RecordStatus::Paused;
    let hover_bg = if is_active {
        theme.red().opacity(0.18)
    } else {
        style::hover(theme)
    };

    let mut widget = div()
        .id("bar-widget-record")
        .relative()
        .child(tracker)
        .h(px(style::WIDGET_HEIGHT))
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
                    crate::bar::shared_panel::SharedPanelModule::Record,
                    target,
                    window,
                    cx,
                );
            }),
        );

    if is_active {
        let mins = duration_secs / 60;
        let secs = duration_secs % 60;
        let time_str = format!("{mins:02}:{secs:02}");
        let icon_color = if status == RecordStatus::Recording {
            theme.red()
        } else {
            theme.yellow()
        };

        widget = widget
            .px(px(8.0))
            .gap(px(6.0))
            .child(
                svg()
                    .path("camera.svg")
                    .size(px(15.0))
                    .text_color(icon_color),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(icon_color)
                    .child(time_str),
            );
    } else {
        widget = widget.size(px(style::WIDGET_HEIGHT)).child(
            svg()
                .path("camera.svg")
                .size(px(15.0))
                .text_color(theme.foreground()),
        );
    }

    widget.into_any_element()
}
