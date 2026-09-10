use gpui::prelude::FluentBuilder;
use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, MouseButton, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px, svg,
};
use services::{AppState, RecordStatus};
use ui::theme::Theme;

use crate::capsule::modules::record::RecordModule;

pub fn render_record_bar(
    status: RecordStatus,
    duration_str: impl Into<SharedString>,
    cx: &mut Context<RecordModule>,
) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let is_recording = status == RecordStatus::Recording;
    let is_paused = status == RecordStatus::Paused;

    let (lbl_record, lbl_paused) = if cx.has_global::<AppState>() {
        let lang = &cx.global::<AppState>().language;
        (lang.get("record.record_screen"), lang.get("record.paused"))
    } else {
        ("Grabar pantalla".to_string(), "Pausa".to_string())
    };

    let duration: SharedString = duration_str.into();

    let surface_btn = theme.surface().opacity(0.45);
    let surface_btn_hover = theme.surface().opacity(0.75);
    let fg = theme.foreground();
    let fg_muted = theme.foreground_muted();

    div()
        .size_full()
        .when(status == RecordStatus::Stopped, |this| {
            this.child(
                div()
                    .id("record-trigger")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .px(px(20.0))
                    .gap(px(9.0))
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start_recording(cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, _, _, cx| {
                            this.close(cx);
                        }),
                    )
                    .child(div().size(px(10.0)).rounded_full().bg(gpui::rgb(0xef4444)))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(fg)
                            .child(lbl_record),
                    ),
            )
        })
        .when(status != RecordStatus::Stopped, |this| {
            this.child(
                div()
                    .id("record-active-bar")
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .size_full()
                    .px(px(16.0))
                    .gap(px(20.0))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, _, _, cx| {
                            this.close(cx);
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().size(px(8.0)).rounded_full().bg(if is_recording {
                                gpui::rgb(0xef4444)
                            } else {
                                gpui::rgb(0xf59e0b)
                            }))
                            .child(
                                div()
                                    .whitespace_nowrap()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(fg)
                                    .child(duration.to_string()),
                            )
                            .when(is_paused, |this| {
                                this.child(
                                    div()
                                        .whitespace_nowrap()
                                        .text_size(px(11.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(fg_muted)
                                        .child(lbl_paused),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .id("record-pause-btn")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(26.0))
                                    .rounded_full()
                                    .bg(surface_btn)
                                    .hover(move |s| s.bg(surface_btn_hover))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.toggle_pause(cx);
                                    }))
                                    .child(
                                        svg()
                                            .path(if is_paused { "play.svg" } else { "pause.svg" })
                                            .size(px(12.0))
                                            .text_color(fg),
                                    ),
                            )
                            .child(
                                div()
                                    .id("record-stop-btn")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(26.0))
                                    .rounded_full()
                                    .bg(gpui::rgb(0xef4444))
                                    .hover(|s| s.bg(gpui::rgb(0xdc2626)))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.stop_recording(cx);
                                    }))
                                    .child(div().size(px(8.5)).rounded(px(1.5)).bg(gpui::white())),
                            ),
                    ),
            )
        })
}
