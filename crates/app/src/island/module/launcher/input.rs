use gpui::{Context, KeyDownEvent, Window};

use super::LauncherModule;
use crate::capsule::module::CapsuleModuleEvent;

impl LauncherModule {
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        // When enter is pressed, launch the selected app / calculator result
        if key == "enter" || key == "return" {
            self.activate_selected(cx);
            return;
        }

        // When escape is pressed, clear search first, or close launcher if already empty
        if key == "escape" {
            if !self.search_input.is_empty() {
                self.reset_search(cx);
            } else {
                cx.emit(CapsuleModuleEvent::Close);
            }
            return;
        }

        // Grid navigation with vertical arrows
        if (key == "up" || key == "down") && !shift && !control {
            self.navigate(0, if key == "up" { -1 } else { 1 }, cx);
            return;
        }

        // Grid navigation with left/right arrows when input is empty and no selection/modifiers
        if (key == "left" || key == "right") && !shift && !control && self.search_input.is_empty() {
            self.navigate(if key == "left" { -1 } else { 1 }, 0, cx);
            return;
        }

        // Delegate all other editing keys (Ctrl+A, Ctrl+C, Ctrl+V, Shift+Arrows, Backspace, Delete, typing)
        // directly to search_input!
        if self.search_input.handle_key_down(event, cx) {
            let text = self.search_input.text();
            self.calculator = services::launcher::calculator::evaluate(&text);
            self.apps = self.service.search(&text);
            self.query = text;
            self.selected = 0;
            self.launch_error = None;
            self.scroll.scroll_to_item(0);
            cx.notify();
        }
    }
}
