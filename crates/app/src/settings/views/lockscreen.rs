use gpui::{
    AnyElement, App, Context, ElementId, IntoElement, MouseMoveEvent, MouseUpEvent, ParentElement,
    Styled, Window, div, px,
};
use services::AppState;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::SettingsView;
use super::common::{
    card, control_row, heading, hero_header, pill_button, row_divider, slider_row, toggle_row,
};
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

pub struct LockScreenView {
    timeout_tracker: DimensionTracker,
    dragging_timeout: bool,
}

impl LockScreenView {
    pub fn new() -> Self {
        Self {
            timeout_tracker: DimensionTracker::new(),
            dragging_timeout: false,
        }
    }

    fn update_timeout(&mut self, x: f32, cx: &mut Context<Settings>) {
        let left = self.timeout_tracker.left();
        let width = self.timeout_tracker.width(0.0);
        if width > 0.0 {
            let frac = ((x - left) / width).clamp(0.0, 1.0);
            let mins = (frac * 60.0).round().clamp(0.0, 60.0);
            let secs = (mins * 60.0) as u64;

            if cx.has_global::<AppState>() {
                let _ = cx.global::<AppState>().config.update(|cfg| {
                    cfg.lockscreen.idle_timeout = secs;
                });
            }
            cx.notify();
        }
    }
}

impl SettingsView for LockScreenView {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Settings>) -> bool {
        if self.dragging_timeout {
            self.update_timeout(event.position.x.into(), cx);
            return true;
        }
        false
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _cx: &mut Context<Settings>) -> bool {
        if self.dragging_timeout {
            self.dragging_timeout = false;
            return true;
        }
        false
    }

    fn render(&mut self, _window: &mut Window, cx: &mut Context<Settings>) -> AnyElement {
        let theme = cx.global::<Theme>().clone();
        let config = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get()
        } else {
            Default::default()
        };

        let idle_secs = config.lockscreen.idle_timeout;
        let idle_mins = (idle_secs / 60) as f32;
        let display_timeout = if idle_secs == 0 {
            text("lockscreen.disabled", cx)
        } else {
            format!("{idle_mins:.0} min")
        };
        let timeout_pct = (idle_mins / 60.0).clamp(0.0, 1.0);

        let time_presets = [("%H:%M", "24h (14:30)"), ("%I:%M %p", "12h (02:30 PM)")];

        let date_presets = [
            ("%A, %B %e, %Y", "%A, %B %e, %Y"),
            ("%d/%m/%Y", "DD/MM/YYYY"),
            ("%Y-%m-%d", "YYYY-MM-DD"),
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            // Hero Header
            .child(hero_header(
                "lock.svg",
                &text("settings.tab_lockscreen", cx),
                &text("settings.lockscreen_header_subtitle", cx),
                &theme,
            ))
            // Behavior Card
            .child(
                card(&theme)
                    .child(heading(&text("settings.general_heading", cx), &theme))
                    .child(slider_row(
                        ElementId::Name("slider-idle-timeout".into()),
                        &text("settings.idle_timeout_title", cx),
                        &display_timeout,
                        timeout_pct,
                        &self.timeout_tracker,
                        &theme,
                        cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                            if let Some(view) = this.view_as_mut::<LockScreenView>() {
                                view.dragging_timeout = true;
                                view.update_timeout(event.position.x.into(), cx);
                            }
                        }),
                        cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                            if let Some(view) = this.view_as_mut::<LockScreenView>() {
                                if view.handle_mouse_up(event, cx) {
                                    cx.notify();
                                }
                            }
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("lock-show-clock-toggle".into()),
                        &text("settings.show_clock_title", cx),
                        Some(&text("settings.show_clock_subtitle", cx)),
                        config.lockscreen.show_clock,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.lockscreen.show_clock = !cfg.lockscreen.show_clock;
                                });
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("lock-show-media-toggle".into()),
                        &text("settings.show_media_title", cx),
                        Some(&text("settings.show_media_subtitle", cx)),
                        config.lockscreen.show_media_player,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.lockscreen.show_media_player =
                                        !cfg.lockscreen.show_media_player;
                                });
                            }
                            cx.notify();
                        }),
                    )),
            )
            // Time & Date Formats Card
            .child(
                card(&theme)
                    .child(heading(
                        &text("settings.lockscreen_formats_title", cx),
                        &theme,
                    ))
                    .child(control_row(
                        &text("settings.time_format_title", cx),
                        Some(&text("settings.time_format_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            time_presets
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (fmt, label))| {
                                    let is_selected = config.lockscreen.time_format == fmt;
                                    let fmt_str = fmt.to_string();
                                    pill_button(
                                        ElementId::NamedInteger("time-fmt-pill".into(), idx as u64),
                                        label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = fmt_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.lockscreen.time_format = target;
                                                    });
                                            }
                                            cx.notify();
                                        }),
                                    )
                                }),
                        ),
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.date_format_title", cx),
                        Some(&text("settings.date_format_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            date_presets
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (fmt, label))| {
                                    let is_selected = config.lockscreen.date_format == fmt;
                                    let fmt_str = fmt.to_string();
                                    pill_button(
                                        ElementId::NamedInteger("date-fmt-pill".into(), idx as u64),
                                        label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = fmt_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.lockscreen.date_format = target;
                                                    });
                                            }
                                            cx.notify();
                                        }),
                                    )
                                }),
                        ),
                        &theme,
                    )),
            )
            .into_any_element()
    }
}
