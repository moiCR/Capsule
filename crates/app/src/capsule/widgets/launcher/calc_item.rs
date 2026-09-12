use gpui::{ClipboardItem, Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::launcher::{LauncherEvent, LauncherModule};

pub fn render_calc_item(
    result: &str,
    query: &str,
    is_selected: bool,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let result_str = result.to_string();
    let result_for_click = result_str.clone();

    let indicator_color = if is_selected {
        theme.accent()
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    let item_bg = if is_selected {
        theme.surface().opacity(0.55)
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    let subtitle = format!("= {query}");

    div()
        .id("launcher-calc-result-item")
        .flex()
        .items_center()
        .gap_2p5()
        .px_2()
        .py(px(6.0))
        .rounded(px(12.0))
        .cursor_pointer()
        .bg(item_bg)
        .hover(|s| s.bg(theme.surface().opacity(0.4)))
        .active(|s| s.bg(theme.surface().opacity(0.6)))
        .on_hover(cx.listener(move |this, &hovered, _window, cx| {
            if hovered && this.mouse_moved && this.selected_index != 0 {
                this.selected_index = 0;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _window, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(result_for_click.clone()));
            this.reset_search(cx);
            cx.emit(LauncherEvent::Close);
        }))
        .child(
            div()
                .w(px(3.0))
                .h(px(18.0))
                .rounded_full()
                .bg(indicator_color),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(32.0))
                .h(px(32.0))
                .rounded(px(10.0))
                .bg(theme.surface().opacity(0.6))
                .overflow_hidden()
                .child(
                    svg()
                        .path("calculator.svg")
                        .w(px(20.0))
                        .h(px(20.0))
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .gap_0p5()
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.foreground())
                        .text_ellipsis()
                        .child(result_str),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(subtitle),
                ),
        )
}
