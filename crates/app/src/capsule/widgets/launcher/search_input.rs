use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::modules::launcher::LauncherModule;

pub fn render_search_input(
    query: &str,
    apps_count: usize,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let placeholder = if query.is_empty() {
        lang.launcher.search_placeholder
    } else {
        String::new()
    };

    div()
        .flex()
        .items_center()
        .gap_2p5()
        .w_full()
        .px_3p5()
        .py_2p5()
        .bg(theme.surface())
        .rounded(px(42.0))
        .child(
            svg()
                .path("search.svg")
                .w_4()
                .h_4()
                .text_color(theme.foreground_muted()),
        )
        .child(div().flex_1().text_sm().child(if query.is_empty() {
            div()
                .text_color(theme.foreground_muted())
                .child(placeholder)
        } else {
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child(query.to_string())
        }))
        .child(if !query.is_empty() {
            div()
                .id("clear-search-btn")
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.reset_search(cx);
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .w_3p5()
                        .h_3p5()
                        .text_color(theme.foreground_muted()),
                )
                .into_any_element()
        } else {
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.foreground_muted())
                .child(format!("{apps_count}"))
                .into_any_element()
        })
}
