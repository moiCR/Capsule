use std::time::Duration;

use chrono::Local;
use gpui::{
    Context, EventEmitter, IntoElement, Pixels, Render, Size, Subscription, Task, Window, div,
    prelude::*, px,
};
use services::AppState;
use ui::theme::Theme;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::default::{
    GAP, PADDING, StatusSnapshot, WorkspaceKey, module_size, render_clock, render_status,
    render_workspaces, workspace_keys,
};

pub struct DefaultModule {
    time: String,
    keys: [WorkspaceKey; 6],
    status: StatusSnapshot,
    height: f32,
    animation_duration: Duration,
    _theme_subscription: Subscription,
    _refresh_task: Task<()>,
    _workspace_task: Task<()>,
    workspace_service_task: tokio::task::JoinHandle<()>,
}

impl DefaultModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = cx.global::<AppState>();
        let compositor = state.compositor.clone();
        let initial = compositor.get_workspace();
        let config = state.config.get();
        let status = StatusSnapshot::new(&state.network.get_status(), state.power.get_battery());
        let height = config.ui.idle_height;
        let animation_duration = Duration::from_millis(config.ui.animation_duration_ms as u64);
        let mut changes = compositor.on_change_workspace();
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let workspace_service_task = services::spawn_tokio(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let mut last_current = compositor.get_workspace();
            let mut last_workspaces = compositor.get_workspaces();
            let _ = sender.send((last_current.clone(), last_workspaces.clone()));

            loop {
                let mut should_send = false;
                tokio::select! {
                    change = changes.recv() => {
                        match change {
                            Ok(ws) => {
                                last_current = ws;
                                last_workspaces = compositor.get_workspaces();
                                should_send = true;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                                last_current = compositor.get_workspace();
                                last_workspaces = compositor.get_workspaces();
                                should_send = true;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                changes = compositor.on_change_workspace();
                                last_current = compositor.get_workspace();
                                last_workspaces = compositor.get_workspaces();
                                should_send = true;
                            }
                        }
                    }
                    _ = interval.tick() => {
                        let current = compositor.get_workspace();
                        let workspaces = compositor.get_workspaces();
                        if current != last_current || workspaces != last_workspaces {
                            last_current = current;
                            last_workspaces = workspaces;
                            should_send = true;
                        }
                    }
                }

                if should_send && sender.send((last_current.clone(), last_workspaces.clone())).is_err() {
                    break;
                }
            }
        });

        let workspace_task = cx.spawn(async move |this, cx| {
            while let Some((current, workspaces)) = receiver.recv().await {
                let keys = workspace_keys(&current, &workspaces);
                if this
                    .update(cx, |module: &mut Self, cx| {
                        if module.keys != keys {
                            module.keys = keys;
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        let refresh_task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(250))
                    .await;
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let state = cx.global::<AppState>();
                        let status = StatusSnapshot::new(
                            &state.network.get_status(),
                            state.power.get_battery(),
                        );
                        let config = state.config.get();
                        let height = config.ui.idle_height;
                        let duration =
                            Duration::from_millis(config.ui.animation_duration_ms as u64);
                        let time = Local::now().format("%H:%M").to_string();
                        let resized = module.height != height;
                        let changed = resized
                            || module.animation_duration != duration
                            || module.status != status
                            || module.time != time;
                        module.height = height;
                        module.animation_duration = duration;
                        module.time = time;
                        module.status = status;
                        if resized {
                            cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Default));
                        }
                        if changed {
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        let theme_subscription = cx.observe_global::<Theme>(|_, cx| cx.notify());
        Self {
            _theme_subscription: theme_subscription,
            time: Local::now().format("%H:%M").to_string(),
            keys: workspace_keys(&initial, &[]),
            status,
            height,
            animation_duration,
            _refresh_task: refresh_task,
            _workspace_task: workspace_task,
            workspace_service_task,
        }
    }
}

impl Drop for DefaultModule {
    fn drop(&mut self) {
        self.workspace_service_task.abort();
    }
}

impl CapsuleModule for DefaultModule {
    fn size(&self) -> Size<Pixels> {
        module_size(self.height)
    }
}

impl EventEmitter<CapsuleModuleEvent> for DefaultModule {}

impl Render for DefaultModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .id("default-module")
            .size_full()
            .flex()
            .items_center()
            .font_family(theme.font_family())
            .px(px(PADDING))
            .gap(px(GAP))
            .child(render_workspaces(
                &self.keys,
                self.animation_duration,
                &theme,
                cx,
            ))
            .child(render_clock(&self.time, &theme))
            .child(render_status(&self.status, &theme))
    }
}
