mod catalog;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::new_capsule::widgets::dashboard::{self as widgets, DashboardAction, DashboardView};
use catalog::{Catalog, WallpaperEntry};
use chrono::Local;
use gpui::{
    Context, EventEmitter, FocusHandle, KeyDownEvent, Pixels, Render, Size, Subscription, Task,
    Window, div, prelude::*, px,
};
use services::{
    AppState, BatteryStatus, MediaTrack, NetworkStatus, NotificationItem, NotificationStore,
    SniItem, SystemStatus,
};
use std::{path::PathBuf, time::Duration};
use ui::{
    theme::{
        Theme,
        theme_manager::{ThemeItem, ThemeManager},
    },
    tracker::DimensionTracker,
};

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
    pub view: DashboardView,
    pub snapshot: Snapshot,
    pub player_bus: Option<String>,
    pub selected_ssid: Option<String>,
    pub password: String,
    pub error: Option<String>,
    pub pending: bool,
    pub themes: Vec<ThemeItem>,
    pub wallpapers: Vec<WallpaperEntry>,
    pub catalog_ready: bool,
    pub volume_bounds: DimensionTracker,
    pub seek_bounds: DimensionTracker,
    dragging_volume: bool,
    dragging_seek: bool,
    focus: FocusHandle,
    _refresh: Task<()>,
    _catalog: Task<()>,
    _theme: Subscription,
    catalog_worker: tokio::task::JoinHandle<()>,
    action_task: Option<Task<()>>,
}

impl DashboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let snapshot = Snapshot::read(cx);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let catalog_worker = services::spawn_tokio(async move {
            loop {
                let Ok(catalog) = tokio::task::spawn_blocking(Catalog::load).await else {
                    break;
                };
                if sender.send(catalog).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        });
        let catalog_task = cx.spawn(async move |this, cx| {
            while let Some(catalog) = receiver.recv().await {
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let changed = !module.catalog_ready
                            || module.wallpapers != catalog.wallpapers
                            || module.themes.len() != catalog.themes.len()
                            || module
                                .themes
                                .iter()
                                .zip(&catalog.themes)
                                .any(|(a, b)| a.path != b.path || a.theme != b.theme);
                        module.catalog_ready = true;
                        module.themes = catalog.themes;
                        module.wallpapers = catalog.wallpapers;
                        if changed {
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let refresh = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let next = Snapshot::read(cx);
                        if next != module.snapshot {
                            module.player_bus =
                                retain_player(module.player_bus.as_deref(), &next.players);
                            if module
                                .selected_ssid
                                .as_ref()
                                .is_some_and(|ssid| *ssid == next.network.wifi_ssid)
                            {
                                module.selected_ssid = None;
                                module.password.clear();
                            }
                            module.snapshot = next;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            player_bus: retain_player(None, &snapshot.players),
            snapshot,
            view: DashboardView::Home,
            selected_ssid: None,
            password: String::new(),
            error: None,
            pending: false,
            themes: Vec::new(),
            wallpapers: Vec::new(),
            catalog_ready: false,
            volume_bounds: DimensionTracker::new(),
            seek_bounds: DimensionTracker::new(),
            dragging_volume: false,
            dragging_seek: false,
            focus: cx.focus_handle(),
            _refresh: refresh,
            _catalog: catalog_task,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
            catalog_worker,
            action_task: None,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.view = DashboardView::Home;
        self.selected_ssid = None;
        self.password.clear();
        self.error = None;
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
    pub fn dispatch(&mut self, action: DashboardAction, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>().clone();
        match action {
            DashboardAction::View(view) => {
                self.selected_ssid = None;
                self.password.clear();
                self.error = None;
                match &view {
                    DashboardView::Wifi => state.network.rescan_wifi(),
                    DashboardView::Bluetooth => state.network.start_bluetooth_scan(),
                    _ => {}
                }
                self.view = view;
            }
            DashboardAction::Back => {
                self.view = DashboardView::Home;
                self.selected_ssid = None;
                self.password.clear();
                self.error = None;
            }
            DashboardAction::Close => cx.emit(CapsuleModuleEvent::Close),
            DashboardAction::Wifi => state.network.toggle_wifi(),
            DashboardAction::Bluetooth => state.network.toggle_bluetooth(),
            DashboardAction::Dnd => {
                NotificationStore::global().toggle_dnd();
            }
            DashboardAction::ScanWifi => state.network.rescan_wifi(),
            DashboardAction::ScanBluetooth => state.network.start_bluetooth_scan(),
            DashboardAction::SelectWifi(ssid) => {
                if let Some(ap) = self
                    .snapshot
                    .network
                    .wifi_ap_list
                    .iter()
                    .find(|ap| ap.ssid == ssid)
                {
                    if ap.is_connected {
                        state.network.disconnect_wifi(&ssid);
                    } else if ap.security.is_empty() || ap.security == "--" || ap.is_saved {
                        state.network.connect_wifi(&ssid, None);
                    } else {
                        self.selected_ssid = Some(ssid);
                        self.password.clear();
                    }
                }
            }
            DashboardAction::ConnectWifi => {
                if let Some(ssid) = &self.selected_ssid {
                    state.network.connect_wifi(ssid, Some(&self.password));
                    self.password.clear();
                }
            }
            DashboardAction::BluetoothDevice(mac) => {
                if self
                    .snapshot
                    .network
                    .bluetooth_device_list
                    .iter()
                    .any(|d| d.mac == mac && d.is_connected)
                {
                    state.network.disconnect_bluetooth(&mac);
                } else {
                    state.network.connect_bluetooth(&mac);
                }
            }
            DashboardAction::Month(offset) => match offset {
                -1 => state.calendar.prev_month(),
                1 => state.calendar.next_month(),
                _ => state.calendar.reset_to_today(),
            },
            DashboardAction::Settings => {
                cx.emit(CapsuleModuleEvent::Close);
                crate::panel::SettingsPanel::open(cx);
            }
            DashboardAction::Lock => {
                cx.emit(CapsuleModuleEvent::Close);
                crate::panel::LockScreenPanel::open_all(cx);
            }
            DashboardAction::ClearNotifications => {
                NotificationStore::global().clear_all_notifications()
            }
            DashboardAction::RemoveNotification(id) => {
                NotificationStore::global().remove_notification(id)
            }
            DashboardAction::Notification(id, key) => {
                NotificationStore::global().invoke_action(id, key)
            }
            DashboardAction::Player(bus) => self.player_bus = Some(bus),
            DashboardAction::TrayActivate(bus) => {
                if let Some(index) = state
                    .sni_host
                    .get_items()
                    .iter()
                    .position(|item| item.bus_name == bus)
                {
                    state.sni_host.activate_item(index);
                }
            }
            DashboardAction::TrayMenu(bus, path, id) => state.sni_host.trigger_menu(bus, path, id),
            action => self.run_action(action, state, cx),
        }
        self.snapshot = Snapshot::read(cx);
        cx.notify();
    }

    fn run_action(&mut self, action: DashboardAction, state: AppState, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        if let DashboardAction::Media(direction) = &action
            && self.player().is_none_or(|player| {
                (*direction == -1 && !player.can_go_previous)
                    || (*direction == 1 && !player.can_go_next)
            })
        {
            return;
        }
        let bus = self.player_bus.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.pending = true;
        self.error = None;
        services::spawn_tokio(async move {
            let result: Result<Option<Theme>, String> = match action {
                DashboardAction::Mute => state
                    .system
                    .toggle_mute()
                    .await
                    .map(|_| None)
                    .map_err(|e| e.to_string()),
                DashboardAction::Sink(name) => state
                    .system
                    .set_default_sink(&name)
                    .await
                    .map(|_| None)
                    .map_err(|e| e.to_string()),
                DashboardAction::Media(command) => {
                    let ok = if let Some(bus) = bus {
                        match command {
                            -1 => services::MprisService::previous_bus(&bus).await,
                            1 => services::MprisService::next_bus(&bus).await,
                            _ => services::MprisService::play_pause_bus(&bus).await,
                        }
                    } else {
                        false
                    };
                    if ok {
                        Ok(None)
                    } else {
                        Err("dashboard_new.media_error".into())
                    }
                }
                DashboardAction::Seek(seconds) => {
                    if let Some(bus) = bus {
                        if services::MprisService::seek_to(&bus, seconds).await {
                            Ok(None)
                        } else {
                            Err("dashboard_new.media_error".into())
                        }
                    } else {
                        Ok(None)
                    }
                }
                DashboardAction::Theme(theme) => tokio::task::spawn_blocking(move || {
                    ThemeManager::save_current_theme(&theme).map(|_| Some(theme))
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|result| result),
                DashboardAction::Wallpaper(path) => tokio::task::spawn_blocking(move || {
                    if state.wallpaper.set_wallpaper(path) {
                        Ok(None)
                    } else {
                        Err("dashboard_new.wallpaper_error".into())
                    }
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|result| result),
                _ => Ok(None),
            };
            let _ = sender.send(result);
        });
        self.action_task = Some(cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |module, cx| {
                module.pending = false;
                match result {
                    Ok(Ok(Some(theme))) => {
                        if cx.has_global::<ThemeManager>() {
                            let manager = cx.global_mut::<ThemeManager>();
                            manager.current_theme = theme.clone();
                            manager.apply_theme_to_apps();
                        }
                        cx.set_global(theme);
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        module.error = Some(if error.starts_with("dashboard_new.") {
                            module.text(&error, cx)
                        } else {
                            error
                        })
                    }
                    Err(_) => module.error = Some(module.text("dashboard_new.action_error", cx)),
                }
                module.snapshot = Snapshot::read(cx);
                cx.notify();
            });
        }));
    }

    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" => self.dispatch(
                if self.view == DashboardView::Home {
                    DashboardAction::Close
                } else {
                    DashboardAction::Back
                },
                cx,
            ),
            "enter" if self.selected_ssid.is_some() => {
                self.dispatch(DashboardAction::ConnectWifi, cx)
            }
            "backspace" if self.selected_ssid.is_some() => {
                self.password.pop();
                cx.notify();
            }
            "v" if self.selected_ssid.is_some() && event.keystroke.modifiers.control => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.password
                        .extend(text.chars().filter(|c| !c.is_control()));
                    cx.notify();
                }
            }
            _ if self.selected_ssid.is_some()
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt =>
            {
                if let Some(text) = &event.keystroke.key_char {
                    self.password
                        .extend(text.chars().filter(|c| !c.is_control()));
                    cx.notify();
                }
            }
            _ => {}
        }
    }
    pub fn volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        if let Some(fraction) =
            slider_fraction(x, self.volume_bounds.left(), self.volume_bounds.width(0.0))
        {
            cx.global::<AppState>()
                .system
                .set_volume_fast((fraction * 100.0).round() as u32);
            self.snapshot.audio = cx.global::<AppState>().system.get_status();
            cx.notify();
        }
    }
    pub fn start_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_volume = true;
        self.volume_at(x, cx);
    }
    pub fn start_seek(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_seek = true;
        let _ = (x, cx);
    }
    fn seek_at(&mut self, x: f32, cx: &mut Context<Self>) {
        if let Some(fraction) =
            slider_fraction(x, self.seek_bounds.left(), self.seek_bounds.width(0.0))
            && let Some(length) = self.player().and_then(|p| p.length_micros)
        {
            self.dispatch(
                DashboardAction::Seek(f64::from(fraction) * length as f64 / 1_000_000.0),
                cx,
            );
        }
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
impl Drop for DashboardModule {
    fn drop(&mut self) {
        self.catalog_worker.abort();
    }
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
        div()
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
            .flex()
            .flex_col()
            .w(self.size().width)
            .h(self.size().height)
            .p(px(widgets::PADDING))
            .gap(px(widgets::GAP))
            .overflow_hidden()
            .text_size(px(14.0))
            .text_color(theme.foreground())
            .child(widgets::header::render(self, &theme, cx))
            .child(if self.view == DashboardView::Home {
                widgets::home::render(self, &theme, cx)
            } else {
                widgets::details::render(self, &theme, cx)
            })
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
