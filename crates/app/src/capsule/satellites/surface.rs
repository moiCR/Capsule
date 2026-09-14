use gpui::{
    AnyElement, BoxShadow, Div, Hsla, PathBuilder, Pixels, Point, Size, Window, canvas, div, point,
    prelude::*, px,
};
use ui::theme::Theme;

use super::{Lane, satellite_retract};

const SNAP_PHASE: f32 = 0.7;
const RELEASE_PHASE: f32 = 0.85;

pub struct PanelReveal {
    pub size: Size<f32>,
    pub y: f32,
}

impl PanelReveal {
    pub fn new(
        content_size: Size<f32>,
        target_y: f32,
        capsule_size: Size<f32>,
        capsule_radius: f32,
        phase: f32,
    ) -> Self {
        let inset = capsule_radius.clamp(0.0, ((capsule_size.height - 1.0) * 0.5).max(0.0));
        let start_width = content_size.width.min((capsule_size.width - 60.0).max(1.0));
        let start_height = content_size
            .height
            .min((capsule_size.height - 2.0 * inset).max(1.0));
        let start_y = target_y.clamp(
            inset,
            (capsule_size.height - inset - start_height).max(inset),
        );
        let growth = satellite_retract((phase - 0.35) / (SNAP_PHASE - 0.35));
        Self {
            size: Size::new(
                start_width + (content_size.width - start_width) * growth,
                start_height + (content_size.height - start_height) * growth,
            ),
            y: start_y + (target_y - start_y) * growth,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PanelSurface {
    pub lane: Lane,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub phase: f32,
    pub closing: bool,
    pub radius: f32,
}

impl PanelSurface {
    fn corner_radius(self) -> f32 {
        let limit = self.width.min(self.height).max(0.0) * 0.5;
        let resting = self.radius.clamp(0.0, limit);
        let rounded = (resting + 24.0).min(limit);
        rounded + (resting - rounded) * satellite_retract(self.phase / SNAP_PHASE)
    }

    fn content_opacity(self) -> f32 {
        let progress = ((self.phase - 0.48) / 0.24).clamp(0.0, 1.0);
        1.0 - (1.0 - progress).powi(3)
    }

    fn release(self) -> f32 {
        satellite_retract((self.phase - SNAP_PHASE) / (RELEASE_PHASE - SNAP_PHASE))
    }

    pub fn render(self, content: AnyElement, content_size: Size<f32>, theme: &Theme) -> Div {
        let radius = self.corner_radius();
        let opacity = self.content_opacity();
        let release = self.release();
        let background = div()
            .absolute()
            .inset_0()
            .rounded(px(radius))
            .bg(theme.background())
            .border_1()
            .border_color(theme.surface().opacity(0.6 - 0.25 * release))
            .shadow(vec![BoxShadow {
                color: gpui::black().opacity(0.18 * release),
                offset: point(px(0.0), px(4.0)),
                blur_radius: px(16.0),
                spread_radius: px(0.0),
                inset: false,
            }]);

        div()
            .relative()
            .w(px(self.width))
            .h(px(self.height))
            .child(background)
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .rounded(px(radius))
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .w(px(content_size.width))
                            .h(px(content_size.height))
                            .top(px(-6.0 * (1.0 - opacity)))
                            .opacity(opacity)
                            .when(opacity <= 0.001, |content| content.invisible())
                            .child(content),
                    ),
            )
            .when(self.closing || opacity < 0.95, |panel| {
                panel.child(div().absolute().inset_0().occlude())
            })
    }

    fn neck(self, capsule_width: f32, capsule_height: f32, capsule_radius: f32) -> Option<Neck> {
        if self.phase <= 0.0 || self.phase >= RELEASE_PHASE {
            return None;
        }
        let direction = if self.lane == Lane::Left { -1.0 } else { 1.0 };
        let capsule_x = if self.lane == Lane::Left {
            0.0
        } else {
            capsule_width
        };
        let panel_x = if self.lane == Lane::Left {
            self.x + self.width
        } else {
            self.x
        };
        let gap = (panel_x - capsule_x) * direction;
        let outer_x = if self.lane == Lane::Left {
            self.x
        } else {
            self.x + self.width
        };
        let exposed = (outer_x - capsule_x) * direction;
        if exposed <= 0.0 {
            return None;
        }
        let radius = self.corner_radius();
        let capsule_radius = capsule_radius.clamp(0.0, capsule_height.max(0.0) * 0.5);
        let root = ((self.height - 2.0 * radius).max(0.0) * 0.35)
            .min((capsule_height - 2.0 * capsule_radius).max(0.0) * 0.25)
            .min(26.0)
            .min(exposed);
        if root <= 0.01 {
            return None;
        }
        let panel_y = self.y + self.height * 0.5;
        let capsule_y = panel_y.clamp(
            capsule_radius + root,
            capsule_height - capsule_radius - root,
        );
        let stretch = satellite_retract((self.phase - 0.35) / (SNAP_PHASE - 0.35));
        let release = self.release();
        Some(Neck {
            start: (capsule_x - direction * 1.5 * (1.0 - release), capsule_y),
            end: (
                capsule_x + direction * (gap.max(0.0) + 1.5 * (1.0 - release)),
                panel_y,
            ),
            root: root * (1.0 - release),
            waist: root * 0.65 * (1.0 - stretch).powi(2),
            release,
            split: self.phase >= SNAP_PHASE,
        })
    }
}

#[derive(Clone, Copy)]
struct Neck {
    start: (f32, f32),
    end: (f32, f32),
    root: f32,
    waist: f32,
    release: f32,
    split: bool,
}

impl Neck {
    fn paint(self, origin: Point<Pixels>, background: Hsla, border: Hsla, window: &mut Window) {
        let point = |x, y| origin + gpui::point(px(x), px(y));
        let (ax, ay) = self.start;
        let (bx, by) = self.end;
        let mx = (ax + bx) * 0.5;
        let my = (ay + by) * 0.5;
        if self.split {
            self.paint_lobe(self.start, (mx, my), origin, background, border, window);
            self.paint_lobe(self.end, (mx, my), origin, background, border, window);
            return;
        }
        let dx = bx - ax;
        let upper = |path: &mut PathBuilder| {
            path.move_to(point(ax, ay - self.root));
            path.cubic_bezier_to(
                point(mx, my - self.waist),
                point(ax, ay - self.root * 0.2),
                point(mx - dx * 0.2, my - self.waist),
            );
            path.cubic_bezier_to(
                point(bx, by - self.root),
                point(mx + dx * 0.2, my - self.waist),
                point(bx, by - self.root * 0.2),
            );
        };
        let lower = |path: &mut PathBuilder| {
            path.cubic_bezier_to(
                point(mx, my + self.waist),
                point(bx, by + self.root * 0.2),
                point(mx + dx * 0.2, my + self.waist),
            );
            path.cubic_bezier_to(
                point(ax, ay + self.root),
                point(mx - dx * 0.2, my + self.waist),
                point(ax, ay + self.root * 0.2),
            );
        };
        let mut fill = PathBuilder::fill();
        upper(&mut fill);
        fill.line_to(point(bx, by + self.root));
        lower(&mut fill);
        fill.close();
        paint_path(fill, background, window);
        let mut stroke = PathBuilder::stroke(px(1.0));
        upper(&mut stroke);
        stroke.move_to(point(bx, by + self.root));
        lower(&mut stroke);
        paint_path(stroke, border, window);
    }

    fn paint_lobe(
        self,
        root: (f32, f32),
        tip: (f32, f32),
        origin: Point<Pixels>,
        background: Hsla,
        border: Hsla,
        window: &mut Window,
    ) {
        let (x, y) = root;
        let tip_x = x + (tip.0 - x) * (1.0 - self.release);
        let tip_y = y + (tip.1 - y) * (1.0 - self.release);
        let point = |x, y| origin + gpui::point(px(x), px(y));
        let outline = |path: &mut PathBuilder| {
            path.move_to(point(x, y - self.root));
            path.cubic_bezier_to(
                point(tip_x, tip_y),
                point(x, y - self.root * 0.2),
                point(tip_x - (tip_x - x) * 0.4, tip_y),
            );
            path.cubic_bezier_to(
                point(x, y + self.root),
                point(tip_x - (tip_x - x) * 0.4, tip_y),
                point(x, y + self.root * 0.2),
            );
        };
        let mut fill = PathBuilder::fill();
        outline(&mut fill);
        fill.close();
        paint_path(fill, background, window);
        let mut stroke = PathBuilder::stroke(px(1.0));
        outline(&mut stroke);
        paint_path(stroke, border, window);
    }
}

fn paint_path(builder: PathBuilder, color: Hsla, window: &mut Window) {
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

pub fn connections(
    panels: Vec<PanelSurface>,
    capsule_width: f32,
    capsule_height: f32,
    capsule_radius: f32,
    theme: &Theme,
) -> impl IntoElement {
    let background = theme.background();
    let border = theme.surface().opacity(0.6);
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            for panel in panels {
                if let Some(neck) = panel.neck(capsule_width, capsule_height, capsule_radius) {
                    neck.paint(bounds.origin, background, border, window);
                }
            }
        },
    )
    .absolute()
    .inset_0()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_and_stacked_panels_fit_before_leaving_capsule() {
        use super::super::{LANE_GAP, PanelManager};
        let capsule_size = Size::new(490.0, 400.0);
        for content_size in [
            Size::new(280.0, 650.0),
            Size::new(700.0, 600.0),
            Size::new(280.0, 120.0),
        ] {
            for target_y in [0.0, 360.0, 510.0] {
                for lane in [Lane::Left, Lane::Right] {
                    for step in 0..=35 {
                        let phase = step as f32 / 100.0;
                        let reveal =
                            PanelReveal::new(content_size, target_y, capsule_size, 24.0, phase);
                        assert!(reveal.y >= 24.0);
                        assert!(reveal.y + reveal.size.height <= 376.0);
                        let lane_x = if lane == Lane::Left {
                            -reveal.size.width - LANE_GAP
                        } else {
                            490.0 + LANE_GAP
                        };
                        let (x, _) = PanelManager::animated_position(
                            lane,
                            490.0,
                            400.0,
                            reveal.size.width,
                            lane_x,
                            reveal.y,
                            phase,
                            false,
                        );
                        if step == 0 {
                            assert!(x >= 0.0);
                            assert!(x + reveal.size.width <= 490.0);
                        }
                        if step == 35 {
                            if lane == Lane::Left {
                                assert!((x + reveal.size.width).abs() < 0.001);
                            } else {
                                assert!((x - 490.0).abs() < 0.001);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn reveal_preserves_final_dimensions_and_is_continuous() {
        let size = Size::new(700.0, 650.0);
        let reveal = |phase| PanelReveal::new(size, 420.0, Size::new(490.0, 400.0), 24.0, phase);
        for phase in [0.7, 0.85, 1.0] {
            assert_eq!(reveal(phase).size, size);
            assert_eq!(reveal(phase).y, 420.0);
        }
        for boundary in [0.35, 0.7] {
            let before = reveal(boundary - 0.00001);
            let after = reveal(boundary + 0.00001);
            assert!((before.size.width - after.size.width).abs() < 0.001);
            assert!((before.size.height - after.size.height).abs() < 0.001);
            assert!((before.y - after.y).abs() < 0.001);
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clipped_surface_keeps_full_content_measurements() {
        use gpui::{Context, Render};
        use std::sync::{Arc, Mutex};
        use ui::tracker::DimensionTracker;

        struct LayoutView {
            phase: f32,
            content: DimensionTracker,
            surface: DimensionTracker,
        }
        impl Render for LayoutView {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                let content_size = Size::new(700.0, 650.0);
                let reveal =
                    PanelReveal::new(content_size, 0.0, Size::new(490.0, 400.0), 24.0, self.phase);
                let surface = PanelSurface {
                    width: reveal.size.width,
                    height: reveal.size.height,
                    ..panel(self.phase)
                };
                let content = self
                    .content
                    .track(div().w(px(700.0)).h(px(650.0)).child("Full-size content"))
                    .into_any_element();
                div().size_full().child(self.surface.track(surface.render(
                    content,
                    content_size,
                    &Theme::default(),
                )))
            }
        }
        let measurements = Arc::new(Mutex::new(Vec::new()));
        let output = measurements.clone();
        gpui_platform::headless()
            .with_assets(assets::Assets {})
            .run(move |cx| {
                let window = cx
                    .open_window(gpui::WindowOptions::default(), |_, cx| {
                        cx.new(|_| LayoutView {
                            phase: 0.0,
                            content: DimensionTracker::new(),
                            surface: DimensionTracker::new(),
                        })
                    })
                    .expect("headless window");
                let mut app = cx.to_async();
                cx.foreground_executor()
                    .spawn(async move {
                        for phase in [0.0, 0.2, 0.5, 1.0, 0.5, 0.2, 0.0] {
                            app.update_window(window.into(), |root, window, cx| {
                                let root = root.downcast::<LayoutView>().unwrap();
                                root.update(cx, |view, cx| {
                                    view.phase = phase;
                                    cx.notify();
                                });
                                window.draw(cx).clear(cx);
                                let view = root.read(cx);
                                output.lock().unwrap().push((
                                    phase,
                                    view.content.width(0.0),
                                    view.content.height(0.0),
                                    view.surface.width(0.0),
                                    view.surface.height(0.0),
                                ));
                            })
                            .unwrap();
                        }
                        app.update(|cx| cx.quit());
                    })
                    .detach();
            });
        let measurements = measurements.lock().unwrap();
        assert_eq!(measurements.len(), 7);
        for &(phase, content_width, content_height, surface_width, surface_height) in
            measurements.iter()
        {
            assert!(
                (content_width - 700.0).abs() < 0.01,
                "phase {phase}: {content_width}"
            );
            assert!(
                (content_height - 650.0).abs() < 0.01,
                "phase {phase}: {content_height}"
            );
            let reveal = PanelReveal::new(
                Size::new(700.0, 650.0),
                0.0,
                Size::new(490.0, 400.0),
                24.0,
                phase,
            );
            assert!(
                (surface_width - reveal.size.width).abs() <= 0.5,
                "phase {phase}: surface width {surface_width}, expected {}",
                reveal.size.width
            );
            assert!(
                (surface_height - reveal.size.height).abs() <= 0.5,
                "phase {phase}: surface height {surface_height}, expected {}",
                reveal.size.height
            );
        }
    }

    fn panel(phase: f32) -> PanelSurface {
        PanelSurface {
            lane: Lane::Left,
            x: -288.0,
            y: 0.0,
            width: 280.0,
            height: 120.0,
            phase,
            closing: false,
            radius: 20.0,
        }
    }

    #[test]
    fn content_reveals_without_scaling_and_radius_settles() {
        assert_eq!(panel(0.0).content_opacity(), 0.0);
        assert_eq!(panel(0.48).content_opacity(), 0.0);
        assert_eq!(panel(1.0).content_opacity(), 1.0);
        assert!(panel(0.6).content_opacity() > 0.0);
        assert!(panel(0.0).corner_radius() > panel(1.0).corner_radius());
        assert_eq!(panel(1.0).corner_radius(), 20.0);
    }

    #[test]
    fn neck_narrows_splits_and_disappears() {
        let early = panel(0.4).neck(490.0, 520.0, 24.0).unwrap();
        let stretched = panel(0.69).neck(490.0, 520.0, 24.0).unwrap();
        let split = panel(0.7).neck(490.0, 520.0, 24.0).unwrap();
        assert!(early.waist > stretched.waist);
        assert!(!stretched.split);
        assert!(split.split);
        assert_eq!(split.waist, 0.0);
        assert!(panel(0.85).neck(490.0, 520.0, 24.0).is_none());
        assert!(panel(1.0).neck(490.0, 520.0, 24.0).is_none());
    }

    #[test]
    fn connections_mirror_and_anchor_inside_capsule() {
        let left = panel(0.5).neck(490.0, 520.0, 24.0).unwrap();
        let mut right = panel(0.5);
        right.lane = Lane::Right;
        right.x = 498.0;
        let right = right.neck(490.0, 520.0, 24.0).unwrap();
        assert_eq!(left.start.0, 490.0 - right.start.0);
        assert_eq!(left.end.0, 490.0 - right.end.0);
        let mut lower = panel(0.5);
        lower.y = 490.0;
        let neck = lower.neck(490.0, 520.0, 24.0).unwrap();
        assert!(neck.start.1 + neck.root <= 496.0);
        assert!(neck.end.1 > neck.start.1);
    }

    #[test]
    fn reversing_does_not_change_surface_geometry() {
        let opening = panel(0.6);
        let closing = PanelSurface {
            closing: true,
            ..opening
        };
        assert_eq!(opening.corner_radius(), closing.corner_radius());
        assert_eq!(opening.content_opacity(), closing.content_opacity());
        assert_eq!(opening.release(), closing.release());
    }
}
