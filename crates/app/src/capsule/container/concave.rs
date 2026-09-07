use gpui::{
    AnyElement, Context, Div, ElementId, InteractiveElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, svg,
};

use super::{CapsuleContainerRenderer, ContainerParams};
use crate::capsule::{Capsule, CapsuleMode};

pub struct ConcaveContainer;

impl ConcaveContainer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConcaveContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleContainerRenderer for ConcaveContainer {
    fn render(
        &self,
        content: AnyElement,
        params: &ContainerParams,
        cx: &mut Context<Capsule>,
    ) -> Div {
        let wing_size = (params.radius * 0.6).clamp(12.0, 32.0).min(params.height);

        let mut pill_container = div()
            .id(ElementId::NamedInteger("capsule-pill".into(), 0))
            .font_family(params.font_family.clone())
            .w(px(params.width))
            .h(px(params.height))
            .rounded_bl(px(params.radius))
            .rounded_br(px(params.radius))
            .bg(params.bg_color)
            .border_b_1()
            .border_l_1()
            .border_r_1()
            .border_color(params.border_color)
            .shadow_lg()
            .overflow_hidden();

        if params.mode == CapsuleMode::Default {
            pill_container =
                pill_container
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start_transition_internal(CapsuleMode::Dashboard, None, cx);
                    }));
        } else if params.mode == CapsuleMode::Settings {
            pill_container = pill_container.on_click(cx.listener(|_this, _, _, cx| {
                cx.stop_propagation();
            }));
        }

        let left_wing_bg = div()
            .absolute()
            .top_0()
            .left(px(-wing_size))
            .w(px(wing_size))
            .h(px(wing_size))
            .child(
                svg()
                    .path("concave-wing-left.svg")
                    .size(px(wing_size))
                    .text_color(params.bg_color),
            );

        let right_wing_bg = div()
            .absolute()
            .top_0()
            .left(px(params.width))
            .w(px(wing_size))
            .h(px(wing_size))
            .child(
                svg()
                    .path("concave-wing-right.svg")
                    .size(px(wing_size))
                    .text_color(params.bg_color),
            );

        let left_patch = div()
            .absolute()
            .top_0()
            .left(px(-0.5))
            .w(px(2.0))
            .h(px(wing_size))
            .bg(params.bg_color);

        let right_patch = div()
            .absolute()
            .top_0()
            .left(px(params.width - 1.5))
            .w(px(2.0))
            .h(px(wing_size))
            .bg(params.bg_color);

        let left_wing_border = div()
            .absolute()
            .top_0()
            .left(px(-wing_size))
            .w(px(wing_size))
            .h(px(wing_size))
            .child(
                svg()
                    .path("concave-wing-left-border.svg")
                    .size(px(wing_size))
                    .text_color(params.border_color),
            );

        let right_wing_border = div()
            .absolute()
            .top_0()
            .left(px(params.width))
            .w(px(wing_size))
            .h(px(wing_size))
            .child(
                svg()
                    .path("concave-wing-right-border.svg")
                    .size(px(wing_size))
                    .text_color(params.border_color),
            );

        div()
            .relative()
            .w(px(params.width))
            .h(px(params.height))
            .child(left_wing_bg)
            .child(right_wing_bg)
            .child(pill_container.child(content))
            .child(left_patch)
            .child(right_patch)
            .child(left_wing_border)
            .child(right_wing_border)
    }
}
