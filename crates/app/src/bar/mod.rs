pub mod ipc_handler;
pub mod popups;
pub mod shared_panel;
pub mod style;
pub mod widgets;
pub mod window_state;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    AnyWindowHandle, ColorExt, Context, FocusHandle, IntoElement, Pixels, Render, Subscription,
    Task, Window, div, px,
};
use services::{AppState, WorkspaceInfo};
use ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarLocation {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl BarLocation {
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
}

pub struct Bar {
    pub window_state: window_state::WindowState,
    pub location: BarLocation,
    pub height: Pixels,
    pub current_workspace: WorkspaceInfo,
    pub workspaces: Vec<WorkspaceInfo>,
    pub clock_center: Rc<Cell<Pixels>>,
    pub status_center: Rc<Cell<Pixels>>,
    pub shared_panel: shared_panel::SharedPanelManager,
    pub launcher: gpui::Entity<popups::launcher::BarLauncher>,
    pub polkit: gpui::Entity<popups::polkit::BarPolkit>,
    pub focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
    _clock_task: Option<Task<()>>,
    _config_task: Option<Task<()>>,
    _workspace_task: Option<Task<()>>,
    _polkit_task: Option<Task<()>>,
    _notification_task: Option<Task<()>>,
}

impl Bar {
    pub fn new(
        location: BarLocation,
        window_handle: AnyWindowHandle,
        cx: &mut Context<Self>,
    ) -> Self {
        let sub = cx.observe_global::<Theme>(|_, cx| cx.notify());

        let (cur_ws, all_ws) = if let Some(state) = cx.try_global::<AppState>() {
            (
                state.compositor.get_workspace(),
                state.compositor.get_workspaces(),
            )
        } else {
            (WorkspaceInfo::default(), Vec::new())
        };

        let clock_task = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this.update(cx, |_, cx| cx.notify()).is_err() {
                    break;
                }
            }
        });

        let config_task = if cx.has_global::<AppState>() {
            let mut rx = cx.global::<AppState>().config.subscribe();
            Some(cx.spawn(async move |this, cx| {
                while rx.recv().await.is_ok() {
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        let workspace_task = if cx.has_global::<AppState>() {
            let compositor = cx.global::<AppState>().compositor.clone();
            let mut ws_changes = compositor.on_change_workspace();
            Some(cx.spawn(async move |this, cx| {
                while let Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) =
                    ws_changes.recv().await
                {
                    let cur = compositor.get_workspace();
                    let all = compositor.get_workspaces();
                    if this
                        .update(cx, |bar, cx| {
                            bar.current_workspace = cur;
                            bar.workspaces = all;
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }))
        } else {
            None
        };

        let polkit_task = if cx.has_global::<AppState>() {
            let service = cx.global::<AppState>().polkit.clone();
            let bar_window = window_handle.downcast::<Bar>();
            Some(cx.spawn(async move |_this, cx| {
                loop {
                    service.wait_for_change().await;
                    if let Some(bar_window) = bar_window {
                        cx.update(|app| {
                            let _ = bar_window.update(app, |bar, window, cx| {
                                bar.check_polkit_requests(window, cx);
                            });
                        });
                    }
                }
            }))
        } else {
            None
        };

        let mut notif_rx = services::NotificationStore::global().subscribe();
        let bar_window = window_handle.downcast::<Bar>();
        let notification_task = Some(cx.spawn(async move |_this, cx| {
            while notif_rx.changed().await.is_ok() {
                if let Some(bar_window) = bar_window {
                    cx.update(|app| {
                        let _ = bar_window.update(app, |bar, window, cx| {
                            bar.handle_notification_store_change(window, cx);
                        });
                    });
                }
            }
        }));

        let focus_handle = cx.focus_handle();
        let launcher = cx.new(popups::launcher::BarLauncher::new);
        let polkit = cx.new(popups::polkit::BarPolkit::new);
        let shared_panel = shared_panel::SharedPanelManager::new(cx);

        let mut subscriptions = vec![sub];
        subscriptions.extend(shared_panel.subscribe(cx));
        subscriptions.push(
            cx.subscribe(&launcher, |bar, _, event: &popups::PopupEvent, cx| {
                bar.handle_popup_event(*event, cx);
            }),
        );
        subscriptions.push(
            cx.subscribe(&polkit, |bar, _, event: &popups::PopupEvent, cx| {
                bar.handle_popup_event(*event, cx);
            }),
        );

        let bar_window = window_handle.downcast::<Bar>();
        cx.defer(move |cx| {
            if let Some(bar_window) = bar_window {
                let _ = bar_window.update(cx, |bar, window, cx| {
                    bar.sync_window_state(window, cx);
                    bar.check_polkit_requests(window, cx);
                });
            }
        });

        Self {
            window_state: window_state::WindowState::new(window_handle, px(style::BAR_HEIGHT)),
            location,
            height: px(style::BAR_HEIGHT),
            current_workspace: cur_ws,
            workspaces: all_ws,
            clock_center: Rc::new(Cell::new(px(0.0))),
            status_center: Rc::new(Cell::new(px(0.0))),
            shared_panel,
            launcher,
            polkit,
            focus_handle,
            _subscriptions: subscriptions,
            _clock_task: Some(clock_task),
            _config_task: config_task,
            _workspace_task: workspace_task,
            _polkit_task: polkit_task,
            _notification_task: notification_task,
        }
    }

    pub fn sync_window_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.window_state.sync_window(
            &self.shared_panel,
            &self.launcher,
            &self.polkit,
            &self.focus_handle,
            window,
            cx,
        );
    }

    pub fn queue_animation_frame(&self, window: &Window, cx: &Context<Self>) {
        window_state::WindowState::queue_animation_frame(
            window,
            cx.entity().downgrade(),
            &self.window_state.animation_frame_pending,
        );
    }

    pub fn queue_animation_frame_checked(&self, window: &Window, cx: &Context<Self>) {
        window_state::WindowState::queue_animation_frame_checked(
            window,
            cx.entity().downgrade(),
            &self.window_state.animation_frame_pending,
        );
    }

    pub fn open_shared_panel(
        &mut self,
        module: shared_panel::SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.launcher.read(cx).is_open() {
            self.launcher.update(cx, |l, cx| l.close(cx));
        }
        self.shared_panel.open(module, anchor_x, window, cx);
        self.sync_window_state(window, cx);
        self.queue_animation_frame(window, cx);
        cx.notify();
    }

    pub fn close_shared_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shared_panel.request_close(cx);
        self.sync_window_state(window, cx);
        self.queue_animation_frame(window, cx);
        cx.notify();
    }

    pub fn toggle_shared_panel(
        &mut self,
        module: shared_panel::SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.shared_panel.is_open() && self.shared_panel.active_module == Some(module) {
            self.close_shared_panel(window, cx);
        } else {
            self.open_shared_panel(module, anchor_x, window, cx);
        }
    }

    pub fn open_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shared_panel.is_open() {
            self.close_shared_panel(window, cx);
        }
        self.launcher.update(cx, |l, cx| l.open(cx));
        self.sync_window_state(window, cx);
        self.queue_animation_frame(window, cx);
        cx.notify();
    }

    pub fn close_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.launcher.update(cx, |l, cx| l.close(cx));
        self.sync_window_state(window, cx);
        self.queue_animation_frame(window, cx);
        cx.notify();
    }

    pub fn toggle_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.launcher.read(cx).is_open() {
            self.close_launcher(window, cx);
        } else {
            self.open_launcher(window, cx);
        }
    }

    pub fn check_polkit_requests(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let had_request = self.polkit.read(cx).request.is_some();
        let has_new = self.polkit.update(cx, |p, cx| p.process_requests(cx));
        if has_new || (!had_request && self.polkit.read(cx).request.is_some()) {
            if self.shared_panel.is_open() {
                self.close_shared_panel(window, cx);
            }
            if self.launcher.read(cx).is_open() {
                self.close_launcher(window, cx);
            }
            self.polkit.update(cx, |p, cx| p.open(cx));
            self.sync_window_state(window, cx);
            self.queue_animation_frame(window, cx);
            cx.notify();
        }
    }

    pub fn close_polkit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.polkit.update(cx, |p, cx| p.close(cx));
        self.sync_window_state(window, cx);
        self.queue_animation_frame(window, cx);
        cx.notify();
    }

    pub fn toggle_notifications(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.clock_center.get() > px(0.0) {
            self.clock_center.get()
        } else {
            self.window_state.screen_width / 2.0
        };
        self.toggle_shared_panel(
            shared_panel::SharedPanelModule::Calendar,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn show_notifications(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.clock_center.get() > px(0.0) {
            self.clock_center.get()
        } else {
            self.window_state.screen_width / 2.0
        };
        self.open_shared_panel(
            shared_panel::SharedPanelModule::Calendar,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn toggle_dashboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.status_center.get() > px(0.0) {
            self.status_center.get()
        } else {
            self.window_state.screen_width - px(240.0)
        };
        self.toggle_shared_panel(
            shared_panel::SharedPanelModule::Dashboard,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn show_dashboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.status_center.get() > px(0.0) {
            self.status_center.get()
        } else {
            self.window_state.screen_width - px(240.0)
        };
        self.open_shared_panel(
            shared_panel::SharedPanelModule::Dashboard,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn toggle_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.window_state.screen_width / 2.0;
        self.toggle_shared_panel(
            shared_panel::SharedPanelModule::Clipboard,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn show_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.window_state.screen_width / 2.0;
        self.open_shared_panel(
            shared_panel::SharedPanelModule::Clipboard,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn toggle_emoji(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.window_state.screen_width / 2.0;
        self.toggle_shared_panel(shared_panel::SharedPanelModule::Emoji, anchor_x, window, cx);
    }

    pub fn show_emoji(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.window_state.screen_width / 2.0;
        self.open_shared_panel(shared_panel::SharedPanelModule::Emoji, anchor_x, window, cx);
    }

    pub fn toggle_record(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.status_center.get() > px(0.0) {
            self.status_center.get()
        } else {
            self.window_state.screen_width - px(200.0)
        };
        self.toggle_shared_panel(
            shared_panel::SharedPanelModule::Record,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn show_record(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.status_center.get() > px(0.0) {
            self.status_center.get()
        } else {
            self.window_state.screen_width - px(200.0)
        };
        self.open_shared_panel(
            shared_panel::SharedPanelModule::Record,
            anchor_x,
            window,
            cx,
        );
    }

    pub fn handle_shared_panel_event(
        &mut self,
        event: shared_panel::SharedPanelEvent,
        cx: &mut Context<Self>,
    ) {
        let bar_window = self.window_state.handle.downcast::<Bar>();
        if let Some(bar_window) = bar_window {
            cx.defer(move |app| {
                let _ = bar_window.update(app, |bar, window, cx| match event {
                    shared_panel::SharedPanelEvent::Close => {
                        bar.close_shared_panel(window, cx);
                    }
                    shared_panel::SharedPanelEvent::Open { id, anchor_x } => {
                        let default_anchor = bar.window_state.screen_width / 2.0;
                        let target = anchor_x.unwrap_or(default_anchor);
                        bar.open_shared_panel(id, target, window, cx);
                    }
                    shared_panel::SharedPanelEvent::SyncWindow
                    | shared_panel::SharedPanelEvent::SizeChanged(_)
                    | shared_panel::SharedPanelEvent::NotificationChanged => {
                        bar.sync_window_state(window, cx);
                        bar.queue_animation_frame(window, cx);
                    }
                });
            });
        }
    }

    pub fn handle_popup_event(&mut self, event: popups::PopupEvent, cx: &mut Context<Self>) {
        let bar_window = self.window_state.handle.downcast::<Bar>();
        if let Some(bar_window) = bar_window {
            cx.defer(move |app| {
                let _ = bar_window.update(app, |bar, window, cx| match event {
                    popups::PopupEvent::Open | popups::PopupEvent::Close => {
                        bar.sync_window_state(window, cx);
                        bar.queue_animation_frame(window, cx);
                    }
                    popups::PopupEvent::SyncWindow => {
                        bar.sync_window_state(window, cx);
                    }
                    popups::PopupEvent::RequestAnimationFrame => {
                        bar.queue_animation_frame(window, cx);
                    }
                });
            });
        }
    }

    fn handle_notification_store_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let store = services::NotificationStore::global();
        let latest = store.get_latest_active_notification();
        if let Some(item) = latest {
            let item_id = item.id;
            let current_module = self.shared_panel.active_module;
            let is_popup = matches!(
                current_module,
                Some(shared_panel::SharedPanelModule::NotificationPopup(_))
            );
            if (!self.shared_panel.is_open() || is_popup)
                && current_module
                    != Some(shared_panel::SharedPanelModule::NotificationPopup(item_id))
            {
                let anchor_x = if self.status_center.get() > px(0.0) {
                    self.status_center.get()
                } else {
                    self.window_state.screen_width - px(220.0)
                };
                self.open_shared_panel(
                    shared_panel::SharedPanelModule::NotificationPopup(item_id),
                    anchor_x,
                    window,
                    cx,
                );
            }
        } else if let Some(shared_panel::SharedPanelModule::NotificationPopup(_)) =
            self.shared_panel.active_module
        {
            self.close_shared_panel(window, cx);
        }
    }
}

impl Render for Bar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let is_horizontal = self.location.is_horizontal();

        let bar_config = if let Some(state) = cx.try_global::<AppState>() {
            state.config.get().ui.bar.clone()
        } else {
            services::BarConfig::default()
        };

        let border_color = style::border(&theme).opacity(0.35);

        let mut bar = div()
            .id("capsule-bar")
            .relative()
            .font_family(theme.font_family())
            .text_color(style::text(&theme))
            .bg(style::background(&theme))
            .flex()
            .items_center();

        if is_horizontal {
            bar = bar
                .w_full()
                .h(self.height)
                .px(px(style::BAR_PADDING_H))
                .py(px(style::BAR_PADDING_V));

            if self.location == BarLocation::Bottom {
                bar = bar.border_t_1().border_color(border_color);
            }
        } else {
            bar = bar
                .h_full()
                .w(self.height)
                .flex_col()
                .px(px(style::BAR_PADDING_V))
                .py(px(style::BAR_PADDING_H));

            match self.location {
                BarLocation::Left => {
                    bar = bar
                        .border_r_1()
                        .border_color(style::border(&theme).opacity(0.4));
                }
                BarLocation::Right => {
                    bar = bar
                        .border_l_1()
                        .border_color(style::border(&theme).opacity(0.4));
                }
                _ => {}
            }
        }

        let mut left = div()
            .id("bar-left")
            .flex()
            .items_center()
            .gap(px(style::GAP));
        if !is_horizontal {
            left = left.flex_col();
        }
        for &widget_id in &bar_config.left {
            left = left.child(widgets::render_widget(
                widget_id,
                &self.current_workspace,
                &self.workspaces,
                &self.clock_center,
                &self.status_center,
                &theme,
                cx,
            ));
        }

        let mut center = div()
            .id("bar-center")
            .flex()
            .items_center()
            .gap(px(style::GAP));
        if !is_horizontal {
            center = center.flex_col();
        }
        for &widget_id in &bar_config.center {
            center = center.child(widgets::render_widget(
                widget_id,
                &self.current_workspace,
                &self.workspaces,
                &self.clock_center,
                &self.status_center,
                &theme,
                cx,
            ));
        }

        let mut right = div()
            .id("bar-right")
            .flex()
            .items_center()
            .gap(px(style::GAP));
        if !is_horizontal {
            right = right.flex_col();
        }
        for &widget_id in &bar_config.right {
            right = right.child(widgets::render_widget(
                widget_id,
                &self.current_workspace,
                &self.workspaces,
                &self.clock_center,
                &self.status_center,
                &theme,
                cx,
            ));
        }

        if is_horizontal {
            bar = bar
                .child(
                    div()
                        .id("bar-left-container")
                        .flex_1()
                        .h_full()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .justify_start()
                        .child(left),
                )
                .child(
                    div()
                        .id("bar-center-container")
                        .flex_shrink_0()
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(center),
                )
                .child(
                    div()
                        .id("bar-right-container")
                        .flex_1()
                        .h_full()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(right),
                );
        } else {
            bar = bar
                .child(
                    div()
                        .id("bar-left-container")
                        .flex_1()
                        .w_full()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_start()
                        .child(left),
                )
                .child(
                    div()
                        .id("bar-center-container")
                        .flex_shrink_0()
                        .w_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .child(center),
                )
                .child(
                    div()
                        .id("bar-right-container")
                        .flex_1()
                        .w_full()
                        .min_h_0()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_end()
                        .child(right),
                );
        }

        let viewport = window.viewport_size();
        let ear_radius = if bar_config.convex && is_horizontal {
            px(shared_panel::view::CONVEX_EAR_RADIUS)
        } else {
            px(0.0)
        };
        let border_start = ear_radius;
        let border_end = (viewport.width - ear_radius).max(border_start);

        if self.location == BarLocation::Top {
            bar = bar.child(shared_panel::view::render_bottom_border(
                self.shared_panel.animator.as_ref(),
                border_start,
                border_end,
                border_color,
            ));
        }

        let mut root = div()
            .id("capsule-bar-root")
            .size_full()
            .relative()
            .font_family(theme.font_family())
            .track_focus(&self.focus_handle)
            .key_context("capsule-bar")
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    if this.polkit.read(cx).is_open() {
                        this.close_polkit(window, cx);
                        cx.stop_propagation();
                    } else if this.launcher.read(cx).is_open() {
                        this.close_launcher(window, cx);
                        cx.stop_propagation();
                    } else if this.shared_panel.is_open() {
                        this.close_shared_panel(window, cx);
                        cx.stop_propagation();
                    }
                }
            }))
            .child(bar);

        if bar_config.convex && is_horizontal {
            let is_bottom = self.location == BarLocation::Bottom;
            let (left_ear, right_ear) = shared_panel::view::render_convex_ears(
                self.shared_panel.animator.as_ref(),
                viewport.width,
                self.height,
                is_bottom,
                &theme,
            );
            root = root.child(left_ear).child(right_ear);
        }

        if let Some(ref anim) = self.shared_panel.animator
            && let Some(current_view) = self.shared_panel.current()
        {
            let entity = cx.entity().downgrade();
            let (backdrop, panel) = shared_panel::view::render_panel_container(
                anim,
                self.height,
                current_view,
                &theme,
                move |_, window, cx| {
                    if let Some(strong) = entity.upgrade() {
                        strong.update(cx, |bar, cx| {
                            bar.close_shared_panel(window, cx);
                        });
                    }
                },
            );
            root = root.child(backdrop).child(panel);
        }

        if self.launcher.read(cx).is_visible() {
            root = root.child(self.launcher.clone());
        }

        if self.polkit.read(cx).is_visible() {
            root = root.child(self.polkit.clone());
        }

        root
    }
}
