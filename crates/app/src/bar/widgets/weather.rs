use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let weather_opt = cx
        .try_global::<AppState>()
        .and_then(|state| state.weather.get_status());

    let (icon_path, temp_text) = if let Some(ref w) = weather_opt {
        (w.icon, format!("{}°C", w.temp_c))
    } else {
        ("cloud-sun.svg", "--°C".to_string())
    };

    div()
        .id("bar-widget-weather")
        .h(px(style::WIDGET_HEIGHT))
        .px(px(8.0))
        .flex()
        .items_center()
        .gap(px(5.0))
        .child(
            svg()
                .path(icon_path)
                .size(px(15.0))
                .text_color(theme.yellow()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(style::text(theme))
                .child(temp_text),
        )
        .into_any_element()
}
