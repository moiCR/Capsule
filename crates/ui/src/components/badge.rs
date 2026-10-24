use crate::theme::Theme;
use gpui::{
    App, ColorExt, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, Window, div, px, svg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Accent,
    Danger,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeSize {
    Sm,
    #[default]
    Md,
}

impl BadgeSize {
    pub fn height(&self) -> f32 {
        match self {
            BadgeSize::Sm => 18.0,
            BadgeSize::Md => 22.0,
        }
    }

    pub fn text_size(&self) -> f32 {
        match self {
            BadgeSize::Sm => 10.0,
            BadgeSize::Md => 11.0,
        }
    }

    pub fn icon_size(&self) -> f32 {
        match self {
            BadgeSize::Sm => 10.0,
            BadgeSize::Md => 12.0,
        }
    }

    pub fn padding_x(&self) -> f32 {
        match self {
            BadgeSize::Sm => 6.0,
            BadgeSize::Md => 8.0,
        }
    }
}

#[derive(IntoElement)]
pub struct Badge {
    id: Option<ElementId>,
    content: Option<SharedString>,
    icon: Option<SharedString>,
    variant: BadgeVariant,
    size: BadgeSize,
    circle: bool,
}

impl Badge {
    pub fn new(content: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            content: Some(content.into()),
            icon: None,
            variant: BadgeVariant::Neutral,
            size: BadgeSize::Md,
            circle: false,
        }
    }

    pub fn icon_only(icon: impl Into<SharedString>) -> Self {
        Self {
            id: None,
            content: None,
            icon: Some(icon.into()),
            variant: BadgeVariant::Neutral,
            size: BadgeSize::Md,
            circle: true,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    pub fn circle(mut self, circle: bool) -> Self {
        self.circle = circle;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let h = self.size.height();
        let font_size = self.size.text_size();
        let icon_sz = self.size.icon_size();
        let px_val = if self.circle {
            0.0
        } else {
            self.size.padding_x()
        };

        let (bg, text_col, border_col) = match self.variant {
            BadgeVariant::Neutral => (
                theme.surface().opacity(0.6),
                theme.foreground_muted(),
                theme.surface().opacity(0.3),
            ),
            BadgeVariant::Accent => (theme.accent(), theme.on_accent(), theme.accent()),
            BadgeVariant::Danger => (
                theme.red().opacity(0.18),
                theme.red(),
                theme.red().opacity(0.3),
            ),
            BadgeVariant::Success => (
                theme.green().opacity(0.18),
                theme.green(),
                theme.green().opacity(0.3),
            ),
        };

        let id = self.id.unwrap_or_else(|| ElementId::Name("badge".into()));
        let mut el = div()
            .id(id)
            .h(px(h))
            .min_w(px(h))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(4.0))
            .bg(bg)
            .border_1()
            .border_color(border_col)
            .text_color(text_col)
            .text_size(px(font_size))
            .font_family(theme.font_family())
            .font_weight(gpui::FontWeight::MEDIUM)
            .flex_shrink_0();

        if !self.circle {
            el = el.px(px(px_val));
        }

        if let Some(icon) = self.icon {
            el = el.child(svg().path(icon).size(px(icon_sz)).text_color(text_col));
        }

        if let Some(content) = self.content {
            el = el.child(content);
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_sizing() {
        assert_eq!(BadgeSize::Sm.height(), 18.0);
        assert_eq!(BadgeSize::Md.height(), 22.0);
    }
}
