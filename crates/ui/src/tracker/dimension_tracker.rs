use gpui::{Div, IntoElement, ParentElement, Styled, canvas, div};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct DimensionTracker {
    top: Rc<Cell<f32>>,
    bottom: Rc<Cell<f32>>,
    left: Rc<Cell<f32>>,
    right: Rc<Cell<f32>>,
}

impl DimensionTracker {
    pub fn new() -> Self {
        Self {
            top: Rc::new(Cell::new(0.0)),
            bottom: Rc::new(Cell::new(0.0)),
            left: Rc::new(Cell::new(0.0)),
            right: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn width(&self, horizontal_padding: f32) -> f32 {
        let content_width = (self.right.get() - self.left.get()).max(0.0);
        if content_width > 0.0 {
            content_width + horizontal_padding
        } else {
            0.0
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

    pub fn dimensions(&self, horizontal_padding: f32, vertical_padding: f32) -> (f32, f32) {
        (
            self.width(horizontal_padding),
            self.height(vertical_padding),
        )
    }

    pub fn track(&self, element: impl IntoElement) -> Div {
        let top = self.top.clone();
        let bottom = self.bottom.clone();
        let left = self.left.clone();
        let right = self.right.clone();

        div()
            .flex()
            .flex_row()
            .child(
                canvas(
                    move |bounds, _, _| left.set(bounds.origin.x.into()),
                    |_, _, _, _| {},
                )
                .h_full(),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
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
                    ),
            )
            .child(
                canvas(
                    move |bounds, _, _| right.set(bounds.origin.x.into()),
                    |_, _, _, _| {},
                )
                .h_full(),
            )
    }
}
