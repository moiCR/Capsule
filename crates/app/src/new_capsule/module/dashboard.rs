mod actions;
mod catalog;
mod input;
mod navigation;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::new_capsule::widgets::dashboard::{self as widgets, DashboardView};
use catalog::{Catalog, WallpaperEntry};
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
use std::{path::PathBuf, time::Duration};
use ui::{
    theme::{Theme, theme_manager::ThemeItem},
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
    pub navigation: Navigation,
    pub snapshot: Snapshot,
    pub player_bus: Option<String>,
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
                                .navigation
                                .selected_ssid
                                .as_ref()
                                .is_some_and(|ssid| *ssid == next.network.wifi_ssid)
                            {
                                module.navigation.selected_ssid = None;
                                module.navigation.password.clear();
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
            navigation: Navigation::default(),
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
            .overflow_hidden()
            .text_size(px(14.0))
            .text_color(theme.foreground())
            .child(widgets::header::render(self, &theme, cx))
            .child(if self.navigation.view == DashboardView::Home {
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
