use gpui::{IntoElement, Window, div, img, prelude::*, px};
use services::WallpaperTransition;
use super::animator::WallpaperAnimator;

pub fn render_wallpaper(animator: &WallpaperAnimator, window: &mut Window) -> impl IntoElement {
    let viewport = window.viewport_size();
    let root = div()
        .relative()
        .size_full()
        .overflow_hidden()
        .bg(gpui::black());

    match (&animator.previous, &animator.current) {
        (None, None) => root.into_any_element(),
        (None, Some(curr)) => root
            .child(
                img(curr.clone())
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .object_fit(gpui::ObjectFit::Cover),
            )
            .into_any_element(),
        (Some(prev), Some(curr)) => match animator.transition {
            WallpaperTransition::Fade => root
                .child(
                    img(prev.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover)
                        .opacity(1.0 - animator.progress),
                )
                .child(
                    img(curr.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover)
                        .opacity(animator.progress),
                )
                .into_any_element(),
            WallpaperTransition::Slide => {
                let offset = viewport.width * animator.progress;
                root.child(
                    img(prev.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left(-offset)
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .child(
                    img(curr.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left(viewport.width - offset)
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .into_any_element()
            }
            WallpaperTransition::Zoom => {
                let p = animator.progress;
                let scale = 1.05 - p * 0.05;
                root.child(
                    img(prev.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover)
                        .opacity(1.0 - p),
                )
                .child(
                    div()
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .overflow_hidden()
                        .child(
                            img(curr.clone())
                                .w(viewport.width * scale)
                                .h(viewport.height * scale)
                                .object_fit(gpui::ObjectFit::Cover)
                                .opacity(p),
                        ),
                )
                .into_any_element()
            }
            WallpaperTransition::Wipe => {
                let wipe_width = (viewport.width * animator.progress).max(px(0.0));
                root.child(
                    img(prev.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .child(
                    div()
                        .h_full()
                        .w(wipe_width)
                        .absolute()
                        .top_0()
                        .left_0()
                        .overflow_hidden()
                        .child(
                            div().w(viewport.width).h_full().child(
                                img(curr.clone())
                                    .size_full()
                                    .object_fit(gpui::ObjectFit::Cover),
                            ),
                        ),
                )
                .into_any_element()
            }
            WallpaperTransition::None => root
                .child(
                    img(curr.clone())
                        .size_full()
                        .absolute()
                        .top_0()
                        .left_0()
                        .object_fit(gpui::ObjectFit::Cover),
                )
                .into_any_element(),
        },
        (Some(prev), None) => root
            .child(
                img(prev.clone())
                    .size_full()
                    .absolute()
                    .top_0()
                    .left_0()
                    .object_fit(gpui::ObjectFit::Cover)
                    .opacity(1.0 - animator.progress),
            )
            .into_any_element(),
    }
}
