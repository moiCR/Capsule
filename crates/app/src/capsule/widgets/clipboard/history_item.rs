use gpui::{Context, FontWeight, IntoElement, div, img, prelude::*, px, svg};
use services::ClipboardItem;
use ui::theme::Theme;

use crate::capsule::modules::clipboard::{ClipboardEvent, ClipboardModule};

fn render_fallback_image_icon(theme: &Theme) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(56.0))
        .h(px(40.0))
        .rounded(px(8.0))
        .bg(theme.surface().opacity(0.6))
        .border_1()
        .border_color(theme.surface().opacity(0.35))
        .flex_shrink_0()
        .child(
            svg()
                .path("wallpaper.svg")
                .size(px(18.0))
                .text_color(theme.foreground_muted()),
        )
        .into_any_element()
}

pub fn render_history_item(
    idx: usize,
    item: &ClipboardItem,
    is_selected: bool,
    is_pinned: bool,
    theme: &Theme,
    cx: &mut Context<ClipboardModule>,
) -> impl IntoElement {
    let (empty_item_text, image_title_text) = if cx.has_global::<services::AppState>() {
        let lang = &cx.global::<services::AppState>().language;
        let img_txt = lang.get("clipboard.image_item");
        let empty_txt = lang.get("clipboard.empty_item");
        (
            empty_txt,
            if img_txt.is_empty() || img_txt == "clipboard.image_item" {
                "Imagen".to_string()
            } else {
                img_txt
            },
        )
    } else {
        ("[Elemento vacío]".to_string(), "Imagen".to_string())
    };

    let item_clone = item.clone();
    let item_for_pin = item.clone();

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

    let indicator_height = if item.is_image { px(36.0) } else { px(18.0) };

    let mut content = div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2p5()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(10.0))
                .text_color(theme.foreground_muted().opacity(0.7))
                .child(format!("#{}", idx + 1)),
        );

    if item.is_image {
        let thumbnail_element = if let Some(ref path) = item.image_path {
            if path.exists() {
                div()
                    .w(px(56.0))
                    .h(px(40.0))
                    .rounded(px(8.0))
                    .overflow_hidden()
                    .bg(theme.surface().opacity(0.6))
                    .border_1()
                    .border_color(theme.surface().opacity(0.35))
                    .flex_shrink_0()
                    .child(
                        img(path.clone())
                            .size_full()
                            .object_fit(gpui::ObjectFit::Cover),
                    )
                    .into_any_element()
            } else {
                render_fallback_image_icon(theme)
            }
        } else {
            render_fallback_image_icon(theme)
        };

        let mut text_column = div()
            .flex()
            .flex_col()
            .flex_1()
            .overflow_hidden()
            .gap_0p5()
            .child(
                div()
                    .font_weight(if is_selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_size(px(13.0))
                    .text_color(if is_selected {
                        theme.foreground()
                    } else {
                        theme.foreground_muted()
                    })
                    .truncate()
                    .child(image_title_text.to_string()),
            );

        if !item.preview.is_empty() {
            text_column = text_column.child(
                div()
                    .text_size(px(10.5))
                    .text_color(theme.foreground_muted().opacity(0.75))
                    .truncate()
                    .child(item.preview.clone()),
            );
        }

        content = content.child(thumbnail_element).child(text_column);
    } else {
        content = content.child(
            div()
                .font_weight(if is_selected {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_size(px(13.0))
                .text_color(if is_selected {
                    theme.foreground()
                } else {
                    theme.foreground_muted()
                })
                .truncate()
                .child(if item.preview.is_empty() {
                    empty_item_text.to_string()
                } else {
                    item.preview.clone()
                }),
        );
    }

    let pin_button = if !item.is_image {
        Some(
            div()
                .id(format!("pin-clip-{idx}"))
                .flex()
                .items_center()
                .justify_center()
                .w(px(22.0))
                .h(px(22.0))
                .rounded_full()
                .cursor_pointer()
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.opacity(0.6))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.toggle_pin_history(&item_for_pin, cx);
                }))
                .child(
                    svg()
                        .path("pin.svg")
                        .size(px(12.0))
                        .text_color(if is_pinned {
                            theme.accent()
                        } else {
                            theme.foreground_muted().opacity(0.5)
                        }),
                ),
        )
    } else {
        None
    };

    div()
        .id(format!("clip-item-{idx}"))
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .gap_2p5()
        .px_2()
        .py(if item.is_image { px(5.0) } else { px(6.0) })
        .rounded(px(12.0))
        .cursor_pointer()
        .bg(item_bg)
        .hover(|s| s.bg(theme.surface().opacity(0.4)))
        .active(|s| s.bg(theme.surface().opacity(0.6)))
        .on_mouse_move(cx.listener(move |this, _, _, cx| {
            if !this.mouse_moved {
                this.mouse_moved = true;
            }
            if this.selected_index != idx {
                this.selected_index = idx;
                cx.notify();
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.service.copy_item(&item_clone);
            cx.emit(ClipboardEvent::Close);
        }))
        .child(
            div()
                .w(px(3.0))
                .h(indicator_height)
                .rounded_full()
                .bg(indicator_color),
        )
        .child(content)
        .children(pin_button)
}
