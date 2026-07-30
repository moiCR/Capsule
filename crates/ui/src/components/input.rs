use crate::theme::Theme;
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, Window,
    div, px, svg,
};

#[derive(IntoElement)]
pub struct TextInputField {
    id: ElementId,
    value: String,
    placeholder: String,
    icon: Option<&'static str>,
}

impl TextInputField {
    pub fn new(id: impl Into<ElementId>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: String::new(),
            icon: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }
}

impl RenderOnce for TextInputField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let show_placeholder = self.value.is_empty();

        let mut row = div()
            .id(self.id)
            .flex()
            .items_center()
            .w_full()
            .px_3()
            .py_2()
            .bg(theme.surface())
            .rounded(px(24.0))
            .gap_2();

        if let Some(icon_path) = self.icon {
            row = row.child(
                svg()
                    .path(icon_path)
                    .size(px(14.0))
                    .text_color(theme.foreground_muted()),
            );
        }

        row.child(
            div()
                .text_sm()
                .text_color(if show_placeholder {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .child(if show_placeholder {
                    self.placeholder
                } else {
                    self.value
                }),
        )
    }
}
