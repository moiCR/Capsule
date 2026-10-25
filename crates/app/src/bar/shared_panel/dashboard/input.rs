use gpui::{Context, KeyDownEvent, Window};
use services::AppState;

use super::{DashboardModule, DashboardSubView};
use crate::bar::shared_panel::SharedPanelEvent;

impl DashboardModule {
    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            if self.view != DashboardSubView::Home {
                self.close_subview(cx);
            } else {
                cx.emit(SharedPanelEvent::Close);
            }
            cx.stop_propagation();
            return;
        }

        if (key == "enter" || key == "return")
            && let Some(ssid) = self.selected_ssid.take()
        {
            let pwd = self.password_input.text();
            if let Some(state) = cx.try_global::<AppState>() {
                state.network.connect_wifi(&ssid, Some(&pwd));
            }
            self.password_input.clear();
            cx.emit(SharedPanelEvent::SyncWindow);
            cx.notify();
            cx.stop_propagation();
            return;
        }

        if self.selected_ssid.is_some() && self.password_input.handle_key_down(event, cx) {
            cx.notify();
            cx.stop_propagation();
        }
    }

    pub fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.handle_key_down(event, cx);
    }
}
