use crate::theme::Theme;
use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, Window, div, px, svg,
};

#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: &'static str,
    icon_size: f32,
    size: f32,
    on_click: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: &'static str) -> Self {
        Self {
            id: id.into(),
            icon,
            icon_size: 13.0,
            size: 24.0,
            on_click: None,
        }
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn button_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(self.size))
            .h(px(self.size))
            .rounded_full()
            .bg(theme.surface())
            .cursor_pointer()
            .child(
                svg()
                    .path(self.icon)
                    .size(px(self.icon_size))
                    .text_color(theme.accent()),
            );

        if let Some(on_click) = self.on_click {
            el = el.on_click(move |evt, window, app| {
                on_click(evt, window, app);
            });
        }

        el
    }
}

#[derive(IntoElement)]
pub struct TextButton {
    id: ElementId,
    label: String,
    on_click: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl TextButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for TextButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .px_3()
            .py_1p5()
            .rounded(px(24.0))
            .bg(theme.accent())
            .text_color(theme.background())
            .text_xs()
            .font_weight(gpui::FontWeight::BOLD)
            .cursor_pointer()
            .child(self.label);

        if let Some(on_click) = self.on_click {
            el = el.on_click(move |evt, window, app| {
                on_click(evt, window, app);
            });
        }

        el
    }
}
