use crate::theme::Theme;
use gpui::{
    App, ClickEvent, ColorExt, DurationWithEasing, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window, div, ease_in_out,
    millis, px, white,
};

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_click: Option<ClickListener>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
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
            theme.surface().opacity(0.4)
        } else if self.checked {
            theme.accent()
        } else {
            theme.surface().opacity(0.65)
        };

        let hover_bg = if self.checked {
            theme.accent().opacity(0.85)
        } else {
            theme.surface().opacity(0.85)
        };

        let mut track = div()
            .id(self.id)
            .flex()
            .items_center()
            .w(px(44.0))
            .h(px(26.0))
            .p(px(3.0))
            .rounded_full()
            .bg(track_bg)
            .border_1()
            .border_color(if self.checked {
                theme.accent()
            } else {
                theme.surface().opacity(0.9)
            })
            .transitions(|t| {
                t.bg(millis(180).with_easing(ease_in_out))
                    .border_color(millis(180).with_easing(ease_in_out))
            });

        if !self.disabled {
            track = track.cursor_pointer().hover(move |s| s.bg(hover_bg));

            if let Some(on_click) = self.on_click {
                track = track.on_click(move |evt, window, app| {
                    on_click(evt, window, app);
                });
            }
        } else {
            track = track.opacity(0.5);
        }

        let knob_bg = if self.disabled {
            white().opacity(0.6)
        } else {
            white()
        };

        let knob = div()
            .size(px(20.0))
            .rounded_full()
            .bg(knob_bg)
            .shadow_sm()
            .ml(if self.checked { px(18.0) } else { px(0.0) })
            .transitions(|t| {
                t.ml(millis(180).with_easing(ease_in_out))
                    .bg(millis(180).with_easing(ease_in_out))
            });

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
