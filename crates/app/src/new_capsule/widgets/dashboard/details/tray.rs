use super::super::DashboardAction as Action;
use super::list;
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use crate::new_capsule::widgets::style;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use gpui::{MotionDurationExt, ease_in_out};
use ui::theme::Theme;
pub(super) fn tray(
    bus: &str,
    object_path: &str,
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let Some(item) = module
        .snapshot
        .tray
        .iter()
        .find(|item| item.bus_name == bus && item.object_path == object_path)
    else {
        return empty(module.text("dashboard_new.tray_gone", cx), theme);
    };
    let mut content = list("dashboard-tray-menu").child(button(
        "dashboard-tray-open",
        item.title.clone(),
        "chevron-right.svg",
        Action::TrayActivate {
            bus: bus.to_string(),
            path: object_path.to_string(),
        },
        false,
        theme,
        cx,
    ));
    if item.menu_items.is_empty() {
        content = content.child(empty(module.text("dashboard_new.no_menu", cx), theme));
    }
    if let Some(path) = &item.menu_path {
        for entry in &item.menu_items {
            content = content.child(menu_entry(entry, bus, path, module, theme, cx));
        }
    }
    content.into_any_element()
}
pub(super) fn menu_entry(
    entry: &services::tray::dbus_menu::DBusMenuItem,
    bus: &str,
    path: &str,
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    if !entry.visible {
        return div().into_any_element();
    }
    if entry.is_separator {
        return div()
            .h(px(1.0))
            .w_full()
            .bg(theme.surface())
            .into_any_element();
    }
    let mut content = div().flex_shrink_0().flex().flex_col().gap(px(4.0));
    let mut row = div()
        .id(format!("dashboard-tray-menu-{bus}-{path}-{}", entry.id))
        .min_h(px(36.0))
        .px(px(12.0))
        .py(px(8.0))
        .rounded(px(10.0))
        .bg(style::surface(theme))
        .opacity(if entry.enabled { 1.0 } else { 0.4 })
        .transitions(|t| {
            t.bg(module.snapshot.duration.with_easing(ease_in_out))
                .opacity(module.snapshot.duration.with_easing(ease_in_out))
        })
        .child(entry.label.replace('_', ""));
    if entry.enabled {
        let action = Action::TrayMenu(bus.to_string(), path.to_string(), entry.id);
        let hover = style::hover(theme);
        row = row
            .cursor_pointer()
            .hover(move |s| s.bg(hover))
            .on_click(cx.listener(move |this, _, _, cx| this.dispatch(action.clone(), cx)));
    }
    content = content.child(row);
    for child in &entry.children {
        content = content.child(
            div()
                .pl(px(16.0))
                .child(menu_entry(child, bus, path, module, theme, cx)),
        );
    }
    content.into_any_element()
}
