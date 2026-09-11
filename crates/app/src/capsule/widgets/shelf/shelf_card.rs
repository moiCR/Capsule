use gpui::{
    Context, ExternalDragPayload, FileDragPaths, FontWeight, IntoElement, Pixels, Point, Render,
    Window, div, img, prelude::*, px, svg,
};
use services::{AppState, ShelfItem};
use ui::theme::Theme;

use crate::capsule::modules::shelf::{ShelfEvent, ShelfModule};

#[derive(Clone)]
pub struct DraggedShelfItem {
    pub path_str: String,
}

impl Render for DraggedShelfItem {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub fn render_shelf_card(
    idx: usize,
    item: &ShelfItem,
    is_selected: bool,
    theme: &Theme,
    cx: &mut Context<ShelfModule>,
) -> impl IntoElement {
    let item_clone = item.clone();
    let item_id = item.id.clone();
    let icon_path = item.get_icon_path();
    let drag_payload = DraggedShelfItem {
        path_str: item.path.to_string_lossy().to_string(),
    };

    let card_round = if cx.has_global::<AppState>() {
        cx.global::<AppState>().config.get().ui.cards_round
    } else {
        8.0
    };

    let preview_icon = if item.is_image && item.path.exists() {
        div()
            .w(px(44.0))
            .h(px(44.0))
            .rounded(px(card_round))
            .overflow_hidden()
            .border_1()
            .border_color(theme.surface().opacity(0.3))
            .child(
                img(item.path.clone())
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover),
            )
            .into_any_element()
    } else if let Some(icon) = icon_path {
        div()
            .w(px(44.0))
            .h(px(48.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                img(icon)
                    .size(px(42.0))
                    .object_fit(gpui::ObjectFit::Contain),
            )
            .into_any_element()
    } else if item.is_dir {
        div()
            .w(px(44.0))
            .h(px(48.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path("folder.svg")
                    .size(px(42.0))
                    .text_color(theme.accent()),
            )
            .into_any_element()
    } else {
        div()
            .w(px(40.0))
            .h(px(48.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path("file-doc.svg")
                    .w(px(40.0))
                    .h(px(48.0))
                    .text_color(theme.foreground()),
            )
            .into_any_element()
    };

    let remove_btn_id = item.id.clone();
    let group_id = format!("shelf-card-{idx}");

    let item_opacity = if is_selected { 1.0 } else { 0.9 };

    div()
        .id(group_id.clone())
        .group(group_id.clone())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .w(px(84.0))
        .h(px(86.0))
        .cursor_grab()
        .opacity(item_opacity)
        .on_click(cx.listener({
            let item = item_clone.clone();
            move |this, _, _window, cx| {
                this.selected_index = idx;
                this.service.copy_item(&item);
                cx.emit(ShelfEvent::ItemCopied(item.clone()));
                cx.stop_propagation();
                cx.notify();
            }
        }))
        .on_drag(
            drag_payload,
            move |dragged: &DraggedShelfItem, _: Point<Pixels>, _, cx| {
                let d = dragged.clone();
                cx.new(|_| d)
            },
        )
        .immediate_external_drag_payload(|dragged: &DraggedShelfItem, _window, _cx| {
            let path = std::path::PathBuf::from(&dragged.path_str);
            let is_dir = path.is_dir();
            Some(ExternalDragPayload::Files(FileDragPaths::new([(
                path, is_dir,
            )])))
        })
        .child(
            div()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .child(preview_icon)
                .child(
                    div()
                        .id(format!("shelf-remove-{remove_btn_id}"))
                        .absolute()
                        .top(px(-4.0))
                        .right(px(-6.0))
                        .size(px(16.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(theme.surface())
                        .border_1()
                        .border_color(theme.surface().opacity(0.8))
                        .cursor_pointer()
                        .opacity(if is_selected { 1.0 } else { 0.0 })
                        .group_hover(group_id, |s| s.opacity(1.0))
                        .hover(|s| s.bg(theme.background_alt()).border_color(theme.accent()))
                        .on_click(cx.listener({
                            let id = item_id.clone();
                            move |this, _, _window, cx| {
                                this.service.remove_item(&id);
                                this.reload_items(cx);
                                cx.stop_propagation();
                            }
                        }))
                        .child(
                            svg()
                                .path("close.svg")
                                .size(px(8.0))
                                .text_color(theme.foreground()),
                        ),
                ),
        )
        .child(
            div()
                .mt(px(6.0))
                .w(px(80.0))
                .overflow_hidden()
                .text_ellipsis()
                .text_center()
                .font_weight(FontWeight::NORMAL)
                .text_size(px(11.0))
                .text_color(theme.foreground())
                .child(item.name.clone()),
        )
}
