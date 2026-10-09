use super::{DashboardModule, slider_fraction};
use crate::new_capsule::widgets::dashboard::DashboardAction;
use gpui::{Context, KeyDownEvent, Window};
use services::AppState;
impl DashboardModule {
    pub(super) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event.keystroke.key.as_str() {
            "escape" => {
                if self.satellites.iter().any(|satellite| satellite.open) {
                    if let Some(panel) = self.satellites.iter().rev().find(|panel| panel.open) {
                        cx.emit(
                            crate::new_capsule::module::CapsuleModuleEvent::CloseSatelliteId(
                                panel.id.clone(),
                            ),
                        );
                    }
                } else {
                    self.dispatch(DashboardAction::Close, cx);
                }
            }
            "left"
                if self.satellites.iter().any(|panel| {
                    panel.open && panel.id == crate::new_capsule::satellite::SatelliteId::Calendar
                }) =>
            {
                self.dispatch(DashboardAction::Month(-1), cx)
            }
            "right"
                if self.satellites.iter().any(|panel| {
                    panel.open && panel.id == crate::new_capsule::satellite::SatelliteId::Calendar
                }) =>
            {
                self.dispatch(DashboardAction::Month(1), cx)
            }
            "home"
                if self.satellites.iter().any(|panel| {
                    panel.open && panel.id == crate::new_capsule::satellite::SatelliteId::Calendar
                }) =>
            {
                self.dispatch(DashboardAction::Month(0), cx)
            }
            "enter" if self.navigation.selected_ssid.is_some() => {
                self.dispatch(DashboardAction::ConnectWifi, cx)
            }
            "backspace" if self.navigation.selected_ssid.is_some() => {
                self.navigation.password.pop();
                cx.notify();
            }
            "v" if self.navigation.selected_ssid.is_some() && event.keystroke.modifiers.control => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.navigation
                        .password
                        .extend(text.chars().filter(|c| !c.is_control()));
                    cx.notify();
                }
            }
            _ if self.navigation.selected_ssid.is_some()
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt =>
            {
                if let Some(text) = &event.keystroke.key_char {
                    self.navigation
                        .password
                        .extend(text.chars().filter(|c| !c.is_control()));
                    cx.notify();
                }
            }
            _ => {}
        }
    }
    pub fn volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        if let Some(fraction) =
            slider_fraction(x, self.volume_bounds.left(), self.volume_bounds.width(0.0))
        {
            cx.global::<AppState>()
                .system
                .set_volume_fast((fraction * 100.0).round() as u32);
            self.snapshot.audio = cx.global::<AppState>().system.get_status();
            cx.notify();
        }
    }
    pub fn start_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.dragging_volume = true;
        self.volume_at(x, cx);
    }
    pub fn brightness_at(&mut self, x: f32, cx: &mut Context<Self>) {
        if let Some(fraction) = slider_fraction(
            x,
            self.brightness_bounds.left(),
            self.brightness_bounds.width(0.0),
        ) {
            let value = ((fraction * 100.0).round() as u32).clamp(5, 100);
            if self.brightness_preview != Some(value) {
                self.brightness_preview = Some(value);
                cx.notify();
            }
        }
    }
    pub fn start_brightness(&mut self, x: f32, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        self.dragging_brightness = true;
        self.brightness_at(x, cx);
    }
    pub(super) fn finish_brightness(&mut self, cx: &mut Context<Self>) {
        if !self.dragging_brightness {
            return;
        }
        self.dragging_brightness = false;
        if let Some(value) = self.brightness_preview.take() {
            self.dispatch(DashboardAction::Brightness(value), cx);
        }
    }
    pub fn start_seek(&mut self) {
        self.dragging_seek = true;
    }
    pub(super) fn seek_at(&mut self, x: f32, cx: &mut Context<Self>) {
        if let Some(fraction) =
            slider_fraction(x, self.seek_bounds.left(), self.seek_bounds.width(0.0))
            && let Some(length) = self.player().and_then(|p| p.length_micros)
        {
            self.dispatch(
                DashboardAction::Seek(f64::from(fraction) * length as f64 / 1_000_000.0),
                cx,
            );
        }
    }
}
