use gpui::prelude::*;
use gpui::{div, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarLocation {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct Bar {
    pub location: BarLocation,
}

impl Bar {
    pub fn new(location: BarLocation) -> Self {
        Self { location }
    }
}

impl Render for Bar {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}
