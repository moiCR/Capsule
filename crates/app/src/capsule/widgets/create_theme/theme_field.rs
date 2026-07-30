use gpui::{Context, FontWeight, Hsla, IntoElement, div, prelude::*};
use ui::theme::Theme;

use crate::capsule::modules::create_theme::CreateThemeModule;

pub fn render_theme_field(
    label: &'static str,
    val: &str,
    idx: usize,
    is_focused: bool,
    swatch_c: Hsla,
    theme: &Theme,
    cx: &mut Context<CreateThemeModule>,
) -> impl IntoElement {
    let field_bg = if is_focused {
        theme.surface()
    } else {
        theme.background_alt()
    };

    let border_c = if is_focused {
        theme.accent()
    } else {
        theme.surface().opacity(0.5)
    };

    div()
        .id(format!("field-idx-{idx}"))
        .flex()
        .items_center()
        .justify_between()
        .px_2p5()
        .py_1p5()
        .rounded_xl()
        .bg(field_bg)
        .border_1()
        .border_color(border_c)
        .cursor_text()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.active_field = idx;
            cx.notify();
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(if is_focused {
                            theme.accent()
                        } else {
                            theme.foreground_muted()
                        })
                        .child(label),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.foreground())
                        .child(if val.is_empty() {
                            "|".to_string()
                        } else if is_focused {
                            format!("{val}|")
                        } else {
                            val.to_string()
                        }),
                ),
        )
        .child(if idx > 0 {
            div()
                .size_3p5()
                .rounded_full()
                .bg(swatch_c)
                .border_1()
                .border_color(theme.surface())
        } else {
            div()
        })
}
