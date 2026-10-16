mod animator;
mod state;
mod widgets;
mod worker;

use gpui::{Bounds, Context, EventEmitter, Pixels, Size, Task};
use services::{AppState, RecordStatus};
use state::State;
use std::time::{Duration, Instant};

use super::{
    Capsule, CapsuleLocation,
    module::{CapsuleModuleEvent, CapsuleModuleId},
};

pub(crate) const ORB_SIZE: f32 = 26.0;
pub(crate) const USAGE_PANEL_WIDTH: f32 = 220.0;
const USAGE_PANEL_HEIGHT: f32 = 144.0;
const POWER_PANEL_HEIGHT: f32 = 176.0;
const BADGE_OVERHANG: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrbitId {
    Shelf,
    Recording,
    Usage,
    Power,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Snapshot {
    shelf_count: usize,
    record_status: RecordStatus,
    usage: services::system::UsageStatus,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            shelf_count: 0,
            record_status: RecordStatus::Stopped,
            usage: services::system::UsageStatus::default(),
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

#[derive(Clone, Default)]
struct PowerStatus {
    pending: bool,
    error: Option<String>,
}

#[derive(Clone, Copy)]
enum PowerAction {
    Poweroff,
    Reboot,
    Logout,
    Lock,
}

pub(crate) struct Presentation {
    state: Snapshot,
    visuals: Vec<OrbitVisual>,
    capsule: Bounds<Pixels>,
    content_opacity: f32,
    power_opacity: f32,
    power: PowerStatus,
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
    usage_enabled: tokio::sync::watch::Sender<bool>,
    power: PowerStatus,
    _power_action: Option<Task<()>>,
    pub(super) focus_handle: gpui::FocusHandle,
}

impl EventEmitter<OrbitChanged> for Orbit {}

impl Orbit {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let (usage_enabled, enabled) = tokio::sync::watch::channel(false);
        let (mut receiver, worker) = worker::start(
            state.shelf.clone(),
            state.record.clone(),
            state.system.clone(),
            enabled,
        );
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
        let mut orbit_state = State::new();
        orbit_state.configure(
            cx.global::<AppState>().config.get().ui.orbits,
            Duration::ZERO,
            Instant::now(),
        );
        Self {
            state: orbit_state,
            worker,
            _updates: updates,
            usage_enabled,
            power: PowerStatus::default(),
            _power_action: None,
            focus_handle: cx.focus_handle(),
        }
    }

    fn toggle(&mut self, id: OrbitId, duration: Duration, now: Instant) {
        match id {
            OrbitId::Usage => self.state.toggle_usage(duration, now),
            OrbitId::Power => self.state.toggle_power(duration, now),
            _ => return,
        }
        self.usage_enabled.send_replace(self.state.expanded.open);
    }

    pub fn set_motion_options(
        &mut self,
        stagger: bool,
        squash_stretch: bool,
        reduced_motion: bool,
    ) {
        self.state
            .set_motion_options(stagger, squash_stretch, reduced_motion);
    }

    pub fn configure(
        &mut self,
        orbits: services::config::ui_config::OrbitConfig,
        animation: services::CapsuleAnimation,
        duration: Duration,
    ) {
        self.state.set_animation(animation);
        self.state.configure(orbits, duration, Instant::now());
        self.usage_enabled.send_replace(self.state.expanded.open);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        self.state.advance(now)
    }

    pub fn animating(&self) -> bool {
        self.state.animating()
    }

    pub fn expanded(&self) -> bool {
        self.state.expanded.open || self.state.power_expanded.open
    }

    pub fn layout_gap(&self, gap: Pixels) -> Pixels {
        State::satellite_gap(gap)
            + self
                .state
                .expanded
                .extra_width()
                .max(self.state.power_expanded.extra_width())
    }

    pub fn satellite_gap(gap: Pixels) -> Pixels {
        State::satellite_gap(gap)
    }

    pub fn layout(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
        location: CapsuleLocation,
    ) -> impl Iterator<Item = OrbitVisual> {
        self.state.layout(viewport, capsule, gap, location)
    }

    pub fn presentation(
        &self,
        viewport: Size<Pixels>,
        capsule: Bounds<Pixels>,
        gap: Pixels,
        location: CapsuleLocation,
    ) -> Presentation {
        Presentation {
            state: self.state.snapshot,
            visuals: self.layout(viewport, capsule, gap, location).collect(),
            capsule,
            content_opacity: self.state.expanded.content_opacity(),
            power_opacity: self.state.power_expanded.content_opacity(),
            power: self.power.clone(),
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
        let entity = cx.entity().downgrade();
        cx.defer(move |cx| {
            let _ = entity.update(cx, |capsule, cx| {
                let running = capsule
                    .orbit
                    .update(cx, |orbit, _| orbit.advance(Instant::now()));
                capsule.sync_window(cx);
                capsule.start_animation(running, cx);
                cx.notify();
            });
        });
    }

    pub(super) fn close_expanded_orb(&mut self, cx: &mut Context<Self>) {
        if self.orbit.read(cx).expanded() {
            let duration = Duration::from_millis(
                cx.global::<AppState>()
                    .config
                    .get()
                    .ui
                    .animation_duration_ms as u64,
            );
            self.orbit.update(cx, |orbit, _| {
                orbit.state.close(duration, Instant::now());
                orbit.usage_enabled.send_replace(false);
            });
            self.orbit_changed(cx);
        }
    }

    fn execute_power(&mut self, action: PowerAction, cx: &mut Context<Self>) {
        if self.orbit.read(cx).power.pending {
            return;
        }
        if matches!(action, PowerAction::Lock) {
            self.close_expanded_orb(cx);
            crate::panel::LockScreenPanel::open_all(cx);
            return;
        }
        self.orbit.update(cx, |orbit, _| {
            orbit.power.pending = true;
            orbit.power.error = None;
        });
        let compositor = cx.global::<AppState>().compositor.clone();
        let task = cx.spawn(async move |this, cx| {
            let result = services::spawn_tokio(async move {
                match action {
                    PowerAction::Poweroff => services::SystemService::poweroff().await,
                    PowerAction::Reboot => services::SystemService::reboot().await,
                    PowerAction::Logout => compositor.logout().await,
                    PowerAction::Lock => Ok(()),
                }
            })
            .await;
            let error = match result {
                Ok(result) => result.err().map(|error| error.to_string()),
                Err(error) => Some(error.to_string()),
            };
            let _ = this.update(cx, |capsule, cx| {
                let succeeded = error.is_none();
                capsule.orbit.update(cx, |orbit, _| {
                    orbit.power.pending = false;
                    orbit.power.error = error;
                });
                if succeeded {
                    capsule.close_expanded_orb(cx);
                }
                cx.notify();
            });
        });
        self.orbit
            .update(cx, |orbit, _| orbit._power_action = Some(task));
        cx.notify();
    }

    fn open_orbit(&mut self, id: OrbitId, cx: &mut Context<Self>) {
        if matches!(id, OrbitId::Usage | OrbitId::Power) {
            let duration = Duration::from_millis(
                cx.global::<AppState>()
                    .config
                    .get()
                    .ui
                    .animation_duration_ms as u64,
            );
            self.orbit
                .update(cx, |orbit, _| orbit.toggle(id, duration, Instant::now()));
            self.orbit_changed(cx);
            return;
        }
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

pub(crate) struct Preview {
    state: State,
}

impl Preview {
    pub fn new(orbits: services::config::ui_config::OrbitConfig) -> Self {
        let mut state = State::new();
        state.configure(orbits, Duration::ZERO, Instant::now());
        state.update(
            Snapshot {
                shelf_count: 3,
                record_status: RecordStatus::Recording,
                ..Snapshot::default()
            },
            Duration::ZERO,
            Instant::now(),
        );
        Self { state }
    }

    pub fn set_motion_options(
        &mut self,
        stagger: bool,
        squash_stretch: bool,
        reduced_motion: bool,
    ) {
        self.state
            .set_motion_options(stagger, squash_stretch, reduced_motion);
    }

    pub fn transition(
        &mut self,
        orbits: services::config::ui_config::OrbitConfig,
        animation: services::CapsuleAnimation,
        duration: Duration,
        open: bool,
    ) {
        let now = Instant::now();
        self.state.set_animation(animation);
        self.state.configure(orbits, duration, now);
        self.state.preview_transition(open, duration, now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        self.state.advance(now)
    }

    pub fn render(
        &self,
        size: Size<Pixels>,
        gap: Pixels,
        bottom: bool,
        theme: &ui::theme::Theme,
    ) -> impl gpui::IntoElement + use<> {
        widgets::preview(&self.state, size, gap, bottom, theme)
    }
}
