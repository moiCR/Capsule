use std::time::{Duration, Instant};

use gpui::{Pixels, Size, px};
use services::{CapsuleAnimation, ContentTransition};

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

fn spring_eval(t: f32, zeta: f32) -> f32 {
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
    let bell = (16.0 * t * (1.0 - t).powi(3)) / 1.6875;
    let strain_x = (dw / f32::from(start.width).max(40.0)).clamp(-1.5, 1.5);
    let strain_y = (dh / f32::from(start.height).max(20.0)).clamp(-1.5, 1.5);
    let deform = ((strain_x - strain_y * 0.5) / 1.5).clamp(-1.0, 1.0) * 0.028 * bell;
    let w = (f32::from(width) * (1.0 + deform * 0.5)).max(1.0);
    let h = (f32::from(height) * (1.0 - deform)).max(1.0);
    Size {
        width: px(w),
        height: px(h),
    }
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

    pub fn transition_to(&mut self, target: Size<Pixels>, now: Instant) {
        self.transition_to_with_delay(target, Duration::ZERO, now);
    }

    pub fn transition_to_with_delay(&mut self, target: Size<Pixels>, delay: Duration, now: Instant) {
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

        let (width_eased, height_eased) = morph_progress(
            progress,
            expanding,
            self.animation,
            self.spring_bounciness,
        );

        let raw_width = (self.start.width + (self.target.width - self.start.width) * width_eased)
            .max(px(1.0));
        let raw_height =
            (self.start.height + (self.target.height - self.start.height) * height_eased)
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
            let mut animator = Animator::new(size(start), Duration::from_millis(200))
                .with_animation(CapsuleAnimation::Spring);
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
    fn fluid_and_cubic_animations_ease_smoothly_without_overshoot() {
        let now = Instant::now();
        let mut fluid = Animator::new(size(100.0), Duration::from_millis(200));
        assert_eq!(fluid.animation(), CapsuleAnimation::Fluid);
        fluid.transition_to(size(300.0), now);
        for millis in [25, 75, 125, 175] {
            assert!(fluid.advance(now + Duration::from_millis(millis)));
            let t = millis as f32 / 200.0;
            let expected = 100.0 + 200.0 * (1.0 - (1.0 - t).powi(5));
            assert_eq!(fluid.size().width, px(expected));
            assert_eq!(fluid.size().height, px(expected));
            assert!(fluid.size().width <= px(300.0));
        }

        let mut cubic = Animator::new(size(100.0), Duration::from_millis(200))
            .with_animation(CapsuleAnimation::Cubic);
        cubic.transition_to(size(300.0), now);
        for millis in [25, 75, 125, 175] {
            assert!(cubic.advance(now + Duration::from_millis(millis)));
            let t = millis as f32 / 200.0;
            let expected = 100.0 + 200.0 * (1.0 - (1.0 - t).powi(3));
            assert_eq!(cubic.size().width, px(expected));
            assert_eq!(cubic.size().height, px(expected));
            assert!(cubic.size().width <= px(300.0));
        }
    }

    #[test]
    fn content_reveals_even_when_dimensions_do_not_change() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200));
        animator.transition_to(size(100.0), now);
        animator.reveal_content(now);
        assert_eq!(animator.content_scale(), 0.98);
        assert!(animator.advance(now + Duration::from_millis(80)));
        assert!((animator.content_scale() - 0.9975).abs() < f32::EPSILON);
        assert_eq!(animator.size(), size(100.0));
        assert!(!animator.advance(now + Duration::from_millis(160)));
        assert_eq!(animator.content_scale(), 1.0);
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
        assert_eq!(animator.content_scale(), 1.0);
    }

    #[test]
    fn rapid_content_changes_restart_the_reveal_and_settle() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(250));
        animator.reveal_content(now);
        for millis in [40, 80, 120, 160, 200] {
            let switched_at = now + Duration::from_millis(millis);
            assert!(animator.advance(switched_at));
            assert!(animator.content_progress() > 0.0);
            animator.reveal_content(switched_at);
            assert_eq!(animator.content_progress(), 0.0);
        }
        assert!(animator.advance(now + Duration::from_millis(280)));
        assert!(!animator.advance(now + Duration::from_millis(360)));
        assert_eq!(animator.content_progress(), 1.0);
        assert!(!animator.advance(now + Duration::from_secs(1)));
    }

    #[test]
    fn fade_only_transition_maintains_identity_scale() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200))
            .with_content_transition(ContentTransition::Fade);
        animator.reveal_content(now);
        assert_eq!(animator.content_scale(), 1.0);
        assert!(animator.advance(now + Duration::from_millis(80)));
        assert_eq!(animator.content_scale(), 1.0);
        assert!(!animator.advance(now + Duration::from_millis(160)));
        assert_eq!(animator.content_scale(), 1.0);
    }

    #[test]
    fn spring_bounciness_adjusts_damping() {
        let (w_low, _) = apple_island_morph_with_bounciness(0.7, true, 0.0);
        let (w_high, _) = apple_island_morph_with_bounciness(0.7, true, 1.0);
        assert!(w_high > w_low);
    }

    #[test]
    fn zero_duration_finishes_immediately() {
        let mut animator = Animator::new(size(100.0), Duration::ZERO);
        animator.transition_to(size(300.0), Instant::now());
        assert_eq!(animator.size(), size(300.0));
        animator.reveal_content(Instant::now());
        assert_eq!(animator.content_scale(), 1.0);
        assert!(!animator.advance(Instant::now()));
    }

    #[test]
    fn expo_and_back_curves_behave_as_expected() {
        let now = Instant::now();
        let mut expo = Animator::new(size(100.0), Duration::from_millis(200))
            .with_animation(CapsuleAnimation::Expo);
        expo.transition_to(size(300.0), now);
        assert!(expo.advance(now + Duration::from_millis(50)));
        assert!(expo.size().width > px(250.0) && expo.size().width <= px(300.0));
        assert!(!expo.advance(now + Duration::from_millis(200)));
        assert_eq!(expo.size(), size(300.0));

        let mut back = Animator::new(size(100.0), Duration::from_millis(200))
            .with_animation(CapsuleAnimation::Back);
        back.transition_to(size(300.0), now);
        assert!(back.advance(now + Duration::from_millis(130)));
        assert!(back.size().width > px(300.0));
        assert!(!back.advance(now + Duration::from_millis(200)));
        assert_eq!(back.size(), size(300.0));
    }

    #[test]
    fn squash_stretch_deforms_mid_flight_and_settles_at_exact_target() {
        let now = Instant::now();
        let start = Size {
            width: px(100.0),
            height: px(40.0),
        };
        let target = Size {
            width: px(300.0),
            height: px(40.0),
        };
        let mut animator = Animator::new(start, Duration::from_millis(200))
            .with_squash_stretch(true);
        animator.transition_to(target, now);
        assert!(animator.advance(now + Duration::from_millis(50)));
        assert!(animator.size().height < px(40.0));
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.size(), target);
    }

    #[test]
    fn reduced_motion_skips_spatial_morph_and_shortens_fade() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(300))
            .with_reduced_motion(true);
        animator.transition_to(size(300.0), now);
        assert_eq!(animator.size(), size(300.0));
        assert!(!animator.animating());

        animator.reveal_content(now);
        assert_eq!(animator.content_scale(), 1.0);
        assert!(animator.advance(now + Duration::from_millis(25)));
        assert!(!animator.advance(now + Duration::from_millis(50)));
        assert_eq!(animator.content_progress(), 1.0);
    }

    #[test]
    fn delayed_transition_waits_and_finishes_within_total_duration() {
        let now = Instant::now();
        let mut animator = Animator::new(size(100.0), Duration::from_millis(200));
        animator.transition_to_with_delay(size(300.0), Duration::from_millis(40), now);
        assert!(animator.advance(now + Duration::from_millis(20)));
        assert_eq!(animator.size(), size(100.0));
        assert!(animator.advance(now + Duration::from_millis(100)));
        assert!(animator.size().width > px(100.0));
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.size(), size(300.0));
    }
}
