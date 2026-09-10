use gpui::{
    Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Styled, div,
    prelude::*, px,
};
use services::AppState;
use ui::theme::Theme;

use super::setting_item::{
    render_card_container, render_control_row, render_hero_header, render_row_divider,
    render_toggle_row,
};
use crate::capsule::modules::settings::SettingsModule;

pub fn render_record_section(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (
        hero_title,
        hero_sub,
        fps_title,
        fps_sub,
        res_title,
        res_sub,
        quality_title,
        quality_sub,
        container_title,
        container_sub,
        audio_title,
        audio_sub,
        cursor_title,
        cursor_sub,
        backend_title,
        backend_sub,
        output_title,
        output_sub,
    ) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.record_header_title"),
            lang.get("settings.record_header_subtitle"),
            lang.get("settings.record_fps_title"),
            lang.get("settings.record_fps_subtitle"),
            lang.get("settings.record_resolution_title"),
            lang.get("settings.record_resolution_subtitle"),
            lang.get("settings.record_quality_title"),
            lang.get("settings.record_quality_subtitle"),
            lang.get("settings.record_container_title"),
            lang.get("settings.record_container_subtitle"),
            lang.get("settings.record_audio_title"),
            lang.get("settings.record_audio_subtitle"),
            lang.get("settings.record_cursor_title"),
            lang.get("settings.record_cursor_subtitle"),
            lang.get("settings.record_backend_title"),
            lang.get("settings.record_backend_subtitle"),
            lang.get("settings.record_output_title"),
            lang.get("settings.record_output_subtitle"),
        )
    } else {
        (
            "Grabación de Pantalla".to_string(),
            "Configura la tasa de cuadros (FPS), resolución, calidad del video y fuente de audio para gpu-screen-recorder.".to_string(),
            "Cuadros por Segundo (FPS)".to_string(),
            "Fluidez de captura de la grabación de pantalla.".to_string(),
            "Resolución".to_string(),
            "Escala de captura del monitor seleccionado.".to_string(),
            "Calidad de Video".to_string(),
            "Tasa de bits y compresión del codificador de video.".to_string(),
            "Formato de Contenedor".to_string(),
            "Formato de archivo final (MP4 o MKV).".to_string(),
            "Fuente de Audio".to_string(),
            "Audio capturado por defecto durante la grabación.".to_string(),
            "Capturar Cursor".to_string(),
            "Incluir el puntero del ratón en la grabación.".to_string(),
            "Motor de Grabación".to_string(),
            "Backend acelerado por GPU (gpu-screen-recorder).".to_string(),
            "Salida de Video".to_string(),
            "Monitor o pantalla capturada en la grabación.".to_string(),
        )
    };

    div()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .child(render_hero_header(
            "play.svg",
            &hero_title,
            &hero_sub,
            theme,
        ))
        .child(render_card_container(theme).child(render_control_row(
            &output_title,
            Some(&output_sub),
            render_output_selector(
                &module.record_output,
                &module.record_available_monitors,
                theme,
                cx,
            ),
            theme,
        )))
        .child(
            render_card_container(theme)
                .child(render_control_row(
                    &fps_title,
                    Some(&fps_sub),
                    render_fps_selector(module.record_fps, theme, cx),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &res_title,
                    Some(&res_sub),
                    render_resolution_selector(&module.record_resolution, theme, cx),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &quality_title,
                    Some(&quality_sub),
                    render_quality_selector(&module.record_quality, theme, cx),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_control_row(
                    &container_title,
                    Some(&container_sub),
                    render_container_selector(&module.record_container, theme, cx),
                    theme,
                )),
        )
        .child(
            render_card_container(theme)
                .child(render_control_row(
                    &audio_title,
                    Some(&audio_sub),
                    render_audio_selector(&module.record_audio, theme, cx),
                    theme,
                ))
                .child(render_row_divider(theme))
                .child(render_toggle_row(
                    ElementId::Name("record-cursor-toggle".into()),
                    &cursor_title,
                    Some(&cursor_sub),
                    module.record_include_cursor,
                    theme,
                    cx.listener(|this, _, _, cx| {
                        this.toggle_record_include_cursor(cx);
                    }),
                )),
        )
        .child(
            render_card_container(theme).child(render_control_row(
                &backend_title,
                Some(&backend_sub),
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .px_2p5()
                    .py_1()
                    .rounded_full()
                    .bg(theme.accent().opacity(0.15))
                    .border_1()
                    .border_color(theme.accent().opacity(0.3))
                    .child(
                        div()
                            .w(px(6.0))
                            .h(px(6.0))
                            .rounded_full()
                            .bg(theme.accent()),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.accent())
                            .child("gpu-screen-recorder (GPU)"),
                    ),
                theme,
            )),
        )
}

pub(crate) fn render_fps_selector(
    current_fps: u32,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let options = [30, 60, 120];
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for fps in options {
        let is_selected = current_fps == fps;
        let fps_val = fps;

        row = row.child(
            div()
                .id(ElementId::NamedInteger("fps-btn".into(), fps as u64))
                .px_2p5()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_fps(fps_val, cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(format!("{fps} FPS")),
                ),
        );
    }

    row
}

pub(crate) fn render_resolution_selector(
    current_res: &str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let native_label = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .language
            .get("settings.record_res_native")
    } else {
        "Nativa".to_string()
    };
    let options = [
        ("native", native_label),
        ("1920x1080", "1080p".to_string()),
        ("1280x720", "720p".to_string()),
    ];
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for (val, label) in options {
        let is_selected = current_res == val;
        let val_str = val.to_string();

        row = row.child(
            div()
                .id(ElementId::Name(format!("res-btn-{val}").into()))
                .px_2p5()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_resolution(val_str.clone(), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(label),
                ),
        );
    }

    row
}

pub(crate) fn render_quality_selector(
    current_quality: &str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (ultra, very_high, high, medium) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.record_quality_ultra"),
            lang.get("settings.record_quality_very_high"),
            lang.get("settings.record_quality_high"),
            lang.get("settings.record_quality_medium"),
        )
    } else {
        (
            "Ultra".to_string(),
            "Muy Alta".to_string(),
            "Alta".to_string(),
            "Media".to_string(),
        )
    };
    let options = [
        ("ultra", ultra),
        ("very_high", very_high),
        ("high", high),
        ("medium", medium),
    ];
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for (val, label) in options {
        let is_selected = current_quality == val;
        let val_str = val.to_string();

        row = row.child(
            div()
                .id(ElementId::Name(format!("quality-btn-{val}").into()))
                .px_2p5()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_quality(val_str.clone(), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(label),
                ),
        );
    }

    row
}

pub(crate) fn render_container_selector(
    current_container: &str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let options = [("mp4", "MP4"), ("mkv", "MKV")];
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for (val, label) in options {
        let is_selected = current_container == val;
        let val_str = val.to_string();

        row = row.child(
            div()
                .id(ElementId::Name(format!("container-btn-{val}").into()))
                .px_3()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_container(val_str.clone(), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(label),
                ),
        );
    }

    row
}

pub(crate) fn render_audio_selector(
    current_audio: &str,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let (desktop, mic, both, none) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (
            lang.get("settings.record_audio_desktop"),
            lang.get("settings.record_audio_mic"),
            lang.get("settings.record_audio_both"),
            lang.get("settings.record_audio_none"),
        )
    } else {
        (
            "Escritorio".to_string(),
            "Micrófono".to_string(),
            "Ambos".to_string(),
            "Ninguno".to_string(),
        )
    };
    let options = [
        ("desktop", desktop),
        ("mic", mic),
        ("both", both),
        ("none", none),
    ];
    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for (val, label) in options {
        let is_selected = current_audio == val;
        let val_str = val.to_string();

        row = row.child(
            div()
                .id(ElementId::Name(format!("audio-btn-{val}").into()))
                .px_2p5()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_audio(val_str.clone(), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(label),
                ),
        );
    }

    row
}

pub(crate) fn render_output_selector(
    current_output: &str,
    available_monitors: &[String],
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let screen_label = if cx.has_global::<AppState>() {
        cx.global::<AppState>()
            .language
            .get("settings.record_output_fullscreen")
    } else {
        "Pantalla Completa".to_string()
    };
    let mut options = vec![("screen".to_string(), screen_label)];
    for m in available_monitors {
        if m != "screen" {
            options.push((m.clone(), m.clone()));
        }
    }

    let mut row = div().flex().flex_row().items_center().gap(px(4.0));

    for (val, label) in options {
        let is_selected = current_output == val;
        let val_str = val.clone();

        row = row.child(
            div()
                .id(ElementId::Name(format!("output-btn-{val}").into()))
                .px_2p5()
                .py_1()
                .rounded_full()
                .bg(if is_selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.4)
                })
                .hover(|s| {
                    if is_selected {
                        s
                    } else {
                        s.bg(theme.surface().opacity(0.7))
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.set_record_output(val_str.clone(), cx);
                }))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_weight(if is_selected {
                            FontWeight::BOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_selected {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .child(label),
                ),
        );
    }

    row
}
