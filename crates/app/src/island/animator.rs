pub use crate::animator::*;

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
        let mut animator =
            Animator::new(start, Duration::from_millis(200)).with_squash_stretch(true);
        animator.transition_to(target, now);
        assert!(animator.advance(now + Duration::from_millis(50)));
        assert!(animator.size().height < px(40.0));
        assert!(!animator.advance(now + Duration::from_millis(200)));
        assert_eq!(animator.size(), target);
    }

    #[test]
    fn reduced_motion_skips_spatial_morph_and_shortens_fade() {
        let now = Instant::now();
        let mut animator =
            Animator::new(size(100.0), Duration::from_millis(300)).with_reduced_motion(true);
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
