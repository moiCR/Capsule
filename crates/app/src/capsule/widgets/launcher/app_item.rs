use gpui::{Context, FontWeight, IntoElement, div, prelude::*, svg};
use services::Application;
use ui::theme::Theme;

use crate::capsule::modules::launcher::{LauncherEvent, LauncherModule};

pub fn render_app_item(
    idx: usize,
    app: &Application,
    is_selected: bool,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let bg_color = if is_selected {
        theme.surface()
    } else {
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    };

    let text_color = theme.foreground();
    let muted_color = theme.foreground_muted();
    let app_clone = app.clone();

    div()
        .id(idx)
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .rounded_lg()
        .bg(bg_color)
        .cursor_pointer()
        .on_hover(cx.listener(move |this, &hovered, _window, cx| {
            if hovered && this.mouse_moved && this.selected_index != idx {
                this.selected_index = idx;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _window, cx| {
            let _ = app_clone.launch();
            this.reset_search(cx);
            cx.emit(LauncherEvent::Close);
        }))
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .w_8()
                .h_8()
                .rounded_md()
                .bg(theme.background_alt())
                .overflow_hidden()
                .child(if let Some(icon_path) = &app.icon_path {
                    gpui::img(icon_path.clone()).w_5().h_5().into_any_element()
                } else {
                    svg()
                        .path("sparkles.svg")
                        .w_4()
                        .h_4()
                        .text_color(theme.accent())
                        .into_any_element()
                }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .overflow_hidden()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(text_color)
                        .child(app.name.clone()),
                )
                .child(if let Some(desc) = &app.generic_name {
                    div().text_xs().text_color(muted_color).child(desc.clone())
                } else if let Some(comment) = &app.comment {
                    div()
                        .text_xs()
                        .text_color(muted_color)
                        .child(comment.clone())
                } else {
                    div()
                        .text_xs()
                        .text_color(muted_color)
                        .child(app.exec.clone())
                }),
        )
}
