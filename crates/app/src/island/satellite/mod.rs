mod animator;
pub(crate) mod layout;

use super::{Capsule, module::CapsuleModuleId};
use animator::SatelliteAnimator;
use gpui::{Bounds, Context, Pixels, Size, px};
use layout::{SatelliteLayout, Side};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SatelliteId {
    Calendar,
    Wifi,
    Bluetooth,
    Audio,
    Tray { bus: String, path: String },
}

impl SatelliteId {
    pub(crate) fn title_key(&self) -> &'static str {
        match self {
            Self::Calendar => "dashboard_new.calendar",
            Self::Wifi => "quick_settings.wifi_networks",
            Self::Bluetooth => "quick_settings.bt_devices",
            Self::Audio => "dashboard_new.audio_output",
            Self::Tray { .. } => "dashboard_new.tray",
        }
    }

    fn size(&self) -> Size<Pixels> {
        match self {
            Self::Calendar => gpui::size(
                px(super::widgets::satellite::calendar::WIDTH),
                px(super::widgets::satellite::calendar::HEIGHT),
            ),
            Self::Audio => gpui::size(px(320.0), px(300.0)),
            _ => gpui::size(px(320.0), px(400.0)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SatelliteVisual {
    pub id: SatelliteId,
    pub layout: SatelliteLayout,
    pub progress: f32,
    pub open: bool,
}

struct Panel {
    id: SatelliteId,
    side: Side,
    animator: SatelliteAnimator,
}

pub struct Satellite {
    panels: Vec<Panel>,
    animation: services::CapsuleAnimation,
    stagger: bool,
    reduced_motion: bool,
}

impl Satellite {
    pub fn new() -> Self {
        Self {
            panels: Vec::new(),
            animation: services::CapsuleAnimation::default(),
            stagger: true,
            reduced_motion: false,
        }
    }

    pub fn set_animation(&mut self, animation: services::CapsuleAnimation) {
        self.animation = animation;
        for panel in &mut self.panels {
            panel.animator.set_animation(animation);
        }
    }

    pub fn set_stagger(&mut self, stagger: bool) {
        self.stagger = stagger;
    }

    pub fn set_reduced_motion(&mut self, reduced_motion: bool) {
        self.reduced_motion = reduced_motion;
    }

    pub fn open(&self) -> bool {
        self.panels.iter().any(|panel| panel.animator.open())
    }
    pub fn contains(&self, id: &SatelliteId) -> bool {
        self.panels.iter().any(|panel| &panel.id == id)
    }
    pub fn maximum_size() -> Size<Pixels> {
        gpui::size(px(320.0), px(400.0))
    }

    fn next_side(&self, id: &SatelliteId) -> Side {
        if let Some(panel) = self.panels.iter().find(|panel| &panel.id == id) {
            return panel.side;
        }
        [Side::Left, Side::Right]
            .into_iter()
            .find(|side| {
                !self
                    .panels
                    .iter()
                    .any(|panel| panel.side == *side && panel.animator.open())
            })
            .or_else(|| {
                self.panels
                    .iter()
                    .find(|panel| panel.animator.open())
                    .map(|panel| panel.side)
            })
            .unwrap_or(Side::Left)
    }

    #[allow(dead_code)]
    fn transition(&mut self, id: SatelliteId, open: bool, duration: Duration, side: Side) {
        let other_animating = self
            .panels
            .iter()
            .any(|panel| panel.id != id && panel.side == side && panel.animator.animating());
        let delay = if self.stagger && !self.reduced_motion && other_animating {
            Duration::from_millis(25)
        } else {
            Duration::ZERO
        };
        self.transition_with_delay(id, open, duration, delay, side);
    }

    fn transition_with_delay(
        &mut self,
        id: SatelliteId,
        open: bool,
        duration: Duration,
        delay: Duration,
        side: Side,
    ) {
        let duration = if self.reduced_motion {
            Duration::ZERO
        } else {
            duration
        };
        let delay = if self.reduced_motion || !self.stagger {
            Duration::ZERO
        } else {
            delay
        };
        if open {
            self.panels
                .retain(|panel| panel.id == id || panel.side != side);
            if let Some(index) = self.panels.iter().position(|panel| panel.id == id) {
                let panel = self.panels.remove(index);
                self.panels.push(panel);
            }
        } else if !self.contains(&id) {
            return;
        }
        if !self.contains(&id) {
            self.panels.push(Panel {
                id: id.clone(),
                side,
                animator: SatelliteAnimator::new().with_animation(self.animation),
            });
        }
        if let Some(panel) = self.panels.iter_mut().find(|panel| panel.id == id) {
            panel.side = side;
            panel.animator.set_animation(self.animation);
            panel
                .animator
                .transition_with_delay(open, duration, delay, Instant::now());
        }
    }

    fn advance(&mut self, now: Instant) -> bool {
        let mut running = false;
        for panel in &mut self.panels {
            running |= panel.animator.advance(now);
        }
        self.panels
            .retain(|panel| panel.animator.open() || panel.animator.progress() > 0.0);
        running
    }

    #[allow(dead_code)]
    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
        measured: &[(SatelliteId, Bounds<Pixels>)],
    ) -> Vec<SatelliteVisual> {
        self.layout_with_gaps(viewport, capsule, gap, gap, measured)
    }

    pub fn layout_with_gaps(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        left_gap: Pixels,
        right_gap: Pixels,
        measured: &[(SatelliteId, Bounds<Pixels>)],
    ) -> Vec<SatelliteVisual> {
        let mut visuals: Vec<_> = self
            .panels
            .iter()
            .filter(|panel| panel.animator.progress() > 0.0)
            .map(|panel| SatelliteVisual {
                id: panel.id.clone(),
                layout: SatelliteLayout::with_gaps(
                    viewport,
                    capsule,
                    panel.id.size(),
                    left_gap,
                    right_gap,
                    panel.side,
                    panel.animator.progress(),
                ),
                progress: panel.animator.progress(),
                open: panel.animator.open(),
            })
            .collect();
        for visual in &mut visuals {
            if visual.layout.side == Side::Below {
                continue;
            }
            let maximum = visual.id.size().height.min(viewport.height);
            visual.layout.content_size.height = maximum;
            visual.layout.bounds.size.height = maximum;
            let visible = measured
                .iter()
                .find(|(id, _)| *id == visual.id)
                .map(|(_, bounds)| bounds.size.height)
                .unwrap_or(px(72.0))
                .min(maximum);
            visual.layout.bounds.origin.y = layout::centered_y(capsule, visible, viewport.height);
        }
        visuals
    }
}

impl Capsule {
    pub(super) fn toggle_satellite(&mut self, id: SatelliteId, cx: &mut Context<Self>) {
        if self.module_manager.current_id() != CapsuleModuleId::Dashboard {
            return;
        }
        let open = !self
            .satellite
            .panels
            .iter()
            .any(|panel| panel.id == id && panel.animator.open());
        self.transition_satellite_with_delay(id, open, Duration::ZERO, cx);
    }

    pub(super) fn set_satellite_open(&mut self, open: bool, cx: &mut Context<Self>) {
        let panels: Vec<_> = self
            .satellite
            .panels
            .iter()
            .map(|panel| (panel.id.clone(), panel.side))
            .collect();
        let mut left_idx = 0;
        let mut right_idx = 0;
        let mut other_idx = 0;
        for (id, side) in panels {
            let idx = match side {
                Side::Left => {
                    let i = left_idx;
                    left_idx += 1;
                    i
                }
                Side::Right => {
                    let i = right_idx;
                    right_idx += 1;
                    i
                }
                _ => {
                    let i = other_idx;
                    other_idx += 1;
                    i
                }
            };
            let delay = Duration::from_millis(idx as u64 * 30);
            self.transition_satellite_with_delay(id, open, delay, cx);
        }
    }

    pub(super) fn close_satellite(&mut self, id: SatelliteId, cx: &mut Context<Self>) {
        self.transition_satellite_with_delay(id, false, Duration::ZERO, cx);
    }

    fn transition_satellite_with_delay(
        &mut self,
        id: SatelliteId,
        open: bool,
        delay: Duration,
        cx: &mut Context<Self>,
    ) {
        let mut destination = self.satellite.next_side(&id);
        if open {
            let handle = self.window_state.handle;
            let _ = handle.update(cx, |_, window, cx| {
                let viewport = window.viewport_size();
                let capsule = super::window_state::visible_bounds(
                    viewport,
                    self.animator.size(),
                    px(self.window_state.margin),
                    &self.location,
                );
                let (left_gap, right_gap) = self.orbit.read(cx).layout_gaps(px(cx
                    .global::<services::AppState>()
                    .config
                    .get()
                    .ui
                    .gap));
                let gap = match destination {
                    Side::Left => left_gap,
                    Side::Right => right_gap,
                    Side::Below => left_gap.min(right_gap),
                };
                destination =
                    SatelliteLayout::new(viewport, capsule, id.size(), gap, destination, 1.0).side;
            });
        }
        let state = cx.global::<services::AppState>();
        let ui = &state.config.get().ui;
        let duration = if ui.animation_duration_ms == 0 || ui.reduced_motion {
            Duration::ZERO
        } else {
            Duration::from_millis(if open { 180 } else { 130 })
        };
        let compositor = state.compositor.clone();
        if open {
            match &id {
                SatelliteId::Calendar => state.calendar.reset_to_today(),
                SatelliteId::Wifi => state.network.rescan_wifi(),
                SatelliteId::Bluetooth => state.network.start_bluetooth_scan(),
                _ => {}
            }
        }
        self.satellite
            .transition_with_delay(id, open, duration, delay, destination);
        self.sync_window(cx);
        if self.satellite_animation_task.is_none() && self.satellite.advance(Instant::now()) {
            self.satellite_animation_task = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(
                            compositor
                                .get_frame_duration()
                                .max(Duration::from_millis(2)),
                        )
                        .await;
                    let running = this.update(cx, |capsule, cx| {
                        let running = capsule.satellite.advance(Instant::now());
                        if !running {
                            capsule.satellite_animation_task = None;
                        }
                        capsule.sync_window(cx);
                        cx.notify();
                        running
                    });
                    match running {
                        Ok(true) => {}
                        Ok(false) | Err(_) => break,
                    }
                }
            }));
        }
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(satellites: &mut Satellite, id: SatelliteId, open: bool, duration: Duration) {
        let side = satellites.next_side(&id);
        satellites.transition(id, open, duration, side);
    }

    #[test]
    fn panels_fill_both_sides_and_replace_the_oldest() {
        let mut satellites = Satellite::new();
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(
            &mut satellites,
            SatelliteId::Bluetooth,
            true,
            Duration::ZERO,
        );
        assert_eq!(satellites.panels[0].side, Side::Left);
        assert_eq!(satellites.panels[1].side, Side::Right);
        transition(&mut satellites, SatelliteId::Audio, true, Duration::ZERO);
        assert!(!satellites.contains(&SatelliteId::Wifi));
        assert_eq!(satellites.panels[1].side, Side::Left);
        transition(&mut satellites, SatelliteId::Calendar, true, Duration::ZERO);
        assert!(!satellites.contains(&SatelliteId::Bluetooth));
        assert_eq!(satellites.panels[1].side, Side::Right);
        assert_eq!(satellites.panels.len(), 2);
    }

    #[test]
    fn reopening_a_closing_panel_makes_it_the_newest() {
        let mut satellites = Satellite::new();
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(
            &mut satellites,
            SatelliteId::Bluetooth,
            true,
            Duration::ZERO,
        );
        transition(
            &mut satellites,
            SatelliteId::Wifi,
            false,
            Duration::from_millis(130),
        );
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(&mut satellites, SatelliteId::Audio, true, Duration::ZERO);
        assert!(satellites.contains(&SatelliteId::Wifi));
        assert!(!satellites.contains(&SatelliteId::Bluetooth));
    }

    #[test]
    fn a_closed_side_is_reused_without_evicting_the_other() {
        let mut satellites = Satellite::new();
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(
            &mut satellites,
            SatelliteId::Bluetooth,
            true,
            Duration::ZERO,
        );
        transition(&mut satellites, SatelliteId::Wifi, false, Duration::ZERO);
        satellites.advance(Instant::now());
        transition(&mut satellites, SatelliteId::Audio, true, Duration::ZERO);
        assert!(satellites.contains(&SatelliteId::Bluetooth));
        assert_eq!(satellites.panels[1].side, Side::Left);
    }

    #[test]
    fn opening_and_closing_panels_preserves_the_others() {
        let mut satellites = Satellite::new();
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(
            &mut satellites,
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            true,
            Duration::ZERO,
        );
        assert_eq!(satellites.panels.len(), 2);
        transition(
            &mut satellites,
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            false,
            Duration::ZERO,
        );
        assert!(!satellites.advance(Instant::now()));
        assert!(satellites.contains(&SatelliteId::Wifi));
        assert!(!satellites.contains(&SatelliteId::Tray {
            bus: "test".into(),
            path: "/test".into()
        }));
        assert!(satellites.open());
        transition(
            &mut satellites,
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            true,
            Duration::ZERO,
        );
        assert_eq!(satellites.panels.len(), 2);
    }

    #[test]
    fn one_panel_per_side_is_centered_on_the_capsule() {
        let mut satellites = Satellite::new();
        transition(&mut satellites, SatelliteId::Wifi, true, Duration::ZERO);
        transition(
            &mut satellites,
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            true,
            Duration::ZERO,
        );
        transition(&mut satellites, SatelliteId::Calendar, true, Duration::ZERO);
        assert_eq!(satellites.panels.len(), 2);
        assert!(!satellites.contains(&SatelliteId::Wifi));
        assert!(satellites.contains(&SatelliteId::Tray {
            bus: "test".into(),
            path: "/test".into()
        }));
        assert!(satellites.contains(&SatelliteId::Calendar));
        let viewport = gpui::size(px(1400.0), px(900.0));
        let capsule = Bounds {
            origin: gpui::point(px(380.0), px(8.0)),
            size: gpui::size(px(640.0), px(480.0)),
        };
        for height in [72.0, 150.0, 320.0] {
            let measured = [(
                SatelliteId::Tray {
                    bus: "test".into(),
                    path: "/test".into(),
                },
                Bounds {
                    origin: gpui::point(px(0.0), px(0.0)),
                    size: gpui::size(px(320.0), px(height)),
                },
            )];
            let visuals = satellites.layout(viewport, capsule, px(8.0), &measured);
            assert_eq!(visuals.len(), 2);
            let tray = visuals
                .iter()
                .find(|visual| {
                    visual.id
                        == SatelliteId::Tray {
                            bus: "test".into(),
                            path: "/test".into(),
                        }
                })
                .expect("tray satellite");
            assert_eq!(tray.layout.side, Side::Right);
            assert_eq!(
                tray.layout.bounds.origin.y + px(height / 2.0),
                capsule.origin.y + capsule.size.height / 2.0
            );
            let calendar = visuals
                .iter()
                .find(|visual| visual.id == SatelliteId::Calendar)
                .expect("calendar satellite");
            assert_eq!(calendar.layout.side, Side::Left);
            assert_eq!(calendar.layout.content_size.height, px(300.0));
        }
    }
}
