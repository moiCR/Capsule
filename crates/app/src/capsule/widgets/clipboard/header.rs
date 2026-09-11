use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::clipboard::{ClipboardModule, ClipboardTab};

pub fn render_header(
    query: &str,
    current_tab: ClipboardTab,
    theme: &Theme,
    cx: &mut Context<ClipboardModule>,
) -> impl IntoElement {
    let (search_placeholder, history_tab_text, snippets_tab_text) =
        if cx.has_global::<services::AppState>() {
            let lang = &cx.global::<services::AppState>().language;
            (
                lang.get("clipboard.search_placeholder"),
                lang.get("clipboard.history_tab"),
                lang.get("clipboard.snippets_tab"),
            )
        } else {
            (
                "Buscar en el historial...".to_string(),
                "Historial".to_string(),
                "Plantillas".to_string(),
            )
        };

    let has_query = !query.is_empty();
    let query_str = query.to_string();

    let search_bar = div()
        .flex()
        .items_center()
        .gap_3()
        .w_full()
        .px_3()
        .py_2()
        .child(
            svg()
                .path("search.svg")
                .w_4()
                .h_4()
                .text_color(theme.foreground_muted().opacity(0.8)),
        )
        .child(div().flex_1().text_sm().child(if has_query {
            div()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground())
                .child(query_str)
        } else {
            div()
                .text_color(theme.foreground_muted().opacity(0.7))
                .child(search_placeholder)
        }))
        .children(if has_query {
            Some(
                div()
                    .id("clear-search-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.6))
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.update_search(String::new(), cx);
                    }))
                    .child(
                        svg()
                            .path("close.svg")
                            .w_3()
                            .h_3()
                            .text_color(theme.foreground_muted()),
                    ),
            )
        } else {
            None
        })
        .children(if current_tab == ClipboardTab::History {
            Some(
                div()
                    .id("clip-clear-btn")
                    .flex()
                    .items_center()
                    .justify_center()
                    .w(px(22.0))
                    .h(px(22.0))
                    .rounded_full()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.surface().opacity(0.8)))
                    .active(|s| s.opacity(0.6))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear_all(cx);
                    }))
                    .child(
                        svg()
                            .path("trash.svg")
                            .size(px(13.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
        } else {
            None
        });

    let is_history = current_tab == ClipboardTab::History;
    let is_snippets = current_tab == ClipboardTab::Snippets;

    let tab_switcher =
        div()
            .flex()
            .items_center()
            .gap_1()
            .p(px(2.0))
            .rounded(px(10.0))
            .bg(theme.surface().opacity(0.35))
            .child(
                div()
                    .id("tab-history-btn")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2p5()
                    .py(px(4.0))
                    .rounded(px(8.0))
                    .cursor_pointer()
                    .bg(if is_history {
                        theme.surface().opacity(0.7)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| {
                        if !is_history {
                            s.bg(theme.surface().opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.switch_tab(ClipboardTab::History, cx);
                    }))
                    .child(svg().path("clipboard-list.svg").size(px(12.0)).text_color(
                        if is_history {
                            theme.foreground()
                        } else {
                            theme.foreground_muted()
                        },
                    ))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(if is_history {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_history {
                                theme.foreground()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(history_tab_text),
                    ),
            )
            .child(
                div()
                    .id("tab-snippets-btn")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2p5()
                    .py(px(4.0))
                    .rounded(px(8.0))
                    .cursor_pointer()
                    .bg(if is_snippets {
                        theme.surface().opacity(0.7)
                    } else {
                        gpui::hsla(0.0, 0.0, 0.0, 0.0)
                    })
                    .hover(|s| {
                        if !is_snippets {
                            s.bg(theme.surface().opacity(0.35))
                        } else {
                            s
                        }
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.switch_tab(ClipboardTab::Snippets, cx);
                    }))
                    .child(
                        svg()
                            .path("pin.svg")
                            .size(px(12.0))
                            .text_color(if is_snippets {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(if is_snippets {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_snippets {
                                theme.foreground()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(snippets_tab_text),
                    ),
            );

    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .w_full()
        .child(search_bar)
        .child(tab_switcher)
}
