use super::{BADGE_OVERHANG, ORB_SIZE, OrbitId, OrbitVisual, Snapshot, animator::Animator};
use gpui::{Bounds, Pixels, Size, point, px};
use services::RecordStatus;
use std::time::{Duration, Instant};

pub(super) struct State {
    pub snapshot: Snapshot,
    shelf: Animator,
    recording: Animator,
}

impl State {
    pub fn new() -> Self {
        Self {
            snapshot: Snapshot::default(),
            shelf: Animator::new(),
            recording: Animator::new(),
        }
    }

    pub fn update(&mut self, state: Snapshot, duration: Duration, now: Instant) -> bool {
        if self.snapshot == state {
            return false;
        }
        self.snapshot = state;
        self.shelf.transition(state.shelf_count > 0, duration, now);
        self.recording
            .transition(state.record_status != RecordStatus::Stopped, duration, now);
        true
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let shelf = self.shelf.advance(now);
        self.recording.advance(now) || shelf
    }

    pub fn animating(&self) -> bool {
        self.shelf.animating() || self.recording.animating()
    }

    pub fn satellite_gap(gap: Pixels) -> Pixels {
        gap.max(px(12.0)) * 2.0 + px(ORB_SIZE + BADGE_OVERHANG)
    }

    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
    ) -> impl Iterator<Item = OrbitVisual> {
        let gap = gap.max(px(12.0));
        [
            (OrbitId::Shelf, &self.shelf),
            (OrbitId::Recording, &self.recording),
        ]
        .into_iter()
        .filter_map(move |(id, animator)| {
            let progress = animator.progress();
            if progress <= 0.005 {
                return None;
            }
            let offset = px(8.0 * (1.0 - progress));
            let x = match id {
                OrbitId::Shelf => capsule.origin.x - gap - px(ORB_SIZE) + offset,
                OrbitId::Recording => capsule.right() + gap - offset,
            };
            let y = (capsule.origin.y + px(7.0)).clamp(
                px(0.0),
                (viewport.height - px(ORB_SIZE + BADGE_OVERHANG)).max(px(0.0)),
            );
            let bounds = Bounds {
                origin: point(x, y),
                size: gpui::size(px(ORB_SIZE), px(ORB_SIZE)),
            };
            if bounds.origin.x < px(0.0) || bounds.right() + px(BADGE_OVERHANG) > viewport.width {
                return None;
            }
            Some(OrbitVisual {
                id,
                bounds,
                opacity: progress,
                interactive: animator.active() && progress >= 0.85,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_capsule::{
        CapsuleLocation,
        satellite::layout::{SatelliteLayout, Side},
        window_state::visible_bounds,
    };

    fn active() -> State {
        let mut state = State::new();
        state.update(
            Snapshot {
                shelf_count: 3,
                record_status: RecordStatus::Recording,
            },
            Duration::ZERO,
            Instant::now(),
        );
        state
    }

    #[test]
    fn orbits_follow_module_bounds_without_resetting_state() {
        let state = active();
        let viewport = gpui::size(px(1500.0), px(900.0));
        for location in [CapsuleLocation::TOP, CapsuleLocation::BOTTOM] {
            for (width, height) in [
                (250.0, 40.0),
                (560.0, 574.0),
                (650.0, 258.0),
                (560.0, 104.0),
                (400.0, 212.0),
            ] {
                let capsule = visible_bounds(
                    viewport,
                    gpui::size(px(width), px(height)),
                    px(8.0),
                    &location,
                );
                let visuals: Vec<_> = state.layout(viewport, capsule, px(8.0)).collect();
                assert_eq!(visuals.len(), 2);
                assert_eq!(visuals[0].id, OrbitId::Shelf);
                assert_eq!(visuals[1].id, OrbitId::Recording);
                assert!(visuals[0].bounds.right() <= capsule.origin.x - px(12.0));
                assert!(visuals[1].bounds.origin.x >= capsule.right() + px(12.0));
                for visual in visuals {
                    assert!(visual.interactive);
                    assert_eq!(
                        visual.bounds.origin.y + px(ORB_SIZE / 2.0),
                        capsule.origin.y + px(20.0)
                    );
                }
            }
        }
        assert_eq!(state.snapshot.shelf_count, 3);
        assert_eq!(state.snapshot.record_status, RecordStatus::Recording);
        assert!(!state.animating());
    }

    #[test]
    fn satellites_leave_space_for_orbits_and_the_shelf_badge() {
        let state = active();
        let viewport = gpui::size(px(1500.0), px(900.0));
        let capsule = visible_bounds(
            viewport,
            gpui::size(px(560.0), px(574.0)),
            px(0.0),
            &CapsuleLocation::TOP,
        );
        let visuals: Vec<_> = state.layout(viewport, capsule, px(8.0)).collect();
        let panel_size = gpui::size(px(320.0), px(300.0));
        let gap = State::satellite_gap(px(8.0));
        let left = SatelliteLayout::new(viewport, capsule, panel_size, gap, Side::Left, 1.0);
        let right = SatelliteLayout::new(viewport, capsule, panel_size, gap, Side::Right, 1.0);
        assert_eq!(left.side, Side::Left);
        assert_eq!(right.side, Side::Right);
        assert!(left.bounds.right() + px(12.0) <= visuals[0].bounds.origin.x);
        assert!(right.bounds.origin.x >= visuals[1].bounds.right() + px(BADGE_OVERHANG + 12.0));
    }

    #[test]
    fn removal_retracts_only_the_affected_orbit_and_finishes() {
        let now = Instant::now();
        let mut state = active();
        let viewport = gpui::size(px(1000.0), px(800.0));
        let capsule = visible_bounds(
            viewport,
            gpui::size(px(250.0), px(40.0)),
            px(0.0),
            &CapsuleLocation::TOP,
        );
        let snapshot = Snapshot {
            shelf_count: 0,
            record_status: RecordStatus::Paused,
        };
        assert!(state.update(snapshot, Duration::from_millis(200), now));
        assert!(state.animating());
        let visuals: Vec<_> = state.layout(viewport, capsule, px(0.0)).collect();
        assert!(!visuals[0].interactive);
        assert!(visuals[1].interactive);
        assert!(!state.advance(now + Duration::from_millis(300)));
        assert!(!state.animating());
        let visuals: Vec<_> = state.layout(viewport, capsule, px(0.0)).collect();
        assert_eq!(visuals.len(), 1);
        assert_eq!(visuals[0].id, OrbitId::Recording);
        assert!(!state.update(snapshot, Duration::ZERO, now + Duration::from_secs(1)));
        state.update(
            Snapshot::default(),
            Duration::ZERO,
            now + Duration::from_secs(1),
        );
        assert_eq!(state.layout(viewport, capsule, px(0.0)).count(), 0);
    }

    #[test]
    fn narrow_viewports_do_not_create_invisible_input_regions() {
        let state = active();
        let viewport = gpui::size(px(560.0), px(700.0));
        let capsule = visible_bounds(
            viewport,
            gpui::size(px(560.0), px(574.0)),
            px(0.0),
            &CapsuleLocation::TOP,
        );
        assert_eq!(state.layout(viewport, capsule, px(8.0)).count(), 0);
    }
}
