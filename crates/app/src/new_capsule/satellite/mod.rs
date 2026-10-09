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

    fn preferred_side(&self) -> Side {
        match self {
            Self::Tray { .. } => Side::Right,
            Self::Calendar | Self::Wifi | Self::Bluetooth | Self::Audio => Side::Left,
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
    animator: SatelliteAnimator,
}

pub struct Satellite {
    panels: Vec<Panel>,
}

impl Satellite {
    pub fn new() -> Self {
        Self { panels: Vec::new() }
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

    fn transition(&mut self, id: SatelliteId, open: bool, duration: Duration) {
        if open {
            self.panels
                .retain(|panel| panel.id == id || panel.id.preferred_side() != id.preferred_side());
        }
        if !self.contains(&id) {
            self.panels.push(Panel {
                id: id.clone(),
                animator: SatelliteAnimator::new(),
            });
        }
        if let Some(panel) = self.panels.iter_mut().find(|panel| panel.id == id) {
            panel.animator.transition(open, duration, Instant::now());
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

    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
        measured: &[(SatelliteId, Bounds<Pixels>)],
    ) -> Vec<SatelliteVisual> {
        let mut visuals: Vec<_> = self
            .panels
            .iter()
            .filter(|panel| panel.animator.progress() > 0.0)
            .map(|panel| SatelliteVisual {
                id: panel.id.clone(),
                layout: SatelliteLayout::new(
                    viewport,
                    capsule,
                    panel.id.size(),
                    gap,
                    panel.id.preferred_side(),
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
        self.transition_satellite(id, open, cx);
    }

    pub(super) fn set_satellite_open(&mut self, open: bool, cx: &mut Context<Self>) {
        let ids: Vec<_> = self
            .satellite
            .panels
            .iter()
            .map(|panel| panel.id.clone())
            .collect();
        for id in ids {
            self.transition_satellite(id, open, cx);
        }
    }

    pub(super) fn close_satellite(&mut self, id: SatelliteId, cx: &mut Context<Self>) {
        self.transition_satellite(id, false, cx);
    }

    fn transition_satellite(&mut self, id: SatelliteId, open: bool, cx: &mut Context<Self>) {
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
                let gap = px(cx.global::<services::AppState>().config.get().ui.gap);
                let side = |id: &SatelliteId| {
                    SatelliteLayout::new(
                        viewport,
                        capsule,
                        id.size(),
                        gap,
                        id.preferred_side(),
                        1.0,
                    )
                    .side
                };
                let destination = side(&id);
                self.satellite
                    .panels
                    .retain(|panel| panel.id == id || side(&panel.id) != destination);
            });
        }
        let state = cx.global::<services::AppState>();
        let duration = if state.config.get().ui.animation_duration_ms == 0 {
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
        self.satellite.transition(id, open, duration);
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

    #[test]
    fn opening_and_closing_panels_preserves_the_others() {
        let mut satellites = Satellite::new();
        satellites.transition(SatelliteId::Wifi, true, Duration::ZERO);
        satellites.transition(
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            true,
            Duration::ZERO,
        );
        assert_eq!(satellites.panels.len(), 2);
        satellites.transition(
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
        satellites.transition(
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
        satellites.transition(SatelliteId::Wifi, true, Duration::ZERO);
        satellites.transition(
            SatelliteId::Tray {
                bus: "test".into(),
                path: "/test".into(),
            },
            true,
            Duration::ZERO,
        );
        satellites.transition(SatelliteId::Calendar, true, Duration::ZERO);
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
