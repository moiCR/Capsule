use super::super::DashboardAction as Action;
use super::list;
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use gpui::{ColorExt, MotionDurationExt, StyledImage, ease_in_out, img};
use ui::theme::Theme;
pub(super) fn themes(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = list("dashboard-themes-list");
    if module.themes.is_empty() {
        content = content.child(empty(
            module.text(
                if module.catalog_ready {
                    "dashboard_new.no_themes"
                } else {
                    "dashboard_new.loading"
                },
                cx,
            ),
            theme,
        ));
    }
    for item in &module.themes {
        content = content.child(button(
            format!("dashboard-theme-{}", item.path.display()),
            item.name.clone(),
            "palette_2.svg",
            Action::Theme(item.theme.clone()),
            item.theme == *theme,
            theme,
            cx,
        ));
    }
    content.into_any_element()
}
pub(super) fn wallpapers(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = list("dashboard-wallpapers-list");
    if module.wallpapers.is_empty() {
        content = content.child(empty(
            module.text(
                if module.catalog_ready {
                    "dashboard_new.no_wallpapers"
                } else {
                    "dashboard_new.loading"
                },
                cx,
            ),
            theme,
        ));
    }
    for pair in module.wallpapers.chunks(2) {
        let mut row = div().flex().gap(px(12.0)).flex_shrink_0();
        for item in pair {
            let path = item.path.clone();
            let hover = theme.surface().opacity(0.6);
            let mut card = div()
                .id(format!("dashboard-wallpaper-{}", path.display()))
                .w(px(294.0))
                .h(px(136.0))
                .p(px(8.0))
                .flex()
                .flex_col()
                .gap(px(6.0))
                .rounded(px(12.0))
                .bg(
                    if module.snapshot.current_wallpaper.as_ref() == Some(&path) {
                        theme.accent().opacity(0.15)
                    } else {
                        theme.surface().opacity(0.25)
                    },
                )
                .cursor_pointer()
                .transitions(|t| t.bg(module.snapshot.duration.with_easing(ease_in_out)))
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.dispatch(Action::Wallpaper(path.clone()), cx)
                }));
            card = card
                .child(match &item.thumbnail {
                    Some(thumb) => img(thumb.clone())
                        .w_full()
                        .h(px(96.0))
                        .object_fit(gpui::ObjectFit::Cover)
                        .rounded(px(8.0))
                        .into_any_element(),
                    None => div()
                        .w_full()
                        .h(px(96.0))
                        .bg(theme.surface())
                        .into_any_element(),
                })
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_ellipsis()
                        .child(item.name.clone()),
                );
            row = row.child(card);
        }
        content = content.child(row);
    }
    content.into_any_element()
}
