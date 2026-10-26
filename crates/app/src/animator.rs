pub use std::time::{Duration, Instant};

pub use gpui::{Pixels, Size, px};
pub use services::{CapsuleAnimation, ContentTransition};

pub fn spring_eval(t: f32, zeta: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let wd = 2.0 * std::f32::consts::PI;
    let omega0 = wd / (1.0 - zeta * zeta).sqrt();
    let beta = zeta * omega0;
    let scale = 1.0 / (1.0 - (-beta).exp());
    let envelope = (-beta * t).exp();
    let osc = (wd * t).cos() + (beta / wd) * (wd * t).sin();
    (1.0 - envelope * osc) * scale
}

pub fn apple_island_morph(t: f32, expanding: bool) -> (f32, f32) {
    apple_island_morph_with_bounciness(t, expanding, 0.75)
}

pub fn apple_island_morph_with_bounciness(t: f32, expanding: bool, bounciness: f32) -> (f32, f32) {
    if t <= 0.0 {
        return (0.0, 0.0);
    }
    if t >= 1.0 {
        return (1.0, 1.0);
    }
    let b = bounciness.clamp(0.0, 1.0);
    let zeta_w = 0.98 - b * 0.33;
    let zeta_h = 0.98 - b * 0.28;
    let (uw, uh) = if expanding {
        (t.powf(0.92), t.powf(1.04))
    } else {
        (t.powf(1.04), t.powf(0.92))
    };
    (spring_eval(uw, zeta_w), spring_eval(uh, zeta_h))
}

pub fn scalar_progress(t: f32, animation: CapsuleAnimation, bounciness: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    match animation {
        CapsuleAnimation::Fluid => 1.0 - (1.0 - t).powi(5),
        CapsuleAnimation::Cubic => 1.0 - (1.0 - t).powi(3),
        CapsuleAnimation::Expo => {
            let norm = 1.0 - 2.0_f32.powf(-10.0);
            (1.0 - 2.0_f32.powf(-10.0 * t)) / norm
        }
        CapsuleAnimation::Back => {
            let c1 = 1.15;
            let c3 = c1 + 1.0;
            let u = t - 1.0;
            1.0 + c3 * u.powi(3) + c1 * u.powi(2)
        }
        CapsuleAnimation::Spring => apple_island_morph_with_bounciness(t, true, bounciness).0,
    }
}

pub fn morph_progress(
    t: f32,
    expanding: bool,
    animation: CapsuleAnimation,
    bounciness: f32,
) -> (f32, f32) {
    if t <= 0.0 {
        return (0.0, 0.0);
    }
    if t >= 1.0 {
        return (1.0, 1.0);
    }
    match animation {
        CapsuleAnimation::Spring => apple_island_morph_with_bounciness(t, expanding, bounciness),
        _ => {
            let eased = scalar_progress(t, animation, bounciness);
            (eased, eased)
        }
    }
}

pub fn apply_squash_stretch(
    start: Size<Pixels>,
    target: Size<Pixels>,
    width: Pixels,
    height: Pixels,
    t: f32,
) -> Size<Pixels> {
    if t <= 0.0 || t >= 1.0 {
        return Size { width, height };
    }
    let dw = f32::from(target.width - start.width);
    let dh = f32::from(target.height - start.height);
    if dw.abs() < 1.0 && dh.abs() < 1.0 {
        return Size { width, height };
    }
    let norm = (dw * dw + dh * dh).sqrt();
    let norm_w = (dw / norm).abs();
    let norm_h = (dh / norm).abs();
    let pulse = (std::f32::consts::PI * t).sin();
    let scale_factor = 0.04 * pulse;
    let factor_w = 1.0 + scale_factor * norm_w - scale_factor * 0.5 * norm_h;
    let factor_h = 1.0 + scale_factor * norm_h - scale_factor * 0.5 * norm_w;
    Size {
        width: width * factor_w,
        height: height * factor_h,
    }
}

pub struct Animator {
    current: Size<Pixels>,
    start: Size<Pixels>,
    target: Size<Pixels>,
    started_at: Option<Instant>,
    duration: Duration,
    delay: Duration,
    animation: CapsuleAnimation,
    content_transition: ContentTransition,
    spring_bounciness: f32,
    squash_stretch: bool,
    reduced_motion: bool,
    content_started_at: Option<Instant>,
    content_progress: f32,
}

impl Animator {
    pub fn new(size: Size<Pixels>, duration: Duration) -> Self {
        Self {
            current: size,
            start: size,
            target: size,
            started_at: None,
            duration,
            delay: Duration::ZERO,
            animation: CapsuleAnimation::default(),
            content_transition: ContentTransition::default(),
            spring_bounciness: 0.75,
            squash_stretch: false,
            reduced_motion: false,
            content_started_at: None,
            content_progress: 1.0,
        }
    }

    pub fn with_animation(mut self, animation: CapsuleAnimation) -> Self {
        self.animation = animation;
        self
    }

    pub fn with_content_transition(mut self, transition: ContentTransition) -> Self {
        self.content_transition = transition;
        self
    }

    pub fn with_spring_bounciness(mut self, bounciness: f32) -> Self {
        self.spring_bounciness = bounciness;
        self
    }

    pub fn with_squash_stretch(mut self, squash_stretch: bool) -> Self {
        self.squash_stretch = squash_stretch;
        self
    }

    pub fn with_reduced_motion(mut self, reduced_motion: bool) -> Self {
        self.reduced_motion = reduced_motion;
        self
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    pub fn set_animation(&mut self, animation: CapsuleAnimation) {
        self.animation = animation;
    }

    pub fn animation(&self) -> CapsuleAnimation {
        self.animation
    }

    pub fn set_content_transition(&mut self, transition: ContentTransition) {
        self.content_transition = transition;
    }

    #[allow(dead_code)]
    pub fn content_transition(&self) -> ContentTransition {
        self.content_transition
    }

    pub fn set_spring_bounciness(&mut self, bounciness: f32) {
        self.spring_bounciness = bounciness;
    }

    #[allow(dead_code)]
    pub fn spring_bounciness(&self) -> f32 {
        self.spring_bounciness
    }

    pub fn set_squash_stretch(&mut self, squash_stretch: bool) {
        self.squash_stretch = squash_stretch;
    }

    #[allow(dead_code)]
    pub fn squash_stretch(&self) -> bool {
        self.squash_stretch
    }

    pub fn set_reduced_motion(&mut self, reduced_motion: bool) {
        self.reduced_motion = reduced_motion;
    }

    #[allow(dead_code)]
    pub fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    pub fn animating(&self) -> bool {
        self.started_at.is_some() || self.content_started_at.is_some()
    }

    pub fn size(&self) -> Size<Pixels> {
        self.current
    }

    pub fn target(&self) -> Size<Pixels> {
        self.target
    }

    pub fn transition_to(&mut self, target: Size<Pixels>, now: Instant) {
        self.transition_to_with_delay(target, Duration::ZERO, now);
    }

    pub fn transition_to_with_delay(
        &mut self,
        target: Size<Pixels>,
        delay: Duration,
        now: Instant,
    ) {
        if self.target == target {
            return;
        }
        self.start = self.current;
        self.target = target;
        self.delay = if self.duration.is_zero() || self.reduced_motion {
            Duration::ZERO
        } else {
            delay.min(self.duration / 2)
        };
        self.started_at = Some(now);
        self.advance(now);
    }

    pub fn content_progress(&self) -> f32 {
        self.content_progress
    }

    pub fn content_scale(&self) -> f32 {
        if self.reduced_motion {
            return 1.0;
        }
        match self.content_transition {
            ContentTransition::FadeScale => 0.98 + self.content_progress * 0.02,
            ContentTransition::Fade => 1.0,
        }
    }

    pub fn reveal_content(&mut self, now: Instant) {
        self.content_started_at = Some(now);
        self.content_progress = 0.0;
        self.advance(now);
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let size_running = self.advance_size(now);
        let content_running = if let Some(started_at) = self.content_started_at {
            let elapsed = now.saturating_duration_since(started_at);
            if elapsed < self.delay {
                true
            } else {
                let active_elapsed = elapsed - self.delay;
                let max_cap = if self.reduced_motion {
                    Duration::from_millis(50)
                } else {
                    Duration::from_millis(160)
                };
                let duration = self.duration.saturating_sub(self.delay).min(max_cap);
                if active_elapsed >= duration {
                    self.content_progress = 1.0;
                    self.content_started_at = None;
                    false
                } else {
                    let progress = active_elapsed.as_secs_f32() / duration.as_secs_f32();
                    self.content_progress = 1.0 - (1.0 - progress).powi(3);
                    true
                }
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
        if self.reduced_motion || self.duration.is_zero() {
            self.current = self.target;
            self.started_at = None;
            self.delay = Duration::ZERO;
            return false;
        }
        let elapsed = now.saturating_duration_since(started_at);
        if elapsed < self.delay {
            return true;
        }
        let active_duration = self.duration.saturating_sub(self.delay);
        let active_elapsed = elapsed - self.delay;
        if active_elapsed >= active_duration {
            self.current = self.target;
            self.started_at = None;
            self.delay = Duration::ZERO;
            return false;
        }
        let progress = active_elapsed.as_secs_f32() / active_duration.as_secs_f32();
        let expanding =
            self.target.width > self.start.width || self.target.height > self.start.height;

        let (width_eased, height_eased) =
            morph_progress(progress, expanding, self.animation, self.spring_bounciness);

        let raw_width =
            (self.start.width + (self.target.width - self.start.width) * width_eased).max(px(1.0));
        let raw_height = (self.start.height
            + (self.target.height - self.start.height) * height_eased)
            .max(px(1.0));

        self.current = if self.squash_stretch {
            apply_squash_stretch(self.start, self.target, raw_width, raw_height, progress)
        } else {
            Size {
                width: raw_width,
                height: raw_height,
            }
        };
        true
    }
}
