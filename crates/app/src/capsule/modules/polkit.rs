use gpui::{
    EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Render, Window, div, prelude::*, px,
};
use services::{PolkitAuthRequest, authenticate_user};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use ui::theme::Theme;

use crate::capsule::widgets::polkit::{
    auth_dialog::{render_auth_footer, render_auth_header, render_auth_message},
    password_input::render_password_input,
};

pub enum PolkitEvent {
    Authenticated,
    Cancelled,
}

type PolkitPendingResult = Arc<Mutex<Option<Result<(), String>>>>;

pub struct PolkitModule {
    pub request: Option<PolkitAuthRequest>,
    pub password: String,
    pub is_error: bool,
    pub error_msg: Option<String>,
    pub is_authenticating: bool,
    pub focus_handle: FocusHandle,
    pub responder: Option<oneshot::Sender<Result<(), String>>>,
    pub pending_result: Option<PolkitPendingResult>,
}

impl EventEmitter<PolkitEvent> for PolkitModule {}

impl PolkitModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        Self {
            request: None,
            password: String::new(),
            is_error: false,
            error_msg: None,
            is_authenticating: false,
            focus_handle,
            responder: None,
            pending_result: None,
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
        self.error_msg = None;
        self.is_authenticating = false;
        self.responder = Some(responder);
        self.pending_result = None;
        cx.notify();
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(responder) = self.responder.take() {
            let _ = responder.send(Err("Cancelled by user".to_string()));
        }
        self.password.clear();
        self.is_error = false;
        self.error_msg = None;
        self.is_authenticating = false;
        self.request = None;
        self.pending_result = None;
        cx.emit(PolkitEvent::Cancelled);
    }

    pub fn poll_result(&mut self, cx: &mut Context<Self>) {
        if !self.is_authenticating {
            return;
        }
        let finished_res = if let Some(ref slot) = self.pending_result {
            if let Ok(guard) = slot.lock() {
                guard.clone()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(res) = finished_res {
            self.is_authenticating = false;
            self.pending_result = None;
            match res {
                Ok(()) => {
                    if let Some(responder) = self.responder.take() {
                        let _ = responder.send(Ok(()));
                    }
                    self.password.clear();
                    self.is_error = false;
                    self.error_msg = None;
                    self.request = None;
                    cx.emit(PolkitEvent::Authenticated);
                }
                Err(err_msg) => {
                    self.is_error = true;
                    self.error_msg = Some(err_msg);
                    self.password.clear();
                    cx.notify();
                }
            }
        }
    }

    pub fn submit_auth(&mut self, cx: &mut Context<Self>) {
        if self.is_authenticating {
            return;
        }
        let password = self.password.clone();
        if password.is_empty() {
            return;
        }

        let (user_name, cookie) = if let Some(ref req) = self.request {
            (req.user_name.clone(), req.cookie.clone())
        } else {
            ("root".to_string(), "".to_string())
        };

        let result_slot = Arc::new(Mutex::new(None));
        self.pending_result = Some(result_slot.clone());
        self.is_authenticating = true;
        self.is_error = false;
        self.error_msg = None;
        cx.notify();

        tokio::spawn(async move {
            let res = tokio::time::timeout(
                std::time::Duration::from_secs(30),
                authenticate_user(&user_name, &cookie, &password),
            )
            .await;

            let final_res = match res {
                Ok(inner) => inner,
                Err(_) => Err("La autenticación ha tardado demasiado y ha expirado.".to_string()),
            };

            if let Ok(mut guard) = result_slot.lock() {
                *guard = Some(final_res);
            }
        });
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_authenticating {
            return;
        }

        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if ctrl {
            match key {
                "v" => {
                    if let Some(item) = cx.read_from_clipboard()
                        && let Some(text) = item.text()
                    {
                        let clean_text: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !clean_text.is_empty() {
                            self.password.push_str(&clean_text);
                            self.is_error = false;
                            self.error_msg = None;
                            cx.notify();
                        }
                    }
                    return;
                }
                "c" => {
                    if !self.password.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                            self.password.clone(),
                        ));
                    }
                    return;
                }
                "x" => {
                    if !self.password.is_empty() {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                            self.password.clone(),
                        ));
                        self.password.clear();
                        self.is_error = false;
                        self.error_msg = None;
                        cx.notify();
                    }
                    return;
                }
                "u" | "w" => {
                    self.password.clear();
                    self.is_error = false;
                    self.error_msg = None;
                    cx.notify();
                    return;
                }
                _ => {}
            }
        }

        match key {
            "enter" => self.submit_auth(cx),
            "backspace" => {
                if !self.password.is_empty() {
                    self.password.pop();
                    self.is_error = false;
                    self.error_msg = None;
                    cx.notify();
                }
            }
            "escape" => self.cancel(cx),
            "space" => {
                if !ctrl {
                    self.password.push(' ');
                    self.is_error = false;
                    self.error_msg = None;
                    cx.notify();
                }
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());

                if text.chars().count() == 1 && !ctrl {
                    self.password.push_str(text);
                    self.is_error = false;
                    self.error_msg = None;
                    cx.notify();
                }
            }
        }
    }
}

impl Render for PolkitModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        window.focus(&self.focus_handle, cx);

        let (title, placeholder, default_err) = if cx.has_global::<services::AppState>() {
            let lang = &cx.global::<services::AppState>().language;
            (
                lang.get("polkit.auth_required"),
                if self.is_authenticating {
                    lang.get("polkit.verifying")
                } else {
                    "Escribe tu contraseña...".to_string()
                },
                lang.get("polkit.incorrect_password"),
            )
        } else {
            (
                "Autenticación Requerida".to_string(),
                if self.is_authenticating {
                    "Verificando contraseña...".to_string()
                } else {
                    "Escribe tu contraseña...".to_string()
                },
                "Contraseña incorrecta. Reintenta...".to_string(),
            )
        };

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

        let err_text = self.error_msg.as_deref().unwrap_or(&default_err);

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .justify_between()
            .w(px(380.0))
            .h(px(190.0))
            .p_3()
            .overflow_hidden()
            .child(render_auth_header(user_name, &title, &theme, cx))
            .child(render_auth_message(req_message, &theme))
            .child(render_password_input(
                &self.password,
                self.is_error,
                self.is_authenticating,
                &placeholder,
                err_text,
                &theme,
                cx,
            ))
            .child(render_auth_footer(
                !self.password.is_empty(),
                self.is_authenticating,
                if self.is_error { Some(err_text) } else { None },
                &theme,
                cx,
            ))
    }
}
