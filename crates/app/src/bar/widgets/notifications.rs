use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, px, svg};
use services::NotificationStore;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let store = NotificationStore::global();
    let is_dnd = store.is_dnd_enabled();
    let notifs_count = store.get_all_notifications().len();

    let hover_bg = style::hover(theme);
    let (anchor, tracker) = crate::bar::widgets::track_anchor();
    let anchor_click = anchor.clone();

    let icon_path = if is_dnd {
        "bell-off.svg"
    } else if notifs_count > 0 {
        "bell-ring.svg"
    } else {
        "bell.svg"
    };

    let icon_color = if is_dnd {
        theme.foreground_muted()
    } else if notifs_count > 0 {
        theme.accent()
    } else {
        theme.foreground()
    };

    let mut widget = div()
        .id("bar-widget-notifications")
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
                    crate::bar::shared_panel::SharedPanelModule::Calendar,
                    target,
                    window,
                    cx,
                );
            }),
        )
        .on_mouse_down(
            gpui::MouseButton::Right,
            cx.listener(move |_bar, _event, _window, cx| {
                NotificationStore::global().toggle_dnd();
                cx.notify();
            }),
        );

    if notifs_count > 0 && !is_dnd {
        widget = widget
            .px(px(8.0))
            .gap(px(6.0))
            .child(svg().path(icon_path).size(px(15.0)).text_color(icon_color))
            .child(
                div()
                    .px(px(6.0))
                    .py(px(1.0))
                    .rounded_full()
                    .bg(theme.primary())
                    .text_size(px(10.5))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.on_primary())
                    .child(format!("{notifs_count}")),
            );
    } else {
        widget = widget
            .size(px(style::WIDGET_HEIGHT))
            .child(svg().path(icon_path).size(px(15.0)).text_color(icon_color));
    }

    widget.into_any_element()
}
