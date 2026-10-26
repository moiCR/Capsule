use crate::theme::Theme;
use crate::tracker::DimensionTracker;
use gpui::{
    App, Bounds, ElementId, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, RenderOnce, SharedString, Styled, Window, canvas, div, fill, point,
    px, size, svg,
};

pub type MouseDownListener = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type MouseUpListener = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SliderVariant {
    #[default]
    Thumb,
    Filled,
}

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    fraction: f32,
    height: f32,
    variant: SliderVariant,
    thumb_size: f32,
    track_height: f32,
    icon: Option<SharedString>,
    icon_size: f32,
    label: Option<SharedString>,
    tracker: Option<DimensionTracker>,
    disabled: bool,
    active_color: Option<gpui::Hsla>,
    track_color: Option<gpui::Hsla>,
    thumb_color: Option<gpui::Hsla>,
    halo_color: Option<gpui::Hsla>,
    flex_1: bool,
    on_mouse_down: Option<MouseDownListener>,
    on_mouse_up: Option<MouseUpListener>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            fraction: 0.0,
            height: 24.0,
            variant: SliderVariant::Thumb,
            thumb_size: 18.0,
            track_height: 6.0,
            icon: None,
            icon_size: 18.0,
            label: None,
            tracker: None,
            disabled: false,
            active_color: None,
            track_color: None,
            thumb_color: None,
            halo_color: None,
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

    pub fn variant(mut self, variant: SliderVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn thumb(mut self) -> Self {
        self.variant = SliderVariant::Thumb;
        self
    }

    pub fn filled(mut self) -> Self {
        self.variant = SliderVariant::Filled;
        if self.height < 32.0 {
            self.height = 32.0;
        }
        self
    }

    pub fn thumb_size(mut self, size: f32) -> Self {
        self.thumb_size = size;
        self
    }

    pub fn track_height(mut self, height: f32) -> Self {
        self.track_height = height;
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self.variant = SliderVariant::Filled;
        if self.height < 32.0 {
            self.height = 32.0;
        }
        self
    }

    pub fn icon_size(mut self, size: f32) -> Self {
        self.icon_size = size;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.variant = SliderVariant::Filled;
        if self.height < 32.0 {
            self.height = 32.0;
        }
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

    pub fn thumb_color(mut self, color: gpui::Hsla) -> Self {
        self.thumb_color = Some(color);
        self
    }

    pub fn halo_color(mut self, color: gpui::Hsla) -> Self {
        self.halo_color = Some(color);
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
            .unwrap_or_else(|| theme.surface_container_highest());
        let active_color = self.active_color.unwrap_or_else(|| theme.primary());
        let thumb_color = self.thumb_color.unwrap_or(active_color);
        let halo_color = self
            .halo_color
            .unwrap_or_else(|| theme.surface_container());
        let fraction = self.fraction;
        let tracker = self.tracker.clone();
        let variant = self.variant;
        let track_h_val = self.track_height;
        let thumb_s_val = self.thumb_size;

        let mut el = div()
            .id(self.id)
            .relative()
            .w_full()
            .min_w_0()
            .h(px(self.height));

        if variant == SliderVariant::Filled {
            el = el.rounded_full().overflow_hidden();
        }

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
            move |bounds, inner, window, _| match variant {
                SliderVariant::Thumb => {
                    let center_y = bounds.origin.y + bounds.size.height / 2.0;
                    let track_h = px(track_h_val.clamp(2.0, bounds.size.height.into()));
                    let track_r = track_h / 2.0;
                    let track_y = center_y - track_h / 2.0;

                    let thumb_d = px(thumb_s_val.clamp(6.0, 36.0));
                    let thumb_r = thumb_d / 2.0;

                    let travel_width = (bounds.size.width - thumb_d).max(px(0.0));
                    let center_x = bounds.origin.x + thumb_r + travel_width * fraction;

                    // Inactive track (spans full width)
                    let track_bounds = Bounds {
                        origin: point(bounds.origin.x, track_y),
                        size: size(bounds.size.width, track_h),
                    };
                    window.paint_quad(fill(track_bounds, track_color).corner_radii(track_r));

                    // Active track (from left to center_x)
                    let active_w = (center_x - bounds.origin.x).max(px(0.0));
                    if active_w > px(0.0) {
                        let active_bounds = Bounds {
                            origin: point(bounds.origin.x, track_y),
                            size: size(active_w, track_h),
                        };
                        window.paint_quad(fill(active_bounds, active_color).corner_radii(track_r));
                    }

                    // Halo cutout ring around thumb
                    let halo_d = thumb_d + px(3.0);
                    let halo_r = halo_d / 2.0;
                    let halo_bounds = Bounds {
                        origin: point(center_x - halo_r, center_y - halo_r),
                        size: size(halo_d, halo_d),
                    };
                    window.paint_quad(fill(halo_bounds, halo_color).corner_radii(halo_r));

                    // Thumb knob ("la bolita")
                    let thumb_bounds = Bounds {
                        origin: point(center_x - thumb_r, center_y - thumb_r),
                        size: size(thumb_d, thumb_d),
                    };
                    window.paint_quad(fill(thumb_bounds, thumb_color).corner_radii(thumb_r));
                }
                SliderVariant::Filled => {
                    let corner_radius = inner.size.height / 2.0;

                    // Background track
                    window.paint_quad(fill(bounds, track_color).corner_radii(corner_radius));

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
                                    fill(inner, active_color).corner_radii(corner_radius),
                                );
                            },
                        );
                    }
                }
            },
        )
        .size_full();

        el = el.child(canvas_el);

        // Optional inside icon on the left (for Filled variant)
        if let Some(icon_path) = self.icon {
            let icon_top = (self.height - self.icon_size) / 2.0;
            let icon_color = if fraction >= 0.09 {
                theme.on_primary()
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

        // Optional inside label on the right (for Filled variant)
        if let Some(label_text) = self.label {
            let label_top = (self.height - 18.0) / 2.0;
            let text_color = if fraction >= 0.93 {
                theme.on_primary()
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
        assert_eq!(s.variant, SliderVariant::Filled);
        assert_eq!(s.label.as_deref(), Some("100%"));
        assert_eq!(s.icon.as_deref(), Some("volume-2.svg"));

        let s_negative = Slider::new("neg").fraction(-0.5);
        assert_eq!(s_negative.fraction, 0.0);
        assert_eq!(s_negative.variant, SliderVariant::Thumb);
    }
}
