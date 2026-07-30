use gpui::{Context, IntoElement, Render, Window, div, prelude::*};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::widgets::volume::volume_bar::render_volume_bar;

pub struct VolumeModule {
    pub volume: u32,
    pub is_muted: bool,
}

impl VolumeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (volume, is_muted) = if cx.has_global::<AppState>() {
            let status = cx.global::<AppState>().system.get_status();
            (status.volume, status.is_muted)
        } else {
            (50, false)
        };

        Self { volume, is_muted }
    }

    pub fn update_status(&mut self, volume: u32, is_muted: bool, cx: &mut Context<Self>) {
        if self.volume != volume || self.is_muted != is_muted {
            self.volume = volume;
            self.is_muted = is_muted;
            cx.notify();
        }
    }
}

impl Render for VolumeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .size_full()
            .child(render_volume_bar(self.volume, self.is_muted, &theme))
    }
}
