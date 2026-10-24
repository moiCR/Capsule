use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, div, px, svg};
use services::{AppState, WorkspaceInfo};
use ui::theme::Theme;

use crate::bar::{Bar, style};

pub fn render(
    current: &WorkspaceInfo,
    workspaces: &[WorkspaceInfo],
    theme: &Theme,
    _cx: &mut Context<Bar>,
) -> AnyElement {
    let mut pills = div()
        .id("bar-widget-workspaces")
        .flex()
        .items_center()
        .gap(px(4.0))
        .h(px(style::WIDGET_HEIGHT))
        .px(px(4.0));

    let max_workspace = workspaces
        .iter()
        .map(|w| w.num)
        .max()
        .unwrap_or(1)
        .max(current.num)
        .max(4);

    for num in 1..=max_workspace.min(10) {
        let is_active = !current.is_special && current.num == num;
        let is_occupied = workspaces.iter().any(|w| !w.is_special && w.num == num);
        let hover_bg = style::hover(theme);

        let mut pill = div()
            .id(("bar-ws", num as usize))
            .size(px(24.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg));

        if is_active {
            pill = pill.child(
                svg()
                    .path("pacman.svg")
                    .size(px(17.0))
                    .text_color(theme.accent()),
            );
        } else if is_occupied {
            pill = pill.child(
                svg()
                    .path("ghost.svg")
                    .size(px(16.0))
                    .text_color(theme.foreground().opacity(0.85)),
            );
        } else {
            pill = pill.child(
                div()
                    .size(px(6.0))
                    .rounded_full()
                    .bg(theme.foreground_muted().opacity(0.45)),
            );
        }

        pill = pill.on_click(move |_, _, cx| {
            if let Some(state) = cx.try_global::<AppState>() {
                state.compositor.switch_workspace(num as i64);
            }
        });

        pills = pills.child(pill);
    }

    pills.into_any_element()
}
