pub(crate) mod dashboard;
pub(crate) mod default;
pub(crate) mod launcher;

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
    dashboard: Entity<DashboardModule>,
}

impl CapsuleModuleManager {
    pub fn new(cx: &mut impl AppContext) -> Self {
        Self {
            current: CapsuleModuleId::Default,
            default: cx.new(DefaultModule::new),
            launcher: cx.new(LauncherModule::new),
            dashboard: cx.new(DashboardModule::new),
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
        let launcher = self.launcher.read_with(cx, |module, _| module.size());
        let dashboard = self.dashboard.read_with(cx, |module, _| module.size());
        Size {
            width: default.width.max(launcher.width).max(dashboard.width),
            height: default.height.max(launcher.height).max(dashboard.height),
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
        }
    }

    pub fn current_size(&self, cx: &impl AppContext) -> Size<Pixels> {
        match self.current {
            CapsuleModuleId::Default => self.default.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Launcher => self.launcher.read_with(cx, |module, _| module.size()),
            CapsuleModuleId::Dashboard => self.dashboard.read_with(cx, |module, _| module.size()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsuleModuleId {
    Default,
    Launcher,
    Dashboard,
}

pub enum CapsuleModuleEvent {
    Close,
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
        subscriptions.push(cx.observe_global::<ui::theme::Theme>(|_, cx| cx.notify()));
        let size = module_manager.current_size(cx);

        let ipc_task = Self::start_ipc(cx);

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
            animation_task: None,
            _subscriptions: subscriptions,
            _ipc_task: ipc_task,
        };
        capsule.apply_window_geometry(window, cx);
        capsule
    }

    pub(super) fn handle_module_event(
        &mut self,
        event: &CapsuleModuleEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            CapsuleModuleEvent::Open(id) => {
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
                self.module_manager.open(*id);
            }
            CapsuleModuleEvent::Close => self.module_manager.close(),
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
        let compositor = state.compositor.clone();
        self.animator.transition_to(target, Instant::now());
        if self.animation_task.is_none() && self.animator.advance(Instant::now()) {
            self.animation_task = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(
                            compositor
                                .get_frame_duration()
                                .max(Duration::from_millis(2)),
                        )
                        .await;

                    let running = this.update(cx, |capsule, cx| {
                        let running = capsule.animator.advance(Instant::now());
                        if !running {
                            capsule.animation_task = None;
                        }
                        capsule.sync_window(cx);
                        cx.notify();
                        running
                    });

                    match running {
                        Ok(true) => {}
                        Ok(false) | Err(_) => break,
                    }
                }
            }));
        } else if self.animation_task.is_none() {
            self.sync_window(cx);
        }
        cx.notify();
    }
}
