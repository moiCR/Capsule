use std::time::{Duration, Instant};

use gpui::{Pixels, Size, px};
use services::CapsuleAnimation;

use crate::animator::Animator;

pub struct LauncherAnimator {
    pub animator: Animator,
    duration: Duration,
    animation: CapsuleAnimation,
    bounciness: f32,
    entry_start: Instant,
    close_start: Option<Instant>,
    close_offset_y: f32,
    close_opacity: f32,

    pub offset_y: f32,
    pub opacity: f32,
    pub backdrop_opacity: f32,
}

impl LauncherAnimator {
    pub fn new(
        duration: Duration,
        animation: CapsuleAnimation,
        bounciness: f32,
        squash_stretch: bool,
        reduced_motion: bool,
    ) -> Self {
        let initial_size = Size {
            width: px(640.0),
            height: px(54.0),
        };
        let animator = Animator::new(initial_size, duration)
            .with_animation(animation)
            .with_spring_bounciness(bounciness)
            .with_squash_stretch(squash_stretch)
            .with_reduced_motion(reduced_motion);

        Self {
            animator,
            duration,
            animation,
            bounciness,
            entry_start: Instant::now(),
            close_start: None,
            close_offset_y: -18.0,
            close_opacity: 0.0,
            offset_y: -18.0,
            opacity: 0.0,
            backdrop_opacity: 0.0,
        }
    }

    pub fn request_close(&mut self) {
        if self.close_start.is_none() {
            self.close_offset_y = self.offset_y;
            self.close_opacity = self.opacity;
            self.close_start = Some(Instant::now());
        }
    }

    pub fn is_closing(&self) -> bool {
        self.close_start.is_some()
    }

    pub fn transition_to(&mut self, target: Size<Pixels>, now: Instant) {
        if self.animator.target() == target {
            return;
        }
        let expanding = target.width > self.animator.size().width
            || target.height > self.animator.size().height;
        self.animator.transition_to(target, now);
        if expanding {
            self.animator.reveal_content(now);
        }
    }

    pub fn size(&self) -> Size<Pixels> {
        self.animator.size()
    }

    pub fn width(&self) -> Pixels {
        self.animator.size().width
    }

    pub fn height(&self) -> Pixels {
        self.animator.size().height
    }

    pub fn corner_radius(&self) -> Pixels {
        px(16.0)
    }

    pub fn expand_progress(&self) -> f32 {
        ((f32::from(self.animator.size().height) - 54.0) / 40.0).clamp(0.0, 1.0)
    }

    pub fn cascade_opacity(&self) -> f32 {
        let progress = self.expand_progress();
        if progress <= 0.05 {
            0.0
        } else {
            let reveal = ((progress - 0.05) / 0.95).clamp(0.0, 1.0);
            reveal * self.animator.content_progress()
        }
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        let mut running = self.animator.advance(now);
        let mut closed = false;

        if let Some(close_start) = self.close_start {
            let close_duration = self.duration.mul_f32(0.65).max(Duration::from_millis(50));
            let elapsed = now.saturating_duration_since(close_start).as_secs_f32();
            let total = close_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = self.close_offset_y - 12.0;
                self.opacity = 0.0;
                self.backdrop_opacity = 0.0;
                closed = true;
            } else {
                let t = (elapsed / total).clamp(0.0, 1.0);
                let progress = t.powi(2);
                self.offset_y = self.close_offset_y - 12.0 * progress;
                self.opacity = self.close_opacity * (1.0 - progress);
                self.backdrop_opacity = 0.40 * (1.0 - progress);
                running = true;
            }
        } else {
            let entry_duration = self.duration.mul_f32(0.85).max(Duration::from_millis(50));
            let elapsed = now
                .saturating_duration_since(self.entry_start)
                .as_secs_f32();
            let total = entry_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = 0.0;
                self.opacity = 1.0;
                self.backdrop_opacity = 0.40;
            } else {
                let t = (elapsed / total).clamp(0.0, 1.0);
                let eased = crate::animator::scalar_progress(t, self.animation, self.bounciness);
                self.offset_y = -18.0 * (1.0 - eased);
                self.opacity = eased;
                self.backdrop_opacity = 0.40 * eased;
                running = true;
            }
        }

        (running, closed)
    }
}

pub struct PolkitAnimator {
    pub animator: Animator,
    duration: Duration,
    animation: CapsuleAnimation,
    bounciness: f32,
    entry_start: Instant,
    close_start: Option<Instant>,
    close_offset_y: f32,
    close_opacity: f32,

    pub offset_y: f32,
    pub opacity: f32,
    pub backdrop_opacity: f32,
}

impl PolkitAnimator {
    pub fn new(
        target_size: Size<Pixels>,
        duration: Duration,
        animation: CapsuleAnimation,
        bounciness: f32,
        squash_stretch: bool,
        reduced_motion: bool,
    ) -> Self {
        let animator = Animator::new(target_size, duration)
            .with_animation(animation)
            .with_spring_bounciness(bounciness)
            .with_squash_stretch(squash_stretch)
            .with_reduced_motion(reduced_motion);

        Self {
            animator,
            duration,
            animation,
            bounciness,
            entry_start: Instant::now(),
            close_start: None,
            close_offset_y: -18.0,
            close_opacity: 0.0,
            offset_y: -18.0,
            opacity: 0.0,
            backdrop_opacity: 0.0,
        }
    }

    pub fn request_close(&mut self) {
        if self.close_start.is_none() {
            self.close_offset_y = self.offset_y;
            self.close_opacity = self.opacity;
            self.close_start = Some(Instant::now());
        }
    }

    pub fn is_closing(&self) -> bool {
        self.close_start.is_some()
    }

    pub fn transition_to(&mut self, target: Size<Pixels>, now: Instant) {
        if self.animator.target() == target {
            return;
        }
        self.animator.transition_to(target, now);
    }

    pub fn size(&self) -> Size<Pixels> {
        self.animator.size()
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        let mut running = self.animator.advance(now);
        let mut closed = false;

        if let Some(close_start) = self.close_start {
            let close_duration = self.duration.mul_f32(0.65).max(Duration::from_millis(50));
            let elapsed = now.saturating_duration_since(close_start).as_secs_f32();
            let total = close_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = self.close_offset_y - 12.0;
                self.opacity = 0.0;
                self.backdrop_opacity = 0.0;
                closed = true;
            } else {
                let t = (elapsed / total).clamp(0.0, 1.0);
                let progress = t.powi(2);
                self.offset_y = self.close_offset_y - 12.0 * progress;
                self.opacity = self.close_opacity * (1.0 - progress);
                self.backdrop_opacity = 0.40 * (1.0 - progress);
                running = true;
            }
        } else {
            let entry_duration = self.duration.mul_f32(0.85).max(Duration::from_millis(50));
            let elapsed = now
                .saturating_duration_since(self.entry_start)
                .as_secs_f32();
            let total = entry_duration.as_secs_f32();
            if elapsed >= total {
                self.offset_y = 0.0;
                self.opacity = 1.0;
                self.backdrop_opacity = 0.40;
            } else {
                let t = (elapsed / total).clamp(0.0, 1.0);
                let eased = crate::animator::scalar_progress(t, self.animation, self.bounciness);
                self.offset_y = -18.0 * (1.0 - eased);
                self.opacity = eased;
                self.backdrop_opacity = 0.40 * eased;
                running = true;
            }
        }

        (running, closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launcher_animator_initializes_at_resting_bubble_size() {
        let animator = LauncherAnimator::new(
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        assert_eq!(animator.width(), px(640.0));
        assert_eq!(animator.height(), px(54.0));
        assert_eq!(animator.corner_radius(), px(16.0));
        assert_eq!(animator.cascade_opacity(), 0.0);
    }

    #[test]
    fn launcher_animator_morphs_size_fluidly() {
        let now = Instant::now();
        let mut animator = LauncherAnimator::new(
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        let target = Size {
            width: px(640.0),
            height: px(400.0),
        };
        animator.transition_to(target, now);

        let (running, _) = animator.advance(now + Duration::from_millis(100));
        assert!(running);
        assert_eq!(animator.width(), px(640.0));
        assert!(animator.height() > px(54.0) && animator.height() < px(400.0));
        assert!(animator.cascade_opacity() > 0.0);

        let (_, _) = animator.advance(now + Duration::from_millis(210));
        assert_eq!(animator.width(), px(640.0));
        assert_eq!(animator.height(), px(400.0));
        assert_eq!(animator.corner_radius(), px(16.0));
    }

    #[test]
    fn launcher_animator_morphs_back_to_bubble_when_cleared() {
        let now = Instant::now();
        let mut animator = LauncherAnimator::new(
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        let expanded = Size {
            width: px(640.0),
            height: px(400.0),
        };
        animator.transition_to(expanded, now);
        animator.advance(now + Duration::from_millis(210));
        assert_eq!(animator.width(), px(640.0));

        let collapsed = Size {
            width: px(640.0),
            height: px(54.0),
        };
        let t2 = now + Duration::from_millis(300);
        animator.transition_to(collapsed, t2);
        animator.advance(t2 + Duration::from_millis(100));
        assert!(animator.height() < px(400.0) && animator.height() > px(54.0));
        animator.advance(t2 + Duration::from_millis(210));
        assert_eq!(animator.width(), px(640.0));
        assert_eq!(animator.height(), px(54.0));
        assert_eq!(animator.corner_radius(), px(16.0));
    }
}
