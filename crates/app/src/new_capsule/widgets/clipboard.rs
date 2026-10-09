use crate::new_capsule::module::{
    CapsuleModuleEvent,
    clipboard::{ClipboardModule, ClipboardTab},
};
use crate::new_capsule::widgets::style;
use gpui::{
    AnyElement, Context, IntoElement, MotionDurationExt, StyledImage, div, ease_in_out, img,
    prelude::*, px, svg,
};
use ui::theme::Theme;

pub const WIDTH: f32 = 480.0;
pub const COLUMNS: usize = 2;
pub const CARD_HEIGHT: f32 = 144.0;
pub const GRID_GAP: f32 = 10.0;
const PADDING: f32 = 16.0;
const GAP: f32 = 10.0;
const HEADER_HEIGHT: f32 = 32.0;
const SEARCH_HEIGHT: f32 = 40.0;
const TABS_HEIGHT: f32 = 28.0;
const FOOTER_HEIGHT: f32 = 18.0;
pub const BASE_HEIGHT: f32 =
    PADDING * 2.0 + HEADER_HEIGHT + SEARCH_HEIGHT + TABS_HEIGHT + FOOTER_HEIGHT + GAP * 4.0;
pub const MAX_HEIGHT: f32 = BASE_HEIGHT + CARD_HEIGHT * 2.0 + GRID_GAP;
const CARD_WIDTH: f32 = (WIDTH - PADDING * 2.0 - GRID_GAP) / COLUMNS as f32;

#[derive(Clone, Copy)]
enum ItemAction {
    Pin(usize),
    Remove(usize),
    Clear,
}

fn action(
    id: String,
    icon: &'static str,
    enabled: bool,
    highlighted: bool,
    theme: &Theme,
    cx: &mut Context<ClipboardModule>,
    action: ItemAction,
) -> gpui::Stateful<gpui::Div> {
    let hover = style::hover(theme);
    div()
        .id(id)
        .size(px(26.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .opacity(if enabled { 1.0 } else { 0.3 })
        .when(enabled, |s| {
            s.cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |module, _, _, cx| {
                    cx.stop_propagation();
                    match action {
                        ItemAction::Pin(index) => module.pin(index, cx),
                        ItemAction::Remove(index) => module.remove(index, cx),
                        ItemAction::Clear => module.clear(cx),
                    }
                }))
        })
        .child(svg().path(icon).size(px(13.0)).text_color(if highlighted {
            theme.accent()
        } else {
            theme.foreground_muted()
        }))
}

pub(crate) fn render(
    module: &ClipboardModule,
    theme: &Theme,
    cx: &mut Context<ClipboardModule>,
) -> gpui::Stateful<gpui::Div> {
    let config = cx.global::<services::AppState>().config.get();
    let duration = std::time::Duration::from_millis(config.ui.animation_duration_ms as u64);
    let preview_radius = px(style::INNER_RADIUS - 2.0);
    let hover = style::hover(theme);
    let mut header = div()
        .h(px(HEADER_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(19.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(module.text("clipboard.title", cx)),
        );
    let mut tabs = div()
        .h(px(TABS_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(8.0));
    for (tab, id, label) in [
        (
            ClipboardTab::History,
            "clipboard-history-tab",
            "clipboard.history_tab",
        ),
        (
            ClipboardTab::Snippets,
            "clipboard-snippets-tab",
            "clipboard.snippets_tab",
        ),
    ] {
        let active = module.model.tab == tab;
        tabs = tabs.child(
            div()
                .id(id)
                .h(px(TABS_HEIGHT))
                .px(px(8.0))
                .rounded_full()
                .flex()
                .items_center()
                .cursor_pointer()
                .bg(if active {
                    theme.accent()
                } else {
                    style::surface(theme)
                })
                .text_color(if active {
                    style::on_accent(theme)
                } else {
                    theme.foreground_muted()
                })
                .on_click(cx.listener(move |module, _, _, cx| module.switch_tab(tab, cx)))
                .child(div().text_size(px(12.0)).child(module.text(label, cx))),
        );
    }
    header = header
        .child(div().flex_1())
        .when(module.model.tab == ClipboardTab::History, |s| {
            s.child(action(
                "clipboard-clear".into(),
                "trash.svg",
                !module.busy && !module.model.items.is_empty(),
                false,
                theme,
                cx,
                ItemAction::Clear,
            ))
        })
        .child(
            div()
                .id("clipboard-close")
                .size(px(26.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(style::surface(theme))
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(|_, _, _, cx| cx.emit(CapsuleModuleEvent::Close)))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(13.0))
                        .text_color(theme.foreground_muted()),
                ),
        );
    let query = if module.model.query.is_empty() {
        module.text("clipboard.search_placeholder", cx)
    } else {
        module.model.query.clone()
    };
    let search = div()
        .h(px(SEARCH_HEIGHT))
        .flex_shrink_0()
        .px(px(12.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .rounded_full()
        .bg(style::surface(theme))
        .child(
            svg()
                .path("search.svg")
                .size(px(14.0))
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(13.0))
                .text_ellipsis()
                .text_color(if module.model.query.is_empty() {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .child(query),
        );
    let mut gallery = div()
        .id("clipboard-gallery")
        .w_full()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(GRID_GAP))
        .overflow_y_scroll()
        .track_scroll(&module.scroll);
    if module.model.filtered.is_empty() {
        gallery = gallery.items_center().justify_center().child(
            div()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(module.text(
                    if module.loading {
                        "dashboard_new.loading"
                    } else if !module.model.query.is_empty() {
                        "clipboard.no_results"
                    } else if module.model.tab == ClipboardTab::History {
                        "clipboard.empty_history"
                    } else {
                        "clipboard.empty_snippets"
                    },
                    cx,
                )),
        );
    }
    for (row_index, pair) in module.model.filtered.chunks(COLUMNS).enumerate() {
        let mut row = div()
            .id(("clipboard-gallery-row", row_index))
            .w_full()
            .h(px(CARD_HEIGHT))
            .min_h(px(CARD_HEIGHT))
            .flex_shrink_0()
            .flex()
            .justify_center()
            .gap(px(GRID_GAP));
        for (column, &source) in pair.iter().enumerate() {
            let index = row_index * COLUMNS + column;
            let selected = index == module.model.selected;
            let (id, content): (String, AnyElement) = match module.model.tab {
                ClipboardTab::History => {
                    let item = &module.model.items[source];
                    let content = if item.is_image {
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .w(px(CARD_WIDTH - 4.0))
                                    .h(px(104.0))
                                    .flex_shrink_0()
                                    .rounded_tl(preview_radius)
                                    .rounded_tr(preview_radius)
                                    .bg(theme.surface())
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when_some(module.images.get(&item.id), |s, image| {
                                        s.child(
                                            img(image.clone())
                                                .w(px(CARD_WIDTH - 4.0))
                                                .h(px(104.0))
                                                .aspect_ratio((CARD_WIDTH - 4.0) / 104.0)
                                                .rounded_tl(preview_radius)
                                                .rounded_tr(preview_radius)
                                                .object_fit(gpui::ObjectFit::Cover),
                                        )
                                    })
                                    .when(!module.images.contains_key(&item.id), |s| {
                                        s.child(
                                            svg()
                                                .path("wallpaper.svg")
                                                .size(px(24.0))
                                                .text_color(theme.foreground_muted()),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .px(px(10.0))
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        svg()
                                            .path("wallpaper.svg")
                                            .size(px(12.0))
                                            .flex_shrink_0()
                                            .text_color(theme.foreground_muted()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .text_size(px(10.0))
                                            .whitespace_nowrap()
                                            .text_ellipsis()
                                            .child(if item.preview.is_empty() {
                                                module.text("clipboard.image_item", cx)
                                            } else {
                                                item.preview.clone()
                                            }),
                                    ),
                            )
                    } else {
                        div()
                            .size_full()
                            .p(px(12.0))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .h(px(22.0))
                                    .flex_shrink_0()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        svg()
                                            .path("file-text.svg")
                                            .size(px(13.0))
                                            .text_color(theme.foreground_muted()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_size(px(10.0))
                                            .text_color(theme.foreground_muted())
                                            .child(format!("{:02}", index + 1)),
                                    )
                                    .child(action(
                                        format!("clipboard-pin-{}", item.id),
                                        "pin.svg",
                                        !module.busy,
                                        module.pinned.contains(&item.id),
                                        theme,
                                        cx,
                                        ItemAction::Pin(index),
                                    )),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .w_full()
                                    .overflow_hidden()
                                    .text_size(px(13.0))
                                    .line_height(px(19.0))
                                    .child(if item.preview.is_empty() {
                                        module.text("clipboard.empty_item", cx)
                                    } else {
                                        item.preview.clone()
                                    }),
                            )
                    };
                    (
                        format!("clipboard-card-{}", item.id),
                        content.into_any_element(),
                    )
                }
                ClipboardTab::Snippets => {
                    let snippet = &module.model.snippets[source];
                    let content = div()
                        .size_full()
                        .p(px(12.0))
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(
                            div()
                                .h(px(22.0))
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .child(
                                    svg()
                                        .path("pin.svg")
                                        .size(px(13.0))
                                        .flex_shrink_0()
                                        .text_color(theme.accent()),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .text_size(px(11.0))
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .child(snippet.title.clone()),
                                )
                                .child(action(
                                    format!("clipboard-remove-{}", snippet.id),
                                    "close.svg",
                                    !module.busy,
                                    false,
                                    theme,
                                    cx,
                                    ItemAction::Remove(index),
                                )),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .overflow_hidden()
                                .text_size(px(13.0))
                                .line_height(px(19.0))
                                .child(snippet.content.clone()),
                        );
                    (
                        format!("clipboard-card-snippet-{}", snippet.id),
                        content.into_any_element(),
                    )
                }
            };
            row = row.child(
                div()
                    .id(id)
                    .w(px(CARD_WIDTH))
                    .h(px(CARD_HEIGHT))
                    .min_h(px(CARD_HEIGHT))
                    .flex_shrink_0()
                    .rounded(px(style::INNER_RADIUS))
                    .overflow_hidden()
                    .border_2()
                    .border_color(if selected {
                        theme.accent()
                    } else {
                        style::border(theme)
                    })
                    .bg(if selected {
                        style::selected(theme)
                    } else {
                        style::surface(theme)
                    })
                    .transitions(|t| t.bg(duration.with_easing(ease_in_out)))
                    .when(!module.busy, |s| {
                        s.cursor_pointer()
                            .hover(move |s| s.bg(hover))
                            .on_mouse_move(
                                cx.listener(move |module, _, _, cx| {
                                    module.select(index, false, cx)
                                }),
                            )
                            .on_click(cx.listener(move |module, _, _, cx| module.copy(index, cx)))
                    })
                    .child(content),
            );
        }
        gallery = gallery.child(row);
    }
    let footer = if let Some(error) = &module.error {
        error.clone()
    } else if module.busy {
        module.text("clipboard.working", cx)
    } else {
        module.text("clipboard.copy_hint", cx)
    };
    div()
        .id("clipboard-module")
        .size_full()
        .p(px(PADDING))
        .flex()
        .flex_col()
        .gap(px(GAP))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(header)
        .child(search)
        .child(tabs)
        .child(gallery)
        .child(
            div()
                .h(px(FOOTER_HEIGHT))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(10.0))
                        .text_ellipsis()
                        .text_color(if module.error.is_some() {
                            theme.red()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(footer),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(theme.foreground_muted())
                        .child(module.model.filtered.len().to_string()),
                ),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_full_rows_fit_below_the_header_search_tabs_and_above_the_footer() {
        let controls =
            PADDING * 2.0 + HEADER_HEIGHT + SEARCH_HEIGHT + TABS_HEIGHT + FOOTER_HEIGHT + GAP * 4.0;
        assert_eq!(controls, BASE_HEIGHT);
        assert_eq!(MAX_HEIGHT - controls, CARD_HEIGHT * 2.0 + GRID_GAP);
        assert_eq!(
            CARD_WIDTH * COLUMNS as f32 + GRID_GAP + PADDING * 2.0,
            WIDTH
        );
        assert_eq!(CARD_HEIGHT - 4.0 - 104.0, 36.0);
    }
}
