use crate::new_capsule::module::appearance::{AppearanceKind, AppearanceModule};
use gpui::{
    AnyElement, ColorExt, Context, IntoElement, StyledImage, div, img, prelude::*, px, svg,
};
use ui::theme::Theme;

pub const WIDTH: f32 = 560.0;
pub const HEIGHT: f32 = 240.0;
const CAROUSEL_WIDTH: f32 = WIDTH - 32.0;

#[derive(Clone, Copy)]
struct CardGeometry {
    width: f32,
    height: f32,
    left: f32,
    top: f32,
    opacity: f32,
}

fn geometry(position: f32, piano: f32) -> CardGeometry {
    let distance = position.abs();
    let first = distance.min(1.0);
    let second = (distance - 1.0).clamp(0.0, 1.0);
    let width = 160.0 - first * 40.0 - second * 26.0;
    let height = 100.0 - first * 24.0 - second * 16.0;
    CardGeometry {
        width,
        height,
        left: CAROUSEL_WIDTH / 2.0 + position * 148.0 - width / 2.0,
        top: 64.0 - height / 2.0 - (1.0 - first) * 8.0 + second * 3.0 + piano,
        opacity: (1.0 - first * 0.35 - second * 0.35).max(0.0),
    }
}

fn control(
    id: &'static str,
    icon: &'static str,
    enabled: bool,
    theme: &Theme,
    cx: &mut Context<AppearanceModule>,
    direction: Option<i32>,
) -> gpui::Stateful<gpui::Div> {
    let hover = theme.surface().opacity(0.5);
    div()
        .id(id)
        .size(px(28.0))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .opacity(if enabled { 1.0 } else { 0.3 })
        .when(enabled, |s| {
            s.cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |module, _, _, cx| {
                    if let Some(direction) = direction {
                        module.step(direction, cx);
                    } else {
                        module.back(cx);
                    }
                }))
        })
        .child(
            svg()
                .path(icon)
                .size(px(14.0))
                .text_color(theme.foreground_muted()),
        )
}

pub(crate) fn render(
    module: &AppearanceModule,
    theme: &Theme,
    cx: &mut Context<AppearanceModule>,
) -> gpui::Stateful<gpui::Div> {
    let count = module.count();
    let selected = module.carousel().selected;
    let position = module.carousel().position;
    let radius = cx
        .global::<services::AppState>()
        .config
        .get()
        .ui
        .cards_round;
    let title = module.text(
        match module.kind {
            AppearanceKind::Themes => "dashboard_new.themes",
            AppearanceKind::Wallpapers => "dashboard_new.wallpapers",
        },
        cx,
    );
    let mut carousel = div()
        .id("appearance-carousel")
        .relative()
        .w_full()
        .h(px(128.0))
        .flex_shrink_0()
        .overflow_hidden();
    if count == 0 {
        let key = if !module.ready {
            "dashboard_new.loading"
        } else {
            match module.kind {
                AppearanceKind::Themes => "dashboard_new.no_themes",
                AppearanceKind::Wallpapers => "dashboard_new.no_wallpapers",
            }
        };
        carousel = carousel.flex().items_center().justify_center().child(
            div()
                .text_size(px(12.0))
                .text_color(theme.foreground_muted())
                .child(module.text(key, cx)),
        );
    }
    for index in 0..count {
        let relative = index as f32 - position;
        if relative.abs() > 2.8 {
            continue;
        }
        let props = geometry(relative, module.carousel().piano_offset(index));
        let preview_width = (props.width - 2.0).max(1.0);
        let preview_height = (props.height - 28.0).max(1.0);
        let preview_radius = px((radius.min(16.0) - 1.0).max(0.0));
        let (id, name, preview, applied): (String, String, AnyElement, bool) = match module.kind {
            AppearanceKind::Themes => {
                let item = &module.themes[index];
                let colors = [
                    item.theme.red(),
                    item.theme.green(),
                    item.theme.accent(),
                    item.theme.foreground(),
                    item.theme.foreground_muted(),
                    item.theme.surface(),
                ];
                let preview =
                    div()
                        .w(px(preview_width))
                        .h(px(preview_height))
                        .flex_shrink_0()
                        .rounded_tl(preview_radius)
                        .rounded_tr(preview_radius)
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(item.theme.background())
                        .child(div().flex().gap(px(4.0)).children(colors.into_iter().map(
                            |color| {
                                div()
                                    .size(px(if relative.abs() < 0.5 { 10.0 } else { 8.0 }))
                                    .rounded_full()
                                    .bg(color)
                            },
                        )));
                (
                    format!("appearance-theme-{}", item.path.display()),
                    item.name.clone(),
                    preview.into_any_element(),
                    item.theme == *theme,
                )
            }
            AppearanceKind::Wallpapers => {
                let item = &module.wallpapers[index];
                let preview = div()
                    .relative()
                    .w(px(preview_width))
                    .h(px(preview_height))
                    .flex_shrink_0()
                    .rounded_tl(preview_radius)
                    .rounded_tr(preview_radius)
                    .overflow_hidden()
                    .bg(theme.surface())
                    .when_some(item.thumbnail.as_ref(), |s, path| {
                        s.child(
                            img(path.clone())
                                .w(px(preview_width))
                                .h(px(preview_height))
                                .aspect_ratio(preview_width / preview_height)
                                .rounded_tl(preview_radius)
                                .rounded_tr(preview_radius)
                                .object_fit(gpui::ObjectFit::Cover),
                        )
                    });
                (
                    format!("appearance-wallpaper-{}", item.path.display()),
                    item.name.clone(),
                    preview.into_any_element(),
                    module.current_wallpaper.as_ref() == Some(&item.path),
                )
            }
        };
        let hover = theme.surface().opacity(0.6);
        carousel = carousel.child(
            div()
                .id(id)
                .absolute()
                .left(px(props.left))
                .top(px(props.top))
                .w(px(props.width))
                .h(px(props.height))
                .flex()
                .flex_col()
                .rounded(px(radius.min(16.0)))
                .overflow_hidden()
                .bg(theme.surface().opacity(0.35))
                .border_1()
                .border_color(if index == selected {
                    theme.accent()
                } else {
                    theme.surface().opacity(0.7)
                })
                .opacity(props.opacity)
                .cursor_pointer()
                .hover(move |s| s.bg(hover))
                .on_click(cx.listener(move |module, _, _, cx| module.choose(index, cx)))
                .child(preview)
                .child(
                    div()
                        .h(px(26.0))
                        .flex_shrink_0()
                        .px(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_ellipsis()
                                .text_size(px(11.0))
                                .child(name),
                        )
                        .when(applied, |s| {
                            s.child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(theme.accent())
                                    .child("✓"),
                            )
                        }),
                ),
        );
    }
    carousel = carousel.when_some(module.error.as_ref(), |s, error| {
        s.child(
            div()
                .absolute()
                .bottom_0()
                .left_0()
                .w_full()
                .text_size(px(11.0))
                .text_color(theme.red())
                .text_ellipsis()
                .child(error.clone()),
        )
    });
    let applied = match module.kind {
        AppearanceKind::Themes => module
            .themes
            .get(selected)
            .is_some_and(|item| item.theme == *theme),
        AppearanceKind::Wallpapers => module
            .wallpapers
            .get(selected)
            .is_some_and(|item| module.current_wallpaper.as_ref() == Some(&item.path)),
    };
    let enabled = count > 0 && !module.pending && !applied;
    let hover = theme.surface().opacity(0.6);
    div()
        .id("appearance-module")
        .size_full()
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(
            div()
                .h(px(28.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(control(
                    "appearance-back",
                    "chevron-left.svg",
                    true,
                    theme,
                    cx,
                    None,
                ))
                .child(div().flex_1().text_size(px(13.0)).child(title))
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(theme.foreground_muted())
                        .child(format!(
                            "{} / {}",
                            if count == 0 { 0 } else { selected + 1 },
                            count
                        )),
                ),
        )
        .child(carousel)
        .child(
            div()
                .h(px(28.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(control(
                    "appearance-prev",
                    "chevron-left.svg",
                    selected > 0,
                    theme,
                    cx,
                    Some(-1),
                ))
                .child(
                    div()
                        .id("appearance-apply")
                        .h(px(28.0))
                        .px(px(12.0))
                        .rounded(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .bg(theme.accent().opacity(0.15))
                        .opacity(if enabled || applied { 1.0 } else { 0.4 })
                        .when(enabled, |s| {
                            s.cursor_pointer()
                                .hover(move |s| s.bg(hover))
                                .on_click(cx.listener(|module, _, _, cx| module.apply(cx)))
                        })
                        .child(
                            svg()
                                .path(if module.pending {
                                    "rotate-ccw.svg"
                                } else {
                                    match module.kind {
                                        AppearanceKind::Themes => "palette_2.svg",
                                        AppearanceKind::Wallpapers => "wallpaper.svg",
                                    }
                                })
                                .size(px(13.0))
                                .text_color(theme.accent()),
                        )
                        .child(div().text_size(px(11.0)).child(module.text(
                            if module.pending {
                                "dashboard_new.pending"
                            } else if applied {
                                "dashboard_new.applied"
                            } else {
                                "dashboard_new.apply"
                            },
                            cx,
                        ))),
                )
                .child(control(
                    "appearance-next",
                    "chevron-right.svg",
                    selected + 1 < count,
                    theme,
                    cx,
                    Some(1),
                )),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cards_share_compact_geometry_and_are_centered() {
        let center = geometry(0.0, 0.0);
        assert_eq!(center.width, 160.0);
        assert_eq!(center.height, 100.0);
        assert_eq!(center.left + center.width / 2.0, CAROUSEL_WIDTH / 2.0);
        let left = geometry(-1.0, 4.0);
        let right = geometry(1.0, 4.0);
        assert_eq!(left.width, right.width);
        assert_eq!(left.top, right.top);
        assert!(left.left + left.width < center.left);
        assert!(right.left > center.left + center.width);
        assert!(center.width < 204.0 && center.height < 126.0);
    }
}
