use crate::new_capsule::module::shelf::ShelfModule;
use gpui::{
    ColorExt, Context, ExternalDragPayload, FileDragPaths, IntoElement, Pixels, Point, Render,
    Window, div, img, prelude::*, px, svg,
};
use services::{AppState, ShelfItem};
use ui::{components::button::IconButton, theme::Theme};

pub const COLUMNS: usize = 4;
pub const WIDTH: f32 = 480.0;
pub const CARD_HEIGHT: f32 = 112.0;
pub const GAP: f32 = 8.0;
pub const MAX_HEIGHT: f32 = 314.0;
const BASE_HEIGHT: f32 = 82.0;
const CARD_WIDTH: f32 = (WIDTH - 24.0 - GAP * 3.0) / COLUMNS as f32;

pub fn height(count: usize) -> f32 {
    let rows = count.div_ceil(COLUMNS).min(2);
    BASE_HEIGHT
        + if rows == 0 {
            80.0
        } else {
            rows as f32 * CARD_HEIGHT + rows.saturating_sub(1) as f32 * GAP
        }
}

pub fn next_selection(current: usize, count: usize, key: &str) -> usize {
    if count == 0 {
        return 0;
    }
    let current = current.min(count - 1);
    match key {
        "left" => current.saturating_sub(1),
        "right" => (current + 1).min(count - 1),
        "up" => current.saturating_sub(COLUMNS),
        "down" => (current + COLUMNS).min(count - 1),
        "home" => 0,
        "end" => count - 1,
        _ => current,
    }
}

#[derive(Clone)]
struct DraggedFile {
    path: std::path::PathBuf,
    is_dir: bool,
}
impl Render for DraggedFile {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size(px(36.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path(if self.is_dir {
                        "folder.svg"
                    } else {
                        "file-text.svg"
                    })
                    .size(px(24.0))
                    .text_color(cx.global::<Theme>().foreground()),
            )
    }
}

fn card(
    item: &ShelfItem,
    index: usize,
    module: &ShelfModule,
    theme: &Theme,
    cx: &mut Context<ShelfModule>,
) -> impl IntoElement {
    let selected = index == module.selected;
    let radius = cx
        .global::<AppState>()
        .config
        .get()
        .ui
        .cards_round
        .clamp(0.0, 12.0);
    let hover = theme.surface().opacity(0.45);
    let remove_id = item.id.clone();
    let payload = DraggedFile {
        path: item.path.clone(),
        is_dir: item.is_dir,
    };
    let preview = if let Some(image) = module.previews.get(&item.id) {
        img(image.clone())
            .w(px(84.0))
            .h(px(60.0))
            .aspect_ratio(84.0 / 60.0)
            .rounded(px((radius - 2.0).max(0.0)))
            .object_fit(gpui::ObjectFit::Cover)
            .into_any_element()
    } else if let Some(path) = &item.icon_path {
        img(path.clone())
            .size(px(40.0))
            .aspect_ratio(1.0)
            .object_fit(gpui::ObjectFit::Contain)
            .into_any_element()
    } else {
        svg()
            .path(if item.is_dir {
                "folder.svg"
            } else {
                "file-text.svg"
            })
            .size(px(36.0))
            .text_color(theme.foreground_muted())
            .into_any_element()
    };
    let description = if item.is_dir {
        cx.global::<AppState>().language.get("shelf.folder")
    } else {
        item.formatted_size()
    };
    div()
        .id(format!("shelf-file-{}", item.id))
        .relative()
        .w(px(CARD_WIDTH))
        .h(px(CARD_HEIGHT))
        .flex_shrink_0()
        .p(px(6.0))
        .rounded(px(radius))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .bg(if selected {
            theme.accent().opacity(0.13)
        } else {
            theme.surface().opacity(0.18)
        })
        .hover(move |s| s.bg(hover))
        .cursor_grab()
        .on_hover(cx.listener(move |module, &hovered, _, cx| {
            if hovered && module.mouse_moved {
                module.select(index, cx);
            }
        }))
        .when(!module.busy, |s| {
            s.on_click(cx.listener(move |module, _, _, cx| {
                module.select(index, cx);
                module.copy(false, cx);
            }))
        })
        .on_drag(payload, |file: &DraggedFile, _: Point<Pixels>, _, cx| {
            let file = file.clone();
            cx.new(|_| file)
        })
        .immediate_external_drag_payload(|file: &DraggedFile, _, _| {
            Some(ExternalDragPayload::Files(FileDragPaths::new([(
                file.path.clone(),
                file.is_dir,
            )])))
        })
        .child(
            div()
                .w_full()
                .h(px(60.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(preview),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .text_size(px(11.0))
                .text_ellipsis()
                .child(item.name.clone()),
        )
        .child(
            div()
                .w_full()
                .text_center()
                .text_size(px(10.0))
                .text_color(theme.foreground_muted())
                .text_ellipsis()
                .child(description),
        )
        .child(
            div().absolute().top(px(2.0)).right(px(2.0)).child(
                IconButton::new(format!("shelf-remove-{}", item.id), "close.svg")
                    .button_size(20.0)
                    .icon_size(10.0)
                    .on_click(cx.listener(move |module, _, _, cx| {
                        cx.stop_propagation();
                        module.remove(remove_id.clone(), cx);
                    })),
            ),
        )
}

pub fn render(module: &ShelfModule, cx: &mut Context<ShelfModule>) -> gpui::Stateful<gpui::Div> {
    let theme = cx.global::<Theme>().clone();
    let language = &cx.global::<AppState>().language;
    let count = language
        .get("shelf.items_count")
        .replace("{count}", &module.items.len().to_string());
    let footer = language.get(if module.error {
        "shelf.action_error"
    } else if module.busy {
        "shelf.working"
    } else if module.copied {
        "shelf.copied_hint"
    } else {
        "shelf.navigate_hint"
    });
    let empty = language.get("shelf.empty_title");
    let drop_background = theme.accent().opacity(0.08);
    let mut gallery = div()
        .id("shelf-gallery")
        .w_full()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(GAP))
        .overflow_y_scroll()
        .track_scroll(&module.scroll);
    for (row_index, items) in module.items.chunks(COLUMNS).enumerate() {
        let mut row = div()
            .id(("shelf-row", row_index))
            .w_full()
            .h(px(CARD_HEIGHT))
            .flex_shrink_0()
            .flex()
            .gap(px(GAP));
        for (column, item) in items.iter().enumerate() {
            row = row.child(card(item, row_index * COLUMNS + column, module, &theme, cx));
        }
        gallery = gallery.child(row);
    }
    div()
        .id("shelf-module")
        .size_full()
        .p(px(12.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .font_family(theme.font_family())
        .on_mouse_move(cx.listener(|module, _, _, _| module.mouse_moved = true))
        .drag_over::<gpui::ExternalPaths>(move |s, _, _, _| s.bg(drop_background))
        .on_drop(cx.listener(|module, paths: &gpui::ExternalPaths, _, cx| {
            cx.stop_propagation();
            module.add_paths(paths.paths().to_vec());
        }))
        .child(
            div()
                .w_full()
                .h(px(24.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child(count),
                )
                .when(!module.items.is_empty(), |s| {
                    s.child(
                        IconButton::new("shelf-copy-all", "pin.svg")
                            .on_click(cx.listener(|module, _, _, cx| module.copy(true, cx))),
                    )
                    .child(
                        IconButton::new("shelf-clear", "trash.svg")
                            .on_click(cx.listener(|module, _, _, cx| module.clear(cx))),
                    )
                })
                .child(
                    IconButton::new("shelf-close", "close.svg")
                        .on_click(cx.listener(|module, _, _, cx| module.close(cx))),
                ),
        )
        .child(if module.items.is_empty() {
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(empty)
                .into_any_element()
        } else {
            gallery.into_any_element()
        })
        .child(
            div()
                .h(px(18.0))
                .flex_shrink_0()
                .text_size(px(10.0))
                .text_ellipsis()
                .text_color(if module.error {
                    theme.red()
                } else {
                    theme.foreground_muted()
                })
                .child(footer),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_handles_partial_last_rows() {
        assert_eq!(next_selection(0, 0, "down"), 0);
        assert_eq!(next_selection(3, 6, "down"), 5);
        assert_eq!(next_selection(5, 6, "up"), 1);
        assert_eq!(next_selection(0, 6, "left"), 0);
        assert_eq!(next_selection(2, 6, "end"), 5);
    }
    #[test]
    fn shelf_height_is_bounded_and_matches_content() {
        assert_eq!(height(0), 162.0);
        assert_eq!(height(1), 194.0);
        assert_eq!(height(4), height(1));
        assert_eq!(height(5), MAX_HEIGHT);
        assert_eq!(height(100), MAX_HEIGHT);
    }
}
