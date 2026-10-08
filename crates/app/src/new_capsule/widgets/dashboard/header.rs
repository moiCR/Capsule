use super::{DashboardAction as Action, DashboardView as View, HEADER, button};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{AnyElement, Context, IntoElement, div, img, prelude::*, px, svg};
use std::path::PathBuf;
use ui::theme::Theme;

pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let title = if module.navigation.view == View::Home
        || module.navigation.view.satellite_id().is_some()
    {
        module.snapshot.date.clone()
    } else {
        module.text(module.navigation.view.title_key(), cx)
    };
    let mut header = div()
        .id("dashboard-header")
        .flex()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(HEADER))
        .flex_shrink_0()
        .gap(px(8.0));
    let leading = if module.navigation.view == View::Home
        || module.navigation.view.satellite_id().is_some()
    {
        button(
            "dashboard-calendar",
            title,
            "calendar-days.svg",
            Action::View(View::Calendar),
            false,
            theme,
            cx,
        )
    } else {
        button(
            "dashboard-back",
            title,
            "chevron-left.svg",
            Action::Back,
            false,
            theme,
            cx,
        )
    };
    let mut trailing = div().flex().items_center().gap(px(6.0)).min_w_0();
    if module.navigation.view == View::Home || module.navigation.view.satellite_id().is_some() {
        let mut tray = div()
            .id("dashboard-tray")
            .flex()
            .items_center()
            .h(px(36.0))
            .min_w(px(0.0))
            .max_w(px(160.0))
            .overflow_x_scroll()
            .gap(px(4.0));
        for item in &module.snapshot.tray {
            let bus = item.bus_name.clone();
            let path = item.object_path.clone();
            let icon = if let Some(path) = &item.icon_file_path {
                if path.to_lowercase().ends_with(".svg") {
                    svg()
                        .path(path.clone())
                        .size(px(18.0))
                        .text_color(theme.foreground())
                        .into_any_element()
                } else {
                    img(PathBuf::from(path)).size(px(18.0)).into_any_element()
                }
            } else {
                div()
                    .text_size(px(12.0))
                    .child(item.title.chars().next().unwrap_or('?').to_string())
                    .into_any_element()
            };
            let hover = theme.surface();
            tray = tray.child(
                div()
                    .id(format!(
                        "dashboard-tray-{}-{}",
                        item.bus_name, item.object_path
                    ))
                    .size(px(28.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(10.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.dispatch(
                            Action::View(View::Tray {
                                bus: bus.clone(),
                                path: path.clone(),
                            }),
                            cx,
                        )
                    }))
                    .child(icon),
            );
        }
        trailing = trailing.child(tray);
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
    }
    trailing = trailing.child(button(
        "dashboard-close",
        String::new(),
        "close.svg",
        Action::Close,
        false,
        theme,
        cx,
    ));
    header = header.child(leading).child(trailing);
    header.into_any_element()
}
