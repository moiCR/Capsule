use std::time::Duration;

use gpui::{
    BoxShadow, ColorExt, Context, IntoElement, MotionDurationExt, div, ease_in_out, point,
    prelude::*, px,
};
use services::{AppState, WorkspaceInfo};
use ui::theme::Theme;

use super::WORKSPACES_WIDTH;
use crate::new_capsule::module::default::DefaultModule;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceKey {
    pub id: Option<i64>,
    pub active: bool,
}

pub fn workspace_keys(current: &WorkspaceInfo, workspaces: &[WorkspaceInfo]) -> [WorkspaceKey; 6] {
    std::array::from_fn(|index| {
        if index == 5 && (current.is_special || current.num > 6) {
            return WorkspaceKey {
                id: Some(current.id),
                active: true,
            };
        }
        let number = index as i32 + 1;
        let active = !current.is_special && current.num.max(1) == number;
        let id = if active {
            Some(current.id)
        } else {
            workspaces
                .iter()
                .find(|workspace| !workspace.is_special && workspace.num == number)
                .map(|workspace| workspace.id)
        };
        WorkspaceKey { id, active }
    })
}

pub fn render_workspaces(
    keys: &[WorkspaceKey; 6],
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> impl IntoElement {
    div()
        .w(px(WORKSPACES_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(3.0))
        .children(keys.iter().enumerate().map(|(index, key)| {
            let id = key.id;
            let width = if key.active { 8.0 } else { 5.0 };
            let height = if key.active { 16.0 } else { 10.0 };
            let color = if key.active {
                theme.accent()
            } else {
                theme.foreground_muted().opacity(0.5)
            };
            div()
                .id(("workspace-key", index))
                .w(px(10.0))
                .h_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .when(id.is_some(), |element| element.cursor_pointer())
                .on_click(cx.listener(move |_, _, _, cx| {
                    if let Some(id) = id {
                        let compositor = cx.global::<AppState>().compositor.clone();
                        services::spawn_tokio(async move {
                            let _ = tokio::task::spawn_blocking(move || {
                                compositor.switch_workspace(id)
                            })
                            .await;
                        });
                    }
                }))
                .child(
                    div()
                        .id(("workspace-oval", index))
                        .relative()
                        .w(px(width))
                        .h(px(height))
                        .rounded_full()
                        .bg(color)
                        .transitions(|transitions| {
                            transitions
                                .w(duration.with_easing(ease_in_out))
                                .h(duration.with_easing(ease_in_out))
                                .bg(duration.with_easing(ease_in_out))
                        })
                        .child(
                            div()
                                .id(("workspace-glow", index))
                                .absolute()
                                .inset_0()
                                .rounded_full()
                                .bg(theme.accent())
                                .opacity(if key.active { 1.0 } else { 0.0 })
                                .shadow(vec![BoxShadow {
                                    color: theme.accent().opacity(0.4).into(),
                                    offset: point(px(0.0), px(0.0)),
                                    blur_radius: px(4.0),
                                    spread_radius: px(1.0),
                                    inset: false,
                                }])
                                .transitions(|transitions| {
                                    transitions.opacity(duration.with_easing(ease_in_out))
                                }),
                        ),
                )
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(num: i32, id: i64, special: bool) -> WorkspaceInfo {
        WorkspaceInfo {
            num,
            id,
            is_special: special,
            ..WorkspaceInfo::default()
        }
    }

    #[test]
    fn keys_use_real_ids_and_replace_last_with_high_or_special_workspace() {
        let existing = vec![workspace(1, 101, false), workspace(2, 202, false)];
        let keys = workspace_keys(&existing[1], &existing);
        assert_eq!(keys[0].id, Some(101));
        assert!(keys[1].active);
        assert_eq!(keys[2].id, None);
        for current in [workspace(8, 808, false), workspace(-1, -99, true)] {
            let keys = workspace_keys(&current, &existing);
            assert_eq!(
                keys[5],
                WorkspaceKey {
                    id: Some(current.id),
                    active: true
                }
            );
            assert_eq!(keys.iter().filter(|key| key.active).count(), 1);
            assert_eq!(keys[0].id, Some(101));
        }
    }
}
