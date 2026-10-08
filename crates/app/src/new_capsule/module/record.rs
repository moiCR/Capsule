mod worker;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::record as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render, Size,
    Subscription, Task, Window, prelude::*, px,
};
use services::{AppState, RecordStatus};
use ui::theme::Theme;
use worker::Request;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingAction {
    Start,
    Pause,
    Resume,
    Stop,
}

pub(crate) struct RecordModule {
    pub status: RecordStatus,
    pub seconds: u64,
    pub(crate) pending: Option<PendingAction>,
    pub error: Option<String>,
    focus: FocusHandle,
    active: bool,
    generation: u64,
    sender: tokio::sync::mpsc::UnboundedSender<Request>,
    worker: tokio::task::JoinHandle<()>,
    _updates: Task<()>,
    _theme: Subscription,
}
impl RecordModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = cx.global::<AppState>().record.clone();
        let status = service.get_status();
        let (sender, mut receiver, worker) = worker::start(service);
        let updates = cx.spawn(async move |this, cx| {
            while let Some(reply) = receiver.recv().await {
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let previous_size = module.size();
                        module.status = reply.status;
                        module.seconds = reply.seconds;
                        if reply.finished.is_some() && reply.finished == module.pending {
                            module.pending = None;
                            module.error = reply.error;
                        }
                        if module.active {
                            if previous_size != module.size() {
                                cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Record));
                            }
                            if reply.stopped == Some(module.generation) {
                                cx.emit(CapsuleModuleEvent::Close);
                            }
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            status,
            seconds: 0,
            pending: None,
            error: None,
            focus: cx.focus_handle(),
            active: false,
            generation: 0,
            sender,
            worker,
            _updates: updates,
            _theme: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }
    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn set_active(&mut self, active: bool) {
        if self.active != active {
            self.active = active;
            let _ = self.sender.send(Request::Active(active));
        }
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        let _ = self.sender.send(Request::Refresh);
        cx.notify();
    }
    pub fn primary_action(&mut self, cx: &mut Context<Self>) {
        let request = match self.status {
            RecordStatus::Stopped => {
                Request::Start(cx.global::<AppState>().config.get().record.clone())
            }
            RecordStatus::Recording => Request::Pause,
            RecordStatus::Paused => Request::Resume,
        };
        self.request(request, cx);
    }
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if self.status != RecordStatus::Stopped || self.pending == Some(PendingAction::Start) {
            self.request(Request::Stop(self.generation), cx);
        }
    }
    fn request(&mut self, request: Request, cx: &mut Context<Self>) {
        let action = match request {
            Request::Start(_) => PendingAction::Start,
            Request::Pause => PendingAction::Pause,
            Request::Resume => PendingAction::Resume,
            Request::Stop(_) => PendingAction::Stop,
            Request::Refresh | Request::Active(_) => return,
        };
        if self.pending.is_some()
            && !(self.pending == Some(PendingAction::Start) && action == PendingAction::Stop)
        {
            return;
        }
        let previous_size = self.size();
        self.error = None;
        self.pending = Some(action);
        if self.sender.send(request).is_err() {
            self.pending = None;
            self.error = Some(cx.global::<AppState>().language.get("record.action_error"));
        }
        if previous_size != self.size() {
            cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Record));
        }
        cx.notify();
    }
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if self.pending == Some(PendingAction::Start) {
            self.stop(cx);
            return;
        }
        cx.emit(CapsuleModuleEvent::Close);
    }
    fn handle_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "escape" => self.close(cx),
            "enter" | "space" => self.primary_action(cx),
            "s" if event.keystroke.modifiers.control => self.stop(cx),
            _ => {}
        }
    }
}
impl Drop for RecordModule {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
impl EventEmitter<CapsuleModuleEvent> for RecordModule {}
impl CapsuleModule for RecordModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(
            px(widgets::WIDTH),
            px(widgets::height(self.error.is_some())),
        )
    }
}
impl Render for RecordModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(self, cx)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key))
    }
}
