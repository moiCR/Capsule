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
                } else if self.navigation.escape() {
                    self.dispatch(DashboardAction::Close, cx);
                } else {
                    cx.notify();
                }
            }
            "left"
                if self.navigation.view
                    == crate::new_capsule::widgets::dashboard::DashboardView::Calendar =>
            {
                self.dispatch(DashboardAction::Month(-1), cx)
            }
            "right"
                if self.navigation.view
                    == crate::new_capsule::widgets::dashboard::DashboardView::Calendar =>
            {
                self.dispatch(DashboardAction::Month(1), cx)
            }
            "home"
                if self.navigation.view
                    == crate::new_capsule::widgets::dashboard::DashboardView::Calendar =>
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
