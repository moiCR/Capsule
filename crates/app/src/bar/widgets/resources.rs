use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, IntoElement, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(theme: &Theme, cx: &mut Context<Bar>) -> AnyElement {
    let (cpu_pct, mem_str) = if let Some(state) = cx.try_global::<AppState>() {
        let usage = state.system.get_usage_status();
        let cpu = usage.cpu.unwrap_or(0);
        let mem_gib = usage.memory_used_kib as f64 / 1024.0 / 1024.0;
        (cpu, format!("{mem_gib:.1}G"))
    } else {
        (0, "0.0G".to_string())
    };

    div()
        .id("bar-widget-resources")
        .h(px(style::WIDGET_HEIGHT))
        .px(px(8.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(
                    svg()
                        .path("cpu.svg")
                        .size(px(14.0))
                        .text_color(if cpu_pct > 80 {
                            theme.red()
                        } else {
                            theme.foreground_muted()
                        }),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(style::text(theme))
                        .child(format!("{cpu_pct}%")),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(
                    svg()
                        .path("memory-stick.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(style::text(theme))
                        .child(mem_str),
                ),
        )
        .into_any_element()
}
