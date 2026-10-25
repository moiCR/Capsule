use std::time::{Duration, Instant};

use gpui::Context;
use services::AppState;

use super::{DashboardModule, DashboardSubView, slider_fraction};
use crate::bar::shared_panel::SharedPanelEvent;

impl DashboardModule {
    pub fn start_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_volume = true;
        self.volume_at(x, cx);
    }

    pub fn volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.volume_bounds;
        if let Some(fraction) = slider_fraction(x, bounds.left(), bounds.width(0.0)) {
            if let Some(state) = cx.try_global::<AppState>() {
                state
                    .system
                    .set_volume_fast((fraction * 100.0).round() as u32);
            }
            cx.notify();
        }
    }

    pub fn start_input_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_input_volume = true;
        self.input_volume_at(x, cx);
    }

    pub fn input_volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.input_volume_bounds;
        if let Some(fraction) = slider_fraction(x, bounds.left(), bounds.width(0.0)) {
            if let Some(state) = cx.try_global::<AppState>() {
                state
                    .system
                    .set_input_volume_fast((fraction * 100.0).round() as u32);
            }
            cx.notify();
        }
    }

    pub fn start_brightness(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_brightness = true;
        self.brightness_at(x, cx);
    }

    pub fn brightness_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.brightness_bounds;
        if let Some(fraction) = slider_fraction(x, bounds.left(), bounds.width(0.0)) {
            let value = ((fraction * 100.0).round() as u32).clamp(5, 100);
            if self.brightness_preview != Some(value) {
                self.brightness_preview = Some(value);
                cx.notify();
            }
        }
    }

    pub fn finish_brightness(&mut self, cx: &mut Context<Self>) {
        if !self.dragging_brightness {
            return;
        }
        self.dragging_brightness = false;
        if let Some(value) = self.brightness_preview.take() {
            if let Some(state) = cx.try_global::<AppState>() {
                let system = state.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.set_brightness(value).await;
                });
            }
            cx.notify();
        }
    }

    pub fn finish_sliders(&mut self, cx: &mut Context<Self>) {
        let was_audio = self.dragging_volume || self.dragging_input_volume;
        self.dragging_volume = false;
        self.dragging_input_volume = false;
        self.finish_brightness(cx);
        if was_audio {
            if let Some(state) = cx.try_global::<AppState>() {
                let sys = state.system.clone();
                services::spawn_tokio(async move {
                    let _ = sys.refresh().await;
                });
            }
            cx.notify();
        }
    }

    pub fn open_subview(&mut self, subview: DashboardSubView, cx: &mut Context<Self>) {
        if subview == DashboardSubView::Home {
            self.close_subview(cx);
            return;
        }
        self.view = subview;
        self.overlay_view = Some(subview);
        self.overlay_closing = false;
        self.overlay_anim_start = Some(Instant::now());
        if self.overlay_progress <= 0.01 {
            self.overlay_progress = 0.01;
        }
        self.spawn_overlay_animation(cx);
        cx.emit(SharedPanelEvent::SyncWindow);
        cx.notify();
    }

    pub fn close_subview(&mut self, cx: &mut Context<Self>) {
        if self.overlay_view.is_none() {
            self.view = DashboardSubView::Home;
            self.selected_ssid = None;
            cx.emit(SharedPanelEvent::SyncWindow);
            cx.notify();
            return;
        }
        self.view = DashboardSubView::Home;
        self.selected_ssid = None;
        self.password_input.clear();
        self.overlay_closing = true;
        self.overlay_anim_start = Some(Instant::now());
        self.spawn_overlay_animation(cx);
        cx.emit(SharedPanelEvent::SyncWindow);
        cx.notify();
    }

    fn spawn_overlay_animation(&mut self, cx: &mut Context<Self>) {
        self.overlay_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let mut running = false;
                let updated = this.update(cx, |module, cx| {
                    running = module.advance(Instant::now());
                    cx.notify();
                });
                if updated.is_err() || !running {
                    break;
                }
            }
        }));
    }
}
