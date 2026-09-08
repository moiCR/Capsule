use gpui::{
    Element, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div, px, svg,
};
use ui::theme::Theme;

pub fn render_power_menu(theme: &Theme) -> impl Element {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .child(
            div()
                .id("power-suspend")
                .flex()
                .items_center()
                .justify_center()
                .w(px(32.0))
                .h(px(32.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.opacity(0.6))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("suspend")
                        .spawn();
                })
                .child(
                    svg()
                        .path("moon.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .child(
            div()
                .id("power-reboot")
                .flex()
                .items_center()
                .justify_center()
                .w(px(32.0))
                .h(px(32.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.opacity(0.6))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("reboot")
                        .spawn();
                })
                .child(
                    svg()
                        .path("rotate-ccw.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
        .child(
            div()
                .id("power-shutdown")
                .flex()
                .items_center()
                .justify_center()
                .w(px(32.0))
                .h(px(32.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .border_1()
                .border_color(theme.surface().opacity(0.25))
                .hover(|s| {
                    s.bg(theme.red().opacity(0.25))
                        .border_color(theme.red().opacity(0.5))
                })
                .active(|s| s.opacity(0.6))
                .cursor_pointer()
                .on_click(|_, _, _| {
                    let _ = std::process::Command::new("systemctl")
                        .arg("poweroff")
                        .spawn();
                })
                .child(
                    svg()
                        .path("power.svg")
                        .size(px(14.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
}
