use std::path::PathBuf;

use crate::new_capsule::module::appearance::{AppearanceKind, AppearanceModule};
use crate::new_capsule::widgets::style;
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
    let hover = style::hover(theme);
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

fn render_theme_preview(item_theme: &Theme) -> AnyElement {
    div()
        .size_full()
        .rounded(px(style::CARD_RADIUS - 2.0))
        .overflow_hidden()
        .p(px(8.0))
        .bg(item_theme.background())
        .flex()
        .gap(px(6.0))
        .child(
            div()
                .w(px(20.0))
                .h_full()
                .rounded(px(5.0))
                .bg(item_theme.surface())
                .p(px(3.0))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(3.0))
                .child(div().size(px(5.0)).rounded_full().bg(item_theme.accent()))
                .child(
                    div()
                        .size(px(4.0))
                        .rounded_full()
                        .bg(item_theme.foreground_muted().opacity(0.4)),
                )
                .child(
                    div()
                        .size(px(4.0))
                        .rounded_full()
                        .bg(item_theme.foreground_muted().opacity(0.4)),
                )
                .child(
                    div()
                        .size(px(4.0))
                        .rounded_full()
                        .bg(item_theme.foreground_muted().opacity(0.4)),
                ),
        )
        .child(
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .w_full()
                        .h(px(12.0))
                        .rounded(px(3.0))
                        .bg(item_theme.surface())
                        .flex()
                        .items_center()
                        .px(px(4.0))
                        .child(
                            div()
                                .w(px(22.0))
                                .h(px(3.0))
                                .rounded_full()
                                .bg(item_theme.accent().opacity(0.8)),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(12.0))
                        .rounded(px(3.0))
                        .bg(item_theme.surface())
                        .flex()
                        .items_center()
                        .px(px(4.0))
                        .child(
                            div()
                                .w(px(30.0))
                                .h(px(3.0))
                                .rounded_full()
                                .bg(item_theme.foreground_muted().opacity(0.4)),
                        ),
                )
                .child(
                    div()
                        .w_full()
                        .flex_1()
                        .rounded(px(5.0))
                        .bg(item_theme.surface())
                        .p(px(4.0))
                        .flex()
                        .items_center()
                        .gap(px(5.0))
                        .child(div().size(px(22.0)).rounded_full().bg(item_theme.accent()))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.0))
                                .child(
                                    div()
                                        .w_full()
                                        .h(px(4.0))
                                        .rounded_full()
                                        .bg(item_theme.foreground().opacity(0.8)),
                                )
                                .child(
                                    div()
                                        .w(px(24.0))
                                        .h(px(3.0))
                                        .rounded_full()
                                        .bg(item_theme.foreground_muted().opacity(0.4)),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn render_wallpaper_preview(thumbnail: Option<&PathBuf>, theme: &Theme) -> AnyElement {
    if let Some(path) = thumbnail {
        img(path.clone())
            .size_full()
            .rounded(px(style::CARD_RADIUS))
            .object_fit(gpui::ObjectFit::Cover)
            .into_any_element()
    } else {
        div()
            .size_full()
            .rounded(px(style::CARD_RADIUS))
            .bg(theme.surface())
            .flex()
            .items_center()
            .justify_center()
            .child(
                svg()
                    .path("image.svg")
                    .size(px(28.0))
                    .text_color(theme.foreground_muted()),
            )
            .into_any_element()
    }
}

pub(crate) fn render(
    module: &AppearanceModule,
    theme: &Theme,
    cx: &mut Context<AppearanceModule>,
) -> gpui::Stateful<gpui::Div> {
    let count = module.count();
    let selected = module.carousel().selected;
    let position = module.carousel().position;
    let title = match module.kind {
        AppearanceKind::Themes => "Temas",
        AppearanceKind::Wallpapers => "Fondos",
    };
    let mut carousel = div()
        .id("appearance-carousel")
        .relative()
        .w_full()
        .h(px(140.0))
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
        let (id, name, preview, applied): (String, String, AnyElement, bool) = match module.kind {
            AppearanceKind::Themes => {
                let item = &module.themes[index];
                (
                    format!("appearance-theme-{}", item.path.display()),
                    item.name.clone(),
                    render_theme_preview(&item.theme),
                    item.theme == *theme,
                )
            }
            AppearanceKind::Wallpapers => {
                let item = &module.wallpapers[index];
                (
                    format!("appearance-wallpaper-{}", item.path.display()),
                    item.name.clone(),
                    render_wallpaper_preview(item.thumbnail.as_ref(), theme),
                    module.current_wallpaper.as_ref() == Some(&item.path),
                )
            }
        };
        let name = if module.kind == AppearanceKind::Wallpapers {
            let mut characters = name.chars();
            let mut label: String = characters.by_ref().take(12).collect();
            if characters.next().is_some() {
                label.push_str("...");
            }
            label
        } else {
            name
        };
        carousel = carousel
            .child(
                div()
                    .id(id)
                    .absolute()
                    .left(px(props.left))
                    .top(px(props.top))
                    .w(px(props.width))
                    .h(px(props.height))
                    .rounded(px(style::CARD_RADIUS))
                    .overflow_hidden()
                    .bg(style::surface(theme))
                    .border_2()
                    .border_color(if index == selected {
                        theme.accent()
                    } else {
                        theme.foreground().opacity(0.1)
                    })
                    .opacity(props.opacity)
                    .cursor_pointer()
                    .hover(move |s| s.opacity(1.0))
                    .on_click(cx.listener(move |module, _, _, cx| module.choose(index, cx)))
                    .child(preview)
                    .when(applied, |s| {
                        s.child(
                            div()
                                .absolute()
                                .top(px(8.0))
                                .right(px(8.0))
                                .child(style::check_badge(18.0, theme)),
                        )
                    }),
            )
            .child(
                div()
                    .absolute()
                    .left(px(props.left))
                    .top(px(props.top + props.height + 6.0))
                    .w(px(props.width))
                    .h(px(20.0))
                    .overflow_hidden()
                    .text_center()
                    .text_ellipsis()
                    .text_size(if index == selected {
                        px(13.0)
                    } else {
                        px(11.0)
                    })
                    .font_weight(if index == selected {
                        gpui::FontWeight::MEDIUM
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .text_color(if index == selected {
                        theme.foreground()
                    } else {
                        theme.foreground_muted()
                    })
                    .opacity(props.opacity)
                    .child(name),
            );
    }
    carousel = carousel
        .child(div().absolute().left(px(8.0)).top(px(50.0)).child(control(
            "appearance-prev",
            "chevron-left.svg",
            selected > 0,
            theme,
            cx,
            Some(-1),
        )))
        .child(div().absolute().right(px(8.0)).top(px(50.0)).child(control(
            "appearance-next",
            "chevron-right.svg",
            selected + 1 < count,
            theme,
            cx,
            Some(1),
        )));
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
    div()
        .id("appearance-module")
        .size_full()
        .p(px(14.0))
        .flex()
        .flex_col()
        .justify_between()
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .child(
            div()
                .h(px(28.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            svg()
                                .path(match module.kind {
                                    AppearanceKind::Themes => "palette_2.svg",
                                    AppearanceKind::Wallpapers => "image.svg",
                                })
                                .size(px(18.0))
                                .text_color(theme.foreground()),
                        )
                        .child(
                            div()
                                .text_size(px(14.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(title),
                        ),
                )
                .child(control(
                    "appearance-back",
                    "close.svg",
                    true,
                    theme,
                    cx,
                    None,
                )),
        )
        .child(carousel)
        .child(
            div()
                .h(px(32.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .child(div().w(px(80.0)))
                .child(
                    div()
                        .flex_1()
                        .text_center()
                        .text_size(px(12.0))
                        .text_color(theme.foreground_muted())
                        .child(format!(
                            "{} / {}",
                            if count == 0 { 0 } else { selected + 1 },
                            count
                        )),
                )
                .child(
                    div().w(px(80.0)).flex().justify_end().child(
                        style::primary_button(
                            "appearance-apply",
                            module.text(
                                if module.pending {
                                    "dashboard_new.pending"
                                } else if applied {
                                    "dashboard_new.applied"
                                } else {
                                    "dashboard_new.apply"
                                },
                                cx,
                            ),
                            enabled,
                            theme,
                        )
                        .on_click(cx.listener(|module, _, _, cx| module.apply(cx))),
                    ),
                ),
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
