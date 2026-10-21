use super::{
    default::{render_status, render_workspaces},
    style,
};
use crate::capsule::module::{
    CapsuleModuleEvent, CapsuleModuleId, default_expanded::DefaultExpandedModule,
};
use gpui::{Context, FontWeight, IntoElement, StyledImage, div, img, prelude::*, px, svg};
use services::AppState;
use std::{path::PathBuf, time::Duration};
use ui::theme::Theme;

pub const WIDTH: f32 = 700.0;
pub const HEIGHT: f32 = 140.0;

pub fn render(
    module: &DefaultExpandedModule,
    cx: &mut Context<DefaultExpandedModule>,
) -> gpui::Stateful<gpui::Div> {
    let theme = cx.global::<Theme>().clone();
    let state = cx.global::<AppState>();
    let duration = Duration::from_millis(state.config.get().ui.animation_duration_ms as u64);
    let snapshot = &module.snapshot;
    let notification_label = if snapshot.dnd {
        state.language.get("dashboard.dnd")
    } else if snapshot.notifications == 0 {
        state.language.get("dashboard.no_notifications")
    } else {
        format!(
            "{} {}",
            snapshot.notifications,
            state.language.get("notifications")
        )
    };
    let no_media = state.language.get("dashboard.no_media");
    let hover = style::hover(&theme);
    let app_icon = snapshot.icon.as_ref().map_or_else(
        || {
            svg()
                .path("grid-dots.svg")
                .size(px(16.0))
                .text_color(theme.accent())
                .into_any_element()
        },
        |path| {
            img(path.clone())
                .size(px(16.0))
                .object_fit(gpui::ObjectFit::Contain)
                .into_any_element()
        },
    );
    let left = div()
        .w(px(164.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .min_w_0()
                .child(app_icon)
                .child(
                    div()
                        .min_w_0()
                        .text_ellipsis()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .child(snapshot.application.clone()),
                ),
        )
        .child(div().h(px(20.0)).child(render_workspaces(
            &snapshot.keys,
            snapshot.special,
            duration,
            &theme,
            cx,
        )));
    let clock = div()
        .id("collapse-default")
        .flex()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .h(px(30.0))
        .cursor_pointer()
        .on_click(cx.listener(|_, _, _, cx| cx.emit(CapsuleModuleEvent::Close)))
        .child(
            div()
                .text_size(px(24.0))
                .font_weight(FontWeight::MEDIUM)
                .child(snapshot.time.clone()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(snapshot.date.clone()),
        );
    let mut music = div()
        .id("expanded-media")
        .w_full()
        .h(px(54.0))
        .px(px(10.0))
        .rounded(px(16.0))
        .bg(style::surface(&theme))
        .flex()
        .items_center()
        .gap(px(10.0))
        .overflow_hidden();
    if let Some(track) = &snapshot.track {
        let cover = div()
            .size(px(34.0))
            .flex_shrink_0()
            .rounded_full()
            .overflow_hidden()
            .bg(style::raised(&theme))
            .flex()
            .items_center()
            .justify_center();
        let cover = if let Some(path) = &track.local_art_path {
            cover.child(
                img(PathBuf::from(path))
                    .size_full()
                    .rounded_full()
                    .object_fit(gpui::ObjectFit::Cover),
            )
        } else {
            cover.child(
                svg()
                    .path("music.svg")
                    .size(px(18.0))
                    .text_color(theme.accent()),
            )
        };
        music = music
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .on_click(cx.listener(|this, _, _, _| this.toggle_playback()))
            .child(cover)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_ellipsis()
                            .child(track.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(theme.foreground_muted())
                            .text_ellipsis()
                            .child(track.artist.clone()),
                    ),
            )
            .child(
                div()
                    .w(px(57.0))
                    .h(px(20.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(3.0))
                    .children(module.audio_levels.iter().map(|level| {
                        div()
                            .w(px(2.0))
                            .h(px(2.0 + level * 18.0))
                            .rounded_full()
                            .bg(theme.accent())
                    })),
            );
    } else {
        music = music
            .justify_center()
            .text_size(px(12.0))
            .text_color(theme.foreground_muted())
            .child(no_media);
    }
    let center = div()
        .w(px(280.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(10.0))
        .child(clock)
        .child(music);
    let right = div()
        .w(px(164.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_end()
        .gap(px(10.0))
        .child(
            div()
                .id("expanded-notifications")
                .w_full()
                .flex()
                .justify_end()
                .items_center()
                .gap(px(8.0))
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    services::push_ipc_command(services::IpcCommand::ShowNotification);
                    cx.stop_propagation();
                }))
                .child(
                    div()
                        .min_w_0()
                        .text_size(px(12.0))
                        .text_ellipsis()
                        .child(notification_label),
                )
                .child(
                    svg()
                        .path("bell.svg")
                        .size(px(14.0))
                        .flex_shrink_0()
                        .text_color(theme.accent()),
                ),
        )
        .child(
            div()
                .id("expanded-system-status")
                .cursor_pointer()
                .on_click(cx.listener(|_, _, _, cx| {
                    cx.emit(CapsuleModuleEvent::Open(CapsuleModuleId::Dashboard))
                }))
                .child(render_status(&snapshot.status, &theme)),
        );
    div()
        .id("default-expanded-module")
        .size_full()
        .px(px(28.0))
        .flex()
        .items_center()
        .gap(px(18.0))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(left)
        .child(center)
        .child(right)
}
