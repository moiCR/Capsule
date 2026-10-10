use super::style;
use crate::new_capsule::module::{
    CapsuleModuleEvent,
    emoji::{
        EmojiModule,
        model::{COLUMNS, PAGE_SIZE, ROWS},
    },
};
use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use services::AppState;
use ui::theme::Theme;

pub const WIDTH: f32 = 440.0;
pub const HEIGHT: f32 = 430.0;
const CELL_WIDTH: f32 = 54.0;
const CELL_HEIGHT: f32 = 48.0;
const GRID_HEIGHT: f32 = ROWS as f32 * CELL_HEIGHT + (ROWS - 1) as f32 * 6.0;
const CATEGORIES: &[(Option<&str>, &str)] = &[
    (None, ""),
    (Some("people"), "😊"),
    (Some("animals"), "🐾"),
    (Some("nature"), "🌿"),
    (Some("food"), "🍔"),
    (Some("activity"), "⚽"),
    (Some("travel"), "✈️"),
    (Some("objects"), "💡"),
    (Some("symbols"), "🔣"),
    (Some("flags"), "🏁"),
];

pub fn render(module: &EmojiModule, cx: &mut Context<EmojiModule>) -> gpui::Stateful<gpui::Div> {
    let theme = cx.global::<Theme>().clone();
    let language = &cx.global::<AppState>().language;
    let placeholder = language.get("emoji.search_placeholder");
    let empty = language.get("emoji.no_emojis");
    let query = &module.model.query;
    let search = div()
        .id("emoji-search")
        .h(px(40.0))
        .flex_1()
        .min_w_0()
        .px(px(12.0))
        .rounded_full()
        .bg(style::surface(&theme))
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            svg()
                .path("search.svg")
                .size(px(16.0))
                .text_color(theme.foreground_muted()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(13.0))
                .text_ellipsis()
                .text_color(if query.is_empty() {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .child(if query.is_empty() {
                    placeholder
                } else {
                    query.clone()
                }),
        )
        .when(!query.is_empty(), |element| {
            element.child(
                style::circle_button("emoji-search-clear", "close.svg", 22.0, &theme)
                    .on_click(cx.listener(|module, _, _, cx| module.search(String::new(), cx))),
            )
        });
    let categories = div()
        .h(px(32.0))
        .flex_shrink_0()
        .w_full()
        .flex()
        .gap(px(4.0))
        .children(
            CATEGORIES
                .iter()
                .enumerate()
                .map(|(index, &(category, label))| {
                    let selected = module.model.category == category;
                    let content = if category.is_none() {
                        svg()
                            .path("sparkles.svg")
                            .size(px(16.0))
                            .text_color(if selected {
                                style::on_accent(&theme)
                            } else {
                                theme.foreground_muted()
                            })
                            .into_any_element()
                    } else {
                        div()
                            .font_family("Noto Color Emoji")
                            .text_size(px(17.0))
                            .line_height(px(22.0))
                            .child(label)
                            .into_any_element()
                    };
                    div()
                        .id(("emoji-category", index))
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .rounded(px(style::INNER_RADIUS))
                        .bg(if selected {
                            theme.accent()
                        } else {
                            style::surface(&theme)
                        })
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .hover(|s| s.bg(style::hover(&theme)))
                        .on_click(
                            cx.listener(move |module, _, _, cx| module.category(category, cx)),
                        )
                        .child(content)
                }),
        );
    let page_start = module.model.page() * PAGE_SIZE;
    let mut grid = div()
        .h(px(GRID_HEIGHT))
        .flex_shrink_0()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.0));
    if module.model.filtered.is_empty() {
        grid = grid
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path("search.svg")
                    .size(px(26.0))
                    .text_color(theme.foreground_muted()),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.foreground_muted())
                    .child(empty),
            );
    } else {
        let page_end = (page_start + PAGE_SIZE).min(module.model.filtered.len());
        let items = module
            .model
            .filtered
            .get(page_start..page_end)
            .unwrap_or(&[]);
        for (row_index, row_items) in items.chunks(COLUMNS).enumerate() {
            let mut row = div().h(px(CELL_HEIGHT)).flex_shrink_0().flex().gap(px(5.0));
            for (column, item) in row_items.iter().enumerate() {
                let index = page_start + row_index * COLUMNS + column;
                let selected = module.model.selected == index;
                row = row.child(
                    div()
                        .id(("emoji-cell", index))
                        .w(px(CELL_WIDTH))
                        .h(px(CELL_HEIGHT))
                        .flex_shrink_0()
                        .rounded(px(style::INNER_RADIUS))
                        .border_1()
                        .border_color(if selected {
                            theme.accent()
                        } else {
                            style::surface(&theme)
                        })
                        .bg(if selected {
                            style::selected(&theme)
                        } else {
                            style::surface(&theme)
                        })
                        .cursor_pointer()
                        .hover(|s| s.bg(style::hover(&theme)))
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_mouse_move(
                            cx.listener(move |module, _, _, cx| module.select(index, cx)),
                        )
                        .on_click(cx.listener(move |module, _, _, cx| {
                            module.select(index, cx);
                            module.copy_selected(cx);
                        }))
                        .child(
                            div()
                                .font_family("Noto Color Emoji")
                                .text_size(px(24.0))
                                .line_height(px(28.0))
                                .child(item.emoji.clone()),
                        ),
                );
            }
            grid = grid.child(row);
        }
    }
    let footer = div()
        .h(px(32.0))
        .flex_shrink_0()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .when_some(module.model.selected_item(), |element, item| {
                    element
                        .child(
                            div()
                                .font_family("Noto Color Emoji")
                                .text_size(px(20.0))
                                .line_height(px(24.0))
                                .child(item.emoji.clone()),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(11.0))
                                .text_ellipsis()
                                .text_color(theme.foreground_muted())
                                .child(item.name.clone()),
                        )
                }),
        )
        .child(
            style::circle_button("emoji-page-previous", "chevron-left.svg", 26.0, &theme)
                .on_click(cx.listener(|module, _, _, cx| module.navigate("pageup", cx))),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .child(format!(
                    "{}/{}",
                    module.model.page() + 1,
                    module.model.pages()
                )),
        )
        .child(
            style::circle_button("emoji-page-next", "chevron-right.svg", 26.0, &theme)
                .on_click(cx.listener(|module, _, _, cx| module.navigate("pagedown", cx))),
        );
    div()
        .id("emoji-module")
        .size_full()
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .h(px(40.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(search)
                .child(
                    style::circle_button("emoji-close", "close.svg", 30.0, &theme)
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(CapsuleModuleEvent::Close))),
                ),
        )
        .child(categories)
        .child(grid)
        .child(footer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_fit_five_complete_rows_with_search_categories_and_footer() {
        assert_eq!(
            WIDTH,
            32.0 + COLUMNS as f32 * CELL_WIDTH + (COLUMNS - 1) as f32 * 5.0
        );
        assert_eq!(HEIGHT, 32.0 + 40.0 + 32.0 + GRID_HEIGHT + 32.0 + 3.0 * 10.0);
    }
}
