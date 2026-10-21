use std::{path::PathBuf, time::Duration};

use chrono::{Datelike, Local};
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, Pixels, Render, Size, Subscription, Task,
    Window, prelude::*,
};
use services::{AppState, MediaTrack, NotificationStore};
use ui::theme::Theme;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::capsule::widgets::{
    default::{StatusSnapshot, WorkspaceKey, workspace_keys},
    default_expanded as widgets,
};

#[derive(PartialEq)]
pub(crate) struct Snapshot {
    pub time: String,
    pub date: String,
    pub application: String,
    pub icon: Option<PathBuf>,
    pub keys: [WorkspaceKey; 6],
    pub special: bool,
    pub status: StatusSnapshot,
    pub track: Option<MediaTrack>,
    pub notifications: usize,
    pub dnd: bool,
}

impl Snapshot {
    fn read(state: &AppState, application: Option<&str>) -> Self {
        let workspace = state.compositor.get_workspace();
        let apps = state.launcher.get_apps();
        let app = application.and_then(|name| {
            apps.iter().find(|app| {
                app.id
                    .trim_end_matches(".desktop")
                    .eq_ignore_ascii_case(name)
                    || app.name.eq_ignore_ascii_case(name)
                    || app
                        .icon_name
                        .as_deref()
                        .is_some_and(|icon| icon.eq_ignore_ascii_case(name))
            })
        });
        let players = state.mpris.get_all_players();
        let track = players
            .iter()
            .find(|track| track.has_media && track.is_playing)
            .or_else(|| players.iter().find(|track| track.has_media))
            .cloned();
        let now = Local::now();
        let days = state.language.get_list("datetime.days");
        let months = state.language.get_list("datetime.months");
        let short = |value: Option<&String>| {
            value
                .map(|text| text.chars().take(3).collect::<String>())
                .unwrap_or_default()
        };
        let date = format!(
            "{}, {} {}",
            short(days.get(now.weekday().num_days_from_monday() as usize)),
            short(months.get(now.month0() as usize)),
            now.day()
        );
        Self {
            time: now.format(state.config.get().ui.clock_format()).to_string(),
            date,
            application: app
                .map(|app| app.name.clone())
                .or_else(|| application.map(str::to_owned))
                .unwrap_or_else(|| workspace.name.clone()),
            icon: app.and_then(|app| app.icon_path.clone()),
            keys: workspace_keys(&workspace, &state.compositor.get_workspaces()),
            special: workspace.is_special,
            status: StatusSnapshot::new(&state.network.get_status(), state.power.get_battery()),
            track,
            notifications: NotificationStore::global().get_all_notifications().len(),
            dnd: NotificationStore::global().is_dnd_enabled(),
        }
    }
}

pub(crate) struct DefaultExpandedModule {
    pub snapshot: Snapshot,
    pub audio_levels: [f32; 12],
    audio_worker: Option<tokio::task::JoinHandle<()>>,
    audio_updates: Option<Task<()>>,
    focus: FocusHandle,
    refresh: Option<Task<()>>,
    _theme: Subscription,
}

impl DefaultExpandedModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            snapshot: Snapshot::read(cx.global::<AppState>(), None),
            audio_levels: [0.0; 12],
            audio_worker: None,
            audio_updates: None,
            focus: cx.focus_handle(),
            refresh: None,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }

    pub fn set_active(&mut self, active: bool, cx: &mut Context<Self>) {
        if !active {
            self.refresh = None;
            self.stop_visualizer();
        } else if self.refresh.is_none() {
            self.snapshot = Snapshot::read(cx.global::<AppState>(), None);
            let state = cx.global::<AppState>().clone();
            self.refresh = Some(cx.spawn(async move |this, cx| {
                loop {
                    let compositor = state.compositor.clone();
                    let application =
                        services::spawn_tokio(
                            async move { compositor.focused_application().await },
                        )
                        .await
                        .ok()
                        .flatten();
                    if this
                        .update(cx, |module, cx| {
                            let next = Snapshot::read(&state, application.as_deref());
                            if next != module.snapshot {
                                module.snapshot = next;
                                cx.notify();
                            }
                            module.sync_visualizer(cx);
                        })
                        .is_err()
                    {
                        break;
                    }
                    cx.background_executor().timer(Duration::from_secs(1)).await;
                }
            }));
            cx.notify();
        }
    }

    fn stop_visualizer(&mut self) {
        if let Some(worker) = self.audio_worker.take() {
            worker.abort();
        }
        self.audio_updates = None;
        self.audio_levels = [0.0; 12];
    }

    fn sync_visualizer(&mut self, cx: &mut Context<Self>) {
        if !self
            .snapshot
            .track
            .as_ref()
            .is_some_and(|track| track.is_playing)
        {
            if self.audio_worker.is_some() {
                self.stop_visualizer();
                cx.notify();
            }
            return;
        }
        if self.audio_worker.is_some() {
            return;
        }
        let (sender, mut receiver) = tokio::sync::watch::channel([0.0; 12]);
        self.audio_worker = Some(services::spawn_tokio(
            services::SystemService::monitor_audio_levels(sender),
        ));
        self.audio_updates = Some(cx.spawn(async move |this, cx| {
            while receiver.changed().await.is_ok() {
                let levels = *receiver.borrow_and_update();
                if this
                    .update(cx, |module, cx| {
                        if module.audio_levels != levels {
                            module.audio_levels = levels;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }

    pub fn toggle_playback(&self) {
        if let Some(track) = &self.snapshot.track {
            let bus = track.bus_name.clone();
            services::spawn_tokio(async move {
                services::MprisService::play_pause_bus(&bus).await;
            });
        }
    }
}

impl Drop for DefaultExpandedModule {
    fn drop(&mut self) {
        self.stop_visualizer();
    }
}

impl EventEmitter<CapsuleModuleEvent> for DefaultExpandedModule {}
impl CapsuleModule for DefaultExpandedModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(gpui::px(widgets::WIDTH), gpui::px(widgets::HEIGHT))
    }
}
impl Render for DefaultExpandedModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(self, cx)
            .track_focus(&self.focus)
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    window.focus(&this.focus, cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => cx.emit(CapsuleModuleEvent::Close),
                    "space" => this.toggle_playback(),
                    _ => {}
                }
            }))
    }
}
