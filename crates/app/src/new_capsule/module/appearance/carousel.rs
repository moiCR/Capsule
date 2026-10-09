use std::time::{Duration, Instant};

#[derive(Default)]
pub(crate) struct Carousel {
    pub selected: usize,
    pub position: f32,
    start: f32,
    started_at: Option<Instant>,
    duration: Duration,
    pub direction: f32,
    elapsed: Duration,
}

impl Carousel {
    pub fn reset(&mut self, selected: usize, count: usize) {
        self.selected = selected.min(count.saturating_sub(1));
        self.position = self.selected as f32;
        self.started_at = None;
    }

    pub fn navigate(
        &mut self,
        selected: usize,
        count: usize,
        duration: Duration,
        now: Instant,
    ) -> bool {
        self.advance(now);
        let selected = selected.min(count.saturating_sub(1));
        if count == 0 || selected == self.selected {
            return false;
        }
        self.direction = if selected > self.selected { 1.0 } else { -1.0 };
        self.selected = selected;
        self.start = self.position;
        self.duration = duration;
        self.elapsed = Duration::ZERO;
        self.started_at = Some(now);
        self.advance(now);
        true
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(started_at) = self.started_at else {
            return false;
        };
        self.elapsed = now.saturating_duration_since(started_at);
        if self.duration.is_zero() || self.elapsed >= self.duration + Duration::from_millis(72) {
            self.position = self.selected as f32;
            self.started_at = None;
            return false;
        }
        let t = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        self.position =
            self.start + (self.selected as f32 - self.start) * (1.0 - (1.0 - t).powi(3));
        true
    }

    pub fn piano_offset(&self, index: usize) -> f32 {
        if self.started_at.is_none() || self.duration.is_zero() {
            return 0.0;
        }
        let relative = index as f32 - self.selected as f32;
        let order = (relative * self.direction + 2.0).clamp(0.0, 4.0);
        let delay = Duration::from_millis((order * 18.0) as u64);
        if self.elapsed < delay {
            return 0.0;
        }
        let t = ((self.elapsed - delay).as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        (t * std::f32::consts::PI).sin() * 9.0
    }

    pub fn animating(&self) -> bool {
        self.started_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn carousel_has_limits_and_handles_empty_catalogs() {
        let now = Instant::now();
        let mut carousel = Carousel::default();
        assert!(!carousel.navigate(5, 0, Duration::ZERO, now));
        assert!(carousel.navigate(20, 3, Duration::ZERO, now));
        assert_eq!(carousel.selected, 2);
        assert_eq!(carousel.position, 2.0);
        assert!(!carousel.navigate(3, 3, Duration::ZERO, now));
        carousel.reset(10, 1);
        assert_eq!(carousel.selected, 0);
    }
    #[test]
    fn navigation_advances_on_each_monitor_frame_without_skipping_to_the_end() {
        let now = Instant::now();
        let duration = Duration::from_millis(350);
        for rate in [60, 144, 240] {
            let interval = Duration::from_secs_f64(1.0 / rate as f64);
            let mut carousel = Carousel::default();
            carousel.navigate(1, 3, duration, now);
            let mut elapsed = interval;
            let mut previous = 0.0;
            while elapsed < duration {
                assert!(carousel.advance(now + elapsed));
                assert!(carousel.position > previous);
                assert!(carousel.position <= 1.0);
                previous = carousel.position;
                elapsed += interval;
            }
            assert!(!carousel.advance(now + duration + Duration::from_millis(72)));
            assert_eq!(carousel.position, 1.0);
        }
    }

    #[test]
    fn interrupted_piano_navigation_preserves_position_and_stops() {
        let now = Instant::now();
        let duration = Duration::from_millis(220);
        let mut carousel = Carousel::default();
        carousel.navigate(1, 4, duration, now);
        let halfway = now + duration / 2;
        assert!(carousel.advance(halfway));
        let displayed = carousel.position;
        assert_ne!(carousel.piano_offset(0), carousel.piano_offset(2));
        carousel.navigate(0, 4, duration, halfway);
        assert_eq!(carousel.position, displayed);
        assert!(!carousel.advance(halfway + duration + Duration::from_millis(72)));
        assert_eq!(carousel.position, 0.0);
        assert_eq!(carousel.piano_offset(0), 0.0);
    }
}
