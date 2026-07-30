use gpui::{Context, FontWeight, IntoElement, div, prelude::*};
use ui::theme::{Theme, theme_manager::ThemeItem};

use crate::capsule::modules::select_theme::SelectThemeModule;

pub fn render_theme_card(
    item: &ThemeItem,
    current_theme: &Theme,
    theme: &Theme,
    cx: &mut Context<SelectThemeModule>,
) -> impl IntoElement {
    let is_active = item.theme.accent_color.hex == current_theme.accent_color.hex
        && item.theme.background_color.hex == current_theme.background_color.hex;

    let item_theme = item.theme.clone();

    let card_bg = if is_active {
        theme.surface()
    } else {
        theme.background_alt()
    };

    let active_border = if is_active {
        theme.accent()
    } else {
        theme.surface().opacity(0.4)
    };

    let active_badge = if is_active {
        div()
            .px_2p5()
            .py_1()
            .rounded_full()
            .bg(theme.accent())
            .text_xs()
            .font_weight(FontWeight::BOLD)
            .text_color(theme.background())
            .child("Activo")
    } else {
        div()
    };

    let mode_label = match item.theme.mode {
        ui::theme::ThemeMode::Dark => "Oscuro",
        ui::theme::ThemeMode::Light => "Claro",
    };

    div()
        .id(format!("theme-item-{}", item.name))
        .flex()
        .items_center()
        .justify_between()
        .p_3()
        .rounded_xl()
        .bg(card_bg)
        .border_1()
        .border_color(active_border)
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.select_theme(item_theme.clone(), cx);
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme.foreground())
                                .child(item.name.clone()),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(theme.background())
                                .text_xs()
                                .text_color(theme.foreground_muted())
                                .child(mode_label),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .size_3p5()
                                .rounded_full()
                                .bg(item.theme.background())
                                .border_1()
                                .border_color(theme.surface()),
                        )
                        .child(
                            div()
                                .size_3p5()
                                .rounded_full()
                                .bg(item.theme.surface())
                                .border_1()
                                .border_color(theme.background()),
                        )
                        .child(div().size_3p5().rounded_full().bg(item.theme.accent()))
                        .child(div().size_3p5().rounded_full().bg(item.theme.green()))
                        .child(div().size_3p5().rounded_full().bg(item.theme.red())),
                ),
        )
        .child(active_badge)
}
