mod worker;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::shelf as widgets;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render, ScrollHandle,
    Size, Subscription, Task, Window, prelude::*, px,
};
use services::{AppState, ShelfItem};
use ui::theme::Theme;
use worker::Request;

pub(crate) struct ShelfModule {
    pub items: Vec<ShelfItem>,
    pub previews: std::collections::HashMap<String, std::sync::Arc<gpui::Image>>,
    pub selected: usize,
    pub scroll: ScrollHandle,
    pub busy: bool,
    pub error: bool,
    pub copied: bool,
    pub mouse_moved: bool,
    focus: FocusHandle,
    active: bool,
    generation: u64,
    sender: tokio::sync::mpsc::UnboundedSender<Request>,
    worker: tokio::task::JoinHandle<()>,
    _updates: Task<()>,
    _theme: Subscription,
}

impl ShelfModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (sender, mut receiver, worker) = worker::start(cx.global::<AppState>().shelf.clone());
        let updates = cx.spawn(async move |this, cx| {
            while let Some(reply) = receiver.recv().await {
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let previous_size = module.size();
                        let changed = module.items != reply.items || reply.finished;
                        let selected_id = module
                            .items
                            .get(module.selected)
                            .map(|item| item.id.clone());
                        module.items = reply.items;
                        module.previews = reply.previews;
                        module.selected = selected_id
                            .and_then(|id| module.items.iter().position(|item| item.id == id))
                            .unwrap_or(module.selected.min(module.items.len().saturating_sub(1)));
                        if reply.finished {
                            module.busy = false;
                            module.error = reply.error;
                        }
                        if let Some(generation) = reply.copied
                            && module.active
                            && generation == module.generation
                        {
                            module.copied = true;
                        }
                        if changed {
                            if module.active && previous_size != module.size() {
                                cx.emit(CapsuleModuleEvent::SizeChanged(CapsuleModuleId::Shelf));
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
            items: Vec::new(),
            previews: std::collections::HashMap::new(),
            selected: 0,
            scroll: ScrollHandle::new(),
            busy: false,
            error: false,
            copied: false,
            mouse_moved: false,
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
        }
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.generation = self.generation.wrapping_add(1);
        self.copied = false;
        self.error = false;
        self.mouse_moved = false;
        let _ = self.sender.send(Request::Refresh);
        cx.notify();
    }
    pub fn add_paths(&mut self, paths: Vec<std::path::PathBuf>) {
        let _ = self.sender.send(Request::Add(paths));
    }
    pub fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.items.len() && self.selected != index {
            self.selected = index;
            self.scroll.scroll_to_item(index / widgets::COLUMNS);
            cx.notify();
        }
    }
    pub fn copy(&mut self, all: bool, cx: &mut Context<Self>) {
        let paths = if all {
            self.items.iter().map(|item| item.path.clone()).collect()
        } else {
            self.items
                .get(self.selected)
                .map(|item| vec![item.path.clone()])
                .unwrap_or_default()
        };
        self.request(Request::Copy(paths, self.generation), cx);
    }
    pub fn remove(&mut self, id: String, cx: &mut Context<Self>) {
        self.request(Request::Remove(id), cx);
    }
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.request(Request::Clear, cx);
    }
    fn request(&mut self, request: Request, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.busy = self.sender.send(request).is_ok();
        self.error = !self.busy;
        self.copied = false;
        cx.notify();
    }
    pub fn close(&mut self, cx: &mut Context<Self>) {
        cx.emit(CapsuleModuleEvent::Close);
    }
    fn handle_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        match key {
            "escape" => self.close(cx),
            "enter" | "c" if key == "enter" || control => self.copy(false, cx),
            "a" | "d" if control => self.copy(true, cx),
            "delete" | "backspace" => {
                if let Some(item) = self.items.get(self.selected) {
                    self.remove(item.id.clone(), cx);
                }
            }
            "up" | "down" | "left" | "right" | "home" | "end" => {
                let next = widgets::next_selection(self.selected, self.items.len(), key);
                self.mouse_moved = false;
                self.select(next, cx);
            }
            _ => {}
        }
    }
}
impl Drop for ShelfModule {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
impl EventEmitter<CapsuleModuleEvent> for ShelfModule {}
impl CapsuleModule for ShelfModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(px(widgets::WIDTH), px(widgets::height(self.items.len())))
    }
}
impl Render for ShelfModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(self, cx)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key))
    }
}
