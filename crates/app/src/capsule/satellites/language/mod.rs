use gpui::{Context, FontWeight, IntoElement, div, prelude::*, px, svg};
use ui::language::language_manager::{LanguageItem, LanguageManager};
use ui::theme::Theme;

use crate::capsule::modules::dashboard::DashboardModule;

pub fn render_language_widget(
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> impl IntoElement {
    let languages: Vec<LanguageItem> = if cx.has_global::<LanguageManager>() {
        cx.global::<LanguageManager>().list_languages()
    } else {
        Vec::new()
    };

    let lang = if cx.has_global::<ui::language::Language>() {
        cx.global::<ui::language::Language>().clone()
    } else {
        ui::language::Language::default()
    };

    let mut rows = div().flex().flex_col().gap_1p5().w_full();

    for item in languages {
        let is_selected = item.is_current;
        let item_lang = item.language.clone();

        let row =
            div()
                .id(format!("lang-item-{}", item.code))
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .rounded_xl()
                .cursor_pointer()
                .bg(if is_selected {
                    theme.accent().opacity(0.2)
                } else {
                    theme.surface().opacity(0.4)
                })
                .border_1()
                .border_color(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.0)
                })
                .hover(|s| {
                    if !is_selected {
                        s.bg(theme.surface().opacity(0.7))
                    } else {
                        s
                    }
                })
                .on_click(cx.listener(move |_this, _, _, cx| {
                    if cx.has_global::<LanguageManager>() {
                        cx.global_mut::<LanguageManager>()
                            .set_language(item_lang.clone());
                        cx.set_global(cx.global::<LanguageManager>().current_language.clone());
                    }
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(svg().path("languages.svg").size(px(14.0)).text_color(
                            if is_selected {
                                theme.accent()
                            } else {
                                theme.foreground_muted()
                            },
                        ))
                        .child(
                            div()
                                .font_weight(if is_selected {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::MEDIUM
                                })
                                .text_size(px(12.0))
                                .text_color(if is_selected {
                                    theme.foreground()
                                } else {
                                    theme.foreground_muted()
                                })
                                .child(item.name),
                        ),
                );

        rows = rows.child(row);
    }

    div()
        .flex()
        .flex_col()
        .w(px(200.0))
        .p_3()
        .gap_2()
        .rounded(px(20.0))
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_lg()
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(11.0))
                .text_color(theme.foreground_muted())
                .px_1()
                .child(lang.language_section.title),
        )
        .child(rows)
}
