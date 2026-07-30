use crate::theme::Theme;
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, px,
};

#[derive(IntoElement)]
pub struct SelectItem {
    id: ElementId,
    label: String,
    selected: bool,
}

impl SelectItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>, selected: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            selected,
        }
    }
}

impl RenderOnce for SelectItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .px_3()
            .py_2()
            .rounded(px(12.0))
            .bg(if self.selected {
                theme.surface().opacity(0.8)
            } else {
                theme.surface()
            })
            .text_color(if self.selected {
                theme.accent()
            } else {
                theme.foreground()
            })
            .cursor_pointer()
            .child(self.label)
    }
}
