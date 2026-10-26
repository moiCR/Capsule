use crate::theme::Theme;
use gpui::{
    App, ClickEvent, DurationWithEasing, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div, ease_in_out, millis,
    px, svg,
};

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChipSize {
    Sm,
    #[default]
    Md,
}

impl ChipSize {
    pub fn height(&self) -> f32 {
        match self {
            ChipSize::Sm => 24.0,
            ChipSize::Md => 28.0,
        }
    }

    pub fn text_size(&self) -> f32 {
        match self {
            ChipSize::Sm => 11.0,
            ChipSize::Md => 12.0,
        }
    }

    pub fn icon_size(&self) -> f32 {
        match self {
            ChipSize::Sm => 11.0,
            ChipSize::Md => 13.0,
        }
    }

    pub fn padding_x(&self) -> f32 {
        match self {
            ChipSize::Sm => 10.0,
            ChipSize::Md => 14.0,
        }
    }
}

#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    right_icon: Option<SharedString>,
    selected: bool,
    disabled: bool,
    size: ChipSize,
    on_click: Option<ClickListener>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            right_icon: None,
            selected: false,
            disabled: false,
            size: ChipSize::Md,
            on_click: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn right_icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.right_icon = Some(icon.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let h = self.size.height();
        let px_val = self.size.padding_x();
        let font_size = self.size.text_size();
        let icon_sz = self.size.icon_size();

        let (bg, hover_bg, text_col, border_col) = if self.selected {
            (
                theme.primary_container(),
                theme.hover_layer(theme.primary_container()),
                theme.on_primary_container(),
                theme.primary_container(),
            )
        } else {
            (
                theme.surface_container_low(),
                theme.surface_container_high(),
                theme.foreground(),
                theme.outline_variant(),
            )
        };

        let mut el = div()
            .id(self.id)
            .h(px(h))
            .px(px(px_val))
            .rounded_full()
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .bg(bg)
            .border_1()
            .border_color(border_col)
            .text_color(text_col)
            .text_size(px(font_size))
            .font_family(theme.font_family())
            .font_weight(gpui::FontWeight::MEDIUM)
            .flex_shrink_0()
            .transitions(|t| {
                t.bg(millis(120).with_easing(ease_in_out))
                    .border_color(millis(120).with_easing(ease_in_out))
                    .text_color(millis(120).with_easing(ease_in_out))
            });

        if self.disabled {
            el = el.opacity(0.45).cursor_not_allowed();
        } else {
            el = el.cursor_pointer().hover(move |s| s.bg(hover_bg));

            if let Some(on_click) = self.on_click {
                el = el.on_click(move |evt, window, app| {
                    on_click(evt, window, app);
                });
            }
        }

        if let Some(icon) = self.icon {
            el = el.child(svg().path(icon).size(px(icon_sz)).text_color(text_col));
        }

        el = el.child(div().text_ellipsis().child(self.label));

        if let Some(right_icon) = self.right_icon {
            el = el.child(
                svg()
                    .path(right_icon)
                    .size(px(icon_sz))
                    .text_color(text_col),
            );
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_sizing() {
        assert_eq!(ChipSize::Sm.height(), 24.0);
        assert_eq!(ChipSize::Md.height(), 28.0);
    }
}
