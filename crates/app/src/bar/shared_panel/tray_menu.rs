use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{
    App, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render,
    Size, Window, div, img, px,
};
use services::AppState;
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub struct TrayModule {
    pub index: usize,
    focus: FocusHandle,
}

impl TrayModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            index: 0,
            focus: cx.focus_handle(),
        }
    }
}

impl EventEmitter<SharedPanelEvent> for TrayModule {}

impl SharedPanelItem for TrayModule {
    fn size(&self, cx: &App) -> Size<Pixels> {
        let items_count = if let Some(state) = cx.try_global::<AppState>() {
            let items = state.sni_host.items_arc();
            items
                .get(self.index)
                .map(|i| i.menu_items.len())
                .unwrap_or(0)
        } else {
            0
        };
        let h = (52.0 + items_count as f32 * 38.0).clamp(150.0, 440.0);
        Size {
            width: px(310.0),
            height: px(h),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

impl Render for TrayModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let idx = self.index;
        let theme = cx.global::<Theme>().clone();
        let items = if let Some(state) = cx.try_global::<AppState>() {
            state.sni_host.items_arc()
        } else {
            std::sync::Arc::new(Vec::new())
        };
        let Some(item) = items.get(idx) else {
            return div()
                .id("shared-panel-tray-empty")
                .track_focus(&self.focus)
                .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                    if event.keystroke.key == "escape" {
                        cx.emit(SharedPanelEvent::Close);
                    }
                }))
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.5))
                .text_color(theme.foreground_muted())
                .child("No item")
                .into_any_element();
        };
        let bus_name = item.bus_name.clone();
        let menu_path = item.menu_path.clone().unwrap_or_default();

        let title = if item.title.is_empty() {
            item.id.clone()
        } else {
            item.title.clone()
        };

        let icon_el = if let Some(path) = &item.icon_file_path {
            img(PathBuf::from(path)).size(px(20.0)).into_any_element()
        } else {
            div()
                .text_size(px(13.5))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(theme.foreground())
                .child(title.chars().next().unwrap_or('?').to_string())
                .into_any_element()
        };

        let mut menu_container = div()
            .id("shared-panel-tray-list")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(3.0))
            .p(px(8.0));

        if item.menu_items.is_empty() {
            let hover_bg = style::hover(&theme);
            menu_container = menu_container.child(
                div()
                    .id("shared-tray-open-default")
                    .flex()
                    .items_center()
                    .h(px(38.0))
                    .flex_shrink_0()
                    .px(px(12.0))
                    .rounded_xl()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .text_size(px(14.0))
                    .text_color(theme.foreground())
                    .child("Abrir")
                    .on_click(cx.listener(move |_, _, _, cx| {
                        if let Some(state) = cx.try_global::<AppState>() {
                            state.sni_host.activate_item(idx);
                        }
                        cx.emit(SharedPanelEvent::Close);
                    })),
            );
        } else {
            for (item_i, menu_item) in item.menu_items.iter().enumerate() {
                if menu_item.is_separator {
                    menu_container = menu_container.child(
                        div()
                            .h(px(1.0))
                            .flex_shrink_0()
                            .my(px(4.0))
                            .w_full()
                            .bg(theme.outline_variant()),
                    );
                } else {
                    let id = menu_item.id;
                    let enabled = menu_item.enabled;
                    let bus = bus_name.clone();
                    let m_path = menu_path.clone();
                    let clean_label = menu_item.label.replace('_', "");
                    let hover_bg = theme.hover_layer(theme.surface_container_high());

                    let mut row = div()
                        .id(("tray-item-row", item_i))
                        .flex()
                        .items_center()
                        .h(px(34.0))
                        .flex_shrink_0()
                        .px(px(10.0))
                        .rounded(px(10.0))
                        .text_size(px(13.0));

                    if enabled {
                        row = row
                            .text_color(theme.foreground())
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover_bg))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                if let Some(state) = cx.try_global::<AppState>() {
                                    state.sni_host.trigger_menu(bus.clone(), m_path.clone(), id);
                                }
                                cx.emit(SharedPanelEvent::Close);
                            }));
                    } else {
                        row = row.text_color(theme.foreground_muted().opacity(0.5));
                    }

                    row = row.child(div().line_clamp(1).child(clean_label));
                    menu_container = menu_container.child(row);
                }
            }
        }

        div()
            .id("shared-panel-tray-menu")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SharedPanelEvent::Close);
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .font_family(theme.font_family())
            .child(
                div()
                    .h(px(44.0))
                    .flex_shrink_0()
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .border_b_1()
                    .border_color(style::border(&theme).opacity(0.16))
                    .child(icon_el)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(14.5))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .line_clamp(1)
                            .child(title),
                    ),
            )
            .child(menu_container)
            .into_any_element()
    }
}
