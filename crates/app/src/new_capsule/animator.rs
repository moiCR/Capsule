use std::time::{Duration, Instant};

use gpui::{Pixels, Size, px};

use crate::capsule::apple_island_morph;

pub struct Animator {
    current: Size<Pixels>,
    start: Size<Pixels>,
    target: Size<Pixels>,
    started_at: Option<Instant>,
    duration: Duration,
    content_started_at: Option<Instant>,
    content_opacity: f32,
}

impl Animator {
    pub fn new(size: Size<Pixels>, duration: Duration) -> Self {
        Self {
            current: size,
            start: size,
            target: size,
            started_at: None,
            duration,
            content_started_at: None,
            content_opacity: 1.0,
        }
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub fn size(&self) -> Size<Pixels> {
        self.current
    }

    pub fn transition_to(&mut self, target: Size<Pixels>, now: Instant) {
        if self.target == target {
            return;
        }
        self.start = self.current;
        self.target = target;
        self.started_at = Some(now);
        self.advance(now);
    }

    pub fn content_opacity(&self) -> f32 {
        self.content_opacity
    }

    pub fn reveal_content(&mut self, now: Instant) {
        self.content_started_at = Some(now);
        self.content_opacity = 0.0;
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let size_running = self.advance_size(now);
        let content_running = if let Some(started_at) = self.content_started_at {
            let elapsed = now.saturating_duration_since(started_at);
            if elapsed >= self.duration {
                self.content_opacity = 1.0;
                self.content_started_at = None;
                false
            } else {
                let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
                self.content_opacity = progress * progress * (3.0 - 2.0 * progress);
                true
            }
        } else {
            false
        };
        size_running || content_running
    }

    fn advance_size(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };
        let elapsed = now.saturating_duration_since(started_at);
        if elapsed >= self.duration {
            self.current = self.target;
            self.started_at = None;
            return false;
        }
        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        let expanding =
            self.target.width > self.start.width || self.target.height > self.start.height;

        let (width_eased, height_eased) = apple_island_morph(progress, expanding);

        self.current = Size {
            width: (self.start.width + (self.target.width - self.start.width) * width_eased)
                .max(px(1.0)),
            height: (self.start.height + (self.target.height - self.start.height) * height_eased)
                .max(px(1.0)),
        };
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::px;

    fn size(value: f32) -> Size<Pixels> {
        Size {
            width: px(value),
            height: px(value),
        }
    }

    #[test]
    fn interrupted_transition_preserves_displayed_size() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200));
        animator.transition_to(size(300.0), now);
        assert!(animator.advance(now + Duration::from_millis(100)));
        let displayed = animator.size();
        animator.transition_to(size(50.0), now + Duration::from_millis(110));
        assert_eq!(animator.size(), displayed);
        assert!(!animator.advance(now + Duration::from_millis(310)));
        assert_eq!(animator.size(), size(50.0));
    }

    #[test]
    fn expansion_and_contraction_match_the_original_spring() {
        let now = Instant::now();
        for (start, target, expanding) in [(100.0, 300.0, true), (300.0, 100.0, false)] {
            let mut animator = Animator::new(size(start), Duration::from_millis(200));
            animator.transition_to(size(target), now);
            for millis in [25, 75, 125, 175] {
                assert!(animator.advance(now + Duration::from_millis(millis)));
                let (width, height) = apple_island_morph(millis as f32 / 200.0, expanding);
                assert_eq!(animator.size().width, px(start + (target - start) * width));
                assert_eq!(
                    animator.size().height,
                    px(start + (target - start) * height)
                );
            }
            assert!(!animator.advance(now + Duration::from_millis(200)));
            assert_eq!(animator.size(), size(target));
        }
    }

    #[test]
    fn content_reveals_even_when_dimensions_do_not_change() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200));
        animator.transition_to(size(100.0), now);
        animator.reveal_content(now);
        assert_eq!(animator.content_opacity(), 0.0);
        assert!(animator.advance(now + Duration::from_millis(100)));
        assert_eq!(animator.content_opacity(), 0.5);
        assert_eq!(animator.size(), size(100.0));
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.content_opacity(), 1.0);
        assert!(!animator.advance(now + Duration::from_secs(1)));
    }

    #[test]
    fn replacing_content_during_a_resize_preserves_the_current_dimensions() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200));
        animator.transition_to(size(300.0), now);
        animator.reveal_content(now);
        animator.advance(now + Duration::from_millis(100));
        let displayed = animator.size();
        animator.reveal_content(now + Duration::from_millis(100));
        assert_eq!(animator.size(), displayed);
        assert!(animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.size(), size(300.0));
        assert!(!animator.advance(now + Duration::from_millis(300)));
        assert_eq!(animator.content_opacity(), 1.0);
    }

    #[test]
    fn zero_duration_finishes_immediately() {
        let mut animator = Animator::new(size(100.0), Duration::ZERO);
        animator.transition_to(size(300.0), Instant::now());
        assert_eq!(animator.size(), size(300.0));
        animator.reveal_content(Instant::now());
        assert_eq!(animator.content_opacity(), 1.0);
        assert!(!animator.advance(Instant::now()));
    }
}
