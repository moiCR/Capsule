mod actions;
mod input;
mod navigation;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::new_capsule::widgets::dashboard::{self as widgets, DashboardView};
use chrono::Local;
use gpui::{
    Context, EventEmitter, FocusHandle, Pixels, Render, Size, Subscription, Task, Window, div,
    prelude::*, px,
};
use navigation::Navigation;
use services::{
    AppState, BatteryStatus, MediaTrack, NetworkStatus, NotificationItem, NotificationStore,
    SniItem, SystemStatus,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use ui::{theme::Theme, tracker::DimensionTracker};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Snapshot {
    pub network: NetworkStatus,
    pub audio: SystemStatus,
    pub battery: Option<BatteryStatus>,
    pub players: Vec<MediaTrack>,
    pub notifications: Vec<NotificationItem>,
    pub tray: Vec<SniItem>,
    pub dnd: bool,
    pub date: String,
    pub month: (i32, u32),
    pub duration: Duration,
    pub current_wallpaper: Option<PathBuf>,
}

impl Snapshot {
    fn read(cx: &gpui::App) -> Self {
        let state = cx.global::<AppState>();
        Self {
            network: state.network.get_status(),
            audio: state.system.get_status(),
            battery: state.power.get_battery(),
            players: (*state.mpris.get_all_players()).clone(),
            notifications: NotificationStore::global().get_all_notifications(),
            tray: state.sni_host.get_items(),
            dnd: NotificationStore::global().is_dnd_enabled(),
            date: format_date(state),
            month: state.calendar.get_view_date(),
            duration: Duration::from_millis(state.config.get().ui.animation_duration_ms as u64),
            current_wallpaper: state.wallpaper.get_current(),
        }
    }
}

pub(crate) struct DashboardModule {
    pub navigation: Navigation,
    pub satellites: Vec<crate::new_capsule::satellite::SatelliteVisual>,
    pub wave_phase: f32,
    pub satellite_heights: Vec<(crate::new_capsule::satellite::SatelliteId, Pixels)>,
    pub snapshot_at: Instant,
    active: bool,
    container_size: Size<Pixels>,
    _media_animation: Option<Task<()>>,
    pub snapshot: Snapshot,
    pub player_bus: Option<String>,
    pub pending: bool,
    pub volume_bounds: DimensionTracker,
    pub seek_bounds: DimensionTracker,
    dragging_volume: bool,
    dragging_seek: bool,
    focus: FocusHandle,
    _refresh: Option<Task<()>>,
    _theme: Subscription,
    action_task: Option<Task<()>>,
}

impl DashboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let snapshot = Snapshot::read(cx);
        Self {
            satellites: Vec::new(),
            satellite_heights: Vec::new(),
            wave_phase: 0.0,
            snapshot_at: Instant::now(),
            active: false,
            container_size: widgets::module_size(),
            _media_animation: None,
            player_bus: retain_player(None, &snapshot.players),
            snapshot,
            navigation: Navigation::default(),
            pending: false,
            volume_bounds: DimensionTracker::new(),
            seek_bounds: DimensionTracker::new(),
            dragging_volume: false,
            dragging_seek: false,
            focus: cx.focus_handle(),
            _refresh: None,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
            action_task: None,
        }
    }

    pub(crate) fn set_active(&mut self, active: bool, size: Size<Pixels>, cx: &mut Context<Self>) {
        if self.active != active {
            self.active = active;
            if active {
                self.refresh_snapshot(cx);
                self._refresh = Some(cx.spawn(async move |this, cx| {
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(250))
                            .await;
                        if this
                            .update(cx, |module: &mut Self, cx| module.refresh_snapshot(cx))
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            } else {
                self._refresh = None;
                self._media_animation = None;
            }
        }
        if self.container_size != size {
            self.container_size = size;
            cx.notify();
        }
    }

    fn refresh_snapshot(&mut self, cx: &mut Context<Self>) {
        let next = Snapshot::read(cx);
        if next != self.snapshot {
            self.player_bus = retain_player(self.player_bus.as_deref(), &next.players);
            if self
                .navigation
                .selected_ssid
                .as_ref()
                .is_some_and(|ssid| *ssid == next.network.wifi_ssid)
            {
                self.navigation.selected_ssid = None;
                self.navigation.password.clear();
            }
            self.snapshot = next;
            self.snapshot_at = Instant::now();
            cx.notify();
        }
        if !self.player().is_some_and(|player| player.is_playing) {
            self._media_animation = None;
        } else if self.active && self._media_animation.is_none() {
            let epoch = Instant::now();
            let compositor = cx.global::<AppState>().compositor.clone();
            self._media_animation = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(
                            compositor
                                .get_frame_duration()
                                .max(Duration::from_millis(16)),
                        )
                        .await;
                    let running = this.update(cx, |module: &mut Self, cx| {
                        if !module.active
                            || !module.player().is_some_and(|player| player.is_playing)
                        {
                            return false;
                        }
                        module.wave_phase = epoch.elapsed().as_secs_f32() * 3.0;
                        cx.notify();
                        true
                    });
                    if !matches!(running, Ok(true)) {
                        break;
                    }
                }
            }));
        }
    }

    pub(crate) fn set_satellites(
        &mut self,
        visuals: Vec<crate::new_capsule::satellite::SatelliteVisual>,
        cx: &mut Context<Self>,
    ) {
        if self.satellites == visuals {
            return;
        }
        if let Some(next) = visuals.iter().rev().find(|next| {
            !self
                .satellites
                .iter()
                .any(|previous| previous.id == next.id)
        }) {
            self.navigation.view = next.id.view();
        } else if let Some(current) = self.navigation.view.satellite_id()
            && !visuals
                .iter()
                .any(|visual| visual.id == current && visual.open)
        {
            self.navigation.view = visuals
                .iter()
                .rev()
                .find(|visual| visual.open)
                .map(|visual| visual.id.view())
                .unwrap_or(DashboardView::Home);
        }
        self.satellite_heights
            .retain(|(id, _)| visuals.iter().any(|visual| &visual.id == id));
        self.satellites = visuals;
        cx.notify();
    }

    pub(crate) fn measure_satellite(
        &mut self,
        id: &crate::new_capsule::satellite::SatelliteId,
        height: Pixels,
        cx: &mut Context<Self>,
    ) {
        if !self.satellites.iter().any(|visual| &visual.id == id) {
            return;
        }
        let height = px(f32::from(height).ceil());
        match self.satellite_heights.iter_mut().find(|(key, _)| key == id) {
            Some((_, previous)) if *previous == height => return,
            Some((_, previous)) => *previous = height,
            None => self.satellite_heights.push((id.clone(), height)),
        }
        cx.notify();
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.navigation.open(DashboardView::Home);
        self.dragging_volume = false;
        self.dragging_seek = false;
        cx.notify();
    }
    pub fn text(&self, key: &str, cx: &gpui::App) -> String {
        cx.global::<AppState>().language.get(key)
    }
    pub fn player(&self) -> Option<&MediaTrack> {
        self.player_bus
            .as_ref()
            .and_then(|bus| self.snapshot.players.iter().find(|p| p.bus_name == *bus))
    }
}

fn format_date(state: &AppState) -> String {
    use chrono::Datelike;
    let now = Local::now();
    let months = state.language.get_list("datetime.months");
    let month = months
        .get(now.month0() as usize)
        .cloned()
        .unwrap_or_else(|| now.month().to_string());
    format!("{} {} · {}", now.day(), month, now.format("%H:%M"))
}

pub(crate) fn retain_player(current: Option<&str>, players: &[MediaTrack]) -> Option<String> {
    players
        .iter()
        .find(|p| Some(p.bus_name.as_str()) == current)
        .or_else(|| players.iter().find(|p| p.is_playing))
        .or_else(|| players.first())
        .map(|p| p.bus_name.clone())
}
pub(crate) fn slider_fraction(x: f32, left: f32, width: f32) -> Option<f32> {
    (width > 0.0).then(|| ((x - left) / width).clamp(0.0, 1.0))
}
impl EventEmitter<CapsuleModuleEvent> for DashboardModule {}
impl CapsuleModule for DashboardModule {
    fn size(&self) -> Size<Pixels> {
        widgets::module_size()
    }
}
impl Render for DashboardModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let dashboard = div()
            .id("dashboard")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.dragging_volume {
                    this.volume_at(event.position.x.into(), cx);
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.dragging_volume = false;
                    if this.dragging_seek {
                        this.dragging_seek = false;
                        this.seek_at(event.position.x.into(), cx);
                    }
                }),
            )
            .on_mouse_up_out(
                gpui::MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.dragging_volume = false;
                    if this.dragging_seek {
                        this.dragging_seek = false;
                        this.seek_at(event.position.x.into(), cx);
                    }
                }),
            )
            .flex()
            .flex_col()
            .w(self.size().width)
            .h(self.size().height)
            .p(px(widgets::PADDING))
            .gap(px(widgets::GAP))
            .rounded(px(cx.global::<AppState>().config.get().ui.capsule_round))
            .overflow_hidden()
            .bg(theme.background())
            .text_size(px(14.0))
            .text_color(theme.foreground())
            .child(widgets::header::render(self, &theme, cx))
            .child(
                if self.navigation.view == DashboardView::Home
                    || self.navigation.view.satellite_id().is_some()
                {
                    widgets::home::render(self, &theme, cx)
                } else {
                    widgets::details::render(self, &theme, cx)
                },
            );
        let mut root = div()
            .relative()
            .w(self.container_size.width)
            .h(self.container_size.height);
        for visual in &self.satellites {
            let radius = cx.global::<AppState>().config.get().ui.satellite_round;
            root = root.child(div().id(format!("satellite-{:?}", visual.id)).child(
                widgets::details::render_satellite(self, visual, radius, &theme, cx),
            ));
        }
        root.child(
            div()
                .absolute()
                .inset_0()
                .rounded(px(cx.global::<AppState>().config.get().ui.capsule_round))
                .overflow_hidden()
                .child(dashboard),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_survives_reordering_and_falls_back_when_player_disappears() {
        let a = MediaTrack {
            bus_name: "a".into(),
            ..Default::default()
        };
        let b = MediaTrack {
            bus_name: "b".into(),
            is_playing: true,
            ..Default::default()
        };
        assert_eq!(retain_player(Some("a"), &[b.clone(), a]), Some("a".into()));
        assert_eq!(retain_player(Some("a"), &[b]), Some("b".into()));
        assert_eq!(retain_player(Some("a"), &[]), None);
    }
    #[test]
    fn slider_uses_actual_bounds_and_clamps_pointer() {
        assert_eq!(slider_fraction(40.0, 50.0, 100.0), Some(0.0));
        assert_eq!(slider_fraction(100.0, 50.0, 100.0), Some(0.5));
        assert_eq!(slider_fraction(200.0, 50.0, 100.0), Some(1.0));
        assert_eq!(slider_fraction(100.0, 50.0, 0.0), None);
    }
}
