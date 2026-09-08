use gpui::{Context, IntoElement, Render, Task, Window, div, prelude::*};
use services::AppState;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::capsule::widgets::volume::volume_bar::render_volume_bar;

pub struct VolumeModule {
    pub target_volume: u32,
    pub display_volume: f32,
    pub is_muted: bool,
    anim_task: Option<Task<()>>,
}

impl VolumeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (volume, is_muted) = if cx.has_global::<AppState>() {
            let status = cx.global::<AppState>().system.get_status();
            (status.volume, status.is_muted)
        } else {
            (50, false)
        };

        Self {
            target_volume: volume,
            display_volume: volume as f32,
            is_muted,
            anim_task: None,
        }
    }

    pub fn update_status(&mut self, volume: u32, is_muted: bool, cx: &mut Context<Self>) {
        let vol_changed = self.target_volume != volume;
        let mute_changed = self.is_muted != is_muted;

        if vol_changed || mute_changed {
            self.target_volume = volume;
            self.is_muted = is_muted;

            if (self.display_volume - volume as f32).abs() < 0.5 {
                self.display_volume = volume as f32;
                self.anim_task = None;
                cx.notify();
            } else {
                self.start_smooth_animation(cx);
            }
        }
    }

    fn start_smooth_animation(&mut self, cx: &mut Context<Self>) {
        let frame_duration = if cx.has_global::<AppState>() {
            cx.global::<AppState>().compositor.get_frame_duration()
        } else {
            Duration::from_millis(16)
        };

        let start_time = Instant::now();
        let duration_ms = 100.0;

        let anim_task = cx.spawn(async move |this, cx| {
            let start_vol = this
                .update(cx, |this: &mut Self, _| this.display_volume)
                .unwrap_or(0.0);

            loop {
                cx.background_executor().timer(frame_duration).await;
                let finished = this
                    .update(cx, |this: &mut Self, cx| {
                        let elapsed = start_time.elapsed().as_secs_f32() * 1000.0;
                        let t = (elapsed / duration_ms).min(1.0);
                        let eased_t = 1.0 - (1.0 - t).powi(3);
                        let target = this.target_volume as f32;
                        this.display_volume = start_vol + (target - start_vol) * eased_t;
                        cx.notify();
                        let done = t >= 1.0;
                        if done {
                            this.display_volume = target;
                            this.anim_task = None;
                        }
                        done
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });
        self.anim_task = Some(anim_task);
    }
}

impl Render for VolumeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let muted_label = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("volume.muted")
        } else {
            "Mute".to_string()
        };
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(render_volume_bar(
                self.display_volume,
                self.is_muted,
                muted_label,
                &theme,
            ))
    }
}
