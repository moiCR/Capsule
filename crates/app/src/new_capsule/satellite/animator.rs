use std::time::{Duration, Instant};

pub(super) struct SatelliteAnimator {
    progress: f32,
    start: f32,
    target: f32,
    started_at: Option<Instant>,
    duration: Duration,
}

impl SatelliteAnimator {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            start: 0.0,
            target: 0.0,
            started_at: None,
            duration: Duration::ZERO,
        }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn open(&self) -> bool {
        self.target == 1.0
    }

    pub fn transition(&mut self, open: bool, duration: Duration, now: Instant) {
        self.advance(now);
        let target = if open { 1.0 } else { 0.0 };
        if self.target == target {
            return;
        }
        self.start = self.progress;
        self.target = target;
        self.duration = duration.mul_f32((target - self.start).abs());
        self.started_at = Some(now);
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };
        let elapsed = now.saturating_duration_since(started_at);
        if elapsed >= self.duration {
            self.progress = self.target;
            self.started_at = None;
            return false;
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let eased = 1.0 - (1.0 - t).powi(3);
        self.progress = self.start + (self.target - self.start) * eased;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversing_preserves_position_and_finishes_without_idle_frames() {
        let now = Instant::now();
        let mut animator = SatelliteAnimator::new();
        animator.transition(true, Duration::from_millis(200), now);
        assert!(animator.advance(now + Duration::from_millis(100)));
        let displayed = animator.progress();
        animator.transition(
            false,
            Duration::from_millis(200),
            now + Duration::from_millis(100),
        );
        assert_eq!(animator.progress(), displayed);
        assert!(!animator.advance(now + Duration::from_millis(300)));
        assert_eq!(animator.progress(), 0.0);
        assert!(!animator.advance(now + Duration::from_secs(1)));
    }

    #[test]
    fn zero_duration_opens_and_closes_immediately() {
        let mut animator = SatelliteAnimator::new();
        let now = Instant::now();
        animator.transition(true, Duration::ZERO, now);
        assert_eq!(animator.progress(), 1.0);
        animator.transition(false, Duration::ZERO, now);
        assert_eq!(animator.progress(), 0.0);
    }
}
