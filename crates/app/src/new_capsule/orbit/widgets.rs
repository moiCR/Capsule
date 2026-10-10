use super::{ORB_SIZE, OrbitId, Presentation};
use crate::new_capsule::{Capsule, widgets::style};
use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use services::RecordStatus;
use ui::theme::Theme;

pub(super) fn render(
    presentation: Presentation,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement + use<> {
    let Presentation {
        state,
        visuals,
        capsule,
    } = presentation;
    div()
        .absolute()
        .top_0()
        .left_0()
        .w(capsule.size.width)
        .h(capsule.size.height)
        .font_family(theme.font_family())
        .children(visuals.into_iter().map(|visual| {
            let id = visual.id;
            let indicator = match id {
                OrbitId::Shelf => svg()
                    .path("pin-tilted.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground())
                    .into_any_element(),
                OrbitId::Recording if state.record_status == RecordStatus::Paused => svg()
                    .path("pause.svg")
                    .size(px(13.0))
                    .text_color(theme.accent())
                    .into_any_element(),
                OrbitId::Recording => div()
                    .size(px(8.0))
                    .rounded_full()
                    .bg(theme.red())
                    .into_any_element(),
            };
            div()
                .id(match id {
                    OrbitId::Shelf => "orbit-shelf",
                    OrbitId::Recording => "orbit-recording",
                })
                .absolute()
                .left(visual.bounds.origin.x - capsule.origin.x)
                .top(visual.bounds.origin.y - capsule.origin.y)
                .size(px(ORB_SIZE))
                .rounded_full()
                .bg(style::background(theme))
                .border_1()
                .border_color(style::border(theme))
                .opacity(visual.opacity)
                .flex()
                .items_center()
                .justify_center()
                .child(indicator)
                .when(id == OrbitId::Shelf, |element| {
                    element.child(
                        gpui::deferred(
                            div()
                                .absolute()
                                .bottom(px(-3.0))
                                .right(px(-3.0))
                                .min_w(px(14.0))
                                .h(px(14.0))
                                .px(px(2.0))
                                .rounded_full()
                                .bg(theme.accent())
                                .text_color(style::on_accent(theme))
                                .text_size(px(8.0))
                                .line_height(px(8.0))
                                .text_center()
                                .font_weight(gpui::FontWeight::BOLD)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(if state.shelf_count > 99 {
                                    "99+".to_owned()
                                } else {
                                    state.shelf_count.to_string()
                                }),
                        )
                        .priority(10),
                    )
                })
                .when(visual.interactive, |element| {
                    let element = element
                        .cursor_pointer()
                        .hover(|s| s.border_color(theme.accent()))
                        .on_click(cx.listener(move |capsule, _, _, cx| {
                            cx.stop_propagation();
                            capsule.open_orbit(id, cx);
                        }));
                    element.when(id == OrbitId::Shelf, |element| {
                        element
                            .on_drop(cx.listener(Capsule::drop_on_shelf))
                            .on_drop(cx.listener(Capsule::drop_shelf_file))
                    })
                })
        }))
}
