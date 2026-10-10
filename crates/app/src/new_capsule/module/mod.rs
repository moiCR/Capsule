pub(crate) mod appearance;
pub(crate) mod clipboard;
pub(crate) mod dashboard;
pub(crate) mod default;
pub(crate) mod launcher;
pub(crate) mod notification;
pub(crate) mod polkit;
pub(crate) mod record;
pub(crate) mod shelf;

use std::time::{Duration, Instant};

use gpui::{AnyView, AppContext, Context, Entity, Pixels, Size, Subscription};

use crate::new_capsule::{
    Capsule, CapsuleLocation, animator::Animator, module::launcher::LauncherModule,
};

use self::{dashboard::DashboardModule, default::DefaultModule};

pub struct CapsuleModuleManager {
    current: CapsuleModuleId,
    default: Entity<DefaultModule>,
    launcher: Entity<LauncherModule>,
    pub(super) dashboard: Entity<DashboardModule>,
    pub(super) appearance: Entity<appearance::AppearanceModule>,
    pub(super) clipboard: Entity<clipboard::ClipboardModule>,
    pub(super) shelf: Entity<shelf::ShelfModule>,
    pub(super) record: Entity<record::RecordModule>,
    pub(super) notification: Entity<notification::NotificationModule>,
    pub(super) polkit: Entity<polkit::PolkitModule>,
}

impl CapsuleModuleManager {
    pub fn new(cx: &mut impl AppContext) -> Self {
        Self {
            current: CapsuleModuleId::Default,
            default: cx.new(DefaultModule::new),
            launcher: cx.new(LauncherModule::new),
            dashboard: cx.new(DashboardModule::new),
            appearance: cx.new(appearance::AppearanceModule::new),
            clipboard: cx.new(clipboard::ClipboardModule::new),
            shelf: cx.new(shelf::ShelfModule::new),
            record: cx.new(record::RecordModule::new),
            notification: cx.new(notification::NotificationModule::new),
            polkit: cx.new(polkit::PolkitModule::new),
        }
    }

    pub fn open(&mut self, id: CapsuleModuleId) {
        self.current = id;
    }

    pub fn close(&mut self) {
        self.current = CapsuleModuleId::Default;
    }

    pub fn subscribe(&self, cx: &mut Context<Capsule>) -> Vec<Subscription> {
        vec![
            cx.subscribe(&self.notification, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.polkit, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.shelf, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.record, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.clipboard, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.appearance, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.dashboard, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx)
            }),
            cx.subscribe(&self.default, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx);
            }),
            cx.subscribe(&self.launcher, |capsule, _, event, cx| {
                capsule.handle_module_event(event, cx);
            }),
        ]
    }

    pub fn maximum_size(&self, cx: &impl AppContext) -> Size<Pixels> {
        let default = self.default.read_with(cx, |module, _| module.size());
        let launcher = gpui::size(
            gpui::px(super::widgets::launcher::WIDTH),
            gpui::px(super::widgets::launcher::HEIGHT),
        );
        let shelf = gpui::size(
            gpui::px(super::widgets::shelf::WIDTH),
            gpui::px(super::widgets::shelf::MAX_HEIGHT),
        );
        let record = gpui::size(
            gpui::px(super::widgets::record::WIDTH),
            gpui::px(super::widgets::record::MAX_HEIGHT),
        );
        let dashboard = super::widgets::dashboard::module_size(true);
        let notification = gpui::size(
            gpui::px(super::widgets::notification::WIDTH),
            gpui::px(super::widgets::notification::HISTORY_HEIGHT),
        );
        let polkit = self.polkit.read_with(cx, |module, _| module.size());
        let appearance = self.appearance.read_with(cx, |module, _| module.size());
        let clipboard = self.clipboard.read_with(cx, |module, _| module.size());
        Size {
            width: default
                .width
                .max(launcher.width)
                .max(dashboard.width)
                .max(appearance.width)
                .max(clipboard.width)
                .max(shelf.width)
                .max(record.width)
                .max(polkit.width)
                .max(notification.width),
            height: default
                .height
                .max(launcher.height)
                .max(dashboard.height)
                .max(appearance.height)
                .max(clipboard.height)
                .max(shelf.height)
                .max(record.height)
                .max(polkit.height)
                .max(notification.height),
        }
    }

    pub(super) fn launcher_focus_handle(&self, cx: &impl AppContext) -> gpui::FocusHandle {
        self.launcher
            .read_with(cx, |module, _| module.focus_handle())
    }

    pub(super) fn dashboard_focus_handle(&self, cx: &impl AppContext) -> gpui::FocusHandle {
        self.dashboard
            .read_with(cx, |module, _| module.focus_handle())
    }

    pub fn current_id(&self) -> CapsuleModuleId {
        self.current
    }

    pub fn current(&self) -> AnyView {
        match self.current {
            CapsuleModuleId::Default => self.default.clone().into(),
            CapsuleModuleId::Launcher => self.launcher.clone().into(),
            CapsuleModuleId::Dashboard => self.dashboard.clone().into(),
            CapsuleModuleId::Clipboard => self.clipboard.clone().into(),
            CapsuleModuleId::Shelf => self.shelf.clone().into(),
            CapsuleModuleId::Record => self.record.clone().into(),
            CapsuleModuleId::Polkit => self.polkit.clone().into(),
            CapsuleModuleId::Notification => self.notification.clone().into(),
            CapsuleModuleId::Themes | CapsuleModuleId::Wallpapers => self.appearance.clone().into(),
        }
    }

    pub fn current_size(&self, cx: &impl AppContext) -> Size<Pixels> {
        match self.current {
            CapsuleModuleId::Default => self.default.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Launcher => self.launcher.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Dashboard => self.dashboard.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Clipboard => self.clipboard.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Shelf => self.shelf.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Record => self.record.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Polkit => self.polkit.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Notification => {
                self.notification.read_with(cx, |module, _| module.size())
            }
            CapsuleModuleId::Themes | CapsuleModuleId::Wallpapers => {
                self.appearance.read_with(cx, |module, _| module.size())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleModuleId {
    Default,
    Launcher,
    Dashboard,
    Themes,
    Wallpapers,
    Clipboard,
    Shelf,
    Record,
    Polkit,
    Notification,
}

pub enum CapsuleModuleEvent {
    NotificationChanged,
    ToggleSatellite(super::satellite::SatelliteId),
    Close,
    CloseSatelliteId(super::satellite::SatelliteId),
    SatelliteBounds(super::satellite::SatelliteId, gpui::Bounds<Pixels>),
    Open(CapsuleModuleId),
    SizeChanged(CapsuleModuleId),
}

pub trait CapsuleModule {
    fn size(&self) -> Size<Pixels>;
}

impl Capsule {
    pub(crate) fn with_module_manager(
        module_manager: CapsuleModuleManager,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = module_manager.subscribe(cx);
        let satellite = super::satellite::Satellite::new();
        subscriptions.push(cx.observe_global::<ui::theme::Theme>(|_, cx| cx.notify()));
        let size = module_manager.current_size(cx);

        let ipc_task = Self::start_ipc(cx);
        let mut config_changes = cx.global::<services::AppState>().config.subscribe();
        let config_task = cx.spawn(async move |this, cx| {
            loop {
                match config_changes.recv().await {
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
                if this
                    .update(cx, |capsule, cx| {
                        let current = capsule.module_manager.current_id();
                        capsule.handle_module_event(&CapsuleModuleEvent::SizeChanged(current), cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        });

        let mut capsule = Self {
            location: CapsuleLocation::TOP,
            window_state: super::window_state::WindowState::new(window),
            module_manager,
            animator: Animator::new(
                size,
                Duration::from_millis(
                    cx.global::<services::AppState>()
                        .config
                        .get()
                        .ui
                        .animation_duration_ms as u64,
                ),
            ),
            animation_frame_pending: false,
            satellite,
            satellite_animation_task: None,
            _subscriptions: subscriptions,
            _ipc_task: ipc_task,
            _config_task: config_task,
        };
        capsule.apply_window_geometry(window, cx);
        capsule
    }

    pub(super) fn handle_module_event(
        &mut self,
        event: &CapsuleModuleEvent,
        cx: &mut Context<Self>,
    ) {
        let previous_module = self.module_manager.current_id();
        if matches!(event, CapsuleModuleEvent::NotificationChanged) {
            let notification = self.module_manager.notification.read(cx);
            let popup_active =
                previous_module == CapsuleModuleId::Notification && !notification.history;
            let show = notification.latest.is_some()
                && (previous_module == CapsuleModuleId::Default || popup_active);
            let close = notification.latest.is_none() && popup_active;
            if show {
                self.module_manager
                    .notification
                    .update(cx, |module, cx| module.open(false, cx));
                self.handle_module_event(
                    &CapsuleModuleEvent::Open(CapsuleModuleId::Notification),
                    cx,
                );
            } else if close {
                self.handle_module_event(&CapsuleModuleEvent::Close, cx);
            }
            return;
        }
        if self.module_manager.polkit.read(cx).has_request() {
            match event {
                CapsuleModuleEvent::Open(id) if *id != CapsuleModuleId::Polkit => return,
                CapsuleModuleEvent::Close => {
                    self.module_manager
                        .polkit
                        .update(cx, |module, cx| module.cancel(cx));
                    return;
                }
                _ => {}
            }
        }
        match event {
            CapsuleModuleEvent::NotificationChanged => return,
            CapsuleModuleEvent::SatelliteBounds(id, bounds) => {
                if self.satellite.contains(id) {
                    let previous = self
                        .window_state
                        .satellite_bounds
                        .iter_mut()
                        .find(|(key, _)| key == id);
                    let changed = match previous {
                        Some((_, previous)) if previous == bounds => false,
                        Some((_, previous)) => {
                            *previous = *bounds;
                            true
                        }
                        None => {
                            self.window_state
                                .satellite_bounds
                                .push((id.clone(), *bounds));
                            true
                        }
                    };
                    if changed {
                        self.sync_window(cx);
                    }
                }
                return;
            }
            CapsuleModuleEvent::CloseSatelliteId(id) => {
                self.close_satellite(id.clone(), cx);
                return;
            }
            CapsuleModuleEvent::ToggleSatellite(id) => {
                self.toggle_satellite(id.clone(), cx);
                return;
            }
            CapsuleModuleEvent::Open(id) => {
                self.set_satellite_open(false, cx);
                if *id == CapsuleModuleId::Launcher && self.module_manager.current_id() != *id {
                    self.module_manager
                        .launcher
                        .update(cx, |module, cx| module.reset_search(cx));
                }
                if *id == CapsuleModuleId::Dashboard && self.module_manager.current_id() != *id {
                    self.module_manager
                        .dashboard
                        .update(cx, |module, cx| module.reset(cx));
                }
                if matches!(id, CapsuleModuleId::Themes | CapsuleModuleId::Wallpapers) {
                    let kind = if *id == CapsuleModuleId::Themes {
                        appearance::AppearanceKind::Themes
                    } else {
                        appearance::AppearanceKind::Wallpapers
                    };
                    self.module_manager
                        .appearance
                        .update(cx, |module, cx| module.open(kind, cx));
                }
                if *id == CapsuleModuleId::Clipboard && self.module_manager.current_id() != *id {
                    self.module_manager
                        .clipboard
                        .update(cx, |module, cx| module.open(cx));
                }
                if *id == CapsuleModuleId::Shelf && self.module_manager.current_id() != *id {
                    self.module_manager
                        .shelf
                        .update(cx, |module, cx| module.open(cx));
                }
                if *id == CapsuleModuleId::Record && self.module_manager.current_id() != *id {
                    self.module_manager
                        .record
                        .update(cx, |module, cx| module.open(cx));
                }
                self.module_manager.open(*id);
            }
            CapsuleModuleEvent::Close => {
                self.set_satellite_open(false, cx);
                self.module_manager.close();
            }
            CapsuleModuleEvent::SizeChanged(id) => {
                if self.module_manager.current_id() != *id {
                    return;
                }
            }
        }

        let target = self.module_manager.current_size(cx);
        let state = cx.global::<services::AppState>();
        self.animator.set_duration(Duration::from_millis(
            state.config.get().ui.animation_duration_ms as u64,
        ));
        let now = Instant::now();
        if previous_module != self.module_manager.current_id() {
            self.animator.reveal_content(now);
        }
        self.animator.transition_to(target, now);
        let animating = self.animator.advance(Instant::now());
        self.sync_window(cx);
        if !self.animation_frame_pending && animating {
            self.animation_frame_pending = true;
            let entity = cx.entity().downgrade();
            let _ = self.window_state.handle.update(cx, |_, window, _| {
                Self::queue_animation_frame(window, entity);
            });
        }
        cx.notify();
    }

    fn queue_animation_frame(window: &gpui::Window, entity: gpui::WeakEntity<Self>) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |capsule, cx| {
                let previous_size = capsule.animator.size();
                let running = capsule.animator.advance(Instant::now());
                capsule.animation_frame_pending = running;
                if previous_size != capsule.animator.size() {
                    capsule.apply_window_geometry(window, cx);
                }
                cx.notify();
                if running {
                    Self::queue_animation_frame(window, cx.entity().downgrade());
                }
            });
        });
    }
}
