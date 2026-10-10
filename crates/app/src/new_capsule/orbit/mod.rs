mod animator;
mod state;
mod widgets;
mod worker;

use gpui::{Bounds, Context, EventEmitter, Pixels, Size, Task};
use services::{AppState, RecordStatus};
use state::State;
use std::time::{Duration, Instant};

use super::{
    Capsule,
    module::{CapsuleModuleEvent, CapsuleModuleId},
};

pub(crate) const ORB_SIZE: f32 = 26.0;
const BADGE_OVERHANG: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrbitId {
    Shelf,
    Recording,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Snapshot {
    shelf_count: usize,
    record_status: RecordStatus,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            shelf_count: 0,
            record_status: RecordStatus::Stopped,
        }
    }
}

pub(crate) struct OrbitVisual {
    pub id: OrbitId,
    pub bounds: Bounds<Pixels>,
    pub opacity: f32,
    pub interactive: bool,
}

pub(crate) struct OrbitChanged;

pub(crate) struct Presentation {
    state: Snapshot,
    visuals: Vec<OrbitVisual>,
    capsule: Bounds<Pixels>,
}

impl Presentation {
    pub fn render(
        self,
        theme: &ui::theme::Theme,
        cx: &mut Context<Capsule>,
    ) -> impl gpui::IntoElement + use<> {
        widgets::render(self, theme, cx)
    }
}

pub(crate) struct Orbit {
    state: State,
    worker: tokio::task::JoinHandle<()>,
    _updates: Task<()>,
}

impl EventEmitter<OrbitChanged> for Orbit {}

impl Orbit {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let (mut receiver, worker) = worker::start(state.shelf.clone(), state.record.clone());
        let updates = cx.spawn(async move |this, cx| {
            loop {
                let snapshot = *receiver.borrow_and_update();
                if this
                    .update(cx, |orbit: &mut Self, cx| {
                        let duration = Duration::from_millis(
                            cx.global::<AppState>()
                                .config
                                .get()
                                .ui
                                .animation_duration_ms as u64,
                        );
                        if orbit.state.update(snapshot, duration, Instant::now()) {
                            cx.emit(OrbitChanged);
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
                if receiver.changed().await.is_err() {
                    break;
                }
            }
        });
        Self {
            state: State::new(),
            worker,
            _updates: updates,
        }
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        self.state.advance(now)
    }

    pub fn animating(&self) -> bool {
        self.state.animating()
    }

    pub fn satellite_gap(gap: Pixels) -> Pixels {
        State::satellite_gap(gap)
    }

    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
    ) -> impl Iterator<Item = OrbitVisual> {
        self.state.layout(viewport, capsule, gap)
    }

    pub fn presentation(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
    ) -> Presentation {
        Presentation {
            state: self.state.snapshot,
            visuals: self.layout(viewport, capsule, gap).collect(),
            capsule,
        }
    }
}

impl Drop for Orbit {
    fn drop(&mut self) {
        self.worker.abort();
    }
}

impl Capsule {
    pub(super) fn orbit_changed(&mut self, cx: &mut Context<Self>) {
        let running = self
            .orbit
            .update(cx, |orbit, _| orbit.advance(Instant::now()));
        self.sync_window(cx);
        self.start_animation(running, cx);
        cx.notify();
    }

    fn open_orbit(&mut self, id: OrbitId, cx: &mut Context<Self>) {
        let state = self.orbit.read(cx).state.snapshot;
        let target = match id {
            OrbitId::Shelf if state.shelf_count > 0 => CapsuleModuleId::Shelf,
            OrbitId::Recording if state.record_status != RecordStatus::Stopped => {
                CapsuleModuleId::Record
            }
            _ => return,
        };
        let entity = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = entity.update(cx, |capsule, cx| {
                capsule.handle_module_event(&CapsuleModuleEvent::Open(target), cx);
            });
        });
    }
}
