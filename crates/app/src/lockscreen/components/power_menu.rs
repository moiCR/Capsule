use gpui::{
    ColorExt, Element, InteractiveElement, ParentElement, StatefulInteractiveElement, Styled, div,
    px, svg,
};
use ui::theme::Theme;

pub fn render_power_menu(theme: &Theme) -> impl Element {
    let btn_hover_suspend = theme.surface_container_highest();
    let btn_hover_reboot = theme.surface_container_highest();
    let shutdown_hover_bg = theme.red().opacity(0.25);
    let shutdown_hover_border = theme.red().opacity(0.5);

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
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.surface_container_high().opacity(0.85))
                .border_1()
                .border_color(theme.outline_variant().opacity(0.4))
                .hover(move |s| s.bg(btn_hover_suspend))
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
                        .size(px(16.0))
                        .text_color(theme.on_surface_variant()),
                ),
        )
        .child(
            div()
                .id("power-reboot")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.surface_container_high().opacity(0.85))
                .border_1()
                .border_color(theme.outline_variant().opacity(0.4))
                .hover(move |s| s.bg(btn_hover_reboot))
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
                        .size(px(16.0))
                        .text_color(theme.on_surface_variant()),
                ),
        )
        .child(
            div()
                .id("power-shutdown")
                .flex()
                .items_center()
                .justify_center()
                .w(px(38.0))
                .h(px(38.0))
                .rounded_full()
                .bg(theme.surface_container_high().opacity(0.85))
                .border_1()
                .border_color(theme.outline_variant().opacity(0.4))
                .hover(move |s| {
                    s.bg(shutdown_hover_bg)
                        .border_color(shutdown_hover_border)
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
                        .size(px(16.0))
                        .text_color(theme.on_surface_variant()),
                ),
        )
}
