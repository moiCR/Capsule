use gpui::{Context, IntoElement, ParentElement, Render, Styled, Subscription, Task, Window, div};

use crate::new_capsule::{animator::Animator, module::CapsuleModuleManager};

mod animator;
mod ipc_handler;
pub(crate) mod module;
pub(crate) mod widgets;
mod window_state;

pub enum CapsuleLocation {
    TOP,
    BOTTOM,
}

pub struct Capsule {
    location: CapsuleLocation,
    window_state: window_state::WindowState,
    module_manager: CapsuleModuleManager,
    animator: Animator,
    animation_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
    _ipc_task: Task<()>,
}

impl Capsule {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let module_manager = CapsuleModuleManager::new(cx);
        Self::with_module_manager(module_manager, window, cx)
    }
}

impl Render for Capsule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.animator.size();
        let theme = cx.global::<ui::theme::Theme>();
        let container = div()
            .w(size.width)
            .h(size.height)
            .flex_shrink_0()
            .overflow_hidden()
            .rounded(gpui::px(self.window_state.radius))
            .bg(theme.background())
            .text_color(theme.foreground())
            .child(self.module_manager.current());
        let root = div().size_full().flex().flex_col().items_center();
        match self.location {
            CapsuleLocation::TOP => root.pt(gpui::px(self.window_state.margin)).child(container),
            CapsuleLocation::BOTTOM => root
                .justify_end()
                .pb(gpui::px(self.window_state.margin))
                .child(container),
        }
    }
}
