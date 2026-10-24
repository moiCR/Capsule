use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, div, img, px};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let items = if let Some(state) = cx.try_global::<AppState>() {
        state.sni_host.items_arc()
    } else {
        std::sync::Arc::new(Vec::new())
    };

    if items.is_empty() {
        return div().id("bar-widget-tray-empty").into_any_element();
    }

    let mut tray = div()
        .id("bar-widget-tray")
        .flex()
        .items_center()
        .gap(px(4.0))
        .h(px(style::WIDGET_HEIGHT))
        .px(px(4.0));

    for (idx, item) in items.iter().enumerate() {
        let icon_el = if let Some(path) = &item.icon_file_path {
            img(PathBuf::from(path)).size(px(16.0)).into_any_element()
        } else {
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground())
                .child(item.title.chars().next().unwrap_or('?').to_string())
                .into_any_element()
        };

        let hover_bg = style::hover(theme);
        let (anchor, tracker) = crate::bar::widgets::track_anchor();
        let anchor_click = anchor.clone();

        tray = tray.child(
            div()
                .id(("bar-tray-item", idx))
                .relative()
                .child(tracker)
                .size(px(24.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .child(icon_el)
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
                            crate::bar::shared_panel::SharedPanelModule::Tray(idx),
                            target,
                            window,
                            cx,
                        );
                    }),
                ),
        );
    }

    tray.into_any_element()
}
