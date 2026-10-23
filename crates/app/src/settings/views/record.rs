use gpui::{
    AnyElement, App, ColorExt, Context, ElementId, FontWeight, IntoElement, ParentElement, Styled,
    Window, div, px,
};
use services::{AppState, RecordService};
use ui::theme::Theme;

use super::SettingsView;
use super::common::{card, control_row, hero_header, pill_button, row_divider, toggle_row};
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

pub struct RecordView {
    available_monitors: Vec<String>,
}

impl RecordView {
    pub fn new() -> Self {
        Self {
            available_monitors: RecordService::list_monitors(),
        }
    }
}

impl SettingsView for RecordView {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn render(&mut self, _window: &mut Window, cx: &mut Context<Settings>) -> AnyElement {
        let theme = cx.global::<Theme>().clone();
        let config = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get()
        } else {
            Default::default()
        };

        // Output monitors options
        let screen_label = text("settings.record_output_fullscreen", cx);
        let mut monitor_options = vec![("screen".to_string(), screen_label)];
        for m in &self.available_monitors {
            if m != "screen" {
                monitor_options.push((m.clone(), m.clone()));
            }
        }

        let fps_options = [30u32, 60, 120];

        let native_label = text("settings.record_res_native", cx);
        let res_options = [
            ("native", native_label),
            ("1920x1080", "1080p".to_string()),
            ("1280x720", "720p".to_string()),
        ];

        let ultra_label = text("settings.record_quality_ultra", cx);
        let very_high_label = text("settings.record_quality_very_high", cx);
        let high_label = text("settings.record_quality_high", cx);
        let medium_label = text("settings.record_quality_medium", cx);
        let quality_options = [
            ("ultra", ultra_label),
            ("very_high", very_high_label),
            ("high", high_label),
            ("medium", medium_label),
        ];

        let container_options = [("mp4", "MP4"), ("mkv", "MKV")];

        let desktop_label = text("settings.record_audio_desktop", cx);
        let mic_label = text("settings.record_audio_mic", cx);
        let both_label = text("settings.record_audio_both", cx);
        let none_label = text("settings.record_audio_none", cx);
        let audio_options = [
            ("desktop", desktop_label),
            ("mic", mic_label),
            ("both", both_label),
            ("none", none_label),
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            // Hero Header
            .child(hero_header(
                "play.svg",
                &text("settings.record_header_title", cx),
                &text("settings.record_header_subtitle", cx),
                &theme,
            ))
            // Video Output Card
            .child(
                card(&theme).child(control_row(
                    &text("settings.record_output_title", cx),
                    Some(&text("settings.record_output_subtitle", cx)),
                    div().flex().gap(px(6.0)).children(
                        monitor_options
                            .into_iter()
                            .enumerate()
                            .map(|(idx, (val, label))| {
                                let is_selected = config.record.output == val;
                                let val_str = val.clone();
                                pill_button(
                                    ElementId::NamedInteger("output-pill".into(), idx as u64),
                                    &label,
                                    is_selected,
                                    &theme,
                                    cx.listener(move |_this, _, _, cx| {
                                        if cx.has_global::<AppState>() {
                                            let target = val_str.clone();
                                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                                cfg.record.output = target;
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
            // Video Capture Settings Card (FPS, Resolution, Quality, Container)
            .child(
                card(&theme)
                    .child(control_row(
                        &text("settings.record_fps_title", cx),
                        Some(&text("settings.record_fps_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            fps_options.into_iter().enumerate().map(|(idx, fps)| {
                                let is_selected = config.record.fps == fps;
                                pill_button(
                                    ElementId::NamedInteger("fps-pill".into(), idx as u64),
                                    &format!("{fps} FPS"),
                                    is_selected,
                                    &theme,
                                    cx.listener(move |_this, _, _, cx| {
                                        if cx.has_global::<AppState>() {
                                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                                cfg.record.fps = fps;
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
                        &text("settings.record_resolution_title", cx),
                        Some(&text("settings.record_resolution_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            res_options
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (val, label))| {
                                    let is_selected = config.record.resolution == val;
                                    let val_str = val.to_string();
                                    pill_button(
                                        ElementId::NamedInteger("res-pill".into(), idx as u64),
                                        &label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = val_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.record.resolution = target;
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
                        &text("settings.record_quality_title", cx),
                        Some(&text("settings.record_quality_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            quality_options
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (val, label))| {
                                    let is_selected = config.record.quality == val;
                                    let val_str = val.to_string();
                                    pill_button(
                                        ElementId::NamedInteger("quality-pill".into(), idx as u64),
                                        &label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = val_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.record.quality = target;
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
                        &text("settings.record_container_title", cx),
                        Some(&text("settings.record_container_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            container_options
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (val, label))| {
                                    let is_selected = config.record.container == val;
                                    let val_str = val.to_string();
                                    pill_button(
                                        ElementId::NamedInteger(
                                            "container-pill".into(),
                                            idx as u64,
                                        ),
                                        label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = val_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.record.container = target;
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
            // Audio & Cursor Card
            .child(
                card(&theme)
                    .child(control_row(
                        &text("settings.record_audio_title", cx),
                        Some(&text("settings.record_audio_subtitle", cx)),
                        div().flex().gap(px(6.0)).children(
                            audio_options
                                .into_iter()
                                .enumerate()
                                .map(|(idx, (val, label))| {
                                    let is_selected = config.record.audio == val;
                                    let val_str = val.to_string();
                                    pill_button(
                                        ElementId::NamedInteger("audio-pill".into(), idx as u64),
                                        &label,
                                        is_selected,
                                        &theme,
                                        cx.listener(move |_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let target = val_str.clone();
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.record.audio = target;
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
                    .child(toggle_row(
                        ElementId::Name("record-cursor-toggle".into()),
                        &text("settings.record_cursor_title", cx),
                        Some(&text("settings.record_cursor_subtitle", cx)),
                        config.record.include_cursor,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.record.include_cursor = !cfg.record.include_cursor;
                                });
                            }
                            cx.notify();
                        }),
                    )),
            )
            // Engine Backend info badge
            .child(
                card(&theme).child(control_row(
                    &text("settings.record_backend_title", cx),
                    Some(&text("settings.record_backend_subtitle", cx)),
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(6.0))
                        .px(px(12.0))
                        .py(px(5.0))
                        .rounded_full()
                        .bg(theme.accent().opacity(0.12))
                        .border_1()
                        .border_color(theme.accent().opacity(0.3))
                        .child(div().size(px(6.0)).rounded_full().bg(theme.accent()))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme.accent())
                                .child("gpu-screen-recorder (GPU)"),
                        ),
                    &theme,
                )),
            )
            .into_any_element()
    }
}
