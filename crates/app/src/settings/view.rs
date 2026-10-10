use gpui::{
    Context, Entity, FocusHandle, IntoElement, KeyDownEvent, ParentElement, Render, Styled, Window,
    div, prelude::*, px,
};
use std::time::Instant;
use ui::theme::Theme;

use crate::capsule::modules::settings::{SettingsEvent, SettingsModule};
use crate::new_capsule::widgets::{
    settings::{HEIGHT, WIDTH},
    style,
};
use crate::panel::SettingsPanel;

pub struct SettingsWindow {
    focus_handle: FocusHandle,
    settings_module: Entity<SettingsModule>,
    entry_start_time: Instant,
    handle: gpui::AnyWindowHandle,
    frame_pending: bool,
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
            entry_start_time: Instant::now(),
            handle: window.window_handle(),
            frame_pending: true,
            is_closing: false,
            close_start_time: None,
            offset_y: 16.0,
            card_opacity: 0.0,
            close_offset_y: 16.0,
            close_opacity: 0.0,
            animation_duration,
        };

        window_obj
            .settings_module
            .update(cx, |module, cx| module.focus(window, cx));
        Self::queue_frame(window, cx.entity().downgrade());
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
        if !self.frame_pending {
            self.frame_pending = true;
            let entity = cx.entity().downgrade();
            let handle = self.handle;
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_, window, _| Self::queue_frame(window, entity));
            });
        }
        cx.notify();
    }

    fn queue_frame(window: &Window, entity: gpui::WeakEntity<Self>) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |this, cx| {
                let running = if let Some(start) = this.close_start_time {
                    let duration = this.animation_duration * 0.6;
                    let elapsed = start.elapsed().as_secs_f32();
                    if elapsed >= duration {
                        window.remove_window();
                        false
                    } else {
                        let progress = (elapsed / duration).clamp(0.0, 1.0).powi(2);
                        this.offset_y = this.close_offset_y + 8.0 * progress;
                        this.card_opacity = this.close_opacity * (1.0 - progress);
                        true
                    }
                } else {
                    let duration = this.animation_duration * 0.88;
                    let elapsed = this.entry_start_time.elapsed().as_secs_f32();
                    if elapsed >= duration {
                        this.offset_y = 0.0;
                        this.card_opacity = 1.0;
                        false
                    } else {
                        let remaining = (1.0 - elapsed / duration).powi(3);
                        this.offset_y = 16.0 * remaining;
                        this.card_opacity = 1.0 - remaining;
                        true
                    }
                };
                this.frame_pending = running;
                cx.notify();
                if running {
                    Self::queue_frame(window, cx.entity().downgrade());
                }
            });
        });
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

        let viewport = window.viewport_size();
        let width = (f32::from(viewport.width) - 48.0).clamp(320.0, WIDTH);
        let height = (f32::from(viewport.height) - 48.0).clamp(240.0, HEIGHT);
        let card = div()
            .id("settings-modal-card")
            .relative()
            .top(px(self.offset_y))
            .w(px(width))
            .h(px(height))
            .rounded(px(capsule_radius))
            .bg(style::background(&theme))
            .border_1()
            .border_color(style::border(&theme))
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
                            .w(px(WIDTH))
                            .h(px(HEIGHT))
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
