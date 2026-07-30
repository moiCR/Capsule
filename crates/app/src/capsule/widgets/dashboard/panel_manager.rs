use std::collections::VecDeque;
use std::time::Instant;

/// Emerge animation duration in seconds.
pub const PANEL_ANIM_DURATION: f32 = 0.25;

/// Satellite panel width (fixed width across satellite panels).
pub const PANEL_W: f32 = 200.0;

/// Gap between consecutive panels in a lane, and between lane and pill.
pub const PANEL_GAP: f32 = 8.0;
pub const LANE_GAP: f32 = 12.0;

/// Default panel height for tray chips.
pub const DEFAULT_PANEL_H: f32 = 120.0;

/// Which side of the Dashboard a panel lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    Left,
    Right,
}

/// The type of satellite panel orbiting the Dashboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelKind {
    Tray(usize),
    Wifi,
    Bluetooth,
    Calendar,
    Volume,
}

use ui::tracker::DimensionTracker;

/// A single open satellite panel.
#[derive(Clone, Debug)]
pub struct OpenPanel {
    pub lane: Lane,
    pub kind: PanelKind,
    /// Logical height of this panel in pixels.
    pub height: f32,
    pub tracker: DimensionTracker,
    /// When this panel was opened — drives the emerge animation.
    pub opened_at: Instant,
    /// When this panel started closing — drives the shrink/disappear animation.
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

/// Manages left/right lane-based dynamic panel layout.
///
/// Panels stack from top within each lane.
/// Lane height is capped at the Dashboard's current height.
/// When a new panel doesn't fit, the oldest in that lane is evicted until
/// there is enough space (or the lane can hold just this one panel).
pub struct PanelManager {
    pub left: VecDeque<OpenPanel>,
    pub right: VecDeque<OpenPanel>,
}

impl Default for PanelManager {
    fn default() -> Self {
        Self {
            left: VecDeque::new(),
            right: VecDeque::new(),
        }
    }
}

impl PanelManager {
    pub fn new() -> Self {
        Self::default()
    }

    // ── Queries ──────────────────────────────────────────────────────────

    /// Total height consumed by active panels in a lane (including inter-panel gaps).
    pub fn lane_used(lane: &VecDeque<OpenPanel>) -> f32 {
        let active: Vec<_> = lane.iter().filter(|p| !p.is_closing()).collect();
        if active.is_empty() {
            return 0.0;
        }
        let h_sum: f32 = active.iter().map(|p| p.height).sum();
        let gaps = (active.len() as f32 - 1.0) * PANEL_GAP;
        h_sum + gaps
    }

    /// Free vertical pixels in a lane given the maximum available height.
    pub fn lane_free(lane: &VecDeque<OpenPanel>, max_h: f32) -> f32 {
        let used = Self::lane_used(lane);
        let active_count = lane.iter().filter(|p| !p.is_closing()).count();
        let free = max_h - used - if active_count == 0 { 0.0 } else { PANEL_GAP };
        free.max(0.0)
    }

    /// True if `kind` is already open (and not closing) in either lane.
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
                    .map_or(false, |c| c.elapsed().as_secs_f32() < PANEL_ANIM_DURATION)
        })
    }

    /// Remove panels that have finished their closing animation.
    pub fn update_animations(&mut self) {
        self.left.retain(|p| !p.is_finished_closing());
        self.right.retain(|p| !p.is_finished_closing());
    }

    // ── Mutation ─────────────────────────────────────────────────────────

    /// Toggle a panel open/closed.
    ///
    /// `panel_h` is the height of this panel.
    /// `max_lane_h` is the current Dashboard height (lane height cap).
    pub fn toggle(&mut self, kind: PanelKind, panel_h: f32, max_lane_h: f32) {
        // If already open and active, start closing it
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

        // Choose the lane that has more free space
        let lf = Self::lane_free(&self.left, max_lane_h);
        let rf = Self::lane_free(&self.right, max_lane_h);
        let use_left = lf >= rf;

        // Make room by evicting the oldest active panel until the new one fits
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

    /// Mark a panel as closing by kind.
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

    /// Prune open panels whose tray sni_idx is no longer valid (e.g. app was closed).
    pub fn prune_invalid(&mut self, valid_tray_len: usize) {
        for p in self.left.iter_mut().chain(self.right.iter_mut()) {
            if let PanelKind::Tray(sni_idx) = p.kind {
                if sni_idx >= valid_tray_len && !p.is_closing() {
                    p.closing_at = Some(Instant::now());
                }
            }
        }
    }

    // ── Animation helpers ─────────────────────────────────────────────────

    /// Emerge animation origin: pill center (in pill_wrapper coordinates).
    pub fn emerge_origin(dash_w: f32, dash_h: f32) -> (f32, f32) {
        (
            dash_w / 2.0 - PANEL_W / 2.0,
            dash_h / 2.0 - DEFAULT_PANEL_H / 2.0,
        )
    }

    pub fn animated_position(
        dash_w: f32,
        dash_h: f32,
        lane_x: f32,
        stack_y: f32,
        t: f32,
        is_closing: bool,
    ) -> (f32, f32) {
        let (ox, oy) = Self::emerge_origin(dash_w, dash_h);

        let e = if is_closing {
            // When closing, t goes from 1.0 down to 0.0.
            // We want the inverse of ease_out_back, which is ease_in_back.
            // But since t is backwards, 1.0 - t gives us the forward closing progress.
            1.0 - ease_in_back(1.0 - t)
        } else {
            ease_out_back(t)
        };

        (ox + (lane_x - ox) * e, oy + (stack_y - oy) * e)
    }
}

/// Ease-in-back: pull back slightly before accelerating.
pub fn ease_in_back(t: f32) -> f32 {
    let c1: f32 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

/// Ease-out-back: snappy pop with slight overshoot.
pub fn ease_out_back(t: f32) -> f32 {
    if t >= 1.0 {
        return 1.0;
    }
    let c1: f32 = 1.70158;
    let c3 = c1 + 1.0;
    let t1 = t - 1.0;
    1.0 + c3 * t1 * t1 * t1 + c1 * t1 * t1
}
