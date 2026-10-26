use gpui::prelude::*;
use gpui::{
    AnyElement, AnyView, ColorExt, Hsla, IntoElement, PathBuilder, Pixels, Size, div, point, px,
};
use ui::theme::Theme;

use super::{DockEdge, animator::SharedPanelAnimator};
use crate::bar::style;

pub fn concave_panel_canvas(
    size: Size<Pixels>,
    dock_edge: DockEdge,
    radius: f32,
    bg_color: Hsla,
    border_color: Hsla,
) -> impl IntoElement {
    let width = f32::from(size.width).max(0.0);
    let height = f32::from(size.height).max(0.0);
    let radius = radius.max(0.0).min(width / 2.0).min(height / 2.0);
    let shoulder_val = 14.0_f32.min(height / 4.0).min(width / 4.0);

    let (shoulder_left, shoulder_right) = match dock_edge {
        DockEdge::Center => (shoulder_val, shoulder_val),
        DockEdge::Left => (0.0, shoulder_val),
        DockEdge::Right => (shoulder_val, 0.0),
    };

    let radius_bl = if dock_edge == DockEdge::Left {
        0.0
    } else {
        radius
    };
    let radius_br = if dock_edge == DockEdge::Right {
        0.0
    } else {
        radius
    };

    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let width = f32::from(size.width);
            let height = f32::from(size.height);
            if width <= 0.0 || height <= 0.0 {
                return;
            }

            let p = |x: f32, y: f32| bounds.origin + point(px(x + shoulder_left), px(y));
            let k = 0.552_284_8;

            let mut fill_path = PathBuilder::fill();
            fill_path.move_to(p(-shoulder_left, 0.0));
            fill_path.line_to(p(width + shoulder_right, 0.0));
            if shoulder_right > 0.0 {
                fill_path.cubic_bezier_to(
                    p(width, shoulder_right),
                    p(width + shoulder_right * (1.0 - k), 0.0),
                    p(width, shoulder_right * (1.0 - k)),
                );
            }
            fill_path.line_to(p(width, height - radius_br));
            if radius_br > 0.0 {
                fill_path.cubic_bezier_to(
                    p(width - radius_br, height),
                    p(width, height - radius_br * (1.0 - k)),
                    p(width - radius_br * (1.0 - k), height),
                );
            }
            fill_path.line_to(p(radius_bl, height));
            if radius_bl > 0.0 {
                fill_path.cubic_bezier_to(
                    p(0.0, height - radius_bl),
                    p(radius_bl * (1.0 - k), height),
                    p(0.0, height - radius_bl * (1.0 - k)),
                );
            }
            if shoulder_left > 0.0 {
                fill_path.line_to(p(0.0, shoulder_left));
                fill_path.cubic_bezier_to(
                    p(-shoulder_left, 0.0),
                    p(0.0, shoulder_left * (1.0 - k)),
                    p(-shoulder_left * (1.0 - k), 0.0),
                );
            } else {
                fill_path.line_to(p(0.0, 0.0));
            }
            fill_path.close();

            if let Ok(path) = fill_path.build() {
                window.paint_path(path, bg_color);
            }

            let mut stroke_path = PathBuilder::stroke(px(1.0));
            stroke_path.move_to(p(width + shoulder_right, 0.0));
            if shoulder_right > 0.0 {
                stroke_path.cubic_bezier_to(
                    p(width, shoulder_right),
                    p(width + shoulder_right * (1.0 - k), 0.0),
                    p(width, shoulder_right * (1.0 - k)),
                );
            }
            stroke_path.line_to(p(width, height - radius_br));
            if radius_br > 0.0 {
                stroke_path.cubic_bezier_to(
                    p(width - radius_br, height),
                    p(width, height - radius_br * (1.0 - k)),
                    p(width - radius_br * (1.0 - k), height),
                );
            }
            stroke_path.line_to(p(radius_bl, height));
            if radius_bl > 0.0 {
                stroke_path.cubic_bezier_to(
                    p(0.0, height - radius_bl),
                    p(radius_bl * (1.0 - k), height),
                    p(0.0, height - radius_bl * (1.0 - k)),
                );
            }
            if shoulder_left > 0.0 {
                stroke_path.line_to(p(0.0, shoulder_left));
                stroke_path.cubic_bezier_to(
                    p(-shoulder_left, 0.0),
                    p(0.0, shoulder_left * (1.0 - k)),
                    p(-shoulder_left * (1.0 - k), 0.0),
                );
            } else {
                stroke_path.line_to(p(0.0, 0.0));
            }

            if let Ok(path) = stroke_path.build() {
                window.paint_path(path, border_color);
            }
        },
    )
    .absolute()
    .top_0()
    .left(px(-shoulder_left))
    .w(size.width + px(shoulder_left + shoulder_right))
    .h(size.height)
}

pub const CONVEX_EAR_RADIUS: f32 = 16.0;

pub fn convex_bar_ear_canvas(
    radius: f32,
    is_right: bool,
    is_bottom_bar: bool,
    bg_color: Hsla,
    border_color: Hsla,
) -> impl IntoElement {
    let r = radius.max(0.0);

    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            if r <= 0.0 {
                return;
            }

            let p = |x: f32, y: f32| {
                bounds.origin + point(px(x), px(if is_bottom_bar { r - y } else { y }))
            };
            let k = 0.552_284_8;

            let mut fill_path = PathBuilder::fill();
            let mut stroke_path = PathBuilder::stroke(px(1.0));

            if !is_right {
                fill_path.move_to(p(0.0, 0.0));
                fill_path.line_to(p(r, 0.0));
                fill_path.cubic_bezier_to(p(0.0, r), p(r * (1.0 - k), 0.0), p(0.0, r * (1.0 - k)));
                fill_path.line_to(p(0.0, 0.0));
                fill_path.close();

                stroke_path.move_to(p(r, 0.0));
                stroke_path.cubic_bezier_to(
                    p(0.0, r),
                    p(r * (1.0 - k), 0.0),
                    p(0.0, r * (1.0 - k)),
                );
            } else {
                fill_path.move_to(p(0.0, 0.0));
                fill_path.line_to(p(r, 0.0));
                fill_path.line_to(p(r, r));
                fill_path.cubic_bezier_to(p(0.0, 0.0), p(r, r * (1.0 - k)), p(r * k, 0.0));
                fill_path.close();

                stroke_path.move_to(p(0.0, 0.0));
                stroke_path.cubic_bezier_to(p(r, r), p(r * k, 0.0), p(r, r * (1.0 - k)));
            }

            if let Ok(path) = fill_path.build() {
                window.paint_path(path, bg_color);
            }
            if border_color.alpha > 0.0
                && let Ok(path) = stroke_path.build()
            {
                window.paint_path(path, border_color);
            }
        },
    )
    .size(px(r))
}

pub fn render_bottom_border(
    anim: Option<&SharedPanelAnimator>,
    border_start: Pixels,
    border_end: Pixels,
    border_color: Hsla,
) -> AnyElement {
    let mut border_bar = div()
        .id("bar-bottom-border")
        .absolute()
        .bottom_0()
        .left_0()
        .right_0()
        .h(px(1.0));

    if let Some(anim) = anim {
        let width = f32::from(anim.current_size.width).max(0.0);
        let height = f32::from(anim.current_size.height).max(0.0);
        let shoulder_val = 14.0_f32.min(height / 4.0).min(width / 4.0);
        let (sl, sr) = match anim.dock_edge {
            DockEdge::Center => (shoulder_val, shoulder_val),
            DockEdge::Left => (0.0, shoulder_val),
            DockEdge::Right => (shoulder_val, 0.0),
        };
        let neck_left = anim.current_x - px(sl);
        let neck_right = anim.current_x + anim.current_size.width + px(sr);

        let left_end = neck_left.min(border_end);
        if left_end > border_start {
            border_bar = border_bar.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(border_start)
                    .w(left_end - border_start)
                    .bg(border_color),
            );
        }
        if anim.opacity < 1.0 {
            let mid_start = neck_left.max(border_start);
            let mid_end = neck_right.min(border_end);
            if mid_end > mid_start {
                border_bar = border_bar.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(mid_start)
                        .w(mid_end - mid_start)
                        .bg(border_color)
                        .opacity(1.0 - anim.opacity),
                );
            }
        }
        let right_start = neck_right.max(border_start);
        if border_end > right_start {
            border_bar = border_bar.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(right_start)
                    .w(border_end - right_start)
                    .bg(border_color),
            );
        }
    } else if border_end > border_start {
        border_bar = border_bar.child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(border_start)
                .w(border_end - border_start)
                .bg(border_color),
        );
    }

    border_bar.into_any_element()
}

pub fn render_convex_ears(
    anim: Option<&SharedPanelAnimator>,
    viewport_width: Pixels,
    bar_height: Pixels,
    is_bottom: bool,
    theme: &Theme,
) -> (AnyElement, AnyElement) {
    let ear_radius = px(CONVEX_EAR_RADIUS);
    let bg_color = style::background(theme);
    let (left_stroke_opacity, right_stroke_opacity) = if let Some(anim) = anim {
        let width = f32::from(anim.current_size.width).max(0.0);
        let height = f32::from(anim.current_size.height).max(0.0);
        let shoulder_val = 14.0_f32.min(height / 4.0).min(width / 4.0);
        let (sl, sr) = match anim.dock_edge {
            DockEdge::Center => (shoulder_val, shoulder_val),
            DockEdge::Left => (0.0, shoulder_val),
            DockEdge::Right => (shoulder_val, 0.0),
        };
        let neck_left = anim.current_x - px(sl);
        let neck_right = anim.current_x + anim.current_size.width + px(sr);
        let l_op = if neck_left < ear_radius {
            (1.0 - anim.opacity).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let r_op = if neck_right > viewport_width - ear_radius {
            (1.0 - anim.opacity).clamp(0.0, 1.0)
        } else {
            1.0
        };
        (l_op, r_op)
    } else {
        (1.0, 1.0)
    };

    let left_ear_border = style::border(theme).opacity(0.35 * left_stroke_opacity);
    let right_ear_border = style::border(theme).opacity(0.35 * right_stroke_opacity);
    let r = CONVEX_EAR_RADIUS;

    let mut left_ear = div()
        .id("bar-convex-ear-left")
        .absolute()
        .left_0()
        .size(px(r))
        .child(convex_bar_ear_canvas(
            r,
            false,
            is_bottom,
            bg_color,
            left_ear_border,
        ));
    let mut right_ear = div()
        .id("bar-convex-ear-right")
        .absolute()
        .right_0()
        .size(px(r))
        .child(convex_bar_ear_canvas(
            r,
            true,
            is_bottom,
            bg_color,
            right_ear_border,
        ));

    if is_bottom {
        left_ear = left_ear.bottom(bar_height - px(1.0));
        right_ear = right_ear.bottom(bar_height - px(1.0));
    } else {
        left_ear = left_ear.top(bar_height - px(1.0));
        right_ear = right_ear.top(bar_height - px(1.0));
    }

    (left_ear.into_any_element(), right_ear.into_any_element())
}

pub fn render_panel_container(
    anim: &SharedPanelAnimator,
    bar_height: Pixels,
    content: AnyView,
    theme: &Theme,
    on_backdrop_down: impl Fn(&gpui::MouseDownEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> (AnyElement, AnyElement) {
    let bg_color = style::background(theme);
    let border_color = style::border(theme).opacity(0.35);
    let current_x = anim.current_x;
    let current_size = anim.current_size;
    let target_size = anim.target_size;
    let opacity = anim.opacity;
    let dock_edge = anim.dock_edge;

    let content_prog = anim.content_progress;
    let content_opacity = if anim.is_closing() {
        content_prog
    } else {
        (content_prog * 1.35).clamp(0.0, 1.0)
    };
    let blur_val = if content_prog < 1.0 {
        (1.0 - content_prog) * 6.0
    } else {
        0.0
    };
    let slide_y = if anim.is_closing() {
        (1.0 - content_prog) * -6.0
    } else {
        (1.0 - content_prog).powi(2) * -10.0
    };
    let content_offset_x = match dock_edge {
        DockEdge::Center => (current_size.width - target_size.width) / 2.0,
        DockEdge::Left => px(0.0),
        DockEdge::Right => current_size.width - target_size.width,
    };

    let backdrop = div()
        .id("shared-panel-backdrop")
        .absolute()
        .top(bar_height)
        .left_0()
        .right_0()
        .bottom_0()
        .on_mouse_down(gpui::MouseButton::Left, on_backdrop_down);

    let panel = div()
        .id("shared-panel-container")
        .key_context("shared-panel")
        .absolute()
        .top(bar_height - px(1.0))
        .left(current_x)
        .w(current_size.width)
        .h(current_size.height + px(1.0))
        .opacity(opacity)
        .overflow_hidden()
        .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(gpui::MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(|_, _, cx| {
            cx.stop_propagation();
        })
        .child(concave_panel_canvas(
            Size {
                width: current_size.width,
                height: current_size.height + px(1.0),
            },
            dock_edge,
            20.0,
            bg_color,
            border_color,
        ))
        .child(
            div()
                .relative()
                .left(content_offset_x)
                .w(target_size.width)
                .h(target_size.height + px(1.0))
                .pt(px(13.0))
                .overflow_hidden()
                .child(
                    div()
                        .size_full()
                        .relative()
                        .top(px(slide_y))
                        .opacity(content_opacity)
                        .when(blur_val > 0.05, |d| d.blur(px(blur_val)))
                        .child(content),
                ),
        );

    (backdrop.into_any_element(), panel.into_any_element())
}
