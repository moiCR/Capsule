use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Render, ScrollHandle, Window,
    div, prelude::*, px, svg,
};
use services::{AppState, ShelfItem, ShelfService};
use ui::theme::Theme;

use crate::capsule::widgets::shelf::render_shelf_card;

pub enum ShelfEvent {
    Close,
    ItemCopied(ShelfItem),
}

pub struct ShelfModule {
    pub service: ShelfService,
    pub items: Vec<ShelfItem>,
    pub selected_index: usize,
    pub focus_handle: FocusHandle,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<ShelfEvent> for ShelfModule {}

impl ShelfModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = if cx.has_global::<AppState>() {
            cx.global::<AppState>().shelf.clone()
        } else {
            ShelfService::new()
        };

        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let items = service.get_items();

        Self {
            service,
            items,
            selected_index: 0,
            focus_handle,
            scroll_handle,
        }
    }

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        self.items = self.service.get_items();
        if self.items.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.items.len() {
            self.selected_index = self.items.len().saturating_sub(1);
        }
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn select_next(&mut self, cx: &mut Context<Self>) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    pub fn select_prev(&mut self, cx: &mut Context<Self>) {
        if !self.items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.items.len().saturating_sub(1);
            } else {
                self.selected_index = self.selected_index.saturating_sub(1);
            }
            self.scroll_handle.scroll_to_item(self.selected_index);
            cx.notify();
        }
    }

    pub fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = self.items.get(self.selected_index) {
            let id = item.id.clone();
            self.service.remove_item(&id);
            self.reload_items(cx);
            if self.items.is_empty() {
                cx.emit(ShelfEvent::Close);
            }
        }
    }

    pub fn copy_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(item) = self.items.get(self.selected_index) {
            let item_clone = item.clone();
            self.service.copy_item(&item_clone);
            cx.emit(ShelfEvent::ItemCopied(item_clone));
            cx.notify();
        }
    }

    pub fn copy_all_items(&mut self, cx: &mut Context<Self>) {
        if self.service.copy_all() {
            cx.notify();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" => {
                cx.emit(ShelfEvent::Close);
            }
            "left" | "up" => {
                self.select_prev(cx);
            }
            "right" | "down" => {
                self.select_next(cx);
            }
            "delete" | "backspace" => {
                self.remove_selected(cx);
            }
            "enter" => {
                self.copy_selected(cx);
            }
            "c" if event.keystroke.modifiers.control => {
                self.copy_selected(cx);
            }
            "a" if event.keystroke.modifiers.control => {
                self.copy_all_items(cx);
            }
            "d" if event.keystroke.modifiers.control => {
                self.copy_all_items(cx);
            }
            "d" => {
                self.copy_selected(cx);
            }
            _ => {}
        }
    }
}

impl Render for ShelfModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let selected_index = self.selected_index;

        let content = if self.items.is_empty() {
            let (empty_title, empty_hint) = if cx.has_global::<AppState>() {
                let lang = &cx.global::<AppState>().language;
                (lang.get("shelf.empty_title"), lang.get("shelf.empty_hint"))
            } else {
                (
                    "Drag files or folders onto Capsule".to_string(),
                    "Drag a file to reorder it or drop it into another application".to_string(),
                )
            };

            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(6.0))
                .child(
                    svg()
                        .path("cloud-upload.svg")
                        .size(px(26.0))
                        .text_color(theme.foreground_muted()),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(3.0))
                        .child(
                            div()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_size(px(12.5))
                                .text_color(theme.foreground())
                                .child(empty_title),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(theme.foreground_muted())
                                .child(empty_hint),
                        ),
                )
                .into_any_element()
        } else {
            let mut list = div()
                .id("shelf-items-list")
                .track_scroll(&self.scroll_handle)
                .size_full()
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .overflow_x_scroll()
                .gap(px(32.0))
                .px(px(24.0));

            for (idx, item) in self.items.iter().enumerate() {
                list = list.child(render_shelf_card(
                    idx,
                    item,
                    idx == selected_index,
                    &theme,
                    cx,
                ));
            }

            list.into_any_element()
        };

        div()
            .id("shelf-module-view")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .on_click(cx.listener(|_, _, _window, cx| {
                cx.stop_propagation();
            }))
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(content)
    }
}
