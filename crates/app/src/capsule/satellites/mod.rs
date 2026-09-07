pub mod bluetooth;
pub mod calendar;
pub mod tray;
pub mod volume;
pub mod wifi;

use std::collections::VecDeque;
use std::time::Instant;
use ui::tracker::DimensionTracker;

pub const PANEL_ANIM_DURATION: f32 = 0.34;
pub const PANEL_MIN_W: f32 = 280.0;
pub const PANEL_GAP: f32 = 8.0;
pub const LANE_GAP: f32 = 12.0;
pub const DEFAULT_PANEL_H: f32 = 120.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelKind {
    Tray(usize),
    Wifi,
    Bluetooth,
    Calendar,
    Volume,
}

#[derive(Clone, Debug)]
pub struct OpenPanel {
    #[allow(dead_code)]
    pub lane: Lane,
    pub kind: PanelKind,
    pub height: f32,
    pub tracker: DimensionTracker,
    pub opened_at: Instant,
    pub closing_at: Option<Instant>,
}

impl OpenPanel {
    pub fn anim_t(&self) -> f32 {
        if let Some(closing_at) = self.closing_at {
            let t = (closing_at.elapsed().as_secs_f32() / PANEL_ANIM_DURATION).min(1.0);
            (1.0 - t).max(0.0)
        } else {
            (self.opened_at.elapsed().as_secs_f32() / PANEL_ANIM_DURATION).min(1.0)
        }
    }

    pub fn is_closing(&self) -> bool {
        self.closing_at.is_some()
    }

    pub fn is_finished_closing(&self) -> bool {
        if let Some(closing_at) = self.closing_at {
            closing_at.elapsed().as_secs_f32() >= PANEL_ANIM_DURATION
        } else {
            false
        }
    }
}

#[derive(Default)]
pub struct PanelManager {
    pub left: VecDeque<OpenPanel>,
    pub right: VecDeque<OpenPanel>,
}

impl PanelManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lane_used(lane: &VecDeque<OpenPanel>) -> f32 {
        let active: Vec<_> = lane.iter().filter(|p| !p.is_closing()).collect();
        if active.is_empty() {
            return 0.0;
        }
        let h_sum: f32 = active.iter().map(|p| p.height).sum();
        let gaps = (active.len() as f32 - 1.0) * PANEL_GAP;
        h_sum + gaps
    }

    pub fn lane_free(lane: &VecDeque<OpenPanel>, max_h: f32) -> f32 {
        let used = Self::lane_used(lane);
        let active_count = lane.iter().filter(|p| !p.is_closing()).count();
        let free = max_h - used - if active_count == 0 { 0.0 } else { PANEL_GAP };
        free.max(0.0)
    }

    #[allow(dead_code)]
    pub fn is_open(&self, kind: &PanelKind) -> bool {
        self.left
            .iter()
            .chain(self.right.iter())
            .any(|p| p.kind == *kind && !p.is_closing())
    }

    pub fn any_animating(&self) -> bool {
        self.left.iter().chain(self.right.iter()).any(|p| {
            p.opened_at.elapsed().as_secs_f32() < PANEL_ANIM_DURATION
                || p.closing_at
                    .is_some_and(|c| c.elapsed().as_secs_f32() < PANEL_ANIM_DURATION)
        })
    }

    pub fn update_animations(&mut self) {
        self.left.retain(|p| !p.is_finished_closing());
        self.right.retain(|p| !p.is_finished_closing());
    }

    pub fn toggle(&mut self, kind: PanelKind, panel_h: f32, max_lane_h: f32) {
        if let Some(p) = self.left.iter_mut().find(|p| p.kind == kind) {
            if !p.is_closing() {
                p.closing_at = Some(Instant::now());
                return;
            } else {
                p.closing_at = None;
                p.opened_at = Instant::now();
                return;
            }
        }
        if let Some(p) = self.right.iter_mut().find(|p| p.kind == kind) {
            if !p.is_closing() {
                p.closing_at = Some(Instant::now());
                return;
            } else {
                p.closing_at = None;
                p.opened_at = Instant::now();
                return;
            }
        }

        let lf = Self::lane_free(&self.left, max_lane_h);
        let rf = Self::lane_free(&self.right, max_lane_h);
        let use_left = lf >= rf;

        if use_left {
            let active_count = self.left.iter().filter(|p| !p.is_closing()).count();
            let needed = panel_h + if active_count == 0 { 0.0 } else { PANEL_GAP };
            while Self::lane_free(&self.left, max_lane_h) < needed {
                if let Some(p) = self.left.iter_mut().find(|p| !p.is_closing()) {
                    p.closing_at = Some(Instant::now());
                } else {
                    break;
                }
            }
            self.left.push_back(OpenPanel {
                lane: Lane::Left,
                kind,
                height: panel_h,
                tracker: DimensionTracker::new(),
                opened_at: Instant::now(),
                closing_at: None,
            });
        } else {
            let active_count = self.right.iter().filter(|p| !p.is_closing()).count();
            let needed = panel_h + if active_count == 0 { 0.0 } else { PANEL_GAP };
            while Self::lane_free(&self.right, max_lane_h) < needed {
                if let Some(p) = self.right.iter_mut().find(|p| !p.is_closing()) {
                    p.closing_at = Some(Instant::now());
                } else {
                    break;
                }
            }
            self.right.push_back(OpenPanel {
                lane: Lane::Right,
                kind,
                height: panel_h,
                tracker: DimensionTracker::new(),
                opened_at: Instant::now(),
                closing_at: None,
            });
        }
    }

    #[allow(dead_code)]
    pub fn close(&mut self, kind: &PanelKind) {
        if let Some(p) = self.left.iter_mut().find(|p| p.kind == *kind) {
            p.closing_at = Some(Instant::now());
            return;
        }
        if let Some(p) = self.right.iter_mut().find(|p| p.kind == *kind) {
            p.closing_at = Some(Instant::now());
        }
    }

    pub fn close_all(&mut self) {
        for p in self.left.iter_mut().chain(self.right.iter_mut()) {
            if !p.is_closing() {
                p.closing_at = Some(Instant::now());
            }
        }
    }

    pub fn prune_invalid(&mut self, valid_tray_len: usize) {
        for p in self.left.iter_mut().chain(self.right.iter_mut()) {
            if matches!(p.kind, PanelKind::Tray(sni_idx) if sni_idx >= valid_tray_len)
                && !p.is_closing()
            {
                p.closing_at = Some(Instant::now());
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn animated_position(
        lane: Lane,
        dash_w: f32,
        _dash_h: f32,
        panel_w: f32,
        lane_x: f32,
        stack_y: f32,
        t: f32,
        is_closing: bool,
    ) -> (f32, f32) {
        let factor = if is_closing {
            satellite_retract(t)
        } else {
            satellite_spring(t)
        };

        let origin_x = match lane {
            Lane::Left => 30.0,
            Lane::Right => dash_w - panel_w - 30.0,
        };

        let x = origin_x + (lane_x - origin_x) * factor;
        let droop_progress = factor.clamp(0.0, 1.0);
        let droop = (4.0 * droop_progress * (1.0 - droop_progress)).powf(1.5) * 16.0;
        let y = stack_y + droop;

        (x, y)
    }
}

fn spring_eval(t: f32, zeta: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let wd = 2.0 * std::f32::consts::PI;
    let omega0 = wd / (1.0 - zeta * zeta).sqrt();
    let beta = zeta * omega0;
    let scale = 1.0 / (1.0 - (-beta).exp());
    let envelope = (-beta * t).exp();
    let osc = (wd * t).cos() + (beta / wd) * (wd * t).sin();
    (1.0 - envelope * osc) * scale
}

pub fn satellite_spring(t: f32) -> f32 {
    spring_eval(t, 0.68)
}

pub fn satellite_retract(t: f32) -> f32 {
    let t_clamped = t.clamp(0.0, 1.0);
    t_clamped * t_clamped * (3.0 - 2.0 * t_clamped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satellite_spring_bounds() {
        assert_eq!(satellite_spring(0.0), 0.0);
        assert_eq!(satellite_spring(-0.2), 0.0);
        assert_eq!(satellite_spring(1.0), 1.0);
        assert_eq!(satellite_spring(1.5), 1.0);
    }

    #[test]
    fn test_satellite_spring_overshoot() {
        let mid = satellite_spring(0.5);
        assert!(mid > 1.03 && mid < 1.08);
    }

    #[test]
    fn test_satellite_retract() {
        assert_eq!(satellite_retract(0.0), 0.0);
        assert_eq!(satellite_retract(1.0), 1.0);
        let mid = satellite_retract(0.5);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_satellite_position() {
        let (x_open_start, y_open_start) = PanelManager::animated_position(
            Lane::Left,
            490.0,
            520.0,
            PANEL_MIN_W,
            -292.0,
            50.0,
            0.0,
            false,
        );
        assert_eq!(x_open_start, 30.0);
        assert_eq!(y_open_start, 50.0);

        let (x_open_end, y_open_end) = PanelManager::animated_position(
            Lane::Left,
            490.0,
            520.0,
            PANEL_MIN_W,
            -292.0,
            50.0,
            1.0,
            false,
        );
        assert!((x_open_end - (-292.0)).abs() < 0.01);
        assert!((y_open_end - 50.0).abs() < 0.01);

        let (x_right_start, _) = PanelManager::animated_position(
            Lane::Right,
            490.0,
            520.0,
            PANEL_MIN_W,
            502.0,
            0.0,
            0.0,
            false,
        );
        assert_eq!(x_right_start, 490.0 - PANEL_MIN_W - 30.0);

        let (x_right_end, _) = PanelManager::animated_position(
            Lane::Right,
            490.0,
            520.0,
            PANEL_MIN_W,
            502.0,
            0.0,
            1.0,
            false,
        );
        assert!((x_right_end - 502.0).abs() < 0.01);
    }
}
