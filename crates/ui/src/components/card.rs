use crate::theme::Theme;
use gpui::{
    AnyElement, App, ClickEvent, ColorExt, DurationWithEasing, ElementId, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div,
    ease_in_out, millis, px,
};

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Card {
    id: Option<ElementId>,
    radius: f32,
    elevated: bool,
    interactive: bool,
    w_full: bool,
    padding: Option<f32>,
    gap: Option<f32>,
    border: bool,
    children: Vec<AnyElement>,
    on_click: Option<ClickListener>,
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl Card {
    pub const DEFAULT_RADIUS: f32 = 16.0;
    pub const INNER_RADIUS: f32 = 12.0;

    pub fn new() -> Self {
        Self {
            id: None,
            radius: Self::DEFAULT_RADIUS,
            elevated: false,
            interactive: false,
            w_full: true,
            padding: None,
            gap: None,
            border: false,
            children: Vec::new(),
            on_click: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn inner(mut self) -> Self {
        self.radius = Self::INNER_RADIUS;
        self
    }

    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn w_full(mut self, w_full: bool) -> Self {
        self.w_full = w_full;
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self.interactive = true;
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let (base_bg, hover_bg) = if self.elevated {
            (theme.surface().opacity(0.7), theme.surface().opacity(0.9))
        } else {
            (theme.surface().opacity(0.4), theme.surface().opacity(0.6))
        };

        if self.interactive || self.on_click.is_some() || self.id.is_some() {
            let id = self.id.unwrap_or_else(|| ElementId::Name("card".into()));
            let mut el = div()
                .id(id)
                .flex()
                .flex_col()
                .rounded(px(self.radius))
                .bg(base_bg)
                .text_color(theme.foreground())
                .font_family(theme.font_family())
                .overflow_hidden()
                .flex_shrink_0();

            if self.w_full {
                el = el.w_full().min_w_0();
            }

            if let Some(p) = self.padding {
                el = el.p(px(p));
            }

            if let Some(g) = self.gap {
                el = el.gap(px(g));
            }

            if self.border {
                el = el.border_1().border_color(theme.surface().opacity(0.2));
            }

            if self.interactive || self.on_click.is_some() {
                el = el
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .transitions(|t| {
                        t.bg(millis(120).with_easing(ease_in_out))
                            .border_color(millis(120).with_easing(ease_in_out))
                    });

                if let Some(on_click) = self.on_click {
                    el = el.on_click(move |evt, window, app| {
                        on_click(evt, window, app);
                    });
                }
            }

            el.children(self.children).into_any_element()
        } else {
            let mut el = div()
                .flex()
                .flex_col()
                .rounded(px(self.radius))
                .bg(base_bg)
                .text_color(theme.foreground())
                .font_family(theme.font_family())
                .overflow_hidden()
                .flex_shrink_0();

            if self.w_full {
                el = el.w_full().min_w_0();
            }

            if let Some(p) = self.padding {
                el = el.p(px(p));
            }

            if let Some(g) = self.gap {
                el = el.gap(px(g));
            }

            if self.border {
                el = el.border_1().border_color(theme.surface().opacity(0.2));
            }

            el.children(self.children).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_defaults() {
        let card = Card::new();
        assert_eq!(card.radius, 16.0);
        let inner = Card::new().inner();
        assert_eq!(inner.radius, 12.0);
    }
}
