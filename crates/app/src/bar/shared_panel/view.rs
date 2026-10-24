use gpui::prelude::*;
use gpui::{Hsla, IntoElement, PathBuilder, Pixels, Size, point, px};

use super::DockEdge;

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
