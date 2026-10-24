use crate::theme::Theme;
use crate::tracker::DimensionTracker;
use gpui::{
    App, Bounds, ColorExt, ElementId, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, RenderOnce, SharedString, Styled, Window, canvas, div, fill, px,
    size, svg, white,
};

pub type MouseDownListener = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type MouseUpListener = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    fraction: f32,
    height: f32,
    icon: Option<SharedString>,
    icon_size: f32,
    label: Option<SharedString>,
    tracker: Option<DimensionTracker>,
    disabled: bool,
    active_color: Option<gpui::Hsla>,
    track_color: Option<gpui::Hsla>,
    flex_1: bool,
    on_mouse_down: Option<MouseDownListener>,
    on_mouse_up: Option<MouseUpListener>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            fraction: 0.0,
            height: 32.0,
            icon: None,
            icon_size: 18.0,
            label: None,
            tracker: None,
            disabled: false,
            active_color: None,
            track_color: None,
            flex_1: false,
            on_mouse_down: None,
            on_mouse_up: None,
        }
    }

    pub fn fraction(mut self, fraction: f32) -> Self {
        self.fraction = fraction.clamp(0.0, 1.0);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn track_bounds(mut self, tracker: &DimensionTracker) -> Self {
        self.tracker = Some(tracker.clone());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn active_color(mut self, color: gpui::Hsla) -> Self {
        self.active_color = Some(color);
        self
    }

    pub fn track_color(mut self, color: gpui::Hsla) -> Self {
        self.track_color = Some(color);
        self
    }

    pub fn flex_1(mut self, flex_1: bool) -> Self {
        self.flex_1 = flex_1;
        self
    }

    pub fn on_mouse_down<F>(mut self, listener: F) -> Self
    where
        F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        self.on_mouse_down = Some(Box::new(listener));
        self
    }

    pub fn on_mouse_up<F>(mut self, listener: F) -> Self
    where
        F: Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    {
        self.on_mouse_up = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let track_color = self
            .track_color
            .unwrap_or_else(|| theme.surface().opacity(0.35));
        let active_color = self.active_color.unwrap_or_else(|| theme.accent());
        let corner_radius = self.height / 2.0;
        let fraction = self.fraction;
        let tracker = self.tracker.clone();

        let mut el = div()
            .id(self.id)
            .relative()
            .w_full()
            .min_w_0()
            .h(px(self.height))
            .rounded_full()
            .overflow_hidden();

        if self.flex_1 {
            el = el.flex_1();
        }

        if self.disabled {
            el = el.opacity(0.5);
        } else {
            el = el.cursor_pointer();

            if let Some(down) = self.on_mouse_down {
                el = el.on_mouse_down(MouseButton::Left, move |evt, window, app| {
                    down(evt, window, app);
                });
            }

            if let Some(up) = self.on_mouse_up {
                let up_rc = std::rc::Rc::new(up);
                let up_clone = up_rc.clone();

                el = el
                    .on_mouse_up(MouseButton::Left, move |evt, window, app| {
                        up_rc(evt, window, app);
                    })
                    .on_mouse_up_out(MouseButton::Left, move |evt, window, app| {
                        up_clone(evt, window, app);
                    });
            }
        }

        let canvas_el = canvas(
            move |bounds, _, _| {
                if let Some(trk) = &tracker {
                    trk.track_bounds(bounds);
                }
                bounds
            },
            move |bounds, inner, window, _| {
                // Background track
                window.paint_quad(fill(bounds, track_color).corner_radii(px(corner_radius)));

                // Active filled portion
                let width = inner.size.width * fraction;
                if width > px(0.0) {
                    let visible = Bounds {
                        origin: inner.origin,
                        size: size(width, inner.size.height),
                    };
                    window.with_content_mask(
                        Some(gpui::ContentMask {
                            bounds: visible,
                            fade_out: Default::default(),
                        }),
                        |window| {
                            window.paint_quad(
                                fill(inner, active_color).corner_radii(px(corner_radius)),
                            );
                        },
                    );
                }
            },
        )
        .size_full();

        el = el.child(canvas_el);

        // Optional inside icon on the left
        if let Some(icon_path) = self.icon {
            let icon_top = (self.height - self.icon_size) / 2.0;
            let icon_color = if fraction >= 0.09 {
                white()
            } else {
                theme.foreground_muted()
            };

            el = el.child(
                div().absolute().left(px(10.0)).top(px(icon_top)).child(
                    svg()
                        .path(icon_path)
                        .size(px(self.icon_size))
                        .text_color(icon_color),
                ),
            );
        }

        // Optional inside label on the right
        if let Some(label_text) = self.label {
            let label_top = (self.height - 18.0) / 2.0;
            let text_color = if fraction >= 0.93 {
                white()
            } else {
                theme.foreground_muted()
            };

            el = el.child(
                div()
                    .absolute()
                    .right(px(12.0))
                    .top(px(label_top))
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(text_color)
                    .child(label_text),
            );
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_builder_and_clamping() {
        let s = Slider::new("test-slider")
            .fraction(1.5)
            .height(36.0)
            .label("100%")
            .icon("volume-2.svg");

        assert_eq!(s.fraction, 1.0);
        assert_eq!(s.height, 36.0);
        assert_eq!(s.label.as_deref(), Some("100%"));
        assert_eq!(s.icon.as_deref(), Some("volume-2.svg"));

        let s_negative = Slider::new("neg").fraction(-0.5);
        assert_eq!(s_negative.fraction, 0.0);
    }
}
