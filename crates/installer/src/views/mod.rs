pub mod auth_modal;
pub mod dependencies;
pub mod finished;
pub mod header;
pub mod installing;
pub mod welcome;

use crate::app::{InstallerApp, Step};
use crate::style;
use gpui::{
    AnyElement, Context, IntoElement, KeyDownEvent, ParentElement, Styled, Window, div, prelude::*,
    px,
};
use ui::theme::Theme;

pub fn render_installer(
    app: &mut InstallerApp,
    window: &mut Window,
    cx: &mut Context<InstallerApp>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let current_step = app.step;
    let is_installing = current_step == Step::Installing;

    let content = match current_step {
        Step::Welcome => welcome::render_welcome(app, window, cx),
        Step::Dependencies => dependencies::render_dependencies(app, window, cx),
        Step::Installing => installing::render_installing(app, window, cx),
        Step::Finished => finished::render_finished(app, window, cx),
    };

    let header_element = header::render_header(app, cx);

    let main_card = div()
        .id("installer-modal-card")
        .occlude()
        .w(px(680.0))
        .h(px(540.0))
        .max_w_full()
        .max_h_full()
        .rounded(px(style::CAPSULE_ROUND))
        .bg(style::background(&theme))
        .border_1()
        .border_color(style::border(&theme))
        .shadow_xl()
        .p(px(24.0))
        .flex()
        .flex_col()
        .justify_between()
        .overflow_hidden()
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(header_element)
        .child(div().flex_1().min_h_0().pt(px(16.0)).child(content));

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
        .flex()
        .items_center()
        .justify_center()
        // Backdrop overlay
        .child(
            div()
                .id("installer-backdrop")
                .absolute()
                .inset_0()
                .bg(gpui::rgba(0x00000088))
                .when(!is_installing, |s| {
                    s.on_click(cx.listener(|this, _, _, cx| {
                        if this.show_password_modal {
                            this.cancel_password_modal(cx);
                        } else {
                            this.cancel_and_exit(cx);
                        }
                    }))
                }),
        )
        .child(main_card)
        .children(modal_overlay)
        .into_any_element()
}
