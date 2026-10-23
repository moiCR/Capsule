use gpui::{AnyElement, Context, KeyDownEvent, MouseMoveEvent, MouseUpEvent, Window};

pub mod appearance;
pub mod apps;
pub mod common;
pub mod lockscreen;
pub mod media;
pub mod record;
pub mod search;
pub mod system;

use super::Settings;

pub trait SettingsView: 'static {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn render(&mut self, window: &mut Window, cx: &mut Context<Settings>) -> AnyElement;
    fn handle_key_down(
        &mut self,
        _event: &KeyDownEvent,
        _window: &mut Window,
        _cx: &mut Context<Settings>,
    ) -> bool {
        false
    }
    fn handle_mouse_move(&mut self, _event: &MouseMoveEvent, _cx: &mut Context<Settings>) -> bool {
        false
    }
    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _cx: &mut Context<Settings>) -> bool {
        false
    }
}
