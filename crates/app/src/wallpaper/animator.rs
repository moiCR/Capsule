use services::WallpaperTransition;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct WallpaperAnimator {
    pub current: Option<PathBuf>,
    pub previous: Option<PathBuf>,
    pub completed_previous: Option<PathBuf>,
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
            completed_previous: None,
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
            self.completed_previous = self.previous.take().or_else(|| self.current.take());
            self.current = Some(path);
            self.previous = None;
            self.transition = transition;
            self.progress = 1.0;
            self.animating = false;
            return;
        }

        if let Some(displaced) = self.previous.take() {
            self.completed_previous = Some(displaced);
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
            self.completed_previous = self.previous.take();
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
            self.completed_previous = self.previous.take();
            false
        } else {
            true
        }
    }

    pub fn take_completed_previous(&mut self) -> Option<PathBuf> {
        self.completed_previous.take()
    }
}
