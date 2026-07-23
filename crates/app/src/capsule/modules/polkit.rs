use gpui::{
    EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render, Window, div,
    prelude::*, svg,
};
use services::{PolkitAuthRequest, authenticate_user};
use tokio::sync::oneshot;
use ui::theme::Theme;

pub enum PolkitEvent {
    Authenticated,
    Cancelled,
}

pub struct PolkitModule {
    pub request: Option<PolkitAuthRequest>,
    pub password: String,
    pub is_error: bool,
    pub focus_handle: FocusHandle,
    pub responder: Option<oneshot::Sender<Result<(), String>>>,
}

impl EventEmitter<PolkitEvent> for PolkitModule {}

impl PolkitModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            request: None,
            password: String::new(),
            is_error: false,
            focus_handle,
            responder: None,
        }
    }

    pub fn set_request(
        &mut self,
        request: PolkitAuthRequest,
        responder: oneshot::Sender<Result<(), String>>,
        cx: &mut Context<Self>,
    ) {
        self.request = Some(request);
        self.password.clear();
        self.is_error = false;
        self.responder = Some(responder);
        cx.notify();
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(responder) = self.responder.take() {
            let _ = responder.send(Err("Cancelled by user".to_string()));
        }
        self.password.clear();
        self.is_error = false;
        self.request = None;
        cx.emit(PolkitEvent::Cancelled);
    }

    fn submit_auth(&mut self, cx: &mut Context<Self>) {
        let password = self.password.clone();
        if password.is_empty() {
            return;
        }

        cx.spawn(async move |this, cx| {
            let success = authenticate_user(&password).await;

            let _ = this.update(cx, |this: &mut Self, cx| {
                if success {
                    if let Some(responder) = this.responder.take() {
                        let _ = responder.send(Ok(()));
                    }
                    this.password.clear();
                    this.is_error = false;
                    this.request = None;
                    cx.emit(PolkitEvent::Authenticated);
                } else {
                    this.is_error = true;
                    this.password.clear();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        match key {
            "enter" => self.submit_auth(cx),
            "backspace" => {
                if !self.password.is_empty() {
                    self.password.pop();
                    self.is_error = false;
                    cx.notify();
                }
            }
            "escape" => self.cancel(cx),
            _ => {
                if let Some(keystroke_text) = &event.keystroke.key_char {
                    if !keystroke_text.chars().any(|c| c.is_control()) {
                        self.password.push_str(keystroke_text);
                        self.is_error = false;
                        cx.notify();
                    }
                }
            }
        }
    }
}

impl Render for PolkitModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let req_message = self
            .request
            .as_ref()
            .map(|r| r.message.as_str())
            .unwrap_or("Se requiere autenticación para continuar.");

        let user_name = self
            .request
            .as_ref()
            .map(|r| r.user_name.as_str())
            .unwrap_or("usuario");

        let masked_password = "•".repeat(self.password.len());

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .size_full()
            .p_4()
            .gap_3()
            // Header
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .w_full()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_7()
                            .h_7()
                            .rounded_lg()
                            .bg(theme.accent())
                            .child(
                                svg()
                                    .path("sparkles.svg")
                                    .w_4()
                                    .h_4()
                                    .text_color(theme.background()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground())
                                    .child("Autenticación Requerida"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.foreground_muted())
                                    .child(format!("Usuario: {user_name}")),
                            ),
                    ),
            )
            // Message Details
            .child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_lg()
                    .bg(theme.surface())
                    .text_xs()
                    .text_color(theme.foreground())
                    .child(req_message.to_string()),
            )
            // Password Input Box
            .child(
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px_3p5()
                    .py_2()
                    .rounded_xl()
                    .bg(theme.surface())
                    .border_1()
                    .border_color(if self.is_error {
                        theme.red_color.to_hsla()
                    } else {
                        theme.background_alt()
                    })
                    .child(div().flex_1().text_sm().child(if self.password.is_empty() {
                        div()
                            .text_color(if self.is_error {
                                theme.red_color.to_hsla()
                            } else {
                                theme.foreground_muted()
                            })
                            .child(if self.is_error {
                                "Contraseña incorrecta. Reintenta..."
                            } else {
                                "Escribe tu contraseña..."
                            })
                    } else {
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.foreground())
                            .child(masked_password)
                    })),
            )
            // Buttons Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_end()
                    .gap_2()
                    .pt_1()
                    .child(
                        div()
                            .id("polkit-cancel-btn")
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(theme.background_alt())
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel(cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.foreground_muted())
                                    .child("Cancelar"),
                            ),
                    )
                    .child(
                        div()
                            .id("polkit-submit-btn")
                            .px_3p5()
                            .py_1p5()
                            .rounded_lg()
                            .bg(theme.accent())
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.submit_auth(cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.background())
                                    .child("Autenticar"),
                            ),
                    ),
            )
    }
}
