use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};
use services::AppState;
use std::time::Instant;
use ui::theme::Theme;

pub struct Visualizer {
    active: bool,
    start_time: Instant,
    bar_heights: [f32; 4],
}

impl Visualizer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let start_time = Instant::now();

        let compositor = cx.global::<AppState>().compositor.clone();

        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(compositor.get_frame_duration())
                    .await;
                let res = this.update(cx, |this: &mut Self, cx| {
                    if this.active {
                        let t = this.start_time.elapsed().as_secs_f32();
                        let frequencies = [9.0, 14.0, 11.0, 16.0];
                        let offsets = [0.0, 1.2, 2.4, 3.6];

                        for i in 0..4 {
                            let wave1 = ((t * frequencies[i] + offsets[i]).sin() + 1.0) * 0.5;
                            let wave2 = ((t * (frequencies[i] * 0.75) + offsets[i] * 1.6).cos()
                                + 1.0)
                                * 0.5;
                            let combined = wave1 * 0.55 + wave2 * 0.45;

                            let target_h = 3.0 + combined * 11.0;
                            this.bar_heights[i] += (target_h - this.bar_heights[i]) * 0.45;
                        }
                        cx.notify();
                    }
                });

                if res.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            active: true,
            start_time,
            bar_heights: [3.0; 4],
        }
    }

    #[allow(dead_code)]
    pub fn set_active(&mut self, active: bool, cx: &mut Context<Self>) {
        if self.active != active {
            self.active = active;
            cx.notify();
        }
    }
}

impl Render for Visualizer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(2.0))
            .h(px(14.0));

        for &h in &self.bar_heights {
            row = row.child(
                div()
                    .w(px(2.5))
                    .h(px(h))
                    .bg(theme.accent())
                    .rounded(px(1.25)),
            );
        }

        row
    }
}
