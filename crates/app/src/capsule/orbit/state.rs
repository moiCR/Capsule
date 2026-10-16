use super::{
    BADGE_OVERHANG, ORB_SIZE, OrbitId, OrbitVisual, POWER_PANEL_HEIGHT, Snapshot,
    USAGE_PANEL_HEIGHT, USAGE_PANEL_WIDTH, animator::Animator,
};
use crate::capsule::CapsuleLocation;
use gpui::{Bounds, Pixels, Size, point, px};
use services::RecordStatus;
use std::time::{Duration, Instant};

pub(super) struct OrbExpanded {
    pub open: bool,
    animator: crate::capsule::animator::Animator,
    height: f32,
}

impl OrbExpanded {
    fn new(height: f32) -> Self {
        Self {
            open: false,
            height,
            animator: crate::capsule::animator::Animator::new(
                gpui::size(px(ORB_SIZE), px(ORB_SIZE)),
                Duration::ZERO,
            ),
        }
    }

    fn set_open(&mut self, open: bool, duration: Duration, now: Instant) {
        self.set_open_with_delay(open, duration, Duration::ZERO, now);
    }

    fn set_open_with_delay(
        &mut self,
        open: bool,
        duration: Duration,
        delay: Duration,
        now: Instant,
    ) {
        self.open = open;
        self.animator.set_duration(duration);
        self.animator.transition_to_with_delay(
            if open {
                gpui::size(px(USAGE_PANEL_WIDTH), px(self.height))
            } else {
                gpui::size(px(ORB_SIZE), px(ORB_SIZE))
            },
            delay,
            now,
        );
        if open {
            self.animator.reveal_content(now);
        }
    }

    fn set_animation(&mut self, animation: services::CapsuleAnimation) {
        self.animator.set_animation(animation);
    }

    fn set_squash_stretch(&mut self, squash_stretch: bool) {
        self.animator.set_squash_stretch(squash_stretch);
    }

    fn set_reduced_motion(&mut self, reduced_motion: bool) {
        self.animator.set_reduced_motion(reduced_motion);
    }

    #[allow(dead_code)]
    pub(super) fn extra_width(&self) -> Pixels {
        self.animator.size().width - px(ORB_SIZE)
    }

    pub(super) fn content_opacity(&self) -> f32 {
        let size = self.animator.size();
        ((f32::from(size.height) - ORB_SIZE) / (self.height - ORB_SIZE)).clamp(0.0, 1.0)
            * self.animator.content_progress()
    }

    fn bounds(
        &self,
        orb: Bounds<Pixels>,
        viewport: Size<Pixels>,
        left: bool,
        bottom: bool,
    ) -> Bounds<Pixels> {
        let available_width = if left {
            orb.right()
        } else {
            viewport.width - orb.origin.x
        };
        let available_height = if bottom {
            orb.bottom()
        } else {
            viewport.height - orb.origin.y
        };
        let animated = self.animator.size();
        let size = gpui::size(
            animated.width.min(available_width),
            animated.height.min(available_height),
        );
        Bounds {
            origin: point(
                if left {
                    orb.right() - size.width
                } else {
                    orb.origin.x
                },
                if bottom {
                    orb.bottom() - size.height
                } else {
                    orb.origin.y
                },
            ),
            size,
        }
    }
}

pub(super) struct State {
    pub snapshot: Snapshot,
    shelf: Animator,
    recording: Animator,
    pub expanded: OrbExpanded,
    pub power_expanded: OrbExpanded,
    orbits: services::config::ui_config::OrbitConfig,
    stagger: bool,
    reduced_motion: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            snapshot: Snapshot::default(),
            shelf: Animator::new(),
            recording: Animator::new(),
            expanded: OrbExpanded::new(USAGE_PANEL_HEIGHT),
            power_expanded: OrbExpanded::new(POWER_PANEL_HEIGHT),
            orbits: services::config::ui_config::OrbitConfig::default(),
            stagger: true,
            reduced_motion: false,
        }
    }

    fn effective_duration(&self, duration: Duration) -> Duration {
        if self.reduced_motion {
            Duration::ZERO
        } else {
            duration
        }
    }

    pub fn update(&mut self, state: Snapshot, duration: Duration, now: Instant) -> bool {
        if self.snapshot == state {
            return false;
        }
        let duration = self.effective_duration(duration);
        let changed = self.snapshot.shelf_count != state.shelf_count
            || self.snapshot.record_status != state.record_status
            || self.expanded.open;
        let shelf_target = self.orbits.shelf && state.shelf_count > 0;
        let record_target = self.orbits.record && state.record_status != RecordStatus::Stopped;
        self.snapshot = state;
        self.shelf
            .transition_with_delay(shelf_target, duration, Duration::ZERO, now);
        self.recording
            .transition_with_delay(record_target, duration, Duration::ZERO, now);
        changed
    }

    pub fn set_animation(&mut self, animation: services::CapsuleAnimation) {
        self.shelf.set_animation(animation);
        self.recording.set_animation(animation);
        self.expanded.set_animation(animation);
        self.power_expanded.set_animation(animation);
    }

    pub fn set_motion_options(
        &mut self,
        stagger: bool,
        squash_stretch: bool,
        reduced_motion: bool,
    ) {
        self.stagger = stagger;
        self.reduced_motion = reduced_motion;
        self.expanded.set_squash_stretch(squash_stretch);
        self.power_expanded.set_squash_stretch(squash_stretch);
        self.expanded.set_reduced_motion(reduced_motion);
        self.power_expanded.set_reduced_motion(reduced_motion);
    }

    pub fn configure(
        &mut self,
        orbits: services::config::ui_config::OrbitConfig,
        duration: Duration,
        now: Instant,
    ) {
        let duration = self.effective_duration(duration);
        self.orbits = orbits;
        if !orbits.usage && self.expanded.open {
            self.expanded.set_open(false, duration, now);
        }
        if !orbits.power && self.power_expanded.open {
            self.power_expanded.set_open(false, duration, now);
        }
        let shelf_target = orbits.shelf && self.snapshot.shelf_count > 0;
        let record_target = orbits.record && self.snapshot.record_status != RecordStatus::Stopped;
        self.shelf
            .transition_with_delay(shelf_target, duration, Duration::ZERO, now);
        self.recording
            .transition_with_delay(record_target, duration, Duration::ZERO, now);
    }

    pub fn toggle_usage(&mut self, duration: Duration, now: Instant) {
        if !self.orbits.usage {
            return;
        }
        let duration = self.effective_duration(duration);
        if self.power_expanded.open {
            self.power_expanded.set_open(false, duration, now);
        }
        self.expanded.set_open(!self.expanded.open, duration, now);
    }

    pub fn toggle_power(&mut self, duration: Duration, now: Instant) {
        if !self.orbits.power {
            return;
        }
        let duration = self.effective_duration(duration);
        if self.expanded.open {
            self.expanded.set_open(false, duration, now);
        }
        self.power_expanded
            .set_open(!self.power_expanded.open, duration, now);
    }

    pub fn preview_transition(&mut self, open: bool, duration: Duration, now: Instant) {
        let duration = self.effective_duration(duration);
        self.expanded
            .set_open_with_delay(open && self.orbits.usage, duration, Duration::ZERO, now);
        self.power_expanded
            .set_open_with_delay(open && self.orbits.power, duration, Duration::ZERO, now);
    }

    pub fn close(&mut self, duration: Duration, now: Instant) {
        let duration = self.effective_duration(duration);
        if self.expanded.open {
            self.expanded.set_open(false, duration, now);
        }
        if self.power_expanded.open {
            self.power_expanded.set_open(false, duration, now);
        }
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let shelf = self.shelf.advance(now);
        let recording = self.recording.advance(now);
        let usage = self.expanded.animator.advance(now);
        self.power_expanded.animator.advance(now) || usage || shelf || recording
    }

    pub fn animating(&self) -> bool {
        self.shelf.animating()
            || self.recording.animating()
            || self.expanded.animator.animating()
            || self.power_expanded.animator.animating()
    }

    pub fn satellite_gap(gap: Pixels) -> Pixels {
        let gap = gap.max(px(12.0));
        gap * 2.0 + px(ORB_SIZE)
    }

    pub fn layout_gaps(&self, gap: Pixels) -> (Pixels, Pixels) {
        let base_gap = gap.max(px(12.0));

        let shelf_active = self.orbits.shelf
            && (self.snapshot.shelf_count > 0 || self.shelf.animating())
            && self.shelf.progress() > 0.005;
        let left_gap = if shelf_active {
            let power_width = if self.orbits.power {
                self.power_expanded.animator.size().width
            } else {
                px(0.0)
            };
            base_gap * 3.0 + power_width + px(ORB_SIZE)
        } else if self.orbits.power {
            base_gap * 2.0 + self.power_expanded.animator.size().width
        } else {
            base_gap
        };

        let record_active = self.orbits.record
            && (self.snapshot.record_status != RecordStatus::Stopped || self.recording.animating())
            && self.recording.progress() > 0.005;
        let right_gap = if record_active {
            let usage_width = if self.orbits.usage {
                self.expanded.animator.size().width
            } else {
                px(0.0)
            };
            base_gap * 3.0 + usage_width + px(ORB_SIZE)
        } else if self.orbits.usage {
            base_gap * 2.0 + self.expanded.animator.size().width
        } else {
            base_gap
        };

        (left_gap, right_gap)
    }

    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
        location: CapsuleLocation,
    ) -> impl Iterator<Item = OrbitVisual> {
        let gap = gap.max(px(12.0));
        let bottom = location == CapsuleLocation::BOTTOM;
        let y = if bottom {
            capsule.bottom() - px(ORB_SIZE + 7.0)
        } else {
            capsule.origin.y + px(7.0)
        };
        [
            OrbitId::Shelf,
            OrbitId::Recording,
            OrbitId::Usage,
            OrbitId::Power,
        ]
        .into_iter()
        .filter_map(move |id| {
            let (enabled, progress, interactive) = match id {
                OrbitId::Shelf => (
                    self.orbits.shelf,
                    self.shelf.progress(),
                    self.shelf.active() && self.shelf.progress() >= 0.85,
                ),
                OrbitId::Recording => (
                    self.orbits.record,
                    self.recording.progress(),
                    self.recording.active() && self.recording.progress() >= 0.85,
                ),
                OrbitId::Usage => (self.orbits.usage, 1.0, true),
                OrbitId::Power => (self.orbits.power, 1.0, true),
            };
            if !enabled || progress <= 0.005 {
                return None;
            }
            let offset = px(8.0 * (1.0 - progress));
            let left_space = if self.orbits.power {
                self.power_expanded.animator.size().width + gap
            } else {
                px(0.0)
            };
            let right_space = if self.orbits.usage {
                self.expanded.animator.size().width + gap
            } else {
                px(0.0)
            };
            let x = match id {
                OrbitId::Shelf => capsule.origin.x - gap - left_space - px(ORB_SIZE) + offset,
                OrbitId::Recording => capsule.right() + gap + right_space - offset,
                OrbitId::Usage => capsule.right() + gap,
                OrbitId::Power => capsule.origin.x - gap - px(ORB_SIZE),
            };
            let orb = Bounds {
                origin: point(x, y),
                size: gpui::size(px(ORB_SIZE), px(ORB_SIZE)),
            };
            if orb.origin.x < px(0.0)
                || orb.right() + px(BADGE_OVERHANG) > viewport.width
                || y < px(0.0)
                || orb.bottom() > viewport.height
            {
                return None;
            }
            let bounds = match id {
                OrbitId::Usage => self.expanded.bounds(orb, viewport, false, bottom),
                OrbitId::Power => self.power_expanded.bounds(orb, viewport, true, bottom),
                _ => orb,
            };
            Some(OrbitVisual {
                id,
                bounds,
                opacity: progress,
                interactive,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capsule::{
        CapsuleLocation,
        satellite::layout::{SatelliteLayout, Side},
        window_state::visible_bounds,
    };

    fn active() -> State {
        let mut state = State::new();
        state.configure(
            services::config::ui_config::OrbitConfig {
                power: false,
                ..Default::default()
            },
            Duration::ZERO,
            Instant::now(),
        );
        state.update(
            Snapshot {
                shelf_count: 3,
                record_status: RecordStatus::Recording,
                ..Snapshot::default()
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
                let visuals: Vec<_> = state.layout(viewport, capsule, px(8.0), location).collect();
                assert_eq!(visuals.len(), 3);
                assert_eq!(visuals[0].id, OrbitId::Shelf);
                assert_eq!(visuals[1].id, OrbitId::Recording);
                assert!(visuals[0].bounds.right() <= capsule.origin.x - px(12.0));
                assert!(visuals[1].bounds.origin.x >= capsule.right() + px(12.0));
                for visual in visuals {
                    assert!(visual.interactive);
                    assert_eq!(
                        visual.bounds.origin.y + px(ORB_SIZE / 2.0),
                        match location {
                            CapsuleLocation::TOP => capsule.origin.y + px(20.0),
                            CapsuleLocation::BOTTOM => capsule.bottom() - px(20.0),
                        }
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
        let visuals: Vec<_> = state
            .layout(viewport, capsule, px(8.0), CapsuleLocation::TOP)
            .collect();
        let panel_size = gpui::size(px(320.0), px(300.0));
        let (left_gap, right_gap) = state.layout_gaps(px(8.0));
        let left = SatelliteLayout::new(viewport, capsule, panel_size, left_gap, Side::Left, 1.0);
        let right = SatelliteLayout::new(viewport, capsule, panel_size, right_gap, Side::Right, 1.0);
        assert_eq!(left.side, Side::Left);
        assert_eq!(right.side, Side::Right);
        assert!(left.bounds.right() <= visuals[0].bounds.origin.x);
        assert!(right.bounds.origin.x >= visuals[1].bounds.right());
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
            ..Snapshot::default()
        };
        assert!(state.update(snapshot, Duration::from_millis(200), now));
        assert!(state.animating());
        let visuals: Vec<_> = state
            .layout(viewport, capsule, px(0.0), CapsuleLocation::TOP)
            .collect();
        assert!(!visuals[0].interactive);
        assert!(visuals[1].interactive);
        assert!(!state.advance(now + Duration::from_millis(300)));
        assert!(!state.animating());
        let visuals: Vec<_> = state
            .layout(viewport, capsule, px(0.0), CapsuleLocation::TOP)
            .collect();
        assert_eq!(visuals.len(), 2);
        assert_eq!(visuals[0].id, OrbitId::Recording);
        assert!(!state.update(snapshot, Duration::ZERO, now + Duration::from_secs(1)));
        state.update(
            Snapshot::default(),
            Duration::ZERO,
            now + Duration::from_secs(1),
        );
        assert_eq!(
            state
                .layout(viewport, capsule, px(0.0), CapsuleLocation::TOP)
                .count(),
            1
        );
    }

    #[test]
    fn expanded_orbs_keep_the_corner_fixed_on_both_sides_and_edges() {
        let now = Instant::now();
        let viewport = gpui::size(px(1000.0), px(800.0));
        let orb = Bounds {
            origin: point(px(400.0), px(300.0)),
            size: gpui::size(px(ORB_SIZE), px(ORB_SIZE)),
        };
        let mut state = State::new();
        state.toggle_usage(Duration::from_millis(200), now);
        for millis in [0, 50, 100, 200] {
            state.advance(now + Duration::from_millis(millis));
            for left in [false, true] {
                for bottom in [false, true] {
                    let bounds = state.expanded.bounds(orb, viewport, left, bottom);
                    assert_eq!(
                        if left {
                            bounds.right()
                        } else {
                            bounds.origin.x
                        },
                        if left { orb.right() } else { orb.origin.x }
                    );
                    assert_eq!(
                        if bottom {
                            bounds.bottom()
                        } else {
                            bounds.origin.y
                        },
                        if bottom { orb.bottom() } else { orb.origin.y }
                    );
                }
            }
        }
        assert!(!state.animating());
        state.toggle_usage(Duration::from_millis(200), now + Duration::from_millis(250));
        assert!(state.advance(now + Duration::from_millis(300)));
        let displayed = state.expanded.animator.size();
        state.toggle_usage(Duration::from_millis(200), now + Duration::from_millis(300));
        assert_eq!(state.expanded.animator.size(), displayed);
        assert!(!state.advance(now + Duration::from_millis(600)));
    }

    #[test]
    fn expansion_stays_inside_the_viewport_and_keeps_recording_clear() {
        let now = Instant::now();
        let mut state = active();
        state.toggle_usage(Duration::ZERO, now);
        let viewport = gpui::size(px(600.0), px(300.0));
        let capsule = visible_bounds(
            viewport,
            gpui::size(px(250.0), px(40.0)),
            px(8.0),
            &CapsuleLocation::TOP,
        );
        let visuals: Vec<_> = state
            .layout(viewport, capsule, px(12.0), CapsuleLocation::TOP)
            .collect();
        let usage = visuals
            .iter()
            .find(|visual| visual.id == OrbitId::Usage)
            .expect("usage panel");
        assert!(usage.bounds.right() <= viewport.width);
        assert!(usage.bounds.bottom() <= viewport.height);
        if let Some(recording) = visuals
            .iter()
            .find(|visual| visual.id == OrbitId::Recording)
        {
            assert!(recording.bounds.origin.x >= usage.bounds.right() + px(12.0));
        }
    }

    #[test]
    fn switches_remove_orbs_and_collapse_disabled_panels() {
        let now = Instant::now();
        let mut state = State::new();
        let viewport = gpui::size(px(1500.0), px(800.0));
        let capsule = visible_bounds(
            viewport,
            gpui::size(px(250.0), px(40.0)),
            px(8.0),
            &CapsuleLocation::TOP,
        );
        assert_eq!(
            state
                .layout(viewport, capsule, px(12.0), CapsuleLocation::TOP)
                .count(),
            2
        );
        state.update(
            Snapshot {
                shelf_count: 1,
                record_status: RecordStatus::Recording,
                ..Snapshot::default()
            },
            Duration::ZERO,
            now,
        );
        assert_eq!(
            state
                .layout(viewport, capsule, px(12.0), CapsuleLocation::TOP)
                .count(),
            4
        );
        state.toggle_usage(Duration::ZERO, now);
        assert!(state.expanded.open);
        state.toggle_power(Duration::ZERO, now);
        assert!(state.power_expanded.open);
        assert!(!state.expanded.open);
        state.configure(
            services::config::ui_config::OrbitConfig {
                power: false,
                usage: false,
                record: false,
                shelf: false,
            },
            Duration::ZERO,
            now,
        );
        assert!(!state.power_expanded.open);
        assert_eq!(
            state
                .layout(viewport, capsule, px(12.0), CapsuleLocation::TOP)
                .count(),
            0
        );
        state.configure(Default::default(), Duration::ZERO, now);
        assert_eq!(
            state
                .layout(viewport, capsule, px(12.0), CapsuleLocation::TOP)
                .count(),
            4
        );
        assert_eq!(state.snapshot.record_status, RecordStatus::Recording);
        assert_eq!(state.snapshot.shelf_count, 1);
    }

    #[test]
    fn settled_orbs_close_after_idle_and_usage_updates_do_not_restart_the_fade() {
        let now = Instant::now();
        let duration = Duration::from_millis(200);
        for id in [OrbitId::Usage, OrbitId::Power] {
            let mut state = State::new();
            match id {
                OrbitId::Usage => state.toggle_usage(duration, now),
                OrbitId::Power => state.toggle_power(duration, now),
                _ => unreachable!(),
            }
            assert!(!state.advance(now + duration));
            state.update(
                Snapshot {
                    usage: services::system::UsageStatus {
                        cpu: Some(42),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                duration,
                now + Duration::from_secs(10),
            );
            let expanded = match id {
                OrbitId::Usage => &state.expanded,
                _ => &state.power_expanded,
            };
            assert!(expanded.open);
            assert_eq!(expanded.content_opacity(), 1.0);
            assert!(!state.animating());
            let clicked_at = now + Duration::from_secs(30);
            match id {
                OrbitId::Usage => state.toggle_usage(duration, clicked_at),
                _ => state.toggle_power(duration, clicked_at),
            }
            assert!(!state.expanded.open && !state.power_expanded.open);
            assert!(state.advance(clicked_at + duration / 2));
            assert!(!state.advance(clicked_at + duration));
            assert_eq!(state.expanded.extra_width(), px(0.0));
            assert_eq!(state.power_expanded.extra_width(), px(0.0));
        }
    }

    #[test]
    fn preview_animates_both_corners_and_stops_after_returning() {
        let now = Instant::now();
        let mut state = State::new();
        let duration = Duration::from_millis(200);
        state.preview_transition(true, duration, now);
        assert!(state.advance(now + Duration::from_millis(100)));
        assert!(!state.advance(now + duration));
        assert!(state.expanded.extra_width() > px(0.0));
        assert!(state.power_expanded.extra_width() > px(0.0));
        state.preview_transition(false, duration, now + duration);
        assert!(!state.advance(now + duration * 2));
        assert_eq!(state.expanded.extra_width(), px(0.0));
        assert_eq!(state.power_expanded.extra_width(), px(0.0));
        assert!(!state.animating());
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
        assert_eq!(
            state
                .layout(viewport, capsule, px(8.0), CapsuleLocation::TOP)
                .count(),
            0
        );
    }
}
