use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    App, BoxShadow, ColorExt, Context, EventEmitter, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render, Size, Task, Window,
    div, point, px, svg,
};
use services::{AppState, CapsuleAnimation, PolkitAuthRequest};
use ui::components::{Button, ButtonVariant, Input, InputState};
use ui::theme::Theme;

use super::{PopupEvent, animator::PolkitAnimator};
use crate::bar::style;

pub fn compute_target_size(has_error: bool, message: &str) -> Size<Pixels> {
    let msg_len = message.len();
    let base_h = if msg_len > 120 {
        290.0
    } else if msg_len > 60 {
        260.0
    } else {
        240.0
    };
    let error_extra = if has_error { 38.0 } else { 0.0 };
    Size {
        width: px(460.0),
        height: px(base_h + error_extra),
    }
}

pub struct BarPolkit {
    pub request: Option<PolkitAuthRequest>,
    pub password_input: InputState,
    pub error: Option<String>,
    pub authenticating: bool,
    pub animator: Option<PolkitAnimator>,
    pub is_open: bool,
    pub focus: FocusHandle,
    pub authentication: Option<tokio::task::JoinHandle<()>>,
    pub response: Option<Task<()>>,
    pub responder: Option<tokio::sync::oneshot::Sender<Result<(), String>>>,
    _polkit_task: Option<Task<()>>,
}

impl EventEmitter<PopupEvent> for BarPolkit {}

impl BarPolkit {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let polkit_task = if cx.has_global::<AppState>() {
            let service = cx.global::<AppState>().polkit.clone();
            Some(cx.spawn(async move |this, cx| {
                loop {
                    service.wait_for_change().await;
                    if this
                        .update(cx, |polkit, cx| {
                            polkit.check_requests(cx);
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

        let mut polkit = Self {
            request: None,
            password_input: InputState::new("").with_focused(true).with_max_len(120),
            error: None,
            authenticating: false,
            animator: None,
            is_open: false,
            focus: cx.focus_handle(),
            authentication: None,
            response: None,
            responder: None,
            _polkit_task: polkit_task,
        };
        polkit.check_requests(cx);
        polkit
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_visible(&self) -> bool {
        self.is_open
            || self
                .animator
                .as_ref()
                .map(|a| a.opacity > 0.0)
                .unwrap_or(false)
    }

    pub fn check_requests(&mut self, cx: &mut Context<Self>) {
        let had_request = self.request.is_some();
        let has_new = self.process_requests(cx);
        if has_new || (!had_request && self.request.is_some()) {
            self.open(cx);
        }
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        let (duration, animation, bounciness, squash_stretch, reduced_motion) =
            if cx.has_global::<AppState>() {
                let cfg = cx.global::<AppState>().config.get();
                (
                    Duration::from_millis(cfg.ui.bar.animation_duration_ms as u64),
                    cfg.ui.bar.animation,
                    cfg.ui.bar.spring_bounciness,
                    cfg.ui.bar.squash_stretch,
                    cfg.ui.bar.reduced_motion || cfg.ui.reduced_motion,
                )
            } else {
                (
                    Duration::from_millis(220),
                    CapsuleAnimation::Fluid,
                    0.75,
                    false,
                    false,
                )
            };

        self.is_open = true;
        self.password_input.clear();
        self.error = None;
        self.authenticating = false;

        let msg = self
            .request
            .as_ref()
            .map(|r| r.message.as_str())
            .unwrap_or("");
        let target_size = compute_target_size(self.error.is_some(), msg);

        let mut anim = PolkitAnimator::new(
            target_size,
            duration,
            animation,
            bounciness,
            squash_stretch,
            reduced_motion,
        );
        anim.advance(Instant::now());
        self.animator = Some(anim);
        cx.emit(PopupEvent::Open);
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut anim) = self.animator
            && !anim.is_closing()
        {
            anim.request_close();
            cx.emit(PopupEvent::Close);
            cx.notify();
        }
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        if let Some(ref mut anim) = self.animator {
            let (running, closed) = anim.advance(now);
            if closed {
                self.is_open = false;
                self.animator = None;
            }
            (running, closed)
        } else {
            (false, false)
        }
    }

    pub fn update_target_size(&mut self) {
        let msg = self
            .request
            .as_ref()
            .map(|r| r.message.as_str())
            .unwrap_or("");
        let target = compute_target_size(self.error.is_some(), msg);
        if let Some(ref mut anim) = self.animator {
            anim.transition_to(target, Instant::now());
        }
    }

    pub fn process_requests(&mut self, cx: &App) -> bool {
        if self.request.is_some() || self.authenticating {
            return false;
        }
        let Some(service) = cx
            .try_global::<AppState>()
            .map(|state| state.polkit.clone())
        else {
            return false;
        };
        let Some((request, responder)) = service.pop_request() else {
            return false;
        };
        self.request = Some(request);
        self.responder = Some(responder);
        self.password_input.clear();
        self.error = None;
        self.authenticating = false;
        self.update_target_size();
        true
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        self.finish(Err("Cancelled by user".into()));
        self.process_requests(cx);
        if self.request.is_none() {
            self.close(cx);
        }
        cx.notify();
    }

    pub fn submit(&mut self, cx: &mut Context<Self>) {
        if self.authenticating || self.password_input.is_empty() {
            return;
        }
        let Some(request) = &self.request else {
            return;
        };
        let user = request.user_name.clone();
        let cookie = request.cookie.clone();
        let response_cookie = cookie.clone();
        let password = self.password_input.text();
        self.password_input.clear();
        let timeout_error = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("polkit.timeout")
        } else {
            "La autenticación ha expirado. Inténtalo de nuevo.".to_string()
        };
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.authenticating = true;
        self.error = None;
        self.update_target_size();
        cx.emit(PopupEvent::RequestAnimationFrame);

        self.authentication = Some(services::spawn_tokio(async move {
            let result = tokio::time::timeout(
                Duration::from_secs(30),
                services::authenticate_user(&user, &cookie, &password),
            )
            .await
            .unwrap_or(Err(timeout_error));
            let _ = sender.send(result);
        }));

        self.response = Some(cx.spawn(async move |this, cx| {
            let Ok(result) = receiver.await else {
                return;
            };
            let _ = this.update(cx, |polkit, cx| {
                if polkit
                    .request
                    .as_ref()
                    .is_none_or(|r| r.cookie != response_cookie)
                {
                    return;
                }
                polkit.authenticating = false;
                polkit.authentication = None;
                match result {
                    Ok(()) => {
                        polkit.finish(Ok(()));
                        polkit.process_requests(cx);
                        if polkit.request.is_none() {
                            polkit.close(cx);
                        }
                    }
                    Err(error) => {
                        polkit.error = Some(error);
                        polkit.update_target_size();
                        cx.emit(PopupEvent::RequestAnimationFrame);
                    }
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    pub fn finish(&mut self, result: Result<(), String>) {
        if let Some(authentication) = self.authentication.take() {
            authentication.abort();
        }
        self.response = None;
        if let Some(responder) = self.responder.take() {
            let _ = responder.send(result);
        }
        self.request = None;
        self.password_input.clear();
        self.error = None;
        self.authenticating = false;
    }

    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        if key == "escape" || (key == "f4" && event.keystroke.modifiers.alt) {
            self.cancel(cx);
            cx.stop_propagation();
            return;
        }

        if key == "enter" || key == "return" {
            self.submit(cx);
            cx.stop_propagation();
            return;
        }

        if self.authenticating {
            cx.stop_propagation();
            return;
        }

        if self.password_input.handle_key_down(event, cx) {
            if self.error.take().is_some() {
                self.update_target_size();
                cx.emit(PopupEvent::RequestAnimationFrame);
            }
            cx.notify();
            cx.stop_propagation();
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.handle_key_down(event, cx);
    }
}

impl Render for BarPolkit {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(ref anim) = self.animator else {
            return div().into_any_element();
        };

        let Some(request) = &self.request else {
            return div().into_any_element();
        };

        let theme = cx.global::<Theme>().clone();
        let size = anim.size();
        let corner_radius = px(22.0);
        let anim_opacity = anim.opacity;
        let anim_offset_y = anim.offset_y;
        let blur_progress = (anim.backdrop_opacity / 0.40).clamp(0.0, 1.0);

        let auth_required_text = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("polkit.auth_required")
        } else {
            "Autenticación Requerida".to_string()
        };

        let user_prefix = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("polkit.user_prefix")
        } else {
            "Usuario: ".to_string()
        };

        let password_placeholder = if cx.has_global::<AppState>() {
            if self.authenticating {
                cx.global::<AppState>().language.get("polkit.verifying")
            } else {
                cx.global::<AppState>()
                    .language
                    .get("polkit.password_placeholder")
            }
        } else if self.authenticating {
            "Verificando contraseña...".to_string()
        } else {
            "Contraseña".to_string()
        };

        let cancel_label = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("cancel")
        } else {
            "Cancelar".to_string()
        };

        let auth_label = if cx.has_global::<AppState>() {
            if self.authenticating {
                cx.global::<AppState>().language.get("polkit.verifying")
            } else {
                cx.global::<AppState>().language.get("authenticate")
            }
        } else if self.authenticating {
            "Verificando...".to_string()
        } else {
            "Autenticar".to_string()
        };

        let enabled = !self.authenticating && !self.password_input.is_empty();

        let field = Input::from_state("polkit-bar-password", &self.password_input)
            .placeholder(password_placeholder)
            .masked(true)
            .focused(true)
            .disabled(self.authenticating)
            .error(self.error.is_some())
            .prefix(
                svg()
                    .path("lock.svg")
                    .size(px(16.0))
                    .flex_shrink_0()
                    .text_color(theme.on_surface_variant())
                    .into_any_element(),
            )
            .h_px(40.0)
            .rounded(px(8.0))
            .bg(theme.surface_container_high())
            .border_color(if self.error.is_some() {
                theme.red()
            } else {
                theme.outline_variant().opacity(0.40)
            })
            .on_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let theme = cx.global::<Theme>().clone();
                this.password_input.handle_mouse_down(event, window, &theme);
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if this.password_input.is_dragging {
                    let theme = cx.global::<Theme>().clone();
                    this.password_input.handle_mouse_move(event, window, &theme);
                    cx.notify();
                }
            }))
            .on_mouse_up(cx.listener(|this, event: &MouseUpEvent, _, cx| {
                this.password_input.handle_mouse_up(event);
                cx.notify();
            }));

        let modal_shadow = BoxShadow {
            color: gpui::black().opacity(0.35).into(),
            offset: point(px(0.0), px(16.0)),
            blur_radius: px(36.0),
            spread_radius: px(0.0),
            inset: false,
        };

        let mut modal_box = div()
            .id("polkit-popup-container")
            .w(size.width)
            .h(size.height)
            .rounded(corner_radius)
            .bg(theme.surface_container().opacity(0.96))
            .border_1()
            .border_color(theme.outline_variant().opacity(0.35))
            .shadow(vec![modal_shadow])
            .flex()
            .flex_col()
            .justify_between()
            .p(px(18.0))
            .overflow_hidden()
            .opacity(anim_opacity)
            .relative()
            .top(px(anim_offset_y))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down(gpui::MouseButton::Right, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation());

        let top_section = div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .size(px(32.0))
                            .rounded(px(8.0))
                            .bg(theme.primary_container())
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_shrink_0()
                            .child(
                                svg()
                                    .path("lock.svg")
                                    .size(px(16.0))
                                    .text_color(theme.primary()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.foreground())
                                    .line_clamp(1)
                                    .child(auth_required_text),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(theme.foreground_muted())
                                    .line_clamp(1)
                                    .child(format!("{user_prefix}{}", request.user_name)),
                            ),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .p(px(10.0))
                    .rounded(px(8.0))
                    .bg(theme.surface_container_high().opacity(0.60))
                    .border_1()
                    .border_color(theme.outline_variant().opacity(0.25))
                    .flex()
                    .flex_col()
                    .gap(px(3.0))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(theme.foreground())
                            .child(request.message.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(10.5))
                            .text_color(theme.foreground_muted().opacity(0.70))
                            .child(request.action_id.clone()),
                    ),
            )
            .child(field);

        let mut bottom_section = div().flex().flex_col().gap(px(8.0));

        if let Some(error) = &self.error {
            bottom_section = bottom_section.child(
                div()
                    .text_size(px(11.5))
                    .text_color(theme.red())
                    .child(error.clone()),
            );
        }

        let buttons_row = div()
            .flex()
            .items_center()
            .justify_end()
            .gap(px(10.0))
            .child(
                Button::new("polkit-bar-cancel")
                    .label(cancel_label)
                    .variant(ButtonVariant::Secondary)
                    .padding_x(16.0)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.cancel(cx);
                    })),
            )
            .child(
                Button::new("polkit-bar-submit")
                    .label(auth_label)
                    .variant(ButtonVariant::Primary)
                    .disabled(!enabled)
                    .padding_x(18.0)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.submit(cx);
                    })),
            );

        bottom_section = bottom_section.child(buttons_row);
        modal_box = modal_box.child(top_section).child(bottom_section);

        div()
            .id("polkit-backdrop")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .absolute()
            .inset_0()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("polkit-backdrop-click")
                    .absolute()
                    .inset_0()
                    .backdrop_blur(px(20.0 * blur_progress))
                    .bg(style::background(&theme).opacity(0.08 * blur_progress)),
            )
            .child(modal_box)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_target_size_scales_with_message_and_error() {
        let size_short = compute_target_size(false, "Short message");
        assert_eq!(size_short.width, px(460.0));
        assert_eq!(size_short.height, px(240.0));

        let size_error = compute_target_size(true, "Short message");
        assert_eq!(size_error.height, px(278.0));

        let long_msg = "This is a very long message that spans multiple lines because it describes a complex system action that needs administrative privileges and authentication from the current user.";
        let size_long = compute_target_size(false, long_msg);
        assert!(size_long.height > size_short.height);
    }

    #[test]
    fn test_polkit_animator_morphs_size_when_error_occurs() {
        let now = Instant::now();
        let initial = compute_target_size(false, "Test");
        let mut animator = PolkitAnimator::new(
            initial,
            Duration::from_millis(200),
            CapsuleAnimation::Fluid,
            0.75,
            false,
            false,
        );

        let error_size = compute_target_size(true, "Test");
        animator.transition_to(error_size, now);

        let (running, _) = animator.advance(now + Duration::from_millis(100));
        assert!(running);
        assert!(
            animator.size().height > initial.height && animator.size().height < error_size.height
        );

        let (_, _) = animator.advance(now + Duration::from_millis(210));
        assert_eq!(animator.size().height, error_size.height);
    }
}
