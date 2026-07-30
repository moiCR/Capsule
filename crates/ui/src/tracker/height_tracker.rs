use gpui::{Div, IntoElement, ParentElement, Styled, canvas, div};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct HeightTracker {
    top: Rc<Cell<f32>>,
    bottom: Rc<Cell<f32>>,
}

impl HeightTracker {
    pub fn new() -> Self {
        Self {
            top: Rc::new(Cell::new(0.0)),
            bottom: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn height(&self, vertical_padding: f32) -> f32 {
        let content_height = (self.bottom.get() - self.top.get()).max(0.0);
        if content_height > 0.0 {
            content_height + vertical_padding
        } else {
            0.0
        }
    }

    pub fn track(&self, element: impl IntoElement) -> Div {
        let top = self.top.clone();
        let bottom = self.bottom.clone();

        div()
            .flex()
            .flex_col()
            .w_full()
            .child(
                canvas(
                    move |bounds, _, _| top.set(bounds.origin.y.into()),
                    |_, _, _, _| {},
                )
                .w_full(),
            )
            .child(element)
            .child(
                canvas(
                    move |bounds, _, _| bottom.set(bounds.origin.y.into()),
                    |_, _, _, _| {},
                )
                .w_full(),
            )
    }
}
