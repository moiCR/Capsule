use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

use crate::capsule::{
    Capsule,
    orbit::{ORB_SIZE, OrbKind},
};

pub fn render_shelf_orb(
    count: usize,
    interactive: bool,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement {
    let accent = theme.accent();
    let button = div()
        .id("shelf-orb-button")
        .size_full()
        .rounded_full()
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface())
        .group_hover("shelf-orb", |style| style.border_color(accent))
        .flex()
        .items_center()
        .justify_center()
        .child(
            svg()
                .path("pin-tilted.svg")
                .size(px(13.0))
                .text_color(theme.foreground()),
        );
    let badge = div()
        .absolute()
        .bottom(px(-3.0))
        .right(px(-3.0))
        .min_w(px(14.0))
        .h(px(14.0))
        .px(px(2.0))
        .rounded_full()
        .bg(accent)
        .border_1()
        .border_color(theme.background())
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .relative()
                .top(px(1.5))
                .text_center()
                .font_weight(gpui::FontWeight::BOLD)
                .text_size(px(8.0))
                .text_color(gpui::white())
                .child(count.to_string()),
        );
    div()
        .id("shelf-orb")
        .group("shelf-orb")
        .relative()
        .size(px(ORB_SIZE))
        .child(button)
        .child(badge)
        .drag_over::<gpui::ExternalPaths>(move |style, _, window, cx| {
            if let Some(Some(root)) = window.root::<Capsule>() {
                root.update(cx, |capsule, cx| capsule.on_drag_over_capsule(cx));
            }
            style.rounded_full().border_2().border_color(accent)
        })
        .on_drop(cx.listener(Capsule::drop_on_shelf))
        .when(interactive, |element| {
            element
                .cursor_pointer()
                .on_click(cx.listener(|capsule, _, _, cx| {
                    cx.stop_propagation();
                    capsule.open_orb(OrbKind::Shelf, cx);
                }))
        })
}
