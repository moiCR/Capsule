use gpui::{
    div, px, svg, Element, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled,
};
use ui::theme::Theme;

pub fn render_power_menu(theme: &Theme) -> impl Element {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .p(px(6.0))
        .rounded_full()
        .bg(theme.surface().opacity(0.4))
        .border_1()
        .border_color(theme.surface().opacity(0.6))
        .shadow_md()
        // Suspend (moon_1.svg)
        .child(
            div()
                .id("power-suspend")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.surface())
                .hover(|s| s.bg(theme.accent().opacity(0.3)))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("suspend")
                        .spawn();
                })
                .child(
                    svg()
                        .path("moon_1.svg")
                        .size(px(18.0))
                        .text_color(theme.foreground()),
                ),
        )
        // Reboot (rotate-ccw.svg)
        .child(
            div()
                .id("power-reboot")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.surface())
                .hover(|s| s.bg(theme.accent().opacity(0.3)))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("reboot")
                        .spawn();
                })
                .child(
                    svg()
                        .path("rotate-ccw.svg")
                        .size(px(18.0))
                        .text_color(theme.foreground()),
                ),
        )
        // Power Off (power.svg)
        .child(
            div()
                .id("power-shutdown")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.red().opacity(0.8))
                .hover(|s| s.bg(theme.red()))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("poweroff")
                        .spawn();
                })
                .child(
                    svg()
                        .path("power.svg")
                        .size(px(18.0))
                        .text_color(theme.foreground()),
                ),
        )
}
