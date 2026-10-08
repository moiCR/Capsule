use gpui::{Bounds, Pixels, Size, point, px};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SatelliteLayout {
    pub bounds: Bounds<Pixels>,
    pub content_size: Size<Pixels>,
    pub side: Side,
}

pub(super) fn centered_y(
    capsule: Bounds<Pixels>,
    height: Pixels,
    viewport_height: Pixels,
) -> Pixels {
    (capsule.origin.y + (capsule.size.height - height) / 2.0)
        .clamp(px(0.0), (viewport_height - height).max(px(0.0)))
}

impl SatelliteLayout {
    pub fn new(
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        desired: Size<Pixels>,
        gap: Pixels,
        preferred: Side,
        progress: f32,
    ) -> Self {
        let gap = gap.max(px(0.0));
        let width = desired.width.min(viewport.width).max(px(1.0));
        let left_space = capsule.origin.x - gap;
        let right_edge = capsule.origin.x + capsule.size.width;
        let right_space = viewport.width - right_edge - gap;
        let side = if preferred == Side::Right && right_space >= width {
            Side::Right
        } else if left_space >= width {
            Side::Left
        } else if right_space >= width {
            Side::Right
        } else {
            Side::Below
        };
        let target_y = match side {
            Side::Below => capsule.origin.y + capsule.size.height + gap,
            _ => capsule.origin.y,
        };
        let height = desired
            .height
            .min((viewport.height - target_y).max(px(1.0)));
        let y = target_y.min((viewport.height - height).max(px(0.0)));
        let content_size = gpui::size(width, height);
        let progress = progress.clamp(0.0, 1.0);
        let offset = px(16.0) * (1.0 - progress);
        let (x, y) = match side {
            Side::Left => (capsule.origin.x - gap - width + offset, y),
            Side::Right => (right_edge + gap - offset, y),
            Side::Below => (
                (capsule.origin.x + (capsule.size.width - width) / 2.0)
                    .clamp(px(0.0), (viewport.width - width).max(px(0.0))),
                (y - offset).max(px(0.0)),
            ),
        };
        Self {
            bounds: Bounds {
                origin: point(x, y),
                size: content_size,
            },
            content_size,
            side,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_side_falls_back_only_when_it_cannot_fit() {
        let viewport = gpui::size(px(1300.0), px(600.0));
        let desired = gpui::size(px(300.0), px(350.0));
        for (x, preferred, expected) in [
            (350.0, Side::Left, Side::Left),
            (350.0, Side::Right, Side::Right),
            (20.0, Side::Left, Side::Right),
            (900.0, Side::Right, Side::Left),
        ] {
            let capsule = Bounds {
                origin: point(px(x), px(8.0)),
                size: gpui::size(px(300.0), px(480.0)),
            };
            let initial = SatelliteLayout::new(viewport, capsule, desired, px(8.0), preferred, 0.0);
            let final_layout =
                SatelliteLayout::new(viewport, capsule, desired, px(8.0), preferred, 1.0);
            assert_eq!(final_layout.side, expected);
            assert_eq!(initial.content_size, final_layout.content_size);
            let direction = if expected == Side::Left { -1.0 } else { 1.0 };
            assert_eq!(
                final_layout.bounds.origin.x - initial.bounds.origin.x,
                px(16.0 * direction)
            );
            assert!(final_layout.bounds.origin.x >= px(0.0));
            assert!(final_layout.bounds.right() <= viewport.width);
        }
    }

    #[test]
    fn panels_fit_and_reveal_from_the_nearest_edge() {
        let capsule = Bounds {
            origin: point(px(350.0), px(8.0)),
            size: gpui::size(px(250.0), px(40.0)),
        };
        let desired = gpui::size(px(300.0), px(350.0));
        for step in 0..=10 {
            let layout = SatelliteLayout::new(
                gpui::size(px(950.0), px(500.0)),
                capsule,
                desired,
                px(8.0),
                Side::Left,
                step as f32 / 10.0,
            );
            assert!(layout.bounds.origin.x >= px(0.0));
            assert!(
                layout.bounds.origin.x + layout.bounds.size.width <= capsule.origin.x + px(8.0)
            );
            assert_eq!(layout.content_size, desired);
            assert_eq!(layout.bounds.size, desired);
        }
        let narrow = SatelliteLayout::new(
            gpui::size(px(400.0), px(300.0)),
            Bounds {
                origin: point(px(75.0), px(8.0)),
                ..capsule
            },
            desired,
            px(8.0),
            Side::Left,
            1.0,
        );
        assert!(matches!(narrow.side, Side::Below));
        assert!(narrow.bounds.origin.x >= px(0.0));
        assert!(narrow.bounds.origin.y + narrow.bounds.size.height <= px(300.0));
    }
}
