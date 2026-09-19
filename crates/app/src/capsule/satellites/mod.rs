pub mod bluetooth;
pub mod calendar;
pub mod media;
pub mod surface;
pub mod tray;
pub mod volume;
pub mod wifi;

use std::collections::VecDeque;
use std::time::Instant;
use ui::tracker::DimensionTracker;

pub const PANEL_MIN_W: f32 = 280.0;
pub const PANEL_GAP: f32 = 8.0;
pub const LANE_GAP: f32 = 12.0;
pub const DEFAULT_PANEL_H: f32 = 120.0;
const GUM_DURATION_SCALE: f32 = 1.35;
const CLOSE_DURATION_SCALE: f32 = 0.7;

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
    #[allow(dead_code)]
    Media,
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
    phase_started_at: Instant,
    phase_start: f32,
    animation_duration: f32,
}

impl OpenPanel {
    fn new(lane: Lane, kind: PanelKind, height: f32, duration: f32, now: Instant) -> Self {
        Self {
            lane,
            kind,
            height,
            tracker: DimensionTracker::new(),
            opened_at: now,
            closing_at: None,
            phase_started_at: now,
            phase_start: 0.0,
            animation_duration: duration,
        }
    }

    fn phase_at(&self, now: Instant) -> f32 {
        if self.animation_duration <= 0.0 {
            return if self.is_closing() { 0.0 } else { 1.0 };
        }
        let elapsed = now
            .saturating_duration_since(self.phase_started_at)
            .as_secs_f32();
        let delta = elapsed / self.animation_duration;
        if self.is_closing() {
            (self.phase_start - delta / CLOSE_DURATION_SCALE).clamp(0.0, 1.0)
        } else {
            (self.phase_start + delta).clamp(0.0, 1.0)
        }
    }

    fn set_closing_at(&mut self, closing: bool, now: Instant) {
        if self.is_closing() == closing {
            return;
        }
        self.phase_start = self.phase_at(now);
        self.phase_started_at = now;
        self.closing_at = closing.then_some(now);
        if !closing {
            self.opened_at = now;
        }
    }

    pub fn anim_t(&self) -> f32 {
        self.phase_at(Instant::now())
    }

    pub fn stack_weight(&self) -> f32 {
        satellite_retract(self.anim_t())
    }

    pub fn is_closing(&self) -> bool {
        self.closing_at.is_some()
    }

    pub fn is_finished_closing(&self) -> bool {
        self.is_closing() && self.anim_t() <= 0.0
    }
}

pub struct PanelManager {
    pub left: VecDeque<OpenPanel>,
    pub right: VecDeque<OpenPanel>,
    animation_duration: f32,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            left: VecDeque::new(),
            right: VecDeque::new(),
            animation_duration: services::config::UIConfig::default().animation_duration_ms as f32
                / 1000.0
                * GUM_DURATION_SCALE,
        }
    }
}

impl PanelManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_animation_duration(&mut self, seconds: f32) {
        let duration = if seconds.is_finite() {
            (seconds.max(0.0) * GUM_DURATION_SCALE).min(f32::MAX)
        } else {
            0.0
        };
        if self.animation_duration == duration {
            return;
        }
        let now = Instant::now();
        for panel in self.left.iter_mut().chain(self.right.iter_mut()) {
            panel.phase_start = panel.phase_at(now);
            panel.phase_started_at = now;
            panel.animation_duration = duration;
        }
        self.animation_duration = duration;
    }

    pub fn lane_used(lane: &VecDeque<OpenPanel>) -> f32 {
        let mut used = 0.0;
        let mut count: usize = 0;
        for panel in lane.iter().filter(|panel| !panel.is_closing()) {
            used += panel.height;
            count += 1;
        }
        used + count.saturating_sub(1) as f32 * PANEL_GAP
    }

    pub fn lane_free(lane: &VecDeque<OpenPanel>, max_h: f32) -> f32 {
        let used = Self::lane_used(lane);
        let has_active = lane.iter().any(|panel| !panel.is_closing());
        (max_h - used - if has_active { PANEL_GAP } else { 0.0 }).max(0.0)
    }

    pub fn has_open(&self) -> bool {
        self.left
            .iter()
            .chain(self.right.iter())
            .any(|panel| !panel.is_closing())
    }

    #[allow(dead_code)]
    pub fn is_open(&self, kind: &PanelKind) -> bool {
        self.left
            .iter()
            .chain(self.right.iter())
            .any(|panel| panel.kind == *kind && !panel.is_closing())
    }

    pub fn any_animating(&self) -> bool {
        self.left
            .iter()
            .chain(self.right.iter())
            .any(|panel| panel.is_closing() || panel.anim_t() < 1.0)
    }

    pub fn update_animations(&mut self) {
        self.left.retain(|panel| !panel.is_finished_closing());
        self.right.retain(|panel| !panel.is_finished_closing());
    }

    pub fn toggle(&mut self, kind: PanelKind, panel_h: f32, max_lane_h: f32) {
        self.toggle_in_lane(kind, panel_h, max_lane_h, None);
    }

    pub fn toggle_in_lane(
        &mut self,
        kind: PanelKind,
        panel_h: f32,
        max_lane_h: f32,
        preferred_lane: Option<Lane>,
    ) {
        let now = Instant::now();
        if let Some(panel) = self
            .left
            .iter_mut()
            .chain(self.right.iter_mut())
            .find(|panel| panel.kind == kind)
        {
            panel.set_closing_at(!panel.is_closing(), now);
            return;
        }

        let lane = preferred_lane.unwrap_or_else(|| {
            if Self::lane_free(&self.left, max_lane_h) >= Self::lane_free(&self.right, max_lane_h) {
                Lane::Left
            } else {
                Lane::Right
            }
        });
        let panels = match lane {
            Lane::Left => &mut self.left,
            Lane::Right => &mut self.right,
        };
        while Self::lane_free(panels, max_lane_h) < panel_h {
            if let Some(panel) = panels.iter_mut().find(|panel| !panel.is_closing()) {
                panel.set_closing_at(true, now);
            } else {
                break;
            }
        }
        panels.push_back(OpenPanel::new(
            lane,
            kind,
            panel_h,
            self.animation_duration,
            now,
        ));
    }

    #[allow(dead_code)]
    pub fn close(&mut self, kind: &PanelKind) {
        let now = Instant::now();
        if let Some(panel) = self
            .left
            .iter_mut()
            .chain(self.right.iter_mut())
            .find(|panel| panel.kind == *kind)
        {
            panel.set_closing_at(true, now);
        }
    }

    #[allow(dead_code)]
    pub fn close_all(&mut self) {
        let now = Instant::now();
        for panel in self.left.iter_mut().chain(self.right.iter_mut()) {
            panel.set_closing_at(true, now);
        }
    }

    pub fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
    }

    pub fn prune_invalid(&mut self, valid_tray_len: usize) {
        let now = Instant::now();
        for panel in self.left.iter_mut().chain(self.right.iter_mut()) {
            if matches!(panel.kind, PanelKind::Tray(index) if index >= valid_tray_len) {
                panel.set_closing_at(true, now);
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
        _is_closing: bool,
    ) -> (f32, f32) {
        let t = t.clamp(0.0, 1.0);
        let inset = ((dash_w - panel_w) * 0.5).clamp(0.0, 30.0);
        let (origin_x, contact_x, direction) = match lane {
            Lane::Left => (inset, -panel_w, -1.0),
            Lane::Right => (dash_w - panel_w - inset, dash_w, 1.0),
        };
        let stretched_x = contact_x + (lane_x - contact_x) * 0.9;
        let settled_x = lane_x + direction * 1.5;
        let x = if t < 0.35 {
            interpolate(origin_x, contact_x, t / 0.35)
        } else if t < 0.7 {
            interpolate(contact_x, stretched_x, (t - 0.35) / 0.35)
        } else if t < 0.85 {
            interpolate(stretched_x, settled_x, (t - 0.7) / 0.15)
        } else {
            interpolate(settled_x, lane_x, (t - 0.85) / 0.15)
        };
        (x, stack_y)
    }
}

fn interpolate(start: f32, end: f32, t: f32) -> f32 {
    start + (end - start) * satellite_retract(t)
}

pub fn satellite_spring(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let zeta = 0.68_f32;
    let wd = 2.0 * std::f32::consts::PI;
    let omega0 = wd / (1.0 - zeta * zeta).sqrt();
    let beta = zeta * omega0;
    let scale = 1.0 / (1.0 - (-beta).exp());
    let envelope = (-beta * t).exp();
    let osc = (wd * t).cos() + (beta / wd) * (wd * t).sin();
    (1.0 - envelope * osc) * scale
}

pub fn satellite_retract(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn assert_near(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
    }

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
    fn phase_is_linear_and_reverses_from_current_position() {
        let start = Instant::now();
        let mut panel = OpenPanel::new(Lane::Left, PanelKind::Wifi, 120.0, 1.0, start);
        let reverse = start + Duration::from_millis(400);
        assert_near(panel.phase_at(reverse), 0.4);
        panel.set_closing_at(true, reverse);
        assert_near(panel.phase_at(reverse), 0.4);
        let reopen = reverse + Duration::from_millis(140);
        assert_near(panel.phase_at(reopen), 0.2);
        panel.set_closing_at(false, reopen);
        assert_near(panel.phase_at(reopen), 0.2);
        assert_near(panel.phase_at(reopen + Duration::from_millis(800)), 1.0);
    }

    #[test]
    fn repeated_close_does_not_restart_motion() {
        let start = Instant::now();
        let mut panel = OpenPanel::new(Lane::Left, PanelKind::Wifi, 120.0, 1.0, start);
        let close = start + Duration::from_secs(1);
        panel.set_closing_at(true, close);
        panel.set_closing_at(true, close + Duration::from_millis(350));
        assert_eq!(panel.phase_started_at, close);
        assert_near(panel.phase_at(close + Duration::from_millis(700)), 0.0);
    }

    #[test]
    fn geometry_is_direction_independent_and_has_no_droop() {
        for lane in [Lane::Left, Lane::Right] {
            let lane_x = if lane == Lane::Left { -292.0 } else { 502.0 };
            for step in 0..=100 {
                let t = step as f32 / 100.0;
                let open = PanelManager::animated_position(
                    lane, 490.0, 520.0, 280.0, lane_x, 50.0, t, false,
                );
                let close = PanelManager::animated_position(
                    lane, 490.0, 520.0, 280.0, lane_x, 50.0, t, true,
                );
                assert_eq!(open, close);
                assert_eq!(open.1, 50.0);
            }
            let position = |t| {
                PanelManager::animated_position(lane, 490.0, 520.0, 280.0, lane_x, 50.0, t, false).0
            };
            let (origin, contact, direction) = if lane == Lane::Left {
                (30.0, -280.0, -1.0)
            } else {
                (180.0, 490.0, 1.0)
            };
            assert_near(position(0.0), origin);
            assert_near(position(0.35), contact);
            assert_near(position(0.7), contact + (lane_x - contact) * 0.9);
            assert_near(position(0.85), lane_x + direction * 1.5);
            assert_near(position(1.0), lane_x);
            for boundary in [0.35, 0.7, 0.85] {
                assert!(
                    (position(boundary - 0.00001) - position(boundary + 0.00001)).abs() < 0.001
                );
            }
        }
    }

    #[test]
    fn stack_weight_is_bounded_and_continuous_on_reversal() {
        let start = Instant::now();
        let mut panel = OpenPanel::new(Lane::Left, PanelKind::Wifi, 120.0, 1.0, start);
        let now = start + Duration::from_millis(450);
        let weight = satellite_retract(panel.phase_at(now));
        panel.set_closing_at(true, now);
        assert_near(satellite_retract(panel.phase_at(now)), weight);
        for step in 0..=100 {
            let phase = step as f32 / 100.0;
            assert!((0.0..=1.0).contains(&satellite_retract(phase)));
        }
        assert_eq!(satellite_retract(0.0), 0.0);
        assert_eq!(satellite_retract(1.0), 1.0);
    }

    #[test]
    fn zero_duration_is_instant_for_every_close_path() {
        let mut manager = PanelManager::new();
        manager.set_animation_duration(0.0);
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        assert_eq!(manager.left[0].anim_t(), 1.0);
        assert_eq!(manager.left[0].stack_weight(), 1.0);
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        assert_eq!(manager.left[0].anim_t(), 0.0);
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        assert_eq!(manager.left[0].anim_t(), 1.0);
        manager.close(&PanelKind::Wifi);
        manager.update_animations();
        assert!(manager.left.is_empty());
        manager.toggle(PanelKind::Tray(0), 120.0, 200.0);
        manager.prune_invalid(0);
        manager.update_animations();
        assert!(manager.left.is_empty());
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        manager.toggle(PanelKind::Bluetooth, 120.0, 200.0);
        manager.toggle(PanelKind::Calendar, 120.0, 200.0);
        assert!(manager.left[0].is_finished_closing());
        manager.update_animations();
        manager.close_all();
        manager.update_animations();
        assert!(manager.left.is_empty() && manager.right.is_empty());
        assert!(!manager.any_animating());
    }

    #[test]
    fn duration_changes_rebase_instead_of_resetting() {
        let mut manager = PanelManager::new();
        manager.set_animation_duration(1.0);
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        manager.left[0].phase_start = 0.4;
        let before = manager.left[0].anim_t();
        manager.set_animation_duration(2.0);
        assert!((manager.left[0].anim_t() - before).abs() < 0.01);
        assert_near(manager.animation_duration, 2.7);
        manager.set_animation_duration(0.0);
        assert_eq!(manager.left[0].anim_t(), 1.0);
    }

    #[test]
    fn clear_removes_all_panels_immediately() {
        let mut manager = PanelManager::new();
        manager.toggle(PanelKind::Wifi, 120.0, 200.0);
        assert_eq!(manager.left.len(), 1);
        manager.clear();
        assert!(manager.left.is_empty() && manager.right.is_empty());
        assert!(!manager.any_animating());
    }

    #[test]
    fn media_panel_toggle_and_height() {
        let mut manager = PanelManager::new();
        let height = media::compute_media_panel_height();
        assert_eq!(height, 195.0);
        manager.toggle(PanelKind::Media, height, 300.0);
        assert_eq!(manager.left.len(), 1);
        assert_eq!(manager.left[0].kind, PanelKind::Media);
        assert_eq!(manager.left[0].height, 195.0);
        manager.toggle(PanelKind::Media, height, 300.0);
        assert!(manager.left[0].is_closing());
    }
}
