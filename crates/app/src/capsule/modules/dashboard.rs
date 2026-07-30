use chrono::{Datelike, Local, Timelike, Weekday};
use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, Render, Window, div,
    prelude::*, px,
};
use services::{AppState, MediaTrack};
use std::time::Instant;
use ui::theme::Theme;

use crate::capsule::widgets::dashboard::{
    header::render_header, media_player::render_media_player_widget,
    notifications::render_notifications_widget, quick_settings::render_quick_settings_widget,
    tray::render_tray_widget, volume::render_volume_widget,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DashboardEvent {
    CloseRequested,
    SelectThemeRequested,
    TrayIconClicked(usize),
    WifiChevronClicked,
    BluetoothChevronClicked,
    CalendarClicked,
    VolumeChevronClicked,
    WallpaperRequested,
    PowerClicked,
    LanguageClicked,
}

pub struct DashboardModule {
    pub focus_handle: FocusHandle,
    pub time_str: String,
    pub date_str: String,
    pub greeting_str: String,
    pub greeting_icon: &'static str,
    pub battery_percentage: Option<i32>,
    pub battery_charging: bool,
    pub media_players: Vec<MediaTrack>,
    pub selected_player_idx: usize,
    pub last_user_action: Option<Instant>,
    pub open_panel_indices: Vec<usize>,
    pub is_dragging_volume: bool,
}

fn weekday_es(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "Lunes",
        Weekday::Tue => "Martes",
        Weekday::Wed => "Miércoles",
        Weekday::Thu => "Jueves",
        Weekday::Fri => "Viernes",
        Weekday::Sat => "Sábado",
        Weekday::Sun => "Domingo",
    }
}

fn month_es(month: u32) -> &'static str {
    match month {
        1 => "Enero",
        2 => "Febrero",
        3 => "Marzo",
        4 => "Abril",
        5 => "Mayo",
        6 => "Junio",
        7 => "Julio",
        8 => "Agosto",
        9 => "Septiembre",
        10 => "Octubre",
        11 => "Noviembre",
        12 => "Diciembre",
        _ => "",
    }
}

impl DashboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let now = Local::now();
        let hour = now.hour();

        let (greeting, icon) = match hour {
            5..=11 => ("Buenos días", "sun.svg"),
            12..=18 => ("Buenas tardes", "sun.svg"),
            _ => ("Buenas noches", "moon.svg"),
        };

        let time_str = format!("{:02}:{:02}", now.hour(), now.minute());

        let mut bat: Option<i32> = None;
        let mut charging = false;

        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("BAT") {
                    let cap_path = entry.path().join("capacity");
                    if let Ok(cap_str) = std::fs::read_to_string(cap_path) {
                        if let Ok(val) = cap_str.trim().parse::<i32>() {
                            bat = Some(val);
                        }
                    }
                    let stat_path = entry.path().join("status");
                    if let Ok(stat_str) = std::fs::read_to_string(stat_path) {
                        if stat_str.trim().to_lowercase().contains("charging") {
                            charging = true;
                        }
                    }
                    break;
                }
            }
        }

        cx.spawn(async move |this, cx| {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if this.update(cx, |_view, cx| cx.notify()).is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            focus_handle,
            time_str,
            date_str: format!(
                "{}, {} de {}",
                weekday_es(now.weekday()),
                now.day(),
                month_es(now.month())
            ),
            greeting_str: greeting.to_string(),
            greeting_icon: icon,
            battery_percentage: bat,
            battery_charging: charging,
            media_players: Vec::new(),
            selected_player_idx: 0,
            last_user_action: None,
            open_panel_indices: Vec::new(),
            is_dragging_volume: false,
        }
    }

    pub fn get_selected_player(&self) -> Option<&MediaTrack> {
        self.media_players.get(self.selected_player_idx)
    }

    pub fn get_selected_player_mut(&mut self) -> Option<&mut MediaTrack> {
        self.media_players.get_mut(self.selected_player_idx)
    }

    pub fn touch_user_action(&mut self) {
        self.last_user_action = Some(Instant::now());
    }

    #[allow(dead_code)]
    pub fn update_players(&mut self, players: Vec<MediaTrack>) {
        self.media_players = players;
        self.last_user_action = None;
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
            cx.emit(DashboardEvent::CloseRequested);
        }
    }
}

impl EventEmitter<DashboardEvent> for DashboardModule {}

impl Focusable for DashboardModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DashboardModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        if cx.has_global::<AppState>() {
            let mpris = cx.global::<AppState>().mpris.clone();
            let live_players = mpris.get_all_players();
            let recent_action = self
                .last_user_action
                .map(|t| t.elapsed() < std::time::Duration::from_millis(1500))
                .unwrap_or(false);

            if !recent_action || self.media_players.is_empty() {
                self.media_players = (*live_players).clone();
            } else {
                // Keep length & bus names in sync while preserving user's optimistic playing toggle
                for (old_p, new_p) in self.media_players.iter_mut().zip(live_players.iter()) {
                    if old_p.bus_name == new_p.bus_name {
                        let is_playing_optimistic = old_p.is_playing;
                        *old_p = new_p.clone();
                        old_p.is_playing = is_playing_optimistic;
                    }
                }
            }

            if self.selected_player_idx >= self.media_players.len()
                && !self.media_players.is_empty()
            {
                self.selected_player_idx = 0;
            }
        }

        let active_track = self
            .media_players
            .get(self.selected_player_idx)
            .cloned()
            .unwrap_or_default();

        let total_players = self.media_players.len();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                if this.is_dragging_volume {
                    let win_w: f32 = window.bounds().size.width.into();
                    let pill_x = (win_w - 440.0) / 2.0;
                    let slider_start_x = pill_x + 92.0;
                    let slider_width = 288.0;

                    let x_val = f32::from(event.position.x);
                    let rel_x = x_val - slider_start_x;
                    let pct = ((rel_x / slider_width) * 100.0).clamp(0.0, 100.0) as u32;

                    if cx.has_global::<AppState>() {
                        cx.global::<AppState>().system.set_volume_fast(pct);
                    }
                    cx.notify();
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _event: &gpui::MouseUpEvent, _window, cx| {
                    if this.is_dragging_volume {
                        this.is_dragging_volume = false;
                        cx.notify();
                    }
                }),
            )
            .flex()
            .flex_col()
            .w(px(440.0))
            .p_4()
            .gap_3p5()
            .overflow_hidden()
            .child(render_header(
                self.battery_percentage,
                self.battery_charging,
                &self.greeting_str,
                self.greeting_icon,
                &self.date_str,
                &self.time_str,
                &theme,
                cx,
            ))
            .child(div().w_full().h(px(1.0)).bg(theme.background_alt()))
            .child(render_quick_settings_widget(&theme, cx))
            .child(render_volume_widget(&theme, cx))
            .child(render_media_player_widget(
                &active_track,
                total_players,
                self.selected_player_idx,
                &theme,
                cx,
            ))
            .child(render_notifications_widget(&theme, cx))
            .child(render_tray_widget(&self.open_panel_indices, &theme, cx))
    }
}
