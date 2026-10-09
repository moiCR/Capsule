use crate::new_capsule::widgets::style;
use std::time::Duration;

use gpui::{
    AnyElement, Context, ElementId, IntoElement, MotionDurationExt, div, ease_in_out, prelude::*,
    px, svg,
};
use services::Application;
use ui::theme::Theme;

use super::TILE_HEIGHT;
use crate::new_capsule::module::launcher::LauncherModule;

struct TileContent {
    id: ElementId,
    index: usize,
    selected: bool,
    label: String,
    icon: AnyElement,
}

fn tile(
    content: TileContent,
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let TileContent {
        id,
        index,
        selected,
        label,
        icon,
    } = content;
    let background = if selected {
        theme.accent()
    } else {
        style::surface(theme)
    };
    let hover = if selected {
        style::tint(theme.accent(), theme.background(), 0.08)
    } else {
        style::hover(theme)
    };
    div()
        .id(id)
        .flex_1()
        .min_w_0()
        .h(px(TILE_HEIGHT))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .px(px(8.0))
        .rounded(px(style::CARD_RADIUS))
        .bg(background)
        .cursor_pointer()
        .transitions(|transitions| transitions.bg(duration.with_easing(ease_in_out)))
        .hover(move |style| style.bg(hover))
        .on_hover(cx.listener(move |this, &hovered, _, cx| {
            if hovered && this.mouse_moved {
                this.select(index, cx);
            }
        }))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.select(index, cx);
            this.activate_selected(cx);
        }))
        .child(icon)
        .child(
            div()
                .w_full()
                .text_size(px(12.0))
                .text_color(if selected {
                    style::on_accent(theme)
                } else {
                    theme.foreground()
                })
                .text_center()
                .text_ellipsis()
                .child(label),
        )
}

pub fn render_app_tile(
    app: &Application,
    index: usize,
    selected: bool,
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let icon = match &app.icon_path {
        Some(path) => gpui::img(path.clone()).size(px(44.0)).into_any_element(),
        None => svg()
            .path("sparkles.svg")
            .size(px(44.0))
            .text_color(theme.accent())
            .into_any_element(),
    };
    tile(
        TileContent {
            id: format!("launcher-app-{}", app.id).into(),
            index,
            selected,
            label: app.name.clone(),
            icon,
        },
        duration,
        theme,
        cx,
    )
}

pub fn render_calculator_tile(
    result: &str,
    selected: bool,
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<LauncherModule>,
) -> impl IntoElement {
    let icon = div()
        .h(px(44.0))
        .text_size(px(24.0))
        .text_color(theme.accent())
        .child("=")
        .into_any_element();
    tile(
        TileContent {
            id: "launcher-calculator".into(),
            index: 0,
            selected,
            label: result.to_string(),
            icon,
        },
        duration,
        theme,
        cx,
    )
}
