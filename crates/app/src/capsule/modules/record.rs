use gpui::{Context, EventEmitter, IntoElement, Render, Task, Window};
use services::{AppState, RecordOptions, RecordStatus};
use std::time::Duration;

use crate::capsule::widgets::record::render_record_bar;

pub enum RecordEvent {
    Close,
}

pub struct RecordModule {
    is_starting: bool,
    timer_task: Option<Task<()>>,
}

impl EventEmitter<RecordEvent> for RecordModule {}

impl RecordModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let module = Self {
            is_starting: false,
            timer_task: None,
        };

        if cx.has_global::<AppState>() {
            let mut rx = cx.global::<AppState>().record.subscribe_status();
            cx.spawn(async move |this, cx| {
                loop {
                    match rx.recv().await {
                        Ok(status) => {
                            let _ = this.update(cx, |view: &mut Self, cx| {
                                if status == RecordStatus::Stopped {
                                    view.is_starting = false;
                                    view.timer_task = None;
                                } else {
                                    view.is_starting = false;
                                    if view.timer_task.is_none() {
                                        view.start_timer(cx);
                                    }
                                }
                                cx.notify();
                            });
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    }
                }
            })
            .detach();
        }

        module
    }

    pub fn start_recording(&mut self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

        let app_state = cx.global::<AppState>();
        let cfg = app_state.config.get();
        let record_service = app_state.record.clone();
        let options = RecordOptions::from_config(&cfg.record);

        self.is_starting = true;
        self.start_timer(cx);

        tokio::spawn(async move {
            let _ = record_service.start(options).await;
        });

        cx.notify();
    }

    fn start_timer(&mut self, cx: &mut Context<Self>) {
        if self.timer_task.is_some() {
            return;
        }

        let task = cx.spawn(async move |this, cx| {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let continue_timer = this
                    .update(cx, |view: &mut Self, cx| {
                        if view.is_starting {
                            cx.notify();
                            return true;
                        }
                        if cx.has_global::<AppState>() {
                            let rec = &cx.global::<AppState>().record;
                            if rec.is_recording() || rec.is_paused() {
                                cx.notify();
                                return true;
                            }
                        }
                        false
                    })
                    .unwrap_or(false);

                if !continue_timer {
                    let _ = this.update(cx, |view: &mut Self, _| {
                        view.timer_task = None;
                    });
                    break;
                }
            }
        });

        self.timer_task = Some(task);
    }

    pub fn toggle_pause(&mut self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

        let record_service = cx.global::<AppState>().record.clone();
        tokio::spawn(async move {
            let _ = record_service.toggle_pause().await;
        });

        cx.notify();
    }

    pub fn stop_recording(&mut self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

        let record_service = cx.global::<AppState>().record.clone();
        self.is_starting = false;
        self.timer_task = None;
        record_service.mark_stopped();

        tokio::spawn(async move {
            let _ = record_service.stop().await;
        });

        self.close(cx);
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        cx.emit(RecordEvent::Close);
    }

    pub fn desired_width(&self, cx: &gpui::App) -> f32 {
        let (status, duration_secs) = if cx.has_global::<AppState>() {
            let rec = &cx.global::<AppState>().record;
            (rec.get_status(), rec.get_duration_secs())
        } else {
            (RecordStatus::Stopped, 0)
        };

        let extra_hours_w = if duration_secs >= 3600 { 26.0 } else { 0.0 };

        if self.is_starting || status == RecordStatus::Recording {
            220.0 + extra_hours_w
        } else if status == RecordStatus::Paused {
            264.0 + extra_hours_w
        } else {
            let label = if cx.has_global::<AppState>() {
                cx.global::<AppState>().language.get("record.record_screen")
            } else {
                "Grabar pantalla".to_string()
            };
            let text_w = calc_text_width(&label);
            (text_w + 64.0).max(168.0)
        }
    }
}

fn calc_text_width(text: &str) -> f32 {
    let mut width: f32 = 0.0;
    for c in text.chars() {
        let code = c as u32;
        if (0x3000..=0x9FFF).contains(&code)
            || (0xF900..=0xFAFF).contains(&code)
            || (0xFF00..=0xFFEF).contains(&code)
            || (0x20000..=0x2FA1F).contains(&code)
            || (0x1F300..=0x1F9FF).contains(&code)
        {
            width += 14.5;
        } else if c.is_ascii_uppercase() || c == 'W' || c == 'M' || c == '@' {
            width += 9.2;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() {
            width += 7.8;
        } else if c == ' ' {
            width += 4.5;
        } else {
            width += 8.0;
        }
    }
    width
}

impl Render for RecordModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (status, duration_secs) = if cx.has_global::<AppState>() {
            let rec = &cx.global::<AppState>().record;
            (rec.get_status(), rec.get_duration_secs())
        } else {
            (RecordStatus::Stopped, 0)
        };

        let effective_status = if self.is_starting && status == RecordStatus::Stopped {
            RecordStatus::Recording
        } else {
            status
        };

        if (effective_status == RecordStatus::Recording || effective_status == RecordStatus::Paused)
            && self.timer_task.is_none()
        {
            self.start_timer(cx);
        }

        let duration_str = if duration_secs >= 3600 {
            let hours = duration_secs / 3600;
            let mins = (duration_secs % 3600) / 60;
            let secs = duration_secs % 60;
            format!("{hours:02}:{mins:02}:{secs:02}")
        } else {
            let mins = duration_secs / 60;
            let secs = duration_secs % 60;
            format!("{mins:02}:{secs:02}")
        };

        render_record_bar(effective_status, duration_str, cx)
    }
}
