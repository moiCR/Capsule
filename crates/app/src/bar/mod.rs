pub mod popups;
pub mod shared_panel;
pub mod style;
pub mod widgets;

use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    AnyWindowHandle, ColorExt, Context, FocusHandle, Pixels, Size, Subscription, Task, Window, div,
    px,
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
    pub window_handle: AnyWindowHandle,
    pub location: BarLocation,
    pub height: Pixels,
    pub current_workspace: WorkspaceInfo,
    pub workspaces: Vec<WorkspaceInfo>,
    pub shared_panel: shared_panel::SharedPanel,
    pub launcher: popups::launcher::BarLauncher,
    pub polkit: popups::polkit::BarPolkit,
    pub focus_handle: FocusHandle,
    pub clock_center: std::rc::Rc<std::cell::Cell<Pixels>>,
    pub status_center: std::rc::Rc<std::cell::Cell<Pixels>>,
    pub animation_frame_pending: std::rc::Rc<std::cell::Cell<bool>>,
    last_input_regions: Option<Vec<gpui::Bounds<Pixels>>>,
    last_keyboard: Option<gpui::layer_shell::KeyboardInteractivity>,
    _subscription: Option<Subscription>,
    _clock_task: Option<Task<()>>,
    _config_task: Option<Task<()>>,
    _workspace_task: Option<Task<()>>,
    _polkit_task: Option<Task<()>>,
    _notification_task: Option<Task<()>>,
    _notification_expiry_task: Option<Task<()>>,
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
                loop {
                    match ws_changes.recv().await {
                        Ok(_) => {
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
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
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
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
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
        let notification_task = cx.spawn(async move |this, cx| {
            while notif_rx.changed().await.is_ok() {
                if this
                    .update(cx, |bar, cx| {
                        bar.handle_notification_store_change(cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        let bar_window = window_handle.downcast::<Bar>();
        cx.defer(move |cx| {
            if let Some(bar_window) = bar_window {
                let _ = bar_window.update(cx, |bar, window, cx| {
                    bar.sync_window_state(window, cx);
                    bar.check_polkit_requests(window, cx);
                });
            }
        });

        let focus_handle = cx.focus_handle();
        let animation_frame_pending = std::rc::Rc::new(std::cell::Cell::new(false));
        let launcher = popups::launcher::BarLauncher::new(cx);
        let polkit = popups::polkit::BarPolkit::new();
        let mut shared_panel = shared_panel::SharedPanel::new();
        shared_panel.focus_handle = Some(focus_handle.clone());
        shared_panel.animation_frame_pending = animation_frame_pending.clone();

        Self {
            window_handle,
            location,
            height: px(style::BAR_HEIGHT),
            current_workspace: cur_ws,
            workspaces: all_ws,
            shared_panel,
            launcher,
            polkit,
            focus_handle,
            clock_center: std::rc::Rc::new(std::cell::Cell::new(px(0.0))),
            status_center: std::rc::Rc::new(std::cell::Cell::new(px(0.0))),
            animation_frame_pending,
            last_input_regions: None,
            last_keyboard: None,
            _subscription: Some(sub),
            _clock_task: Some(clock_task),
            _config_task: config_task,
            _workspace_task: workspace_task,
            _polkit_task: polkit_task,
            _notification_task: Some(notification_task),
            _notification_expiry_task: None,
        }
    }

    pub fn needs_exclusive_keyboard(&self) -> bool {
        if self.launcher.is_open() || self.polkit.is_open() {
            return true;
        }
        if !self.shared_panel.is_open() {
            return false;
        }
        match self.shared_panel.active_module {
            Some(shared_panel::SharedPanelModule::Clipboard)
            | Some(shared_panel::SharedPanelModule::Emoji) => true,
            Some(shared_panel::SharedPanelModule::Dashboard) => {
                self.shared_panel.dashboard_state.selected_ssid.is_some()
            }
            _ => false,
        }
    }

    pub fn sync_window_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shared_panel.sync_display_bounds(window, cx);
        let viewport = window.viewport_size();
        let width = if viewport.width > px(0.0) {
            viewport.width
        } else {
            self.shared_panel.screen_width
        };
        let height = if viewport.height > px(0.0) {
            viewport.height
        } else {
            self.shared_panel.screen_height
        };

        let is_popup_open = self.launcher.is_open() || self.polkit.is_open();
        let has_popup = self.launcher.is_visible() || self.polkit.is_visible();
        let is_shared_open = self.shared_panel.is_open();
        let has_panel = is_shared_open || self.shared_panel.animator.is_some();

        let regions = if is_popup_open || has_popup || has_panel {
            vec![gpui::Bounds {
                origin: gpui::point(px(0.0), px(0.0)),
                size: gpui::size(width, height),
            }]
        } else {
            vec![gpui::Bounds {
                origin: gpui::point(px(0.0), px(0.0)),
                size: gpui::size(width, self.height),
            }]
        };

        if self.last_input_regions.as_ref() != Some(&regions) {
            self.last_input_regions = Some(regions.clone());
            window.set_input_region(Some(&regions));
        }

        let desired_keyboard = if self.needs_exclusive_keyboard() {
            gpui::layer_shell::KeyboardInteractivity::Exclusive
        } else {
            gpui::layer_shell::KeyboardInteractivity::None
        };

        if self.last_keyboard != Some(desired_keyboard) {
            self.last_keyboard = Some(desired_keyboard);
            window.set_keyboard_interactivity(desired_keyboard);
            if desired_keyboard == gpui::layer_shell::KeyboardInteractivity::Exclusive {
                window.focus(&self.focus_handle, cx);
            } else {
                window.blur(cx);
            }
        }
    }

    pub fn toggle_shared_panel(
        &mut self,
        module: shared_panel::SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shared_panel.toggle(module, anchor_x, window, cx);
        self.sync_window_state(window, cx);
    }

    fn handle_notification_store_change(&mut self, cx: &mut Context<Self>) {
        let store = services::NotificationStore::global();
        let latest = store.get_latest_active_notification();
        let Some(bar_window) = self.window_handle.downcast::<Bar>() else {
            return;
        };

        if let Some(item) = latest {
            let item_id = item.id;
            let current_module = self.shared_panel.active_module.clone();
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
                    self.shared_panel.screen_width - px(220.0)
                };
                let timeout = item.timeout;
                cx.defer(move |app| {
                    let _ = bar_window.update(app, |bar, window, cx| {
                        let entity = cx.entity().downgrade();
                        bar.shared_panel.open(
                            shared_panel::SharedPanelModule::NotificationPopup(item_id),
                            anchor_x,
                            window,
                            entity,
                            cx,
                        );
                        bar.sync_window_state(window, cx);
                        bar.schedule_notification_expiry(timeout, item_id, cx);
                        cx.notify();
                    });
                });
            }
        } else if let Some(shared_panel::SharedPanelModule::NotificationPopup(_)) =
            self.shared_panel.active_module
        {
            cx.defer(move |app| {
                let _ = bar_window.update(app, |bar, window, cx| {
                    bar.close_shared_panel(window, cx);
                });
            });
        }
    }

    fn schedule_notification_expiry(
        &mut self,
        timeout: Duration,
        item_id: u32,
        cx: &mut Context<Self>,
    ) {
        let delay = if timeout.is_zero() {
            Duration::from_secs(5)
        } else {
            timeout
        };
        let bar_window = self.window_handle.downcast::<Bar>();
        self._notification_expiry_task = Some(cx.spawn(async move |_this, cx| {
            cx.background_executor().timer(delay).await;
            if services::NotificationStore::global().is_hovered() {
                return;
            }
            if let Some(bar_window) = bar_window {
                cx.update(|app| {
                    let _ = bar_window.update(app, |bar, window, cx| {
                        if bar.shared_panel.active_module
                            == Some(shared_panel::SharedPanelModule::NotificationPopup(item_id))
                        {
                            bar.close_shared_panel(window, cx);
                        }
                    });
                });
            }
        }));
    }

    pub fn toggle_notifications(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.clock_center.get() > px(0.0) {
            self.clock_center.get()
        } else {
            self.shared_panel.screen_width / 2.0
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
            self.shared_panel.screen_width / 2.0
        };
        self.shared_panel.open(
            shared_panel::SharedPanelModule::Calendar,
            anchor_x,
            window,
            cx.entity().downgrade(),
            cx,
        );
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn toggle_dashboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = if self.status_center.get() > px(0.0) {
            self.status_center.get()
        } else {
            self.shared_panel.screen_width - px(240.0)
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
            self.shared_panel.screen_width - px(240.0)
        };
        self.shared_panel.open(
            shared_panel::SharedPanelModule::Dashboard,
            anchor_x,
            window,
            cx.entity().downgrade(),
            cx,
        );
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn start_dashboard_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.shared_panel.dashboard_state.dragging_volume = true;
        self.dashboard_volume_at(x, cx);
    }

    pub fn dashboard_volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.shared_panel.dashboard_state.volume_bounds;
        if let Some(fraction) =
            shared_panel::dashboard::slider_fraction(x, bounds.left(), bounds.width(0.0))
        {
            if let Some(state) = cx.try_global::<AppState>() {
                state
                    .system
                    .set_volume_fast((fraction * 100.0).round() as u32);
            }
            cx.notify();
        }
    }

    pub fn start_dashboard_input_volume(&mut self, x: f32, cx: &mut Context<Self>) {
        self.shared_panel.dashboard_state.dragging_input_volume = true;
        self.dashboard_input_volume_at(x, cx);
    }

    pub fn dashboard_input_volume_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.shared_panel.dashboard_state.input_volume_bounds;
        if let Some(fraction) =
            shared_panel::dashboard::slider_fraction(x, bounds.left(), bounds.width(0.0))
        {
            if let Some(state) = cx.try_global::<AppState>() {
                state
                    .system
                    .set_input_volume_fast((fraction * 100.0).round() as u32);
            }
            cx.notify();
        }
    }

    pub fn start_dashboard_brightness(&mut self, x: f32, cx: &mut Context<Self>) {
        self.shared_panel.dashboard_state.dragging_brightness = true;
        self.dashboard_brightness_at(x, cx);
    }

    pub fn dashboard_brightness_at(&mut self, x: f32, cx: &mut Context<Self>) {
        let bounds = &self.shared_panel.dashboard_state.brightness_bounds;
        if let Some(fraction) =
            shared_panel::dashboard::slider_fraction(x, bounds.left(), bounds.width(0.0))
        {
            let value = ((fraction * 100.0).round() as u32).clamp(5, 100);
            if self.shared_panel.dashboard_state.brightness_preview != Some(value) {
                self.shared_panel.dashboard_state.brightness_preview = Some(value);
                cx.notify();
            }
        }
    }

    pub fn finish_dashboard_brightness(&mut self, cx: &mut Context<Self>) {
        if !self.shared_panel.dashboard_state.dragging_brightness {
            return;
        }
        self.shared_panel.dashboard_state.dragging_brightness = false;
        if let Some(value) = self.shared_panel.dashboard_state.brightness_preview.take() {
            if let Some(state) = cx.try_global::<AppState>() {
                let system = state.system.clone();
                services::spawn_tokio(async move {
                    let _ = system.set_brightness(value).await;
                });
            }
            cx.notify();
        }
    }

    pub fn finish_dashboard_sliders(&mut self, cx: &mut Context<Self>) {
        let was_audio = self.shared_panel.dashboard_state.dragging_volume
            || self.shared_panel.dashboard_state.dragging_input_volume;
        self.shared_panel.dashboard_state.dragging_volume = false;
        self.shared_panel.dashboard_state.dragging_input_volume = false;
        self.finish_dashboard_brightness(cx);
        if was_audio {
            if let Some(state) = cx.try_global::<AppState>() {
                let sys = state.system.clone();
                services::spawn_tokio(async move {
                    let _ = sys.refresh().await;
                });
            }
            cx.notify();
        }
    }

    pub fn open_dashboard_subview(
        &mut self,
        view: shared_panel::dashboard::DashboardSubView,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shared_panel.dashboard_state.open_subview(view);
        self.sync_window_state(window, cx);
        Bar::queue_animation_frame_checked(
            window,
            cx.entity().downgrade(),
            &self.animation_frame_pending,
        );
        cx.notify();
    }

    pub fn close_dashboard_subview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shared_panel.dashboard_state.close_subview();
        self.sync_window_state(window, cx);
        Bar::queue_animation_frame_checked(
            window,
            cx.entity().downgrade(),
            &self.animation_frame_pending,
        );
        cx.notify();
    }

    fn handle_dashboard_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            if self.shared_panel.dashboard_state.view
                != shared_panel::dashboard::DashboardSubView::Home
            {
                self.close_dashboard_subview(window, cx);
            } else {
                self.close_shared_panel(window, cx);
            }
            return;
        }

        if (key == "enter" || key == "return")
            && let Some(ssid) = self.shared_panel.dashboard_state.selected_ssid.take()
        {
            let pwd = self.shared_panel.dashboard_state.password_input.text();
            if let Some(state) = cx.try_global::<AppState>() {
                state.network.connect_wifi(&ssid, Some(&pwd));
            }
            self.shared_panel.dashboard_state.password_input.clear();
            self.sync_window_state(window, cx);
            cx.notify();
            return;
        }

        if self.shared_panel.dashboard_state.selected_ssid.is_some()
            && self
                .shared_panel
                .dashboard_state
                .password_input
                .handle_key_down(event, cx)
        {
            cx.notify();
        }
    }

    pub fn toggle_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let is_same = self.shared_panel.is_open()
            && self.shared_panel.active_module == Some(shared_panel::SharedPanelModule::Clipboard);
        if is_same {
            self.close_shared_panel(window, cx);
        } else {
            let anchor_x = self.shared_panel.screen_width / 2.0;
            self.open_clipboard(anchor_x, window, cx);
        }
    }

    pub fn show_clipboard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.shared_panel.screen_width / 2.0;
        self.open_clipboard(anchor_x, window, cx);
    }

    pub fn open_clipboard(
        &mut self,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = cx.try_global::<AppState>() {
            self.shared_panel.clipboard_state.items = state.clipboard.fetch_history();
            self.shared_panel.clipboard_state.snippets = state.clipboard.get_snippets();
        }
        self.shared_panel.clipboard_state.search_input.clear();
        self.shared_panel.clipboard_state.selected = 0;
        self.shared_panel.clipboard_state.scroll.scroll_to_item(0);
        self.shared_panel.open(
            shared_panel::SharedPanelModule::Clipboard,
            anchor_x,
            window,
            cx.entity().downgrade(),
            cx,
        );
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn toggle_emoji(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let is_same = self.shared_panel.is_open()
            && self.shared_panel.active_module == Some(shared_panel::SharedPanelModule::Emoji);
        if is_same {
            self.close_shared_panel(window, cx);
        } else {
            let anchor_x = self.shared_panel.screen_width / 2.0;
            self.open_emoji(anchor_x, window, cx);
        }
    }

    pub fn show_emoji(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let anchor_x = self.shared_panel.screen_width / 2.0;
        self.open_emoji(anchor_x, window, cx);
    }

    pub fn open_emoji(&mut self, anchor_x: Pixels, window: &mut Window, cx: &mut Context<Self>) {
        self.shared_panel.emoji_state.search_input.clear();
        self.shared_panel.emoji_state.category = None;
        self.shared_panel.emoji_state.selected = 0;
        self.shared_panel.emoji_state.page = 0;
        self.shared_panel.open(
            shared_panel::SharedPanelModule::Emoji,
            anchor_x,
            window,
            cx.entity().downgrade(),
            cx,
        );
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn activate_clipboard_history(
        &mut self,
        idx: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let filtered = self.shared_panel.clipboard_state.filtered_history();
        if let Some(item) = filtered.get(idx).copied() {
            let item_clone = item.clone();
            if !item.is_image {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(item.preview.clone()));
            }
            if let Some(state) = cx.try_global::<AppState>() {
                let cb = state.clipboard.clone();
                services::spawn_blocking(move || {
                    cb.copy_item(&item_clone);
                });
            }
            self.close_shared_panel(window, cx);
        }
    }

    pub fn activate_clipboard_snippet(
        &mut self,
        idx: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let filtered = self.shared_panel.clipboard_state.filtered_snippets();
        if let Some(snippet) = filtered.get(idx).copied() {
            let content = snippet.content.clone();
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(content.clone()));
            services::spawn_blocking(move || {
                let _ = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut c| {
                        use std::io::Write;
                        if let Some(mut stdin) = c.stdin.take() {
                            let _ = stdin.write_all(content.as_bytes());
                        }
                        c.wait()
                    });
            });
            self.close_shared_panel(window, cx);
        }
    }

    pub fn pin_clipboard_item(&mut self, idx: usize, cx: &mut Context<Self>) {
        let history = self.shared_panel.clipboard_state.filtered_history();
        if let Some(item) = history.get(idx) {
            let item_clone = (*item).clone();
            if let Some(state) = cx.try_global::<AppState>() {
                let cb = state.clipboard.clone();
                services::spawn_blocking(move || {
                    let _ = cb.pin_from_history(&item_clone);
                });
                self.shared_panel.clipboard_state.snippets = state.clipboard.get_snippets();
                cx.notify();
            }
        }
    }

    pub fn remove_clipboard_item(&mut self, id: &str, cx: &mut Context<Self>) {
        self.shared_panel
            .clipboard_state
            .items
            .retain(|i| i.id != id);
        cx.notify();
    }

    pub fn remove_snippet_item(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let id_str = id.to_string();
            let cb = state.clipboard.clone();
            services::spawn_blocking(move || {
                cb.remove_snippet(&id_str);
            });
        }
        self.shared_panel
            .clipboard_state
            .snippets
            .retain(|s| s.id != id);
        cx.notify();
    }

    pub fn clear_clipboard_history(&mut self, cx: &mut Context<Self>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let cb = state.clipboard.clone();
            services::spawn_blocking(move || {
                cb.clear_history();
            });
        }
        self.shared_panel.clipboard_state.items.clear();
        cx.notify();
    }

    pub fn copy_emoji(&mut self, emoji: &str, window: &mut Window, cx: &mut Context<Self>) {
        let emoji_str = emoji.to_string();
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(emoji_str.clone()));
        services::spawn_blocking(move || {
            services::EmojiService::new().copy_emoji(&emoji_str);
        });
        self.close_shared_panel(window, cx);
    }

    pub fn emoji_total_pages(&self) -> usize {
        self.shared_panel
            .emoji_state
            .filtered_emojis()
            .len()
            .div_ceil(shared_panel::emoji::EMOJIS_PER_PAGE)
            .max(1)
    }

    fn handle_clipboard_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.close_shared_panel(window, cx);
            return;
        }

        let count = match self.shared_panel.clipboard_state.tab {
            shared_panel::clipboard::ClipboardTab::History => {
                self.shared_panel.clipboard_state.filtered_history().len()
            }
            shared_panel::clipboard::ClipboardTab::Snippets => {
                self.shared_panel.clipboard_state.filtered_snippets().len()
            }
        };

        if key == "right" {
            if count > 0 {
                let cur = self.shared_panel.clipboard_state.selected;
                let next = (cur + 1).min(count.saturating_sub(1));
                self.shared_panel.clipboard_state.selected = next;
                let row = next / shared_panel::clipboard::CLIPBOARD_COLUMNS;
                self.shared_panel.clipboard_state.scroll.scroll_to_item(row);
                cx.notify();
            }
            return;
        }

        if key == "left" {
            if count > 0 {
                let cur = self.shared_panel.clipboard_state.selected;
                let next = cur.saturating_sub(1);
                self.shared_panel.clipboard_state.selected = next;
                let row = next / shared_panel::clipboard::CLIPBOARD_COLUMNS;
                self.shared_panel.clipboard_state.scroll.scroll_to_item(row);
                cx.notify();
            }
            return;
        }

        if key == "down" {
            if count > 0 {
                let cur = self.shared_panel.clipboard_state.selected;
                let next =
                    (cur + shared_panel::clipboard::CLIPBOARD_COLUMNS).min(count.saturating_sub(1));
                self.shared_panel.clipboard_state.selected = next;
                let row = next / shared_panel::clipboard::CLIPBOARD_COLUMNS;
                self.shared_panel.clipboard_state.scroll.scroll_to_item(row);
                cx.notify();
            }
            return;
        }

        if key == "up" {
            if count > 0 {
                let cur = self.shared_panel.clipboard_state.selected;
                let next = cur.saturating_sub(shared_panel::clipboard::CLIPBOARD_COLUMNS);
                self.shared_panel.clipboard_state.selected = next;
                let row = next / shared_panel::clipboard::CLIPBOARD_COLUMNS;
                self.shared_panel.clipboard_state.scroll.scroll_to_item(row);
                cx.notify();
            }
            return;
        }

        if key == "enter" || key == "return" {
            let sel = self.shared_panel.clipboard_state.selected;
            match self.shared_panel.clipboard_state.tab {
                shared_panel::clipboard::ClipboardTab::History => {
                    self.activate_clipboard_history(sel, window, cx);
                }
                shared_panel::clipboard::ClipboardTab::Snippets => {
                    self.activate_clipboard_snippet(sel, window, cx);
                }
            }
            return;
        }

        if key == "tab" {
            self.shared_panel.clipboard_state.tab = match self.shared_panel.clipboard_state.tab {
                shared_panel::clipboard::ClipboardTab::History => {
                    shared_panel::clipboard::ClipboardTab::Snippets
                }
                shared_panel::clipboard::ClipboardTab::Snippets => {
                    shared_panel::clipboard::ClipboardTab::History
                }
            };
            self.shared_panel.clipboard_state.selected = 0;
            self.shared_panel.clipboard_state.scroll.scroll_to_item(0);
            cx.notify();
            return;
        }

        if self
            .shared_panel
            .clipboard_state
            .search_input
            .handle_key_down(event, cx)
        {
            self.shared_panel.clipboard_state.selected = 0;
            self.shared_panel.clipboard_state.scroll.scroll_to_item(0);
            cx.notify();
        }
    }

    fn handle_emoji_key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.close_shared_panel(window, cx);
            return;
        }

        let all = self.shared_panel.emoji_state.filtered_emojis();
        let total_pages = all
            .len()
            .div_ceil(shared_panel::emoji::EMOJIS_PER_PAGE)
            .max(1);
        let cur_page = self.shared_panel.emoji_state.page.min(total_pages - 1);
        let page_items_count = all
            .iter()
            .skip(cur_page * shared_panel::emoji::EMOJIS_PER_PAGE)
            .take(shared_panel::emoji::EMOJIS_PER_PAGE)
            .count();

        if key == "right" {
            if page_items_count > 0 {
                let next = (self.shared_panel.emoji_state.selected + 1)
                    .min(page_items_count.saturating_sub(1));
                self.shared_panel.emoji_state.selected = next;
                cx.notify();
            }
            return;
        }

        if key == "left" {
            if page_items_count > 0 {
                let next = self.shared_panel.emoji_state.selected.saturating_sub(1);
                self.shared_panel.emoji_state.selected = next;
                cx.notify();
            }
            return;
        }

        if key == "down" {
            if page_items_count > 0 {
                let next =
                    self.shared_panel.emoji_state.selected + shared_panel::emoji::EMOJI_COLUMNS;
                if next < page_items_count {
                    self.shared_panel.emoji_state.selected = next;
                    cx.notify();
                }
            }
            return;
        }

        if key == "up" {
            if self.shared_panel.emoji_state.selected >= shared_panel::emoji::EMOJI_COLUMNS {
                self.shared_panel.emoji_state.selected -= shared_panel::emoji::EMOJI_COLUMNS;
                cx.notify();
            }
            return;
        }

        if key == "pageup" {
            if self.shared_panel.emoji_state.page > 0 {
                self.shared_panel.emoji_state.page -= 1;
                self.shared_panel.emoji_state.selected = 0;
                cx.notify();
            }
            return;
        }

        if key == "pagedown" {
            if self.shared_panel.emoji_state.page + 1 < total_pages {
                self.shared_panel.emoji_state.page += 1;
                self.shared_panel.emoji_state.selected = 0;
                cx.notify();
            }
            return;
        }

        if key == "enter" || key == "return" {
            let start = cur_page * shared_panel::emoji::EMOJIS_PER_PAGE;
            if let Some(item) = all.get(start + self.shared_panel.emoji_state.selected) {
                let emoji_char = item.emoji.clone();
                self.copy_emoji(&emoji_char, window, cx);
            }
            return;
        }

        if self
            .shared_panel
            .emoji_state
            .search_input
            .handle_key_down(event, cx)
        {
            self.shared_panel.emoji_state.selected = 0;
            self.shared_panel.emoji_state.page = 0;
            cx.notify();
        }
    }

    pub fn close_shared_panel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shared_panel.close(window, cx);
        self.sync_window_state(window, cx);
    }

    pub fn toggle_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.launcher.is_open() {
            self.close_launcher(window, cx);
        } else {
            self.open_launcher(window, cx);
        }
    }

    pub fn open_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.shared_panel.is_open() {
            self.shared_panel.close(window, cx);
        }
        let entity = cx.entity().downgrade();
        self.launcher.open(window, entity, cx);
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn close_launcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        self.launcher.close(window, entity);
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn activate_launcher_item(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let entity = cx.entity().downgrade();
        self.launcher.selected = index;
        self.launcher.activate_selected(window, entity, cx);
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn check_polkit_requests(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let had_request = self.polkit.request.is_some();
        let has_new = self.polkit.process_requests(cx);
        if has_new || (!had_request && self.polkit.request.is_some()) {
            if self.shared_panel.is_open() {
                self.shared_panel.close(window, cx);
            }
            if self.launcher.is_open() {
                let entity = cx.entity().downgrade();
                self.launcher.close(window, entity);
            }
            let entity = cx.entity().downgrade();
            self.polkit.open(window, entity, cx);
            self.sync_window_state(window, cx);
            cx.notify();
        }
    }

    pub fn cancel_polkit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.polkit.cancel(cx);
        if self.polkit.request.is_none() {
            self.close_polkit(window, cx);
        }
        cx.notify();
    }

    pub fn submit_polkit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.polkit.authenticating || self.polkit.password_input.is_empty() {
            return;
        }
        let Some(request) = &self.polkit.request else {
            return;
        };
        let user = request.user_name.clone();
        let cookie = request.cookie.clone();
        let response_cookie = cookie.clone();
        let password = self.polkit.password_input.text();
        self.polkit.password_input.clear();
        let timeout_error = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("polkit.timeout")
        } else {
            "La autenticación ha expirado. Inténtalo de nuevo.".to_string()
        };
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.polkit.authenticating = true;
        self.polkit.error = None;
        self.polkit.update_target_size();
        let entity = cx.entity().downgrade();
        Self::queue_animation_frame(window, entity);

        self.polkit.authentication = Some(services::spawn_tokio(async move {
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                services::authenticate_user(&user, &cookie, &password),
            )
            .await
            .unwrap_or(Err(timeout_error));
            let _ = sender.send(result);
        }));

        let bar_window = self.window_handle.downcast::<Bar>();
        self.polkit.response = Some(cx.spawn(async move |_this, cx| {
            let Ok(result) = receiver.await else {
                return;
            };
            if let Some(bar_window) = bar_window {
                cx.update(|app| {
                    let _ = bar_window.update(app, |bar, window, cx| {
                        if bar
                            .polkit
                            .request
                            .as_ref()
                            .is_none_or(|r| r.cookie != response_cookie)
                        {
                            return;
                        }
                        bar.polkit.authenticating = false;
                        bar.polkit.authentication = None;
                        match result {
                            Ok(()) => {
                                bar.polkit.finish(Ok(()));
                                bar.polkit.process_requests(cx);
                                if bar.polkit.request.is_none() {
                                    bar.close_polkit(window, cx);
                                }
                            }
                            Err(error) => {
                                bar.polkit.error = Some(error);
                                bar.polkit.update_target_size();
                                let entity = cx.entity().downgrade();
                                Self::queue_animation_frame(window, entity);
                            }
                        }
                        cx.notify();
                    });
                });
            }
        }));
        cx.notify();
    }

    pub fn close_polkit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let entity = cx.entity().downgrade();
        self.polkit.close(window, entity);
        self.sync_window_state(window, cx);
        cx.notify();
    }

    pub fn queue_animation_frame_checked(
        window: &Window,
        entity: gpui::WeakEntity<Self>,
        pending: &std::rc::Rc<std::cell::Cell<bool>>,
    ) {
        if pending.get() {
            return;
        }
        pending.set(true);
        Self::schedule_animation_tick(window, entity);
    }

    pub fn queue_animation_frame(window: &Window, entity: gpui::WeakEntity<Self>) {
        if let Some(strong) = entity.upgrade() {
            window.on_next_frame(move |window, cx| {
                let is_pending = strong.read(cx).animation_frame_pending.get();
                if is_pending {
                    return;
                }
                Self::run_animation_tick(window, entity, cx);
            });
        }
    }

    fn schedule_animation_tick(window: &Window, entity: gpui::WeakEntity<Self>) {
        window.on_next_frame(move |window, cx| {
            Self::run_animation_tick(window, entity, cx);
        });
    }

    fn run_animation_tick(window: &mut Window, entity: gpui::WeakEntity<Self>, cx: &mut gpui::App) {
        let entity_clone = entity.clone();
        let _ = entity.update(cx, |this, cx| {
            this.animation_frame_pending.set(false);
            let now = std::time::Instant::now();
            let mut needs_frame = false;

            let (sp_running, sp_closed) = this.shared_panel.advance(now);
            if sp_closed && this.shared_panel.is_open {
                this.shared_panel
                    .request_close(window, entity_clone.clone());
            }
            if sp_running {
                needs_frame = true;
            }

            let (l_running, l_closed) = this.launcher.advance(now);
            if l_closed {
                this.launcher.is_open = false;
            }
            if l_running {
                needs_frame = true;
            }

            let (p_running, p_closed) = this.polkit.advance(now);
            if p_closed {
                this.polkit.is_open = false;
            }
            if p_running {
                needs_frame = true;
            }

            if sp_closed || l_closed || p_closed || !needs_frame {
                this.sync_window_state(window, cx);
            }

            this.animation_frame_pending.set(needs_frame);
            cx.notify();
            if needs_frame {
                Self::schedule_animation_tick(window, entity_clone);
            }
        });
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
            let left_container = div()
                .id("bar-left-container")
                .flex_1()
                .h_full()
                .min_w_0()
                .flex()
                .items_center()
                .justify_start()
                .child(left);

            let center_container = div()
                .id("bar-center-container")
                .flex_shrink_0()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(center);

            let right_container = div()
                .id("bar-right-container")
                .flex_1()
                .h_full()
                .min_w_0()
                .flex()
                .items_center()
                .justify_end()
                .child(right);

            bar = bar
                .child(left_container)
                .child(center_container)
                .child(right_container);
        } else {
            let left_container = div()
                .id("bar-left-container")
                .flex_1()
                .w_full()
                .min_h_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_start()
                .child(left);

            let center_container = div()
                .id("bar-center-container")
                .flex_shrink_0()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(center);

            let right_container = div()
                .id("bar-right-container")
                .flex_1()
                .w_full()
                .min_h_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_end()
                .child(right);

            bar = bar
                .child(left_container)
                .child(center_container)
                .child(right_container);
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
            let mut border_bar = div()
                .id("bar-bottom-border")
                .absolute()
                .bottom_0()
                .left_0()
                .right_0()
                .h(px(1.0));

            if let Some(ref anim) = self.shared_panel.animator {
                let width = f32::from(anim.current_size.width).max(0.0);
                let height = f32::from(anim.current_size.height).max(0.0);
                let shoulder_val = 14.0_f32.min(height / 4.0).min(width / 4.0);
                let (sl, sr) = match anim.dock_edge {
                    shared_panel::DockEdge::Center => (shoulder_val, shoulder_val),
                    shared_panel::DockEdge::Left => (0.0, shoulder_val),
                    shared_panel::DockEdge::Right => (shoulder_val, 0.0),
                };
                let neck_left = anim.current_x - px(sl);
                let neck_right = anim.current_x + anim.current_size.width + px(sr);

                let left_end = neck_left.min(border_end);
                if left_end > border_start {
                    border_bar = border_bar.child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(border_start)
                            .w(left_end - border_start)
                            .bg(border_color),
                    );
                }
                if anim.opacity < 1.0 {
                    let mid_start = neck_left.max(border_start);
                    let mid_end = neck_right.min(border_end);
                    if mid_end > mid_start {
                        border_bar = border_bar.child(
                            div()
                                .absolute()
                                .top_0()
                                .bottom_0()
                                .left(mid_start)
                                .w(mid_end - mid_start)
                                .bg(border_color)
                                .opacity(1.0 - anim.opacity),
                        );
                    }
                }
                let right_start = neck_right.max(border_start);
                if border_end > right_start {
                    border_bar = border_bar.child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left(right_start)
                            .w(border_end - right_start)
                            .bg(border_color),
                    );
                }
            } else if border_end > border_start {
                border_bar = border_bar.child(
                    div()
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left(border_start)
                        .w(border_end - border_start)
                        .bg(border_color),
                );
            }

            bar = bar.child(border_bar);
        }

        let mut root = div()
            .id("capsule-bar-root")
            .size_full()
            .relative()
            .font_family(theme.font_family())
            .track_focus(&self.focus_handle)
            .key_context("capsule-bar")
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.shared_panel.dashboard_state.dragging_volume {
                    this.dashboard_volume_at(f32::from(event.position.x), cx);
                }
                if this.shared_panel.dashboard_state.dragging_input_volume {
                    this.dashboard_input_volume_at(f32::from(event.position.x), cx);
                }
                if this.shared_panel.dashboard_state.dragging_brightness {
                    this.dashboard_brightness_at(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.finish_dashboard_sliders(cx);
                    if this.shared_panel.power_hold.is_some() {
                        this.shared_panel.cancel_power_hold();
                        cx.notify();
                    }
                }),
            )
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if this.polkit.is_open() {
                    let entity = cx.entity().downgrade();
                    match this.polkit.handle_key_down(event, window, entity, cx) {
                        popups::polkit::PolkitKeyAction::Cancel => this.cancel_polkit(window, cx),
                        popups::polkit::PolkitKeyAction::Submit => this.submit_polkit(window, cx),
                        _ => {}
                    }
                    cx.stop_propagation();
                    return;
                }
                if this.launcher.is_open() {
                    let entity = cx.entity().downgrade();
                    this.launcher.handle_key_down(event, window, entity, cx);
                    this.sync_window_state(window, cx);
                    cx.stop_propagation();
                    return;
                }
                if !this.shared_panel.is_open() {
                    return;
                }
                match this.shared_panel.active_module {
                    Some(shared_panel::SharedPanelModule::Dashboard) => {
                        this.handle_dashboard_key_down(event, window, cx);
                        cx.stop_propagation();
                    }
                    Some(shared_panel::SharedPanelModule::Clipboard) => {
                        this.handle_clipboard_key_down(event, window, cx);
                        cx.stop_propagation();
                    }
                    Some(shared_panel::SharedPanelModule::Emoji) => {
                        this.handle_emoji_key_down(event, window, cx);
                        cx.stop_propagation();
                    }
                    _ => {
                        if event.keystroke.key == "escape" {
                            this.close_shared_panel(window, cx);
                            cx.stop_propagation();
                        }
                    }
                }
            }))
            .child(bar);

        if bar_config.convex && is_horizontal {
            let bg_color = style::background(&theme);
            let is_bottom = self.location == BarLocation::Bottom;
            let (left_stroke_opacity, right_stroke_opacity) =
                if let Some(ref anim) = self.shared_panel.animator {
                    let width = f32::from(anim.current_size.width).max(0.0);
                    let height = f32::from(anim.current_size.height).max(0.0);
                    let shoulder_val = 14.0_f32.min(height / 4.0).min(width / 4.0);
                    let (sl, sr) = match anim.dock_edge {
                        shared_panel::DockEdge::Center => (shoulder_val, shoulder_val),
                        shared_panel::DockEdge::Left => (0.0, shoulder_val),
                        shared_panel::DockEdge::Right => (shoulder_val, 0.0),
                    };
                    let neck_left = anim.current_x - px(sl);
                    let neck_right = anim.current_x + anim.current_size.width + px(sr);
                    let l_op = if neck_left < ear_radius {
                        (1.0 - anim.opacity).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    let r_op = if neck_right > viewport.width - ear_radius {
                        (1.0 - anim.opacity).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    (l_op, r_op)
                } else {
                    (1.0, 1.0)
                };

            let left_ear_border = style::border(&theme).opacity(0.35 * left_stroke_opacity);
            let right_ear_border = style::border(&theme).opacity(0.35 * right_stroke_opacity);
            let r = shared_panel::view::CONVEX_EAR_RADIUS;

            let mut left_ear = div()
                .id("bar-convex-ear-left")
                .absolute()
                .left_0()
                .size(px(r))
                .child(shared_panel::view::convex_bar_ear_canvas(
                    r,
                    false,
                    is_bottom,
                    bg_color,
                    left_ear_border,
                ));
            let mut right_ear = div()
                .id("bar-convex-ear-right")
                .absolute()
                .right_0()
                .size(px(r))
                .child(shared_panel::view::convex_bar_ear_canvas(
                    r,
                    true,
                    is_bottom,
                    bg_color,
                    right_ear_border,
                ));

            if is_bottom {
                left_ear = left_ear.bottom(self.height - px(1.0));
                right_ear = right_ear.bottom(self.height - px(1.0));
            } else {
                left_ear = left_ear.top(self.height - px(1.0));
                right_ear = right_ear.top(self.height - px(1.0));
            }

            root = root.child(left_ear).child(right_ear);
        }

        if let Some(ref anim) = self.shared_panel.animator {
            let bg_color = style::background(&theme);
            let current_x = anim.current_x;
            let current_size = anim.current_size;
            let target_size = anim.target_size;
            let opacity = anim.opacity;
            let dock_edge = anim.dock_edge;
            let content = self.shared_panel.render_content(window, cx);

            let backdrop = div()
                .id("shared-panel-backdrop")
                .absolute()
                .top(self.height)
                .left_0()
                .right_0()
                .bottom_0()
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, window, cx| {
                        this.close_shared_panel(window, cx);
                    }),
                );

            let panel = div()
                .id("shared-panel-container")
                .key_context("shared-panel")
                .absolute()
                .top(self.height - px(1.0))
                .left(current_x)
                .w(current_size.width)
                .h(current_size.height + px(1.0))
                .opacity(opacity)
                .overflow_hidden()
                .on_mouse_up(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.shared_panel.dashboard_state.dragging_volume = false;
                        this.shared_panel.dashboard_state.dragging_input_volume = false;
                        this.finish_dashboard_brightness(cx);
                        if this.shared_panel.power_hold.is_some() {
                            this.shared_panel.cancel_power_hold();
                            cx.notify();
                        }
                    }),
                )
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(gpui::MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(|_, _, cx| {
                    cx.stop_propagation();
                })
                .child(shared_panel::view::concave_panel_canvas(
                    Size {
                        width: current_size.width,
                        height: current_size.height + px(1.0),
                    },
                    dock_edge,
                    20.0,
                    bg_color,
                    border_color,
                ))
                .child(
                    div()
                        .w(target_size.width)
                        .h(target_size.height + px(1.0))
                        .pt(px(13.0))
                        .overflow_hidden()
                        .child(content),
                );

            root = root.child(backdrop).child(panel);
        }

        if self.launcher.is_visible() {
            root = root.child(self.launcher.render(window, cx));
        }

        if self.polkit.is_visible() {
            root = root.child(self.polkit.render(window, cx));
        }

        root
    }
}
