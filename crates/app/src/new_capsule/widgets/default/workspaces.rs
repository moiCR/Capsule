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
        if index == 5 && (!current.is_special && current.num > 6) {
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
    special: bool,
    duration: Duration,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> impl IntoElement {
    let mut opposite = theme.accent();
    opposite.color.hue += 180.0;
    let row = div()
        .h(px(16.0))
        .flex()
        .items_center()
        .gap(px(2.0))
        .children(keys.iter().enumerate().map(|(index, key)| {
            let id = key.id;
            let width = if key.active { 8.0 } else { 5.0 };
            let height = if key.active { 16.0 } else { 10.0 };
            let color = if key.active {
                theme.accent()
            } else {
                theme.accent().opacity(0.4)
            };
            div()
                .id(("workspace-key", index))
                .w(px(width))
                .transitions(|t| t.w(duration.with_easing(ease_in_out)))
                .h_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_click(cx.listener(move |_, _, _, cx| {
                    let target_id = id.unwrap_or(index as i64 + 1);
                    cx.global::<AppState>()
                        .compositor
                        .switch_workspace(target_id);
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
        }));
    div()
        .w(px(WORKSPACES_WIDTH))
        .h_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .justify_center()
        .items_start()
        .child(row)
        .child(
            div()
                .id("special-workspace-indicator")
                .w(px(40.0))
                .h(px(if special { 4.0 } else { 0.0 }))
                .mt(px(if special { 4.0 } else { 0.0 }))
                .rounded_full()
                .bg(opposite)
                .opacity(if special { 1.0 } else { 0.0 })
                .transitions(|t| {
                    t.h(duration.with_easing(ease_in_out))
                        .mt(duration.with_easing(ease_in_out))
                        .opacity(duration.with_easing(ease_in_out))
                }),
        )
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
    fn special_workspace_leaves_all_normal_slots_unselected() {
        let existing = vec![workspace(1, 101, false), workspace(6, 606, false)];
        let keys = workspace_keys(&workspace(-1, -99, true), &existing);
        assert!(keys.iter().all(|key| !key.active));
        assert_eq!(keys[0].id, Some(101));
        assert_eq!(keys[5].id, Some(606));
        assert!(!keys.iter().any(|key| key.id == Some(-99)));
    }

    #[test]
    fn keys_use_real_ids_and_replace_last_with_high_workspace() {
        let existing = vec![workspace(1, 101, false), workspace(2, 202, false)];
        let keys = workspace_keys(&existing[1], &existing);
        assert_eq!(keys[0].id, Some(101));
        assert!(keys[1].active);
        assert_eq!(keys[2].id, None);
        {
            let current = workspace(8, 808, false);
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
