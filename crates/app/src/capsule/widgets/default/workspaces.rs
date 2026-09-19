use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px, svg};
use services::WorkspaceInfo;
use ui::theme::Theme;

use crate::capsule::modules::default::{DefaultEvent, DefaultModule};

pub fn render_workspaces_widget(
    active_workspace: &WorkspaceInfo,
    special_progress: f32,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> AnyElement {
    let current_num = active_workspace.num.max(1);
    let slot4 = if current_num > 4 { current_num } else { 4 };
    let normal_slots = [1, 2, 3, slot4];

    let mut normal_items = Vec::new();
    for &num in &normal_slots {
        let is_active = !active_workspace.is_special && (num == current_num);
        let ws_id = num as i64;

        if is_active {
            let icon_path = if num == 4 {
                "pacman-left.svg"
            } else {
                "pacman.svg"
            };
            normal_items.push(
                div()
                    .id(("ws-pacman", num as u32))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(14.0))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _, _, cx| {
                        cx.emit(DefaultEvent::WorkspaceClicked(ws_id));
                    }))
                    .child(
                        svg()
                            .path(icon_path)
                            .size(px(13.0))
                            .text_color(theme.accent()),
                    )
                    .into_any_element(),
            );
        } else {
            let inactive_color = theme.foreground_muted().opacity(0.35);
            let hover_color = theme.foreground_muted().opacity(0.75);
            normal_items.push(
                div()
                    .id(("ws-dot-btn", num as u32))
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(14.0))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _, _, cx| {
                        cx.emit(DefaultEvent::WorkspaceClicked(ws_id));
                    }))
                    .child(
                        div()
                            .size(px(5.0))
                            .rounded_full()
                            .bg(inactive_color)
                            .hover(move |style| style.bg(hover_color)),
                    )
                    .into_any_element(),
            );
        }
    }

    let special_id = active_workspace.id;
    let special_item = div()
        .id("ws-special-pacman")
        .flex()
        .items_center()
        .justify_center()
        .size(px(14.0))
        .cursor_pointer()
        .on_click(cx.listener(move |_this, _, _, cx| {
            cx.emit(DefaultEvent::WorkspaceClicked(special_id));
        }))
        .child(
            svg()
                .path("pacman.svg")
                .size(px(13.0))
                .text_color(theme.accent()),
        );

    let p = special_progress.clamp(0.0, 1.0);
    let normal_top = p * 22.0;
    let normal_opacity = (1.0 - p).clamp(0.0, 1.0);
    let special_top = (1.0 - p) * -22.0;
    let special_opacity = p.clamp(0.0, 1.0);
    let dock_w = 58.0 + (30.0 - 58.0) * p;

    let normal_layer = div()
        .id("workspaces-normal-layer")
        .absolute()
        .top(px(normal_top))
        .left_0()
        .right_0()
        .h(px(22.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(6.0))
        .opacity(normal_opacity)
        .children(normal_items);

    let special_layer = div()
        .id("workspaces-special-layer")
        .absolute()
        .top(px(special_top))
        .left_0()
        .right_0()
        .h(px(22.0))
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .opacity(special_opacity)
        .child(special_item);

    div()
        .id("default-workspaces-dock")
        .flex()
        .flex_shrink_0()
        .items_center()
        .justify_center()
        .h(px(22.0))
        .w(px(dock_w))
        .rounded(px(7.0))
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.2))
        .overflow_hidden()
        .relative()
        .child(normal_layer)
        .child(special_layer)
        .into_any_element()
}
