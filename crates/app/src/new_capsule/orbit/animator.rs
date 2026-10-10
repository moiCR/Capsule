use std::time::{Duration, Instant};

pub(super) struct Animator {
    progress: f32,
    start: f32,
    target: f32,
    started: Option<Instant>,
    duration: Duration,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            start: 0.0,
            target: 0.0,
            started: None,
            duration: Duration::ZERO,
        }
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn active(&self) -> bool {
        self.target == 1.0
    }

    pub fn animating(&self) -> bool {
        self.started.is_some()
    }

    pub fn transition(&mut self, active: bool, duration: Duration, now: Instant) {
        self.advance(now);
        let target = if active { 1.0 } else { 0.0 };
        if self.target == target {
            return;
        }
        self.start = self.progress;
        self.target = target;
        self.duration = duration.mul_f32((target - self.start).abs());
        self.started = Some(now);
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(started) = self.started else {
            return false;
        };
        let elapsed = now.saturating_duration_since(started);
        if elapsed >= self.duration {
            self.progress = self.target;
            self.started = None;
            return false;
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        self.progress = self.start + (self.target - self.start) * (1.0 - (1.0 - t).powi(3));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interruption_preserves_position_and_stops_at_rest() {
        let now = Instant::now();
        let duration = Duration::from_millis(200);
        let mut animator = Animator::new();
        animator.transition(true, duration, now);
        assert!(animator.advance(now + duration / 2));
        let displayed = animator.progress();
        animator.transition(false, duration, now + duration / 2);
        assert_eq!(animator.progress(), displayed);
        assert!(!animator.advance(now + duration * 2));
        assert_eq!(animator.progress(), 0.0);
        assert!(!animator.advance(now + Duration::from_secs(1)));
        animator.transition(true, Duration::ZERO, now + Duration::from_secs(1));
        assert_eq!(animator.progress(), 1.0);
    }
}
