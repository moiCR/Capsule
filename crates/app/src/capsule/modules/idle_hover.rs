use chrono::{Datelike, Local, Timelike, Weekday};
use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, IntoElement, KeyDownEvent, Render, Window,
    canvas, div, prelude::*, px,
};
use services::{AppState, MediaTrack};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::capsule::widgets::{
    header::render_header, media_player::render_media_player_widget,
    notifications::render_notifications_widget, tray::render_tray_widget,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdleHoverEvent {
    CloseRequested,
    SelectThemeRequested,
}

pub struct IdleHoverModule {
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
    pub measured_top: Rc<Cell<f32>>,
    pub measured_bottom: Rc<Cell<f32>>,
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

fn get_greeting_info(hour: u32) -> (&'static str, &'static str) {
    if hour < 6 {
        ("¡Buenas noches, Moi!", "sparkles.svg")
    } else if hour < 12 {
        ("¡Buenos días, Moi!", "sun.svg")
    } else if hour < 18 {
        ("¡Buenas tardes, Moi!", "sun.svg")
    } else {
        ("¡Buenas noches, Moi!", "moon.svg")
    }
}

async fn read_battery() -> Option<(i32, bool)> {
    let cap_str = tokio::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .await
        .ok()?;
    let status_str = tokio::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .await
        .ok()?;

    let cap = cap_str.trim().parse::<i32>().ok()?;
    let charging = status_str.trim() == "Charging";
    Some((cap, charging))
}

impl IdleHoverModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Clock & battery status polling
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;

                let now = Local::now();
                let time_val = format!("{:02}:{:02}", now.hour(), now.minute());
                let date_val = format!(
                    "{}, {} de {}",
                    weekday_es(now.weekday()),
                    now.day(),
                    month_es(now.month())
                );

                let (greeting, icon) = get_greeting_info(now.hour());
                let (bat, charging) = read_battery().await.unwrap_or((100, false));

                let res = this.update(cx, |this: &mut Self, cx| {
                    let changed = this.time_str != time_val
                        || this.date_str != date_val
                        || this.greeting_str != greeting
                        || this.battery_percentage != Some(bat)
                        || this.battery_charging != charging;

                    if changed {
                        this.time_str = time_val;
                        this.date_str = date_val;
                        this.greeting_str = greeting.to_string();
                        this.greeting_icon = icon;
                        this.battery_percentage = Some(bat);
                        this.battery_charging = charging;
                        cx.notify();
                    }
                });
                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        let mpris_service = cx.global::<AppState>().mpris.clone();
        cx.spawn(async move |this, cx| {
            loop {
                let players = mpris_service.get_all_players();

                let res = this.update(cx, |this: &mut Self, cx| {
                    let user_interacting = this
                        .last_user_action
                        .map(|t| t.elapsed() < Duration::from_millis(1500))
                        .unwrap_or(false);

                    if !user_interacting {
                        let active_idx = players.iter().position(|p| p.is_playing).unwrap_or(0);

                        if this.media_players != *players {
                            this.media_players = (*players).clone();
                            this.selected_player_idx =
                                active_idx.min(this.media_players.len().saturating_sub(1));
                            cx.notify();
                        }
                    }
                });

                if res.is_err() {
                    break;
                }

                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
            }
        })
        .detach();

        let now = Local::now();
        let (greeting, icon) = get_greeting_info(now.hour());
        let (bat, charging) = (100, false);

        Self {
            focus_handle,
            time_str: format!("{:02}:{:02}", now.hour(), now.minute()),
            date_str: format!(
                "{}, {} de {}",
                weekday_es(now.weekday()),
                now.day(),
                month_es(now.month())
            ),
            greeting_str: greeting.to_string(),
            greeting_icon: icon,
            battery_percentage: Some(bat),
            battery_charging: charging,
            media_players: Vec::new(),
            selected_player_idx: 0,
            last_user_action: None,
            measured_top: Rc::new(Cell::new(0.0)),
            measured_bottom: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn measured_size(&self) -> (f32, f32) {
        let content_height = (self.measured_bottom.get() - self.measured_top.get()).max(0.0);
        let total_height = if content_height > 0.0 {
            content_height + 32.0
        } else {
            0.0
        };
        (348.0, total_height)
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
            cx.emit(IdleHoverEvent::CloseRequested);
        }
    }
}

impl EventEmitter<IdleHoverEvent> for IdleHoverModule {}

impl Focusable for IdleHoverModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for IdleHoverModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let active_track = self
            .media_players
            .get(self.selected_player_idx)
            .cloned()
            .unwrap_or_default();

        let total_players = self.media_players.len();

        let measured_top = self.measured_top.clone();
        let measured_bottom = self.measured_bottom.clone();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(348.0))
            .p_4()
            .gap_3p5()
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _window, _cx| {
                        measured_top.set(bounds.origin.y.into());
                    },
                    |_, _, _, _| {},
                )
                .w_full(),
            )
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
            .child(render_media_player_widget(
                &active_track,
                total_players,
                self.selected_player_idx,
                &theme,
                cx,
            ))
            .child(render_notifications_widget(&theme, cx))
            .child(render_tray_widget(&theme, cx))
            .child(
                canvas(
                    move |bounds, _window, _cx| {
                        measured_bottom.set(bounds.origin.y.into());
                    },
                    |_, _, _, _| {},
                )
                .w_full(),
            )
    }
}
