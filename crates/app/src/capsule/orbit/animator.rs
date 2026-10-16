use std::time::{Duration, Instant};

use services::CapsuleAnimation;

pub(super) struct Animator {
    progress: f32,
    start: f32,
    target: f32,
    started: Option<Instant>,
    duration: Duration,
    delay: Duration,
    animation: CapsuleAnimation,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            start: 0.0,
            target: 0.0,
            started: None,
            duration: Duration::ZERO,
            delay: Duration::ZERO,
            animation: CapsuleAnimation::default(),
        }
    }

    #[allow(dead_code)]
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

    pub fn active(&self) -> bool {
        self.target == 1.0
    }

    pub fn animating(&self) -> bool {
        self.started.is_some()
    }

    #[allow(dead_code)]
    pub fn transition(&mut self, active: bool, duration: Duration, now: Instant) {
        self.transition_with_delay(active, duration, Duration::ZERO, now);
    }

    pub fn transition_with_delay(
        &mut self,
        active: bool,
        duration: Duration,
        delay: Duration,
        now: Instant,
    ) {
        self.advance(now);
        let target = if active { 1.0 } else { 0.0 };
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
        self.started = Some(now);
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(started) = self.started else {
            return false;
        };
        let elapsed = now.saturating_duration_since(started);
        if elapsed < self.delay {
            return true;
        }
        let active_elapsed = elapsed - self.delay;
        if active_elapsed >= self.duration {
            self.progress = self.target;
            self.started = None;
            self.delay = Duration::ZERO;
            return false;
        }
        let t = active_elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let eased = crate::capsule::animator::scalar_progress(t, self.animation, 0.75);
        self.progress = self.start + (self.target - self.start) * eased;
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

    #[test]
    fn delayed_orbit_transition_waits_and_finishes_within_duration() {
        let now = Instant::now();
        let mut animator = Animator::new().with_animation(CapsuleAnimation::Back);
        animator.transition_with_delay(
            true,
            Duration::from_millis(200),
            Duration::from_millis(30),
            now,
        );
        assert!(animator.advance(now + Duration::from_millis(15)));
        assert_eq!(animator.progress(), 0.0);
        assert!(animator.advance(now + Duration::from_millis(100)));
        assert!(animator.progress() > 0.0);
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.progress(), 1.0);
    }
}
