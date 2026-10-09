use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::polkit as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, KeyDownEvent, Pixels, Render, Size, Subscription, Task,
    Window, prelude::*,
};
use services::{AppState, PolkitAuthRequest};
use ui::theme::Theme;

pub(crate) struct PolkitModule {
    pub request: Option<PolkitAuthRequest>,
    pub password: String,
    pub error: Option<String>,
    pub authenticating: bool,
    focus: FocusHandle,
    responder: Option<tokio::sync::oneshot::Sender<Result<(), String>>>,
    authentication: Option<tokio::task::JoinHandle<()>>,
    response: Option<Task<()>>,
    _events: Task<()>,
    _theme: Subscription,
}

impl PolkitModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = cx.global::<AppState>().polkit.clone();
        let events = cx.spawn(async move |this, cx| {
            loop {
                service.wait_for_change().await;
                if this
                    .update(cx, |module: &mut Self, cx| module.process_requests(cx))
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            request: None,
            password: String::new(),
            error: None,
            authenticating: false,
            focus: cx.focus_handle(),
            responder: None,
            authentication: None,
            response: None,
            _events: events,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    pub fn has_request(&self) -> bool {
        self.request.is_some()
    }

    pub fn text(&self, key: &str, cx: &gpui::App) -> String {
        cx.global::<AppState>().language.get(key)
    }

    fn process_requests(&mut self, cx: &mut Context<Self>) {
        let service = cx.global::<AppState>().polkit.clone();
        let mut cancelled = false;
        while let Some(cookie) = service.pop_cancelled() {
            if self
                .request
                .as_ref()
                .is_some_and(|request| request.cookie == cookie)
            {
                self.finish(Err("Cancelled by Polkit Authority".into()));
                cancelled = true;
            }
        }
        if self
            .responder
            .as_ref()
            .is_some_and(|responder| responder.is_closed())
        {
            self.finish(Err("Authentication request expired".into()));
            cancelled = true;
        }
        if self.request.is_none() {
            while let Some((request, responder)) = service.pop_request() {
                if responder.is_closed() {
                    continue;
                }
                self.request = Some(request);
                self.responder = Some(responder);
                self.password.clear();
                self.error = None;
                cx.emit(CapsuleModuleEvent::Open(CapsuleModuleId::Polkit));
                cx.notify();
                return;
            }
            if cancelled {
                cx.emit(CapsuleModuleEvent::Close);
                cx.notify();
            }
        }
    }

    fn finish(&mut self, result: Result<(), String>) {
        if let Some(authentication) = self.authentication.take() {
            authentication.abort();
        }
        self.response = None;
        if let Some(responder) = self.responder.take() {
            let _ = responder.send(result);
        }
        self.request = None;
        self.password.clear();
        self.error = None;
        self.authenticating = false;
    }

    pub fn cancel(&mut self, cx: &mut Context<Self>) {
        self.finish(Err("Cancelled by user".into()));
        self.process_requests(cx);
        if self.request.is_none() {
            cx.emit(CapsuleModuleEvent::Close);
        }
        cx.notify();
    }

    pub fn submit(&mut self, cx: &mut Context<Self>) {
        if self.authenticating || self.password.is_empty() {
            return;
        }
        let Some(request) = &self.request else {
            return;
        };
        let user = request.user_name.clone();
        let cookie = request.cookie.clone();
        let response_cookie = cookie.clone();
        let password = std::mem::take(&mut self.password);
        let timeout_error = self.text("polkit.timeout", cx);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.authenticating = true;
        self.clear_error(cx);
        self.authentication = Some(services::spawn_tokio(async move {
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(30),
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
            let _ = this.update(cx, |module, cx| {
                if module
                    .request
                    .as_ref()
                    .is_none_or(|request| request.cookie != response_cookie)
                {
                    return;
                }
                module.authenticating = false;
                module.authentication = None;
                match result {
                    Ok(()) => {
                        module.finish(Ok(()));
                        module.process_requests(cx);
                        if module.request.is_none() {
                            cx.emit(CapsuleModuleEvent::Close);
                        }
                    }
                    Err(error) => {
                        module.error = Some(error);
                        cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Polkit));
                    }
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn clear_error(&mut self, cx: &mut Context<Self>) {
        if self.error.take().is_some() {
            cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Polkit));
        }
    }

    pub(crate) fn key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            self.cancel(cx);
            cx.stop_propagation();
            return;
        }
        if self.authenticating {
            return;
        }
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        if control {
            match key {
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        append_password(&mut self.password, &text);
                    }
                }
                "u" | "w" => self.password.clear(),
                _ => return,
            }
        } else if !event.keystroke.modifiers.alt {
            match key {
                "enter" => {
                    self.submit(cx);
                    return;
                }
                "backspace" => {
                    self.password.pop();
                }
                "space" => self.password.push(' '),
                _ => {
                    if let Some(text) = &event.keystroke.key_char {
                        append_password(&mut self.password, text);
                    } else {
                        return;
                    }
                }
            }
        } else {
            return;
        }
        self.clear_error(cx);
        cx.notify();
        cx.stop_propagation();
    }
}

fn append_password(password: &mut String, text: &str) {
    password.extend(text.chars().filter(|character| !character.is_control()));
}

impl Drop for PolkitModule {
    fn drop(&mut self) {
        self.finish(Err("Authentication agent closed".into()));
    }
}

impl EventEmitter<CapsuleModuleEvent> for PolkitModule {}
impl CapsuleModule for PolkitModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(
            gpui::px(widgets::WIDTH),
            gpui::px(widgets::height(self.error.is_some())),
        )
    }
}
impl Render for PolkitModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(self, &self.focus.clone(), cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_input_preserves_unicode_and_spaces_without_protocol_delimiters() {
        let mut password = String::new();
        append_password(&mut password, "  clave ñ🔒\r\n\t\0 ");
        assert_eq!(password, "  clave ñ🔒 ");
        password.pop();
        password.pop();
        assert_eq!(password, "  clave ñ");
    }
}
