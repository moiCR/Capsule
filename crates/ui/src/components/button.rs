use crate::theme::Theme;
use gpui::{
    App, ClickEvent, ColorExt, DurationWithEasing, ElementId, FontWeight, InteractiveElement,
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, ease_in_out, millis, px, svg,
};

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
    Outlined,
    Elevated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    pub fn height(self) -> f32 {
        match self {
            Self::Sm => 28.0,
            Self::Md => 34.0,
            Self::Lg => 42.0,
        }
    }

    pub fn text_size(self) -> f32 {
        match self {
            Self::Sm => 12.0,
            Self::Md => 13.0,
            Self::Lg => 14.5,
        }
    }

    pub fn icon_size(self) -> f32 {
        match self {
            Self::Sm => 14.0,
            Self::Md => 16.0,
            Self::Lg => 18.0,
        }
    }

    pub fn padding_x(self) -> f32 {
        match self {
            Self::Sm => 10.0,
            Self::Md => 14.0,
            Self::Lg => 18.0,
        }
    }
}

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<SharedString>,
    right_icon: Option<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    pill: bool,
    selected: bool,
    disabled: bool,
    w_full: bool,
    height_px: Option<f32>,
    padding_x: Option<f32>,
    tooltip: Option<SharedString>,
    on_click: Option<ClickListener>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            right_icon: None,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            pill: true,
            selected: false,
            disabled: false,
            w_full: false,
            height_px: None,
            padding_x: None,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn height_px(mut self, height: f32) -> Self {
        self.height_px = Some(height);
        self
    }

    pub fn padding_x(mut self, padding: f32) -> Self {
        self.padding_x = Some(padding);
        self
    }

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn right_icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.right_icon = Some(icon.into());
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn pill(mut self, pill: bool) -> Self {
        self.pill = pill;
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

    pub fn w_full(mut self, w_full: bool) -> Self {
        self.w_full = w_full;
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

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let h = self.height_px.unwrap_or_else(|| self.size.height());
        let px_val = self.padding_x.unwrap_or_else(|| self.size.padding_x());
        let font_size = self.size.text_size();
        let icon_sz = self.size.icon_size();

        let (base_bg, hover_bg, text_col, border_col) = if self.selected {
            (
                theme.primary(),
                theme.primary_hover(),
                theme.on_primary(),
                theme.primary(),
            )
        } else {
            match self.variant {
                ButtonVariant::Primary => (
                    theme.primary(),
                    theme.primary_hover(),
                    theme.on_primary(),
                    theme.primary(),
                ),
                ButtonVariant::Secondary => (
                    theme.primary_container(),
                    theme.hover_layer(theme.primary_container()),
                    theme.on_primary_container(),
                    theme.primary_container(),
                ),
                ButtonVariant::Ghost => (
                    gpui::transparent_black(),
                    theme.hover_layer(gpui::transparent_black()),
                    theme.primary(),
                    gpui::transparent_black(),
                ),
                ButtonVariant::Outlined => (
                    gpui::transparent_black(),
                    theme.hover_layer(gpui::transparent_black()),
                    theme.primary(),
                    theme.outline(),
                ),
                ButtonVariant::Elevated => (
                    theme.surface_container_low(),
                    theme.hover_layer(theme.surface_container_low()),
                    theme.primary(),
                    theme.surface_container_low(),
                ),
                ButtonVariant::Danger => (
                    theme.red().opacity(0.18),
                    theme.red().opacity(0.28),
                    theme.red(),
                    theme.red().opacity(0.35),
                ),
            }
        };

        let mut el = div()
            .id(self.id)
            .h(px(h))
            .px(px(px_val))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .bg(base_bg)
            .border_1()
            .border_color(border_col)
            .text_color(text_col)
            .text_size(px(font_size))
            .font_family(theme.font_family())
            .font_weight(FontWeight::MEDIUM)
            .transitions(|t| {
                t.bg(millis(120).with_easing(ease_in_out))
                    .border_color(millis(120).with_easing(ease_in_out))
                    .text_color(millis(120).with_easing(ease_in_out))
            });

        if self.pill {
            el = el.rounded_full();
        } else {
            el = el.rounded(px(10.0));
        }

        if self.w_full {
            el = el.w_full();
        }

        if let Some(tip) = self.tooltip {
            el = el.tooltip(crate::components::tooltip::tooltip(tip));
        }

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

        if let Some(label) = self.label {
            el = el.child(div().text_ellipsis().child(label));
        }

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

#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: SharedString,
    icon_size: f32,
    size: f32,
    variant: ButtonVariant,
    shape_circle: bool,
    selected: bool,
    disabled: bool,
    tooltip: Option<SharedString>,
    on_click: Option<ClickListener>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            icon_size: 14.0,
            size: 28.0,
            variant: ButtonVariant::Secondary,
            shape_circle: true,
            selected: false,
            disabled: false,
            tooltip: None,
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

    pub fn size(self, size: f32) -> Self {
        self.button_size(size)
    }

    pub fn shape_circle(mut self, circle: bool) -> Self {
        self.shape_circle = circle;
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
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

    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
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

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let (base_bg, hover_bg, icon_col) = if self.selected {
            (
                theme.primary(),
                theme.primary_hover(),
                theme.on_primary(),
            )
        } else {
            match self.variant {
                ButtonVariant::Primary => (
                    theme.primary(),
                    theme.primary_hover(),
                    theme.on_primary(),
                ),
                ButtonVariant::Secondary => (
                    theme.surface_container_high(),
                    theme.hover_layer(theme.surface_container_high()),
                    theme.foreground(),
                ),
                ButtonVariant::Ghost => (
                    gpui::transparent_black(),
                    theme.hover_layer(gpui::transparent_black()),
                    theme.foreground(),
                ),
                ButtonVariant::Outlined => (
                    gpui::transparent_black(),
                    theme.hover_layer(gpui::transparent_black()),
                    theme.primary(),
                ),
                ButtonVariant::Elevated => (
                    theme.surface_container_low(),
                    theme.hover_layer(theme.surface_container_low()),
                    theme.primary(),
                ),
                ButtonVariant::Danger => (
                    theme.red().opacity(0.18),
                    theme.red().opacity(0.28),
                    theme.red(),
                ),
            }
        };

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .w(px(self.size))
            .h(px(self.size))
            .bg(base_bg)
            .transitions(|t| {
                t.bg(millis(120).with_easing(ease_in_out))
                    .text_color(millis(120).with_easing(ease_in_out))
            })
            .child(
                svg()
                    .path(self.icon)
                    .size(px(self.icon_size))
                    .text_color(icon_col),
            );

        if self.shape_circle {
            el = el.rounded_full();
        } else {
            el = el.rounded(px(8.0));
        }

        if let Some(tip) = self.tooltip {
            el = el.tooltip(crate::components::tooltip::tooltip(tip));
        }

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

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_builder() {
        let btn = Button::new("test-btn")
            .label("Submit")
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Lg)
            .pill(true)
            .disabled(false);

        assert_eq!(btn.size, ButtonSize::Lg);
        assert_eq!(btn.size.height(), 42.0);
        assert_eq!(btn.variant, ButtonVariant::Primary);
        assert!(btn.pill);
        assert!(!btn.disabled);
    }

    #[test]
    fn test_icon_button_builder() {
        let btn = IconButton::new("test-icon-btn", "close.svg")
            .button_size(32.0)
            .icon_size(16.0)
            .variant(ButtonVariant::Ghost)
            .selected(true);

        assert_eq!(btn.size, 32.0);
        assert_eq!(btn.icon_size, 16.0);
        assert_eq!(btn.variant, ButtonVariant::Ghost);
        assert!(btn.selected);
    }
}
