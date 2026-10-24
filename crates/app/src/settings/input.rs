use super::{Settings, SettingsTab};
use gpui::{Context, KeyDownEvent, Window};

impl Settings {
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        // Global hotkey: Ctrl+F to activate search
        if ctrl && key == "f" {
            self.search_input.focused = true;
            window.focus(&self.focus_handle, cx);
            cx.notify();
            return;
        }

        // Escape: clear query, exit search, or close modal
        if key == "escape" {
            if self.search_input.focused && !self.search_input.is_empty() {
                self.search_input.clear();
                self.scroll_handle.scroll_to_item(0);
                cx.notify();
                return;
            }
            if self.search_input.focused {
                self.search_input.focused = false;
                cx.notify();
                return;
            }
            self.request_close(cx);
            return;
        }

        // When search is active, delegate directly to InputState!
        if self.search_input.focused {
            if self.search_input.handle_key_down(event, cx) {
                self.scroll_handle.scroll_to_item(0);
                cx.notify();
                return;
            }
            if key == "enter" || key == "return" {
                return;
            }
            return;
        }

        // Tab switches sidebar sections when not searching
        if key == "tab" && !self.search_input.focused {
            let next_tab = match self.active_tab {
                SettingsTab::Appearance => SettingsTab::Island,
                SettingsTab::Island => SettingsTab::Bar,
                SettingsTab::Bar => SettingsTab::Apps,
                SettingsTab::Apps => SettingsTab::Media,
                SettingsTab::Media => SettingsTab::LockScreen,
                SettingsTab::LockScreen => SettingsTab::System,
                SettingsTab::System => SettingsTab::Record,
                SettingsTab::Record => SettingsTab::Appearance,
            };
            self.set_tab(next_tab, cx);
            return;
        }

        if let Some((_, view)) = self.views.iter_mut().find(|(t, _)| *t == self.active_tab) {
            if view.handle_key_down(event, window, cx) {
                return;
            }
        }
    }
}
