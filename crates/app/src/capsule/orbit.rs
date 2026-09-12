use gpui::{Context, Task, Window};
use services::{AppState, RecordStatus};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, watch};

use super::satellites::{satellite_retract, satellite_spring};

pub const ORB_SIZE: f32 = 26.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbKind {
    Shelf,
    Recording,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

struct Motion {
    from: f32,
    target: f32,
    started: Instant,
    duration: f32,
}

impl Motion {
    fn new(value: f32, now: Instant) -> Self {
        Self {
            from: value,
            target: value,
            started: now,
            duration: 0.38,
        }
    }

    fn value(&self, now: Instant) -> f32 {
        let progress = (now.duration_since(self.started).as_secs_f32() / self.duration).min(1.0);
        let factor = if self.target < self.from {
            satellite_retract(progress)
        } else {
            satellite_spring(progress)
        };
        self.from + (self.target - self.from) * factor
    }

    fn retarget(&mut self, target: f32, now: Instant) {
        if self.target != target {
            self.from = self.value(now);
            self.target = target;
            self.started = now;
            self.duration = if target < self.from { 0.22 } else { 0.38 };
        }
    }

    fn animating(&self, now: Instant) -> bool {
        self.from != self.target && now.duration_since(self.started).as_secs_f32() < self.duration
    }
}

struct Slot<Kind> {
    kind: Kind,
    side: Side,
    active: bool,
    visibility: Motion,
    rank: Motion,
}

struct Layout<Kind> {
    slots: Vec<Slot<Kind>>,
    visible: bool,
}

impl<Kind: Copy + Eq> Layout<Kind> {
    fn new() -> Self {
        Self {
            slots: Vec::new(),
            visible: true,
        }
    }

    fn set_active(&mut self, kind: Kind, active: bool, now: Instant) {
        if self
            .slots
            .iter()
            .any(|slot| slot.kind == kind && slot.active)
            == active
        {
            return;
        }
        if active {
            let left = self
                .slots
                .iter()
                .filter(|slot| slot.active && slot.side == Side::Left)
                .count();
            let right = self
                .slots
                .iter()
                .filter(|slot| slot.active && slot.side == Side::Right)
                .count();
            let side = if left <= right {
                Side::Left
            } else {
                Side::Right
            };
            let rank = if side == Side::Left { left } else { right } as f32;
            let previous = self
                .slots
                .iter()
                .position(|slot| slot.kind == kind)
                .map(|index| self.slots.remove(index));
            let mut slot = previous.unwrap_or_else(|| Slot {
                kind,
                side,
                active: true,
                visibility: Motion::new(0.0, now),
                rank: Motion::new(rank, now),
            });
            if slot.side != side {
                slot.visibility = Motion::new(0.0, now);
            }
            slot.side = side;
            slot.active = true;
            slot.rank.retarget(rank, now);
            slot.visibility
                .retarget(if self.visible { 1.0 } else { 0.0 }, now);
            self.slots.push(slot);
        } else if let Some(slot) = self.slots.iter_mut().find(|slot| slot.kind == kind) {
            slot.active = false;
            slot.visibility.retarget(0.0, now);
        }
        for side in [Side::Left, Side::Right] {
            for (rank, slot) in self
                .slots
                .iter_mut()
                .filter(|slot| slot.active && slot.side == side)
                .enumerate()
            {
                slot.rank.retarget(rank as f32, now);
            }
        }
    }

    fn set_visible(&mut self, visible: bool, now: Instant) {
        self.visible = visible;
        for slot in &mut self.slots {
            slot.visibility
                .retarget(if visible && slot.active { 1.0 } else { 0.0 }, now);
        }
    }

    fn tick(&mut self, now: Instant) {
        self.slots
            .retain(|slot| slot.active || slot.visibility.animating(now));
    }

    fn animating(&self, now: Instant) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.visibility.animating(now) || slot.rank.animating(now))
    }
}

pub struct OrbGeometry {
    pub kind: OrbKind,
    pub x: f32,
    pub y: f32,
    pub opacity: f32,
    pub interactive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OrbitState {
    shelf_count: usize,
    record_status: RecordStatus,
}

async fn publish_updates(
    sender: watch::Sender<OrbitState>,
    mut animation_updates: watch::Receiver<bool>,
    mut status_updates: broadcast::Receiver<RecordStatus>,
    read_state: impl Fn() -> OrbitState,
    frame_duration: impl Fn() -> Duration,
) {
    let mut animating = false;
    let mut poll = tokio::time::interval(Duration::from_millis(80));
    loop {
        let frame = tokio::select! {
            _ = sender.closed() => break,
            result = animation_updates.changed() => {
                if result.is_err() { break; }
                animating = *animation_updates.borrow_and_update();
                continue;
            }
            _ = tokio::time::sleep(frame_duration()), if animating => true,
            result = status_updates.recv() => {
                if matches!(result, Err(broadcast::error::RecvError::Closed)) { break; }
                false
            }
            _ = poll.tick() => false,
        };
        let current = read_state();
        if frame {
            sender.send_modify(|state| {
                *state = current;
            });
        } else {
            sender.send_if_modified(|state| {
                let changed = *state != current;
                *state = current;
                changed
            });
        }
    }
}

pub struct Orbit {
    layout: Layout<OrbKind>,
    pub shelf_count: usize,
    pub record_status: RecordStatus,
    state: watch::Receiver<OrbitState>,
    animate: watch::Sender<bool>,
    worker: tokio::task::JoinHandle<()>,
    _updates: Task<()>,
}

impl Orbit {
    pub fn new(window: &Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let shelf = state.shelf.clone();
        let record = state.record.clone();
        let compositor = state.compositor.clone();
        let status_updates = record.subscribe_status();
        let initial = OrbitState {
            shelf_count: shelf.count(),
            record_status: record.get_status(),
        };
        let (sender, state) = watch::channel(initial);
        let (animate, animation_updates) = watch::channel(false);
        let worker = tokio::spawn(publish_updates(
            sender,
            animation_updates,
            status_updates,
            move || OrbitState {
                shelf_count: shelf.count(),
                record_status: record.get_status(),
            },
            move || compositor.get_frame_duration(),
        ));
        let updates = Self::receive_updates(window, state.clone(), cx);
        Self {
            layout: Layout::new(),
            shelf_count: initial.shelf_count,
            record_status: initial.record_status,
            state,
            animate,
            worker,
            _updates: updates,
        }
    }

    fn receive_updates(
        window: &Window,
        mut receiver: watch::Receiver<OrbitState>,
        cx: &mut Context<Self>,
    ) -> Task<()> {
        let entity = cx.entity().downgrade();
        let mut window_context = window.to_async(cx);
        cx.foreground_executor().spawn(async move {
            while receiver.changed().await.is_ok() {
                if entity.upgrade().is_none() {
                    break;
                }
                let _ = window_context.update(|window, cx| {
                    if entity.update(cx, |orbit, cx| orbit.sync(cx)).is_ok() {
                        window.refresh();
                    }
                });
            }
        })
    }

    pub fn sync(&mut self, cx: &mut Context<Self>) {
        let state = if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            OrbitState {
                shelf_count: app_state.shelf.count(),
                record_status: app_state.record.get_status(),
            }
        } else {
            *self.state.borrow_and_update()
        };
        let now = Instant::now();
        self.shelf_count = state.shelf_count;
        self.record_status = state.record_status;
        self.layout
            .set_active(OrbKind::Shelf, state.shelf_count > 0, now);
        self.layout.set_active(
            OrbKind::Recording,
            state.record_status != RecordStatus::Stopped,
            now,
        );
        self.layout.tick(now);
        let animating = self.layout.animating(now);
        self.animate.send_if_modified(|previous| {
            let changed = *previous != animating;
            *previous = animating;
            changed
        });
        cx.notify();
    }

    pub fn set_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.sync(cx);
        if self.layout.visible == visible {
            return;
        }
        let now = Instant::now();
        self.layout.set_visible(visible, now);
        self.animate.send_replace(self.layout.animating(now));
        cx.notify();
    }

    pub fn geometry(&self, width: f32, height: f32, gap: f32) -> Vec<OrbGeometry> {
        self.layout.geometry(width, height, gap, Instant::now())
    }
}

impl Drop for Orbit {
    fn drop(&mut self) {
        self.worker.abort();
    }
}

impl Layout<OrbKind> {
    fn geometry(&self, width: f32, height: f32, gap: f32, now: Instant) -> Vec<OrbGeometry> {
        self.slots
            .iter()
            .filter_map(|slot| {
                let factor = slot.visibility.value(now);
                if factor <= 0.0 {
                    return None;
                }
                let distance = (slot.rank.value(now) + 1.0) * (ORB_SIZE + gap.max(8.0));
                let x = match slot.side {
                    Side::Left => -distance * factor,
                    Side::Right => width - ORB_SIZE + distance * factor,
                };
                Some(OrbGeometry {
                    kind: slot.kind,
                    x,
                    y: ((height - ORB_SIZE) / 2.0).max(0.0),
                    opacity: factor.clamp(0.0, 1.0),
                    interactive: self.visible && slot.active && factor >= 0.85,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_window_removes_orbs_without_input_or_mode_changes() {
        use gpui::{AppContext, Entity, IntoElement, Render, WindowOptions, div};
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };

        if std::env::var_os("CAPSULE_ORBIT_HEADLESS_TEST").is_none() {
            let output = std::process::Command::new(
                std::env::current_exe().expect("test executable"),
            )
            .args([
                "--exact",
                "capsule::orbit::tests::idle_window_removes_orbs_without_input_or_mode_changes",
                "--nocapture",
            ])
            .env("CAPSULE_ORBIT_HEADLESS_TEST", "1")
            .env("DBUS_SYSTEM_BUS_ADDRESS", "disabled:")
            .output()
            .expect("headless test process");
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }

        struct IdleWindow {
            orbit: Entity<Orbit>,
            rendered: watch::Sender<usize>,
        }

        impl Render for IdleWindow {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let count = self.orbit.read(cx).geometry(138.0, 42.0, 8.0).len();
                self.rendered.send_replace(count);
                div()
            }
        }

        let runtime = tokio::runtime::Runtime::new().expect("Tokio runtime");
        let _runtime_guard = runtime.enter();
        let initial = OrbitState {
            shelf_count: 0,
            record_status: RecordStatus::Stopped,
        };
        let (source, snapshot) = watch::channel(initial);
        let (status, status_updates) = broadcast::channel(4);
        let (rendered, mut renders) = watch::channel(0);
        let (finished, completion) = tokio::sync::oneshot::channel();
        let succeeded = Arc::new(AtomicBool::new(false));
        let result = succeeded.clone();
        runtime.spawn(async move {
            let outcome = tokio::time::timeout(Duration::from_secs(3), async {
                source.send_replace(OrbitState {
                    shelf_count: 1,
                    record_status: RecordStatus::Recording,
                });
                let _ = status.send(RecordStatus::Recording);
                if renders.wait_for(|count| *count == 2).await.is_err() {
                    return false;
                }
                tokio::time::sleep(Duration::from_millis(450)).await;
                source.send_replace(initial);
                let _ = status.send(RecordStatus::Stopped);
                renders.wait_for(|count| *count == 0).await.is_ok()
            })
            .await
            .unwrap_or(false);
            result.store(outcome, Ordering::SeqCst);
            let _ = finished.send(());
        });

        let (frames, mut frame_updates) = watch::channel(());
        let frame_clock = runtime.spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(16)).await;
                if frames.is_closed() {
                    break;
                }
                frames.send_replace(());
            }
        });
        gpui_platform::headless().run(move |cx| {
            let window = cx
                .open_window(WindowOptions::default(), |window, cx| {
                    cx.new(|cx| {
                        let orbit = cx.new(|cx| {
                            let (sender, state) = watch::channel(initial);
                            let (animate, animation_updates) = watch::channel(false);
                            let worker = tokio::spawn(publish_updates(
                                sender,
                                animation_updates,
                                status_updates,
                                move || *snapshot.borrow(),
                                || Duration::from_millis(16),
                            ));
                            let updates = Orbit::receive_updates(window, state.clone(), cx);
                            Orbit {
                                layout: Layout::new(),
                                shelf_count: 0,
                                record_status: RecordStatus::Stopped,
                                state,
                                animate,
                                worker,
                                _updates: updates,
                            }
                        });
                        cx.observe(&orbit, |_, _, cx| cx.notify()).detach();
                        IdleWindow { orbit, rendered }
                    })
                })
                .expect("headless window");
            let mut app = cx.to_async();
            cx.foreground_executor()
                .spawn(async move {
                    while frame_updates.changed().await.is_ok() {
                        if app
                            .update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
                            .is_err()
                        {
                            break;
                        }
                    }
                })
                .detach();
            let app = cx.to_async();
            cx.foreground_executor()
                .spawn(async move {
                    let _ = completion.await;
                    app.update(|cx| cx.quit());
                })
                .detach();
        });
        frame_clock.abort();
        assert!(
            succeeded.load(Ordering::SeqCst),
            "idle window did not remove both Orbs after the services stopped"
        );
    }

    #[tokio::test]
    async fn stopped_and_empty_state_replaces_frames_without_ui_consumption() {
        let active = OrbitState {
            shelf_count: 1,
            record_status: RecordStatus::Recording,
        };
        let inactive = OrbitState {
            shelf_count: 0,
            record_status: RecordStatus::Stopped,
        };
        let (source, snapshot) = watch::channel(active);
        let (sender, receiver) = watch::channel(active);
        let (animate, animation_updates) = watch::channel(false);
        let (status, status_updates) = broadcast::channel(4);
        let worker = tokio::spawn(publish_updates(
            sender,
            animation_updates,
            status_updates,
            move || *snapshot.borrow(),
            || Duration::from_millis(1),
        ));
        animate.send_replace(true);
        tokio::time::sleep(Duration::from_millis(100)).await;
        source.send_replace(inactive);
        assert!(status.send(RecordStatus::Stopped).is_ok());
        let updated = tokio::time::timeout(Duration::from_secs(1), async {
            while *receiver.borrow() != inactive {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        assert!(updated.is_ok());

        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(OrbKind::Shelf, true, now);
        layout.set_active(OrbKind::Recording, true, now);
        let latest = *receiver.borrow();
        let stopped = now + Duration::from_secs(1);
        layout.set_active(OrbKind::Shelf, latest.shelf_count > 0, stopped);
        layout.set_active(
            OrbKind::Recording,
            latest.record_status != RecordStatus::Stopped,
            stopped,
        );
        layout.tick(stopped + Duration::from_secs(1));
        assert!(
            layout
                .geometry(138.0, 42.0, 8.0, stopped + Duration::from_secs(1))
                .is_empty()
        );
        drop(receiver);
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(1), worker).await,
            Ok(Ok(()))
        ));
    }

    #[tokio::test]
    async fn shelf_changes_are_polled_without_recording_events_or_ui_consumption() {
        let initial = OrbitState {
            shelf_count: 1,
            record_status: RecordStatus::Stopped,
        };
        let (source, snapshot) = watch::channel(initial);
        let (sender, receiver) = watch::channel(initial);
        let (_animate, animation_updates) = watch::channel(false);
        let (_status, status_updates) = broadcast::channel(4);
        let worker = tokio::spawn(publish_updates(
            sender,
            animation_updates,
            status_updates,
            move || *snapshot.borrow(),
            || Duration::from_millis(16),
        ));
        source.send_modify(|state| state.shelf_count = 0);
        let updated = tokio::time::timeout(Duration::from_secs(1), async {
            while receiver.borrow().shelf_count != 0 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await;
        assert!(updated.is_ok());
        drop(receiver);
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(1), worker).await,
            Ok(Ok(()))
        ));
    }

    fn positions(layout: &Layout<u8>) -> Vec<(u8, Side, f32)> {
        layout
            .slots
            .iter()
            .filter(|slot| slot.active)
            .map(|slot| (slot.kind, slot.side, slot.rank.target))
            .collect()
    }

    #[test]
    fn alternates_and_closes_gaps_without_switching_sides() {
        let now = Instant::now();
        let mut layout = Layout::new();
        for kind in 0..4 {
            layout.set_active(kind, true, now);
        }
        assert_eq!(
            positions(&layout),
            vec![
                (0, Side::Left, 0.0),
                (1, Side::Right, 0.0),
                (2, Side::Left, 1.0),
                (3, Side::Right, 1.0),
            ]
        );
        let removal = now + Duration::from_secs(1);
        layout.set_active(0, false, removal);
        assert_eq!(
            positions(&layout),
            vec![
                (1, Side::Right, 0.0),
                (2, Side::Left, 0.0),
                (3, Side::Right, 1.0),
            ]
        );
        assert_eq!(layout.slots[2].rank.value(removal), 1.0);
        layout.tick(removal + Duration::from_secs(1));
        assert_eq!(layout.slots.len(), 3);
        layout.set_active(4, true, removal + Duration::from_secs(1));
        assert_eq!(positions(&layout).last(), Some(&(4, Side::Left, 1.0)));
    }

    #[test]
    fn hiding_preserves_assignments_and_tracks_changes() {
        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(0, true, now);
        layout.set_active(1, true, now);
        layout.set_visible(false, now + Duration::from_secs(1));
        layout.tick(now + Duration::from_secs(2));
        assert_eq!(
            positions(&layout),
            vec![(0, Side::Left, 0.0), (1, Side::Right, 0.0)]
        );
        layout.set_active(0, false, now + Duration::from_secs(2));
        layout.set_active(2, true, now + Duration::from_secs(2));
        layout.set_visible(true, now + Duration::from_secs(3));
        assert_eq!(
            positions(&layout),
            vec![(1, Side::Right, 0.0), (2, Side::Left, 0.0)]
        );
    }

    #[test]
    fn reactivation_uses_the_least_populated_side() {
        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(0, true, now);
        layout.set_active(1, true, now);
        layout.set_active(0, false, now + Duration::from_secs(1));
        layout.tick(now + Duration::from_secs(2));
        layout.set_active(2, true, now + Duration::from_secs(2));
        layout.set_active(3, true, now + Duration::from_secs(2));
        layout.set_active(0, true, now + Duration::from_secs(2));
        assert_eq!(positions(&layout).last(), Some(&(0, Side::Right, 1.0)));
    }

    #[test]
    fn rapid_visibility_reversals_are_continuous() {
        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(0, true, now);
        let closing = now + Duration::from_millis(90);
        let before = layout.slots[0].visibility.value(closing);
        layout.set_visible(false, closing);
        assert_eq!(layout.slots[0].visibility.value(closing), before);
        let reopening = closing + Duration::from_millis(60);
        let before = layout.slots[0].visibility.value(reopening);
        layout.set_visible(true, reopening);
        assert_eq!(layout.slots[0].visibility.value(reopening), before);
        layout.set_active(0, false, reopening + Duration::from_millis(10));
        let reactivation = reopening + Duration::from_millis(20);
        let before = layout.slots[0].visibility.value(reactivation);
        layout.set_active(0, true, reactivation);
        assert_eq!(layout.slots.len(), 1);
        assert_eq!(layout.slots[0].visibility.value(reactivation), before);
        assert!(!layout.animating(now + Duration::from_secs(2)));
    }

    #[test]
    fn geometry_follows_capsule_size_and_disables_retracting_buttons() {
        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(OrbKind::Shelf, true, now);
        layout.set_active(OrbKind::Recording, true, now);
        let settled = now + Duration::from_secs(1);
        let geometry = layout.geometry(138.0, 42.0, 2.0, settled);
        assert_eq!(geometry.len(), 2);
        assert_eq!((geometry[0].x, geometry[0].y), (-34.0, 8.0));
        assert_eq!((geometry[1].x, geometry[1].y), (146.0, 8.0));
        assert!(geometry.iter().all(|orb| orb.interactive));
        let expanded = layout.geometry(200.0, 50.0, 12.0, settled);
        assert_eq!((expanded[0].x, expanded[0].y), (-38.0, 12.0));
        assert_eq!((expanded[1].x, expanded[1].y), (212.0, 12.0));
        layout.set_visible(false, settled);
        assert!(
            layout
                .geometry(200.0, 50.0, 12.0, settled)
                .iter()
                .all(|orb| !orb.interactive)
        );
        assert!(
            layout
                .geometry(200.0, 50.0, 12.0, settled + Duration::from_secs(1))
                .is_empty()
        );
    }

    #[test]
    fn recording_pause_does_not_reassign_and_stop_releases_slot() {
        let now = Instant::now();
        let mut layout = Layout::new();
        layout.set_active(OrbKind::Recording, true, now);
        layout.set_active(OrbKind::Shelf, true, now);
        for status in [RecordStatus::Paused, RecordStatus::Recording] {
            layout.set_active(OrbKind::Recording, status != RecordStatus::Stopped, now);
            assert_eq!(layout.slots.len(), 2);
            assert_eq!(layout.slots[0].side, Side::Left);
            assert_eq!(layout.slots[0].kind, OrbKind::Recording);
        }
        layout.set_active(OrbKind::Recording, false, now + Duration::from_secs(1));
        layout.tick(now + Duration::from_secs(2));
        assert_eq!(layout.slots.len(), 1);
        assert_eq!(layout.slots[0].kind, OrbKind::Shelf);
        assert_eq!(layout.slots[0].side, Side::Right);
    }
}
