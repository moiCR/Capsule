use gpui::{Context, KeyDownEvent, Window};

use super::LauncherModule;
use crate::new_capsule::module::CapsuleModuleEvent;

impl LauncherModule {
    pub(super) fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        if control {
            match key {
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        let clean: String = text
                            .chars()
                            .filter(|character| !character.is_control())
                            .collect();
                        self.update_search(format!("{}{}", self.query, clean), cx);
                    }
                }
                "c" | "x" => {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(self.query.clone()));
                    if key == "x" {
                        self.reset_search(cx);
                    }
                }
                "u" => self.reset_search(cx),
                "w" => {
                    let trimmed = self.query.trim_end();
                    let query = trimmed
                        .rsplit_once(' ')
                        .map(|(prefix, _)| prefix)
                        .unwrap_or("")
                        .to_string();
                    self.update_search(query, cx);
                }
                _ => {}
            }
            return;
        }
        match key {
            "enter" => self.activate_selected(cx),
            "escape" => {
                cx.emit(CapsuleModuleEvent::Close);
            }
            "left" => self.navigate(-1, 0, cx),
            "right" => self.navigate(1, 0, cx),
            "up" => self.navigate(0, -1, cx),
            "down" => self.navigate(0, 1, cx),
            "backspace" => {
                let mut query = self.query.clone();
                query.pop();
                self.update_search(query, cx);
            }
            _ if !event.keystroke.modifiers.alt => {
                if let Some(text) = &event.keystroke.key_char
                    && !text.chars().any(char::is_control)
                {
                    self.update_search(format!("{}{}", self.query, text), cx);
                }
            }
            _ => {}
        }
    }
}
