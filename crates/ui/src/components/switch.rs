use crate::theme::Theme;
use gpui::{
    App, ClickEvent, ColorExt, DurationWithEasing, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, ease_in_out,
    millis, px, svg,
};

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    show_icon: bool,
    on_click: Option<ClickListener>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            show_icon: true,
            on_click: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
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

impl RenderOnce for Switch {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let track_bg = if self.disabled {
            theme.surface_container_highest().opacity(0.4)
        } else if self.checked {
            theme.primary()
        } else {
            theme.surface_container_highest().opacity(0.35)
        };

        let hover_track_bg = if self.checked {
            theme.primary_hover()
        } else {
            theme.surface_container_highest().opacity(0.6)
        };

        let border_color = if self.disabled {
            theme.outline().opacity(0.3)
        } else if self.checked {
            theme.primary()
        } else {
            theme.outline()
        };

        let mut track = div()
            .id(self.id)
            .flex()
            .items_center()
            .w(px(52.0))
            .h(px(32.0))
            .rounded_full()
            .bg(track_bg)
            .border_2()
            .border_color(border_color)
            .transitions(|t| {
                t.bg(millis(200).with_easing(ease_in_out))
                    .border_color(millis(200).with_easing(ease_in_out))
            });

        if !self.disabled {
            track = track.cursor_pointer().hover(move |s| s.bg(hover_track_bg));

            if let Some(on_click) = self.on_click {
                track = track.on_click(move |evt, window, app| {
                    on_click(evt, window, app);
                });
            }
        } else {
            track = track.opacity(0.5);
        }

        let knob_bg = if self.disabled {
            theme.outline().opacity(0.4)
        } else if self.checked {
            theme.on_primary()
        } else {
            theme.outline()
        };

        let knob_size = if self.checked { px(24.0) } else { px(16.0) };
        let knob_ml = if self.checked { px(24.0) } else { px(6.0) };

        let mut knob = div()
            .size(knob_size)
            .rounded_full()
            .bg(knob_bg)
            .ml(knob_ml)
            .flex()
            .items_center()
            .justify_center()
            .transitions(|t| {
                t.ml(millis(200).with_easing(ease_in_out))
                    .bg(millis(200).with_easing(ease_in_out))
                    .w(millis(200).with_easing(ease_in_out))
                    .h(millis(200).with_easing(ease_in_out))
            });

        if self.checked && self.show_icon && !self.disabled {
            knob = knob.child(
                svg()
                    .path("check.svg")
                    .size(px(14.0))
                    .text_color(theme.primary()),
            );
        }

        track.child(knob)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_builder() {
        let sw = Switch::new("my-switch").checked(true).disabled(false);
        assert!(sw.checked);
        assert!(!sw.disabled);

        let sw_disabled = Switch::new("disabled-sw").checked(false).disabled(true);
        assert!(!sw_disabled.checked);
        assert!(sw_disabled.disabled);
    }
}
