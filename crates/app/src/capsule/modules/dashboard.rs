use chrono::{Datelike, Local, Timelike, Weekday};
use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, Render, Window, div,
    prelude::*, px,
};
use services::{AppState, MediaTrack};
use std::time::Instant;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::capsule::widgets::dashboard::{
    header::render_header, notifications::render_notifications_widget,
    quick_settings::render_quick_settings_section, volume::render_volume_widget,
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
    SettingsRequested,
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
    pub slider_tracker: DimensionTracker,
}

fn format_dashboard_date(
    now: &chrono::DateTime<Local>,
    cx: &mut Context<DashboardModule>,
) -> String {
    if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        let days = lang.get_list("datetime.days");
        let months = lang.get_list("datetime.months");
        let weekday_idx = now.weekday().num_days_from_monday() as usize;
        let weekday_name = days
            .get(weekday_idx)
            .cloned()
            .unwrap_or_else(|| match now.weekday() {
                Weekday::Mon => "Monday".to_string(),
                Weekday::Tue => "Tuesday".to_string(),
                Weekday::Wed => "Wednesday".to_string(),
                Weekday::Thu => "Thursday".to_string(),
                Weekday::Fri => "Friday".to_string(),
                Weekday::Sat => "Saturday".to_string(),
                Weekday::Sun => "Sunday".to_string(),
            });
        let month_idx = (now.month() as usize).saturating_sub(1);
        let month_name = months.get(month_idx).cloned().unwrap_or_default();
        let day_str = now.day().to_string();
        let template = lang.get("datetime.header_date_format");
        if template != "datetime.header_date_format" && !template.is_empty() {
            template
                .replace("{weekday}", &weekday_name)
                .replace("{day}", &day_str)
                .replace("{month}", &month_name)
        } else if lang.current_language_code() == "en" {
            format!("{weekday_name}, {month_name} {day_str}")
        } else {
            format!("{weekday_name}, {day_str} de {month_name}")
        }
    } else {
        format!("{}, {} {}", now.weekday(), now.month(), now.day())
    }
}

fn format_greeting(hour: u32, cx: &mut Context<DashboardModule>) -> (&'static str, String) {
    let (icon, key, fallback) = match hour {
        5..=11 => ("sun.svg", "dashboard.greeting_morning", "Buenos días"),
        12..=18 => ("sun.svg", "dashboard.greeting_afternoon", "Buenas tardes"),
        _ => ("moon.svg", "dashboard.greeting_evening", "Buenas noches"),
    };
    let text = if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        fallback.to_string()
    };
    (icon, text)
}

impl DashboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

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
            time_str: String::new(),
            date_str: String::new(),
            greeting_str: String::new(),
            greeting_icon: "sun.svg",
            battery_percentage: bat,
            battery_charging: charging,
            media_players: Vec::new(),
            selected_player_idx: 0,
            last_user_action: None,
            open_panel_indices: Vec::new(),
            is_dragging_volume: false,
            slider_tracker: DimensionTracker::new(),
        }
    }

    pub fn desired_width(&self, cx: &gpui::App) -> f32 {
        let sni_count = if cx.has_global::<AppState>() {
            cx.global::<AppState>().sni_host.get_items().len()
        } else {
            0
        };
        let tray_w = if sni_count > 0 { sni_count as f32 * 30.0 } else { 0.0 };
        let battery_w = if self.battery_percentage.is_some() { 55.0 } else { 0.0 };
        let header_needed_w = 220.0 + tray_w + battery_w + 16.0 + 32.0;
        490.0_f32.max(header_needed_w)
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

        let now = Local::now();
        let time_str = format!("{:02}:{:02}", now.hour(), now.minute());
        let date_str = format_dashboard_date(&now, cx);
        let (greeting_icon, greeting_str) = format_greeting(now.hour(), cx);
        self.time_str = time_str;
        self.date_str = date_str;
        self.greeting_str = greeting_str;
        self.greeting_icon = greeting_icon;

        let dashboard_w = self.desired_width(cx);

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, window, cx| {
                    if this.is_dragging_volume {
                        let slider_x = this.slider_tracker.left();
                        let slider_w = this.slider_tracker.width(0.0);
                        let (start_x, width) = if slider_w > 0.0 {
                            (slider_x, slider_w)
                        } else {
                            let win_w: f32 = window.bounds().size.width.into();
                            let pill_x = (win_w - dashboard_w) / 2.0;
                            (pill_x + 28.0, dashboard_w - 56.0)
                        };

                        let x_val = f32::from(event.position.x);
                        let rel_x = x_val - start_x;
                        let pct = ((rel_x / width) * 100.0).clamp(0.0, 100.0) as u32;

                        if cx.has_global::<AppState>() {
                            cx.global::<AppState>().system.set_volume_fast(pct);
                        }
                        cx.notify();
                    }
                }),
            )
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
            .w(px(dashboard_w))
            .p_4()
            .gap_2p5()
            .overflow_hidden()
            .child(render_header(
                self.battery_percentage,
                self.battery_charging,
                &self.open_panel_indices,
                &self.greeting_str,
                self.greeting_icon,
                &self.date_str,
                &self.time_str,
                &theme,
                cx,
            ))
            .child(render_quick_settings_section(
                &active_track,
                total_players,
                self.selected_player_idx,
                &theme,
                cx,
            ))
            .child(render_volume_widget(&self.slider_tracker, &theme, cx))
            .child(render_notifications_widget(dashboard_w, &theme, cx))
    }
}
