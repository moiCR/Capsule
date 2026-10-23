use std::time::{Duration, Instant};

pub struct SettingsAnimator {
    duration: Duration,
    entry_start: Instant,
    close_start: Option<Instant>,
    close_offset_y: f32,
    close_opacity: f32,

    pub offset_y: f32,
    pub card_opacity: f32,

    section_start: Instant,
    section_duration: Duration,
    section_progress: f32,
}

impl SettingsAnimator {
    pub fn new(duration_ms: u64) -> Self {
        let duration = Duration::from_millis(duration_ms);
        Self {
            duration,
            entry_start: Instant::now(),
            close_start: None,
            close_offset_y: 16.0,
            close_opacity: 0.0,
            offset_y: 16.0,
            card_opacity: 0.0,

            section_start: Instant::now(),
            section_duration: Duration::ZERO,
            section_progress: 1.0,
        }
    }

    pub fn request_close(&mut self) {
        if self.close_start.is_none() {
            self.close_offset_y = self.offset_y;
            self.close_opacity = self.card_opacity;
            self.close_start = Some(Instant::now());
        }
    }

    pub fn is_closing(&self) -> bool {
        self.close_start.is_some()
    }

    pub fn start_section_transition(&mut self, duration: Duration) {
        self.section_duration = duration;
        self.section_start = Instant::now();
        self.section_progress = if duration.is_zero() { 1.0 } else { 0.0 };
    }

    pub fn section_progress(&self) -> f32 {
        self.section_progress
    }

    /// Advances the animations. Returns `(running, is_closed)`.
    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        let mut running = false;
        let mut closed = false;

        if let Some(close_start) = self.close_start {
            let close_duration = self.duration.mul_f32(0.6).max(Duration::from_millis(50));
            let elapsed = now.saturating_duration_since(close_start).as_secs_f32();
            let total = close_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = self.close_offset_y + 8.0;
                self.card_opacity = 0.0;
                closed = true;
            } else {
                let progress = (elapsed / total).clamp(0.0, 1.0).powi(2);
                self.offset_y = self.close_offset_y + 8.0 * progress;
                self.card_opacity = self.close_opacity * (1.0 - progress);
                running = true;
            }
        } else {
            let entry_duration = self.duration.mul_f32(0.88).max(Duration::from_millis(50));
            let elapsed = now
                .saturating_duration_since(self.entry_start)
                .as_secs_f32();
            let total = entry_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = 0.0;
                self.card_opacity = 1.0;
            } else {
                let remaining = (1.0 - (elapsed / total).clamp(0.0, 1.0)).powi(3);
                self.offset_y = 16.0 * remaining;
                self.card_opacity = 1.0 - remaining;
                running = true;
            }
        }

        if !self.section_duration.is_zero() && self.section_progress < 1.0 {
            let elapsed = now
                .saturating_duration_since(self.section_start)
                .as_secs_f32();
            let total = self.section_duration.as_secs_f32();
            if elapsed >= total {
                self.section_progress = 1.0;
            } else {
                self.section_progress = (elapsed / total).clamp(0.0, 1.0);
                running = true;
            }
        }

        (running, closed)
    }
}
