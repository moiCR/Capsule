use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
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
    let app_clone = app.clone();

    div()
        .id(idx)
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py(if is_selected { px(10.0) } else { px(8.0) })
        .rounded(px(16.0))
        .cursor_pointer()
        .bg(if is_selected {
            theme.accent().opacity(0.1)
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        })
        .border_1()
        .border_color(if is_selected {
            theme.accent().opacity(0.25)
        } else {
            gpui::hsla(0.0, 0.0, 0.0, 0.0)
        })
        .hover(|s| {
            s.bg(theme.surface().opacity(0.35))
                .border_color(theme.surface().opacity(0.3))
        })
        .active(|s| s.bg(theme.surface().opacity(0.5)))
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
                .w(if is_selected { px(40.0) } else { px(36.0) })
                .h(if is_selected { px(40.0) } else { px(36.0) })
                .rounded(px(12.0))
                .bg(if is_selected {
                    theme.accent().opacity(0.12)
                } else {
                    theme.background_alt()
                })
                .overflow_hidden()
                .child(if let Some(icon_path) = &app.icon_path {
                    gpui::img(icon_path.clone())
                        .w(if is_selected { px(24.0) } else { px(20.0) })
                        .h(if is_selected { px(24.0) } else { px(20.0) })
                        .into_any_element()
                } else {
                    svg()
                        .path("sparkles.svg")
                        .w(if is_selected { px(24.0) } else { px(20.0) })
                        .h(if is_selected { px(24.0) } else { px(20.0) })
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
                .gap_0p5()
                .child(
                    div()
                        .text_size(if is_selected { px(14.0) } else { px(13.0) })
                        .font_weight(if is_selected {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.accent()
                        } else {
                            theme.foreground()
                        })
                        .text_ellipsis()
                        .child(app.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .text_ellipsis()
                        .child(
                            app.generic_name
                                .as_deref()
                                .or(app.comment.as_deref())
                                .unwrap_or(&app.exec)
                                .to_string(),
                        ),
                ),
        )
        .child(if is_selected {
            div()
                .flex_none()
                .flex()
                .items_center()
                .justify_center()
                .px(px(8.0))
                .py(px(3.0))
                .rounded(px(8.0))
                .bg(theme.accent().opacity(0.15))
                .child(
                    div()
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.accent())
                        .child("↵"),
                )
                .into_any_element()
        } else {
            div().into_any_element()
        })
}
