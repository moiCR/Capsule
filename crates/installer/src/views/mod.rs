pub mod auth_modal;
pub mod compositor;
pub mod dependencies;
pub mod finished;
pub mod installing;
pub mod sidebar;
pub mod welcome;

use crate::app::{InstallerApp, Step};
use gpui::{
    AnyElement, Context, IntoElement, KeyDownEvent, ParentElement, Styled, Window, div, prelude::*,
};
use ui::theme::Theme;

pub fn render_installer(
    app: &mut InstallerApp,
    window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let current_step = app.step;

    let content = match current_step {
        Step::Welcome => welcome::render_welcome(app, window, cx),
        Step::Dependencies => dependencies::render_dependencies(app, window, cx),
        Step::Compositor => compositor::render_compositor(app, window, cx),
        Step::Installing => installing::render_installing(app, window, cx),
        Step::Finished => finished::render_finished(app, window, cx),
    };

    let sidebar_element = sidebar::render_sidebar(app, cx);

    let main_layout = div()
        .id("installer-layout")
        .size_full()
        .flex()
        .flex_row()
        .bg(theme.background())
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(sidebar_element)
        .child(
            div()
                .flex_1()
                .min_w_0()
                .h_full()
                .overflow_hidden()
                .child(content),
        );

    let modal_overlay = if app.show_password_modal {
        Some(auth_modal::render_auth_modal(app, window, cx))
    } else {
        None
    };

    div()
        .id("installer-root")
        .track_focus(&app.focus_handle)
        .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
            if this.show_password_modal {
                let key = event.keystroke.key.as_str();
                if key == "escape" {
                    this.cancel_password_modal(cx);
                } else if key == "enter" {
                    if this.submit_password() {
                        this.execute_pending_action(cx);
                    }
                    cx.notify();
                } else if this.password_input.handle_key_down(event, cx) {
                    cx.notify();
                }
            } else {
                let key = event.keystroke.key.as_str();
                if key == "escape" && this.step != Step::Installing {
                    this.cancel_and_exit(cx);
                }
            }
        }))
        .relative()
        .size_full()
        .overflow_hidden()
        .bg(theme.background())
        .child(main_layout)
        .children(modal_overlay)
        .into_any_element()
}
