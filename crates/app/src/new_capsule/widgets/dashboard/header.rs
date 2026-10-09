use super::{DashboardAction as Action, HEADER};
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::satellite::SatelliteId;
use crate::new_capsule::widgets::style;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px, svg};
use ui::theme::Theme;

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut header = div()
        .id("dashboard-header")
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(HEADER))
        .flex_shrink_0()
        .gap(px(8.0));
    let leading =
        div()
            .id("dashboard-calendar")
            .min_w_0()
            .text_size(px(12.0))
            .text_color(theme.foreground_muted())
            .whitespace_nowrap()
            .text_ellipsis()
            .cursor_pointer()
            .hover(|s| s.text_color(theme.foreground()))
            .on_click(cx.listener(|this, _, _, cx| {
                this.dispatch(Action::Satellite(SatelliteId::Calendar), cx)
            }))
            .child(module.snapshot.date.clone());

    let mut trailing = div().flex().items_center().gap(px(8.0)).flex_shrink_0();
    if let Some(battery) = module.snapshot.battery {
        trailing = trailing.child(
            div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .text_size(px(12.0))
                .child(
                    svg()
                        .path(if battery.is_charging {
                            "battery-charging.svg"
                        } else if battery.percentage <= 20 {
                            "battery-low.svg"
                        } else {
                            "battery-full.svg"
                        })
                        .size(px(16.0))
                        .text_color(theme.foreground_muted()),
                )
                .child(format!("{}%", battery.percentage)),
        );
    }

    trailing = trailing.child(
        div()
            .id("dashboard-settings")
            .size(px(28.0))
            .flex_shrink_0()
            .rounded_full()
            .bg(style::surface(theme))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(style::hover(theme)))
            .on_click(cx.listener(|this, _, _, cx| this.dispatch(Action::Settings, cx)))
            .child(
                svg()
                    .path("settings.svg")
                    .size(px(16.0))
                    .text_color(theme.foreground()),
            ),
    );

    header = header.child(leading).child(trailing);
    header.into_any_element()
}
