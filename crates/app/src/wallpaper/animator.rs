use services::WallpaperTransition;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct WallpaperAnimator {
    pub current: Option<PathBuf>,
    pub previous: Option<PathBuf>,
    pub transition: WallpaperTransition,
    pub duration: Duration,
    pub start_time: Instant,
    pub progress: f32,
    pub animating: bool,
}

impl WallpaperAnimator {
    pub fn new(initial: Option<PathBuf>) -> Self {
        Self {
            current: initial,
            previous: None,
            transition: WallpaperTransition::Fade,
            duration: Duration::from_millis(600),
            start_time: Instant::now(),
            progress: 1.0,
            animating: false,
        }
    }

    pub fn set_wallpaper(
        &mut self,
        path: PathBuf,
        transition: WallpaperTransition,
        duration: Duration,
        now: Instant,
    ) {
        if self.current.as_ref() == Some(&path) {
            return;
        }

        if self.current.is_none() || transition == WallpaperTransition::None || duration.is_zero() {
            self.current = Some(path);
            self.previous = None;
            self.transition = transition;
            self.progress = 1.0;
            self.animating = false;
            return;
        }

        self.previous = self.current.take();
        self.current = Some(path);
        self.transition = transition;
        self.duration = duration;
        self.start_time = now;
        self.progress = 0.0;
        self.animating = true;
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        if !self.animating {
            return false;
        }

        if self.duration.is_zero() || self.transition == WallpaperTransition::None {
            self.progress = 1.0;
            self.animating = false;
            self.previous = None;
            return false;
        }

        let elapsed = now.saturating_duration_since(self.start_time);
        let linear = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);

        // Smooth cubic ease-out curve
        let inv = 1.0 - linear;
        self.progress = 1.0 - inv * inv * inv;

        if linear >= 1.0 {
            self.progress = 1.0;
            self.animating = false;
            self.previous = None;
            false
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_is_not_animating() {
        let animator = WallpaperAnimator::new(Some(PathBuf::from("/tmp/bg.jpg")));
        assert!(!animator.animating);
        assert_eq!(animator.progress, 1.0);
        assert!(animator.previous.is_none());
    }

    #[test]
    fn transition_advances_and_drops_previous_upon_completion() {
        let now = Instant::now();
        let mut animator = WallpaperAnimator::new(Some(PathBuf::from("/tmp/first.jpg")));
        animator.set_wallpaper(
            PathBuf::from("/tmp/second.jpg"),
            WallpaperTransition::Fade,
            Duration::from_millis(500),
            now,
        );

        assert!(animator.animating);
        assert_eq!(animator.previous, Some(PathBuf::from("/tmp/first.jpg")));
        assert_eq!(animator.current, Some(PathBuf::from("/tmp/second.jpg")));
        assert_eq!(animator.progress, 0.0);

        // Advance midway
        assert!(animator.advance(now + Duration::from_millis(250)));
        assert!(animator.progress > 0.0 && animator.progress < 1.0);
        assert!(animator.previous.is_some());

        // Advance to finish
        assert!(!animator.advance(now + Duration::from_millis(500)));
        assert_eq!(animator.progress, 1.0);
        assert!(!animator.animating);
        assert!(animator.previous.is_none());
    }

    #[test]
    fn none_transition_finishes_instantly() {
        let now = Instant::now();
        let mut animator = WallpaperAnimator::new(Some(PathBuf::from("/tmp/first.jpg")));
        animator.set_wallpaper(
            PathBuf::from("/tmp/second.jpg"),
            WallpaperTransition::None,
            Duration::from_millis(500),
            now,
        );

        assert!(!animator.animating);
        assert_eq!(animator.progress, 1.0);
        assert!(animator.previous.is_none());
    }
}
