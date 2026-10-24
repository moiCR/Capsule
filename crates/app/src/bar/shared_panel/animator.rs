use std::time::{Duration, Instant};

use gpui::{Pixels, Size, px};
use services::CapsuleAnimation;

use crate::animator::{apply_squash_stretch, morph_progress, scalar_progress};

pub struct SharedPanelAnimator {
    pub current_x: Pixels,
    pub start_x: Pixels,
    pub target_x: Pixels,

    pub current_size: Size<Pixels>,
    pub start_size: Size<Pixels>,
    pub target_size: Size<Pixels>,

    pub dock_edge: super::DockEdge,

    pub opacity: f32,
    pub backdrop_opacity: f32,
    pub content_progress: f32,

    duration: Duration,
    animation: CapsuleAnimation,
    bounciness: f32,
    squash_stretch: bool,
    reduced_motion: bool,

    open_start: Instant,
    close_start: Option<Instant>,
    close_start_height: f32,
    close_start_opacity: f32,

    switch_start: Option<Instant>,
}

impl SharedPanelAnimator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        initial_x: Pixels,
        target_size: Size<Pixels>,
        dock_edge: super::DockEdge,
        duration: Duration,
        animation: CapsuleAnimation,
        bounciness: f32,
        squash_stretch: bool,
        reduced_motion: bool,
    ) -> Self {
        let initial_size = if reduced_motion {
            target_size
        } else {
            Size {
                width: target_size.width,
                height: px(0.0),
            }
        };

        Self {
            current_x: initial_x,
            start_x: initial_x,
            target_x: initial_x,
            current_size: initial_size,
            start_size: initial_size,
            target_size,
            dock_edge,
            opacity: if reduced_motion { 1.0 } else { 0.0 },
            backdrop_opacity: 0.0,
            content_progress: 1.0,
            duration,
            animation,
            bounciness,
            squash_stretch,
            reduced_motion,
            open_start: Instant::now(),
            close_start: None,
            close_start_height: 0.0,
            close_start_opacity: 0.0,
            switch_start: None,
        }
    }

    pub fn request_close(&mut self) {
        if self.close_start.is_none() {
            self.close_start_height = f32::from(self.current_size.height);
            self.close_start_opacity = self.opacity;
            self.close_start = Some(Instant::now());
        }
    }

    pub fn is_closing(&self) -> bool {
        self.close_start.is_some()
    }

    pub fn switch_to(
        &mut self,
        target_x: Pixels,
        target_size: Size<Pixels>,
        dock_edge: super::DockEdge,
        now: Instant,
    ) {
        self.dock_edge = dock_edge;
        self.start_x = self.current_x;
        self.target_x = target_x;
        self.start_size = self.current_size;
        self.target_size = target_size;
        self.content_progress = 0.0;
        self.switch_start = Some(now);
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        if let Some(close_start) = self.close_start {
            if self.reduced_motion {
                return (false, true);
            }
            let elapsed = now.duration_since(close_start);
            let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
            let (_, prog_h) = morph_progress(t, false, self.animation, self.bounciness);
            self.current_size.height = px((self.close_start_height * (1.0 - prog_h)).max(0.0));
            self.opacity = if t < 0.65 {
                self.close_start_opacity
            } else {
                (self.close_start_opacity * (1.0 - (t - 0.65) / 0.35)).clamp(0.0, 1.0)
            };
            self.content_progress = (1.0 - t / 0.4).clamp(0.0, 1.0);
            self.backdrop_opacity = 0.0;
            if t >= 1.0 {
                self.current_size.height = px(0.0);
                self.opacity = 0.0;
                return (false, true);
            }
            return (true, false);
        }

        let mut running = false;

        if let Some(switch_start) = self.switch_start {
            if self.reduced_motion {
                self.current_x = self.target_x;
                self.current_size = self.target_size;
                self.content_progress = 1.0;
                self.switch_start = None;
            } else {
                let elapsed = now.duration_since(switch_start);
                let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
                let expanding = self.target_size.width > self.start_size.width
                    || self.target_size.height > self.start_size.height;
                let (prog_w, prog_h) =
                    morph_progress(t, expanding, self.animation, self.bounciness);

                let x_progress = scalar_progress(t, self.animation, self.bounciness);
                let x_diff = f32::from(self.target_x - self.start_x);
                self.current_x = self.start_x + px(x_diff * x_progress);

                let w_diff = f32::from(self.target_size.width - self.start_size.width);
                let h_diff = f32::from(self.target_size.height - self.start_size.height);
                let raw_w = self.start_size.width + px(w_diff * prog_w);
                let raw_h = self.start_size.height + px(h_diff * prog_h);

                if self.squash_stretch {
                    self.current_size =
                        apply_squash_stretch(self.start_size, self.target_size, raw_w, raw_h, t);
                } else {
                    self.current_size = Size {
                        width: raw_w,
                        height: raw_h,
                    };
                }

                self.content_progress = t;

                if t < 1.0 {
                    running = true;
                } else {
                    self.current_x = self.target_x;
                    self.current_size = self.target_size;
                    self.content_progress = 1.0;
                    self.switch_start = None;
                }
            }
        } else if !self.reduced_motion {
            let elapsed = now.duration_since(self.open_start);
            let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
            let prog = scalar_progress(t, self.animation, self.bounciness);
            self.current_size.height = px(f32::from(self.target_size.height) * prog);
            self.opacity = prog;
            self.backdrop_opacity = 0.0;
            if t < 1.0 {
                running = true;
            } else {
                self.current_size = self.target_size;
                self.opacity = 1.0;
                self.backdrop_opacity = 0.0;
            }
        }

        (running, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_panel_animator_initialization() {
        let animator = SharedPanelAnimator::new(
            px(100.0),
            Size {
                width: px(300.0),
                height: px(200.0),
            },
            super::super::DockEdge::Center,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        assert_eq!(animator.current_x, px(100.0));
        assert_eq!(animator.current_size.width, px(300.0));
        assert_eq!(animator.current_size.height, px(0.0));
        assert_eq!(animator.opacity, 0.0);
    }

    #[test]
    fn test_shared_panel_animator_advance_to_full() {
        let mut animator = SharedPanelAnimator::new(
            px(100.0),
            Size {
                width: px(300.0),
                height: px(200.0),
            },
            super::super::DockEdge::Center,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        let future = Instant::now() + Duration::from_millis(250);
        let (running, closed) = animator.advance(future);
        assert!(!running);
        assert!(!closed);
        assert_eq!(animator.current_size.height, px(200.0));
        assert_eq!(animator.opacity, 1.0);
    }

    #[test]
    fn test_shared_panel_animator_switch_glide() {
        let mut animator = SharedPanelAnimator::new(
            px(100.0),
            Size {
                width: px(300.0),
                height: px(200.0),
            },
            super::super::DockEdge::Center,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        let now = Instant::now();
        animator.advance(now + Duration::from_millis(250));

        let new_target = Size {
            width: px(350.0),
            height: px(400.0),
        };
        animator.switch_to(px(500.0), new_target, super::super::DockEdge::Right, now);
        assert_eq!(animator.target_x, px(500.0));
        assert_eq!(animator.target_size, new_target);
        assert_eq!(animator.dock_edge, super::super::DockEdge::Right);

        let mid = now + Duration::from_millis(100);
        let (running, _) = animator.advance(mid);
        assert!(running);
        assert!(animator.current_x > px(100.0) && animator.current_x < px(500.0));

        let end = now + Duration::from_millis(250);
        let (running_end, _) = animator.advance(end);
        assert!(!running_end);
        assert_eq!(animator.current_x, px(500.0));
        assert_eq!(animator.current_size, new_target);
    }

    #[test]
    fn test_shared_panel_animator_close() {
        let mut animator = SharedPanelAnimator::new(
            px(100.0),
            Size {
                width: px(300.0),
                height: px(200.0),
            },
            super::super::DockEdge::Center,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );
        let now = Instant::now();
        animator.advance(now + Duration::from_millis(250));
        animator.request_close();
        assert!(animator.is_closing());

        let future = now + Duration::from_millis(500);
        let (running, closed) = animator.advance(future);
        assert!(!running);
        assert!(closed);
        assert_eq!(animator.opacity, 0.0);
    }

    #[test]
    fn test_shared_panel_animator_reduced_motion() {
        let mut animator = SharedPanelAnimator::new(
            px(100.0),
            Size {
                width: px(300.0),
                height: px(200.0),
            },
            super::super::DockEdge::Center,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            true,
        );
        assert_eq!(animator.current_size.height, px(200.0));
        assert_eq!(animator.opacity, 1.0);

        animator.request_close();
        let (_, closed) = animator.advance(Instant::now());
        assert!(closed);
    }
}
