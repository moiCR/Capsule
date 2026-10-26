use crate::theme::Theme;
use gpui::{
    AnyView, App, AppContext, Context, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, prelude::FluentBuilder, px,
};

pub struct TooltipView {
    pub text: SharedString,
    pub shortcut: Option<SharedString>,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .px(px(8.0))
            .py(px(4.0))
            .rounded(px(8.0))
            .bg(theme.surface_container_highest())
            .border_1()
            .border_color(theme.outline_variant())
            .text_size(px(11.0))
            .text_color(theme.foreground())
            .font_family(theme.font_family())
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(self.text.clone())
            .when_some(self.shortcut.clone(), |el, sc| {
                el.child(
                    div()
                        .px(px(4.0))
                        .py(px(1.0))
                        .rounded(px(4.0))
                        .bg(theme.surface_container())
                        .text_color(theme.foreground_muted())
                        .text_size(px(10.0))
                        .child(sc),
                )
            })
    }
}

pub fn tooltip(
    text: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    move |_, cx| {
        let text = text.clone();
        cx.new(|_| TooltipView {
            text,
            shortcut: None,
        })
        .into()
    }
}

pub fn tooltip_with_shortcut(
    text: impl Into<SharedString>,
    shortcut: impl Into<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    let text = text.into();
    let shortcut = shortcut.into();
    move |_, cx| {
        let text = text.clone();
        let shortcut = shortcut.clone();
        cx.new(|_| TooltipView {
            text,
            shortcut: Some(shortcut),
        })
        .into()
    }
}
