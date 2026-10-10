use gpui::{
    AnyView, Context, IntoElement, ParentElement, Render, Styled, Subscription, Task, Window, div,
    prelude::*,
};

use crate::new_capsule::{animator::Animator, module::CapsuleModuleManager};

pub(crate) mod animator;
mod ipc_handler;
pub(crate) mod module;
pub(crate) mod orbit;
pub(crate) mod satellite;
pub(crate) mod widgets;
mod window_state;

pub enum CapsuleLocation {
    TOP,
    BOTTOM,
}

struct OutgoingContent {
    view: AnyView,
    size: gpui::Size<gpui::Pixels>,
    scale: f32,
    opacity: f32,
}

pub struct Capsule {
    location: CapsuleLocation,
    window_state: window_state::WindowState,
    module_manager: CapsuleModuleManager,
    animator: Animator,
    outgoing_content: Option<OutgoingContent>,
    orbit: gpui::Entity<orbit::Orbit>,
    satellite: satellite::Satellite,
    satellite_animation_task: Option<Task<()>>,
    animation_frame_pending: bool,
    _subscriptions: Vec<Subscription>,
    _ipc_task: Task<()>,
    _config_task: Task<()>,
}

impl Capsule {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let module_manager = CapsuleModuleManager::new(cx);
        Self::with_module_manager(module_manager, window, cx)
    }
}

impl Render for Capsule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = self.animator.size();
        let content_progress = self.animator.content_progress();
        let theme = cx.global::<ui::theme::Theme>().clone();
        let concave = self.window_state.style == services::CapsuleStyle::Concave;
        let bottom = matches!(self.location, CapsuleLocation::BOTTOM);
        let (radius, _) = widgets::style::concave_radii(size, self.window_state.radius);
        let container = div()
            .id("capsule-drop-target")
            .relative()
            .on_drop(cx.listener(Self::drop_on_shelf))
            .on_drop(cx.listener(Self::drop_shelf_file))
            .on_drag_move::<widgets::shelf::DraggedFile>(cx.listener(
                |capsule, event: &gpui::DragMoveEvent<widgets::shelf::DraggedFile>, _, cx| {
                    capsule
                        .set_shelf_drop_hovered(event.bounds.contains(&event.event.position), cx);
                },
            ))
            .on_drag_move::<gpui::ExternalPaths>(cx.listener(
                |capsule, event: &gpui::DragMoveEvent<gpui::ExternalPaths>, _, cx| {
                    capsule
                        .set_shelf_drop_hovered(event.bounds.contains(&event.event.position), cx);
                },
            ))
            .on_mouse_exit(cx.listener(|capsule, _, _, cx| {
                capsule.set_shelf_drop_hovered(false, cx);
            }))
            .w(size.width)
            .h(size.height)
            .flex_shrink_0()
            .when(
                self.module_manager.current_id() != module::CapsuleModuleId::Dashboard
                    || content_progress < 1.0,
                |s| s.overflow_hidden(),
            )
            .when(!concave, |s| {
                s.rounded(gpui::px(self.window_state.radius))
                    .bg(widgets::style::background(&theme))
            })
            .when(concave, |s| {
                s.when(bottom, |s| {
                    s.rounded_tl(gpui::px(radius)).rounded_tr(gpui::px(radius))
                })
                .when(!bottom, |s| {
                    s.rounded_bl(gpui::px(radius)).rounded_br(gpui::px(radius))
                })
            })
            .font_family(theme.font_family())
            .text_color(theme.foreground())
            .when_some(self.outgoing_content.as_ref(), |s, outgoing| {
                s.child(
                    div()
                        .id("capsule-outgoing-content")
                        .absolute()
                        .top((size.height - outgoing.size.height * outgoing.scale) * 0.5)
                        .left((size.width - outgoing.size.width * outgoing.scale) * 0.5)
                        .w(outgoing.size.width * outgoing.scale)
                        .h(outgoing.size.height * outgoing.scale)
                        .opacity(outgoing.opacity * (1.0 - content_progress))
                        .capture_any_mouse_down(|_, _, cx| cx.stop_propagation())
                        .capture_any_mouse_up(|_, _, cx| {
                            if !cx.has_active_drag() {
                                cx.stop_propagation();
                            }
                        })
                        .child(outgoing.view.clone()),
                )
            })
            .child(
                div()
                    .id("capsule-module-content")
                    .relative()
                    .top(size.height * (1.0 - self.animator.content_scale()) * 0.5)
                    .left(size.width * (1.0 - self.animator.content_scale()) * 0.5)
                    .w(size.width * self.animator.content_scale())
                    .h(size.height * self.animator.content_scale())
                    .occlude()
                    .on_drop(cx.listener(Self::drop_on_shelf))
                    .on_drop(cx.listener(Self::drop_shelf_file))
                    .opacity(if self.outgoing_content.is_some() {
                        content_progress
                    } else {
                        1.0
                    })
                    .child(self.module_manager.current()),
            );
        let container = div()
            .relative()
            .w(size.width)
            .h(size.height)
            .flex_shrink_0()
            .when(concave, |s| {
                s.child(widgets::style::concave_background(
                    size,
                    self.window_state.radius,
                    bottom,
                    widgets::style::background(&theme),
                ))
            })
            .child(container)
            .child(
                self.orbit
                    .read(cx)
                    .presentation(
                        window.viewport_size(),
                        window_state::visible_bounds(
                            window.viewport_size(),
                            size,
                            gpui::px(self.window_state.margin),
                            &self.location,
                        ),
                        gpui::px(cx.global::<services::AppState>().config.get().ui.gap),
                    )
                    .render(&theme, cx),
            );
        let root = div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .items_center();
        match self.location {
            CapsuleLocation::TOP => root.pt(gpui::px(self.window_state.margin)).child(container),
            CapsuleLocation::BOTTOM => root
                .justify_end()
                .pb(gpui::px(self.window_state.margin))
                .child(container),
        }
    }
}
