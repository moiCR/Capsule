use gpui::{
    Context, Entity, FocusHandle, IntoElement, KeyDownEvent, ParentElement, Render, Styled, Window,
    div, prelude::*, px, svg,
};
use std::time::Instant;
use ui::theme::Theme;

use crate::capsule::apple_island_spring;
use crate::capsule::modules::settings::{SettingsEvent, SettingsModule};
use crate::panel::SettingsPanel;

pub struct SettingsWindow {
    focus_handle: FocusHandle,
    settings_module: Entity<SettingsModule>,
    entry_start_time: Option<Instant>,
    is_closing: bool,
    close_start_time: Option<Instant>,
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

        let window_obj = Self {
            focus_handle,
            settings_module,
            entry_start_time: None,
            is_closing: false,
            close_start_time: None,
        };

        window.focus(&window_obj.focus_handle, cx);
        window_obj
    }

    pub fn request_close(&mut self, cx: &mut Context<Self>) {
        if self.is_closing {
            return;
        }
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

        let win_h: f32 = window.bounds().size.height.into();
        let initial_offset_y = (win_h * 0.45).clamp(320.0, 520.0);

        let (expand_p, offset_y, card_opacity, should_close_now) = if self.is_closing {
            let start = self.close_start_time.get_or_insert_with(Instant::now);
            let elapsed = start.elapsed().as_secs_f32();
            let dur = 0.42;
            if elapsed >= dur {
                (0.0, initial_offset_y, 0.0, true)
            } else {
                window.request_animation_frame();
                let t = (elapsed / dur).clamp(0.0, 1.0);
                let contract_end = 0.50;
                let slide_start = 0.44;

                let expand = if t >= contract_end {
                    0.0
                } else {
                    let ct = (t / contract_end).clamp(0.0, 1.0);
                    (1.0 - apple_island_spring(ct)).max(0.0)
                };

                let (y, op) = if t <= slide_start {
                    (0.0, 1.0)
                } else {
                    let st = ((t - slide_start) / (1.0 - slide_start)).clamp(0.0, 1.0);
                    let y = initial_offset_y * st.powf(1.8);
                    let op = 1.0 - ((st - 0.4) / 0.6).clamp(0.0, 1.0);
                    (y, op)
                };

                (expand, y, op, false)
            }
        } else {
            let start = self.entry_start_time.get_or_insert_with(Instant::now);
            let elapsed = start.elapsed().as_secs_f32();
            let dur = 0.48;
            let t = (elapsed / dur).clamp(0.0, 1.0);
            if t < 1.0 {
                window.request_animation_frame();
            }

            let slide_end = 0.42;
            let expand_start = 0.35;

            let y = if t >= slide_end {
                0.0
            } else {
                let st = (t / slide_end).clamp(0.0, 1.0);
                let sp = apple_island_spring(st);
                initial_offset_y * (1.0 - sp)
            };

            let expand = if t <= expand_start {
                0.0
            } else {
                let et = ((t - expand_start) / (1.0 - expand_start)).clamp(0.0, 1.0);
                apple_island_spring(et)
            };

            let op = (t / 0.10).clamp(0.0, 1.0);

            (expand, y, op, false)
        };

        if should_close_now {
            window.remove_window();
            return div().into_any_element();
        }

        let initial_size = 56.0_f32;
        let target_w = 840.0_f32;
        let target_h = 560.0_f32;
        let initial_r = initial_size / 2.0;
        let target_r = capsule_radius;

        let current_w = (initial_size + (target_w - initial_size) * expand_p).max(initial_size);
        let current_h = (initial_size + (target_h - initial_size) * expand_p).max(initial_size);
        let current_r = (initial_r + (target_r - initial_r) * expand_p).max(initial_r);

        let icon_opacity = (1.0 - (expand_p / 0.35)).clamp(0.0, 1.0);
        let content_opacity = ((expand_p - 0.25) / 0.75).clamp(0.0, 1.0);

        let mut card = div()
            .id("settings-modal-card")
            .relative()
            .top(px(offset_y))
            .w(px(current_w))
            .h(px(current_h))
            .rounded(px(current_r))
            .bg(theme.background())
            .border_1()
            .border_color(
                theme
                    .surface()
                    .opacity(0.35 + 0.25 * expand_p.clamp(0.0, 1.0)),
            )
            .shadow_xl()
            .overflow_hidden()
            .opacity(card_opacity)
            .on_click(cx.listener(|_this, _, _, cx| {
                cx.stop_propagation();
            }));

        if icon_opacity > 0.01 {
            card = card.child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(icon_opacity)
                    .child(
                        svg()
                            .path("settings.svg")
                            .size(px(24.0))
                            .text_color(theme.accent()),
                    ),
            );
        }

        if content_opacity > 0.01 {
            card = card.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .child(
                        div()
                            .w(px(target_w))
                            .h(px(target_h))
                            .flex_shrink_0()
                            .opacity(content_opacity)
                            .child(self.settings_module.clone()),
                    ),
            );
        }

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
