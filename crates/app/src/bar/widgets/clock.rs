use chrono::Local;
use gpui::prelude::*;
use gpui::{AnyElement, Context, IntoElement, div, px};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(
    clock_center: &std::rc::Rc<std::cell::Cell<gpui::Pixels>>,
    theme: &Theme,
    cx: &mut Context<Bar>,
) -> AnyElement {
    let now = Local::now();
    let time_str = if let Some(state) = cx.try_global::<AppState>() {
        let format = state.config.get().ui.clock_format();
        now.format(format).to_string()
    } else {
        now.format("%H:%M").to_string()
    };

    let hover_bg = style::hover(theme);
    let anchor = std::rc::Rc::new(std::cell::Cell::new(px(0.0)));
    let anchor_click = anchor.clone();
    let bar_clock_center = clock_center.clone();
    let anchor_sync = anchor.clone();

    let tracker = gpui::canvas(
        move |bounds, _, _| {
            let cx_pos = bounds.origin.x + bounds.size.width / 2.0;
            anchor_sync.set(cx_pos);
            bar_clock_center.set(cx_pos);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full();

    div()
        .id("bar-widget-clock")
        .relative()
        .child(tracker)
        .flex()
        .items_center()
        .h(px(style::WIDGET_HEIGHT))
        .px(px(10.0))
        .rounded_full()
        .cursor_pointer()
        .hover(move |s| s.bg(hover_bg))
        .child(
            div()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_size(px(13.0))
                .text_color(theme.foreground())
                .child(time_str),
        )
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
        .into_any_element()
}
