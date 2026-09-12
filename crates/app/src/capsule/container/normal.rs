use gpui::{
    AnyElement, Context, Div, ElementId, InteractiveElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px,
};

use super::{CapsuleContainerRenderer, ContainerParams};
use crate::capsule::{Capsule, CapsuleMode};

pub struct NormalContainer;

impl NormalContainer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NormalContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CapsuleContainerRenderer for NormalContainer {
    fn render(
        &self,
        content: AnyElement,
        params: &ContainerParams,
        cx: &mut Context<Capsule>,
    ) -> Div {
        let bg = params.bg_color;

        let mut pill_container = div()
            .id(ElementId::NamedInteger("capsule-pill".into(), 0))
            .font_family(params.font_family.clone())
            .w(px(params.width))
            .h(px(params.height))
            .rounded(px(params.radius))
            .bg(bg)
            .border_1()
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
        } else if params.mode == CapsuleMode::Settings || params.mode == CapsuleMode::Shelf {
            pill_container = pill_container.on_click(cx.listener(|_this, _, _, cx| {
                cx.stop_propagation();
            }));
        }

        div()
            .relative()
            .w(px(params.width))
            .h(px(params.height))
            .child(pill_container.child(content))
    }
}
