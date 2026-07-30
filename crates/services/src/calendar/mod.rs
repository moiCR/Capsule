use chrono::{Datelike, Local};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const CALENDAR_ANIM_DURATION: f32 = 0.20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavDirection {
    Left,
    Right,
}

#[derive(Clone, Debug)]
pub struct CalendarState {
    pub view_year: i32,
    pub view_month: u32,
    pub nav_start: Option<Instant>,
    pub nav_direction: NavDirection,
}

#[derive(Clone)]
pub struct CalendarService {
    state: Arc<Mutex<CalendarState>>,
}

impl Default for CalendarService {
    fn default() -> Self {
        Self::new()
    }
}

impl CalendarService {
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            state: Arc::new(Mutex::new(CalendarState {
                view_year: now.year(),
                view_month: now.month(),
                nav_start: None,
                nav_direction: NavDirection::Right,
            })),
        }
    }

    pub fn get_view_date(&self) -> (i32, u32) {
        if let Ok(g) = self.state.lock() {
            (g.view_year, g.view_month)
        } else {
            let now = Local::now();
            (now.year(), now.month())
        }
    }

    pub fn get_nav_anim(&self) -> (NavDirection, f32) {
        if let Ok(g) = self.state.lock() {
            if let Some(start) = g.nav_start {
                let elapsed = start.elapsed().as_secs_f32();
                let t = (elapsed / CALENDAR_ANIM_DURATION).min(1.0);
                (g.nav_direction, t)
            } else {
                (g.nav_direction, 1.0)
            }
        } else {
            (NavDirection::Right, 1.0)
        }
    }

    pub fn prev_month(&self) {
        if let Ok(mut g) = self.state.lock() {
            if g.view_month == 1 {
                g.view_month = 12;
                g.view_year -= 1;
            } else {
                g.view_month -= 1;
            }
            g.nav_direction = NavDirection::Left;
            g.nav_start = Some(Instant::now());
        }
    }

    pub fn next_month(&self) {
        if let Ok(mut g) = self.state.lock() {
            if g.view_month == 12 {
                g.view_month = 1;
                g.view_year += 1;
            } else {
                g.view_month += 1;
            }
            g.nav_direction = NavDirection::Right;
            g.nav_start = Some(Instant::now());
        }
    }

    pub fn reset_to_today(&self) {
        let now = Local::now();
        if let Ok(mut g) = self.state.lock() {
            let cur_total = g.view_year * 12 + g.view_month as i32;
            let target_total = now.year() * 12 + now.month() as i32;
            g.nav_direction = if target_total >= cur_total {
                NavDirection::Right
            } else {
                NavDirection::Left
            };
            g.view_year = now.year();
            g.view_month = now.month();
            g.nav_start = Some(Instant::now());
        }
    }
}
