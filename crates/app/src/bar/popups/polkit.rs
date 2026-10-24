use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, App, BoxShadow, ColorExt, Context, FontWeight, IntoElement, KeyDownEvent, Pixels,
    Size, Task, WeakEntity, Window, div, point, px, svg,
};
use services::{AppState, CapsuleAnimation, PolkitAuthRequest};
use ui::components::{Button, ButtonVariant, Input, InputState};
use ui::theme::Theme;

use super::animator::PolkitAnimator;
use crate::bar::{Bar, style};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolkitKeyAction {
    None,
    Handled,
    Cancel,
    Submit,
}

pub struct BarPolkit {
    pub request: Option<PolkitAuthRequest>,
    pub password_input: InputState,
    pub error: Option<String>,
    pub authenticating: bool,
    pub animator: Option<PolkitAnimator>,
    pub is_open: bool,
    pub authentication: Option<tokio::task::JoinHandle<()>>,
    pub response: Option<Task<()>>,
    pub responder: Option<tokio::sync::oneshot::Sender<Result<(), String>>>,
}

impl Default for BarPolkit {
    fn default() -> Self {
        Self::new()
    }
}

impl BarPolkit {
    pub fn new() -> Self {
        Self {
            request: None,
            password_input: InputState::new("").with_focused(true).with_max_len(120),
            error: None,
            authenticating: false,
            animator: None,
            is_open: false,
            authentication: None,
            response: None,
            responder: None,
        }
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

    pub fn open(&mut self, window: &Window, entity: WeakEntity<Bar>, cx: &App) {
        let (duration, animation, bounciness, squash_stretch, reduced_motion) =
            if cx.has_global::<AppState>() {
                let cfg = cx.global::<AppState>().config.get();
                (
                    Duration::from_millis(cfg.ui.animation_duration_ms as u64),
                    cfg.ui.capsule_animation,
                    cfg.ui.spring_bounciness,
                    cfg.ui.squash_stretch,
                    cfg.ui.reduced_motion,
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
        Bar::queue_animation_frame(window, entity);
    }

    pub fn request_close(&mut self) {
        if let Some(ref mut anim) = self.animator
            && !anim.is_closing()
        {
            anim.request_close();
        }
    }

    pub fn close(&mut self, window: &Window, entity: WeakEntity<Bar>) {
        if let Some(ref mut anim) = self.animator
            && !anim.is_closing()
        {
            anim.request_close();
            Bar::queue_animation_frame(window, entity);
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

    pub fn cancel(&mut self, cx: &App) {
        self.finish(Err("Cancelled by user".into()));
        self.process_requests(cx);
        if self.request.is_none() {
            self.request_close();
        }
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

    pub fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        entity: WeakEntity<Bar>,
        cx: &mut Context<Bar>,
    ) -> PolkitKeyAction {
        let key = event.keystroke.key.as_str();

        if key == "escape" || (key == "f4" && event.keystroke.modifiers.alt) {
            return PolkitKeyAction::Cancel;
        }

        if key == "enter" || key == "return" {
            return PolkitKeyAction::Submit;
        }

        if self.authenticating {
            return PolkitKeyAction::Handled;
        }

        if self.password_input.handle_key_down(event, cx) {
            if self.error.take().is_some() {
                self.update_target_size();
                Bar::queue_animation_frame(window, entity);
            }
            cx.notify();
            return PolkitKeyAction::Handled;
        }

        PolkitKeyAction::None
    }

    pub fn render(&self, _window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
        let Some(ref anim) = self.animator else {
            return div().into_any_element();
        };

        let Some(request) = &self.request else {
            return div().into_any_element();
        };

        let theme = cx.global::<Theme>().clone();
        let size = anim.size();
        let corner_radius = px(22.0);

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
                    .text_color(theme.foreground_muted())
                    .into_any_element(),
            )
            .h_px(40.0)
            .rounded(px(8.0))
            .bg(style::surface(&theme).opacity(0.60))
            .border_color(if self.error.is_some() {
                theme.red()
            } else {
                style::border(&theme).opacity(0.18)
            });

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
            .bg(style::background(&theme).opacity(0.92))
            .border_1()
            .border_color(style::border(&theme).opacity(0.18))
            .shadow(vec![modal_shadow])
            .flex()
            .flex_col()
            .justify_between()
            .p(px(18.0))
            .overflow_hidden()
            .opacity(anim.opacity)
            .relative()
            .top(px(anim.offset_y))
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
                            .bg(theme.accent().opacity(0.16))
                            .flex()
                            .items_center()
                            .justify_center()
                            .flex_shrink_0()
                            .child(
                                svg()
                                    .path("lock.svg")
                                    .size(px(16.0))
                                    .text_color(theme.accent()),
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
                    .bg(style::surface(&theme).opacity(0.50))
                    .border_1()
                    .border_color(style::border(&theme).opacity(0.14))
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
                    .on_click(cx.listener(|bar, _, window, cx| {
                        bar.cancel_polkit(window, cx);
                    })),
            )
            .child(
                Button::new("polkit-bar-submit")
                    .label(auth_label)
                    .variant(ButtonVariant::Primary)
                    .disabled(!enabled)
                    .padding_x(18.0)
                    .on_click(cx.listener(|bar, _, window, cx| {
                        bar.submit_polkit(window, cx);
                    })),
            );

        bottom_section = bottom_section.child(buttons_row);
        modal_box = modal_box.child(top_section).child(bottom_section);

        let blur_progress = (anim.backdrop_opacity / 0.40).clamp(0.0, 1.0);
        div()
            .id("polkit-backdrop")
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
