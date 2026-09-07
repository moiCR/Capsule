use gpui::{AnyElement, Context, Div, Hsla, SharedString};

use crate::capsule::{Capsule, CapsuleMode};

pub mod concave;
pub mod normal;

pub use concave::ConcaveContainer;
pub use normal::NormalContainer;

pub struct ContainerParams {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub border_color: Hsla,
    pub bg_color: Hsla,
    pub font_family: SharedString,
    pub mode: CapsuleMode,
}

pub trait CapsuleContainerRenderer {
    fn render(
        &self,
        content: AnyElement,
        params: &ContainerParams,
        cx: &mut Context<Capsule>,
    ) -> Div;
}
