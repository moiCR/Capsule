use gpui::{
    Context, Entity, FocusHandle, IntoElement, KeyDownEvent, ParentElement, Render, Styled, Window,
    div, prelude::*, px,
};
use std::time::Instant;
use ui::theme::Theme;

use crate::capsule::modules::settings::{SettingsEvent, SettingsModule};
use crate::panel::SettingsPanel;

pub struct SettingsWindow {
    focus_handle: FocusHandle,
    settings_module: Entity<SettingsModule>,
    entry_start_time: Option<Instant>,
    is_closing: bool,
    close_start_time: Option<Instant>,
    offset_y: f32,
    card_opacity: f32,
    close_offset_y: f32,
    close_opacity: f32,
    animation_duration: f32,
}

impl SettingsWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let settings_module = cx.new(SettingsModule::new);

        cx.subscribe(
            &settings_module,
            |this, _, event: &SettingsEvent, cx| match event {
                SettingsEvent::Close => {
                    this.request_close(cx);
                }
            },
        )
        .detach();

        let animation_duration = if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .config
                .get()
                .ui
                .animation_duration_ms
        } else {
            services::config::UIConfig::default().animation_duration_ms
        } as f32
            / 1000.0;

        let window_obj = Self {
            focus_handle,
            settings_module,
            entry_start_time: None,
            is_closing: false,
            close_start_time: None,
            offset_y: 16.0,
            card_opacity: 0.0,
            close_offset_y: 16.0,
            close_opacity: 0.0,
            animation_duration,
        };

        window.focus(&window_obj.focus_handle, cx);
        window_obj
    }

    pub fn request_close(&mut self, cx: &mut Context<Self>) {
        if self.is_closing {
            return;
        }
        self.close_offset_y = self.offset_y;
        self.close_opacity = self.card_opacity;
        self.is_closing = true;
        self.close_start_time = Some(Instant::now());
        cx.notify();
    }
}

impl Drop for SettingsWindow {
    fn drop(&mut self) {
        SettingsPanel::mark_closed();
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let capsule_radius = if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .config
                .get()
                .ui
                .capsule_round
        } else {
            24.0
        };

        if self.is_closing {
            let start = self.close_start_time.get_or_insert_with(Instant::now);
            let elapsed = start.elapsed().as_secs_f32();
            let duration = self.animation_duration * 0.6;
            if elapsed >= duration {
                window.remove_window();
                return div().into_any_element();
            }

            let t = (elapsed / duration).clamp(0.0, 1.0);
            let progress = t * t;
            self.offset_y = self.close_offset_y + 8.0 * progress;
            self.card_opacity = self.close_opacity * (1.0 - progress);
            window.request_animation_frame();
        } else {
            let start = self.entry_start_time.get_or_insert_with(Instant::now);
            let elapsed = start.elapsed().as_secs_f32();
            let duration = self.animation_duration * 0.88;
            if elapsed >= duration {
                self.offset_y = 0.0;
                self.card_opacity = 1.0;
            } else {
                let t = (elapsed / duration).clamp(0.0, 1.0);
                let remaining = (1.0 - t).powi(3);
                self.offset_y = 16.0 * remaining;
                self.card_opacity = 1.0 - remaining;
                window.request_animation_frame();
            }
        }

        let card = div()
            .id("settings-modal-card")
            .relative()
            .top(px(self.offset_y))
            .w(px(840.0))
            .h(px(560.0))
            .rounded(px(capsule_radius))
            .bg(theme.background())
            .border_1()
            .border_color(theme.surface().opacity(0.6))
            .shadow_xl()
            .overflow_hidden()
            .opacity(self.card_opacity)
            .on_click(cx.listener(|_this, _, _, cx| {
                cx.stop_propagation();
            }))
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .child(
                        div()
                            .w(px(840.0))
                            .h(px(560.0))
                            .flex_shrink_0()
                            .child(self.settings_module.clone()),
                    ),
            );

        div()
            .id("settings-window-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "escape" {
                    this.request_close(cx);
                }
            }))
            .on_click(cx.listener(|this, _, _, cx| {
                this.request_close(cx);
            }))
            .relative()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(card)
            .into_any_element()
    }
}
