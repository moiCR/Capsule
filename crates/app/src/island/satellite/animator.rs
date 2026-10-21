use std::time::{Duration, Instant};

use services::CapsuleAnimation;

pub(super) struct SatelliteAnimator {
    progress: f32,
    start: f32,
    target: f32,
    started_at: Option<Instant>,
    duration: Duration,
    delay: Duration,
    animation: CapsuleAnimation,
}

impl SatelliteAnimator {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            start: 0.0,
            target: 0.0,
            started_at: None,
            duration: Duration::ZERO,
            delay: Duration::ZERO,
            animation: CapsuleAnimation::default(),
        }
    }

    pub fn with_animation(mut self, animation: CapsuleAnimation) -> Self {
        self.animation = animation;
        self
    }

    pub fn set_animation(&mut self, animation: CapsuleAnimation) {
        self.animation = animation;
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn open(&self) -> bool {
        self.target == 1.0
    }

    #[allow(dead_code)]
    pub fn animating(&self) -> bool {
        self.started_at.is_some()
    }

    #[allow(dead_code)]
    pub fn transition(&mut self, open: bool, duration: Duration, now: Instant) {
        self.transition_with_delay(open, duration, Duration::ZERO, now);
    }

    pub fn transition_with_delay(
        &mut self,
        open: bool,
        duration: Duration,
        delay: Duration,
        now: Instant,
    ) {
        self.advance(now);
        let target = if open { 1.0 } else { 0.0 };
        if self.target == target {
            return;
        }
        self.start = self.progress;
        self.target = target;
        let effective_delay = if duration.is_zero() {
            Duration::ZERO
        } else {
            delay.min(duration / 2)
        };
        self.delay = effective_delay;
        self.duration = duration
            .saturating_sub(effective_delay)
            .mul_f32((target - self.start).abs());
        self.started_at = Some(now);
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };
        let elapsed = now.saturating_duration_since(started_at);
        if elapsed < self.delay {
            return true;
        }
        let active_elapsed = elapsed - self.delay;
        if active_elapsed >= self.duration {
            self.progress = self.target;
            self.started_at = None;
            self.delay = Duration::ZERO;
            return false;
        }
        let t = active_elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let eased = crate::island::animator::scalar_progress(t, self.animation, 0.75);
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

    #[test]
    fn staggered_delay_waits_and_finishes_on_time() {
        let now = Instant::now();
        let mut animator = SatelliteAnimator::new().with_animation(CapsuleAnimation::Expo);
        animator.transition_with_delay(
            true,
            Duration::from_millis(200),
            Duration::from_millis(40),
            now,
        );
        assert!(animator.advance(now + Duration::from_millis(20)));
        assert_eq!(animator.progress(), 0.0);
        assert!(animator.advance(now + Duration::from_millis(100)));
        assert!(animator.progress() > 0.0);
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.progress(), 1.0);
    }
}
