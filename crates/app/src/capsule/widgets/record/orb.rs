use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use services::RecordStatus;
use ui::theme::Theme;

use crate::capsule::{
    Capsule,
    orbit::{ORB_SIZE, OrbKind},
};

pub fn render_record_orb(
    status: RecordStatus,
    interactive: bool,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement {
    let indicator = if status == RecordStatus::Paused {
        svg()
            .path("pause.svg")
            .size(px(13.0))
            .text_color(gpui::rgb(0xf59e0b))
            .into_any_element()
    } else {
        div()
            .size(px(8.0))
            .rounded_full()
            .bg(theme.red())
            .into_any_element()
    };
    div()
        .id("recording-orb")
        .size(px(ORB_SIZE))
        .rounded_full()
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface())
        .flex()
        .items_center()
        .justify_center()
        .child(indicator)
        .when(interactive, |element| {
            element
                .cursor_pointer()
                .hover(|style| style.border_color(theme.accent()))
                .on_click(cx.listener(|capsule, _, _, cx| {
                    cx.stop_propagation();
                    capsule.open_orb(OrbKind::Recording, cx);
                }))
        })
}
