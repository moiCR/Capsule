use gpui::{ColorExt, IntoElement, div, prelude::*, px};
use ui::theme::Theme;

use crate::new_capsule::satellite::layout::{SatelliteLayout, Side};

pub(crate) fn render(
    layout: SatelliteLayout,
    progress: f32,
    open: bool,
    radius: f32,
    content: impl IntoElement,
    theme: &Theme,
) -> gpui::Stateful<gpui::Div> {
    let radius = px(radius.max(0.0))
        .min(layout.bounds.size.width / 2.0)
        .min(layout.bounds.size.height / 2.0);
    let body = div()
        .absolute()
        .top_0()
        .when(matches!(layout.side, Side::Left), |s| s.right_0())
        .when(!matches!(layout.side, Side::Left), |s| s.left_0())
        .w(layout.content_size.width)
        .max_h(layout.content_size.height)
        .flex()
        .flex_col()
        .child(content);
    div()
        .id("satellite-surface")
        .absolute()
        .left(layout.bounds.origin.x)
        .top(layout.bounds.origin.y)
        .w(layout.bounds.size.width)
        .h(layout.bounds.size.height)
        .rounded(radius)
        .opacity(progress.clamp(0.0, 1.0))
        .overflow_hidden()
        .bg(theme.background())
        .border_1()
        .border_color(theme.surface().opacity(0.45))
        .child(body)
        .when(!open || progress < 1.0, |s| {
            s.child(div().absolute().inset_0().occlude())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext, Bounds, Context, IntoElement, Render, Window, point};
    use std::sync::{Arc, Mutex};
    use ui::tracker::DimensionTracker;

    struct Content {
        tracker: DimensionTracker,
        rows: Option<usize>,
    }

    impl Render for Content {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let content = if let Some(rows) = self.rows {
                let mut list = crate::new_capsule::widgets::dashboard::details::list("test-list");
                for _ in 0..rows {
                    list = list.child(div().h(px(36.0)).flex_shrink_0());
                }
                div()
                    .w(px(300.0))
                    .max_h(px(348.0))
                    .flex()
                    .flex_col()
                    .p(px(16.0))
                    .gap(px(12.0))
                    .child(div().h(px(28.0)).flex_shrink_0())
                    .child(list)
            } else {
                div().w(px(300.0)).h(px(350.0)).child("Calendar content")
            };
            self.tracker.track(content)
        }
    }

    struct Preview {
        progress: f32,
        content: gpui::Entity<Content>,
        surface: DimensionTracker,
    }

    impl Render for Preview {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut layout = SatelliteLayout::new(
                window.viewport_size(),
                Bounds {
                    origin: point(px(350.0), px(8.0)),
                    size: gpui::size(px(250.0), px(40.0)),
                },
                gpui::size(px(300.0), px(350.0)),
                px(8.0),
                Side::Left,
                self.progress,
            );
            layout.bounds.origin -= point(px(350.0), px(8.0));
            let tracker = self.surface.clone();
            div().relative().size_full().child(
                div()
                    .absolute()
                    .left(px(350.0))
                    .top(px(8.0))
                    .w(px(250.0))
                    .h(px(40.0))
                    .child(
                        render(
                            layout,
                            self.progress,
                            true,
                            20.0,
                            self.content.clone(),
                            &Theme::default(),
                        )
                        .child(
                            gpui::canvas(
                                move |bounds, _, _| tracker.track_bounds(bounds),
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        ),
                    ),
            )
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn slide_preserves_surface_and_content_dimensions() {
        let measurements = Arc::new(Mutex::new(Vec::new()));
        let output = measurements.clone();
        gpui_platform::headless()
            .with_assets(assets::Assets {})
            .run(move |cx| {
                let window = cx
                    .open_window(
                        gpui::WindowOptions {
                            window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
                                origin: point(px(0.0), px(0.0)),
                                size: gpui::size(px(950.0), px(500.0)),
                            })),
                            ..Default::default()
                        },
                        |_, cx| {
                            let content = cx.new(|_| Content {
                                tracker: DimensionTracker::new(),
                                rows: None,
                            });
                            cx.new(|_| Preview {
                                progress: 0.1,
                                content,
                                surface: DimensionTracker::new(),
                            })
                        },
                    )
                    .expect("headless satellite window");
                let mut app = cx.to_async();
                cx.foreground_executor()
                    .spawn(async move {
                        for progress in [0.1, 0.5, 1.0, 0.5, 0.1] {
                            app.update_window(window.into(), |root, window, cx| {
                                let root = root.downcast::<Preview>().expect("satellite preview");
                                root.update(cx, |view, cx| {
                                    view.progress = progress;
                                    cx.notify();
                                });
                                window.draw(cx).clear(cx);
                                let view = root.read(cx);
                                let content = view.content.read(cx);
                                output.lock().expect("measurements").push((
                                    progress,
                                    content.tracker.width(0.0),
                                    content.tracker.height(0.0),
                                    view.surface.width(0.0),
                                ));
                            })
                            .expect("draw satellite preview");
                        }
                        for (rows, expected) in [(3, 196.0), (30, 348.0), (1, 108.0)] {
                            app.update_window(window.into(), |root, window, cx| {
                                let root = root.downcast::<Preview>().expect("satellite preview");
                                let content = root.read(cx).content.clone();
                                content.update(cx, |content, cx| {
                                    content.rows = Some(rows);
                                    cx.notify();
                                });
                                window.draw(cx).clear(cx);
                                let height = content.read(cx).tracker.height(0.0);
                                assert!((height - expected).abs() < 0.5,
                                    "natural content with {rows} rows: {height}, expected {expected}");
                            }).expect("draw natural satellite content");
                        }
                        app.update(|cx| cx.quit());
                    })
                    .detach();
            });
        let measurements = measurements.lock().expect("measurements");
        assert_eq!(measurements.len(), 5);
        for &(progress, width, height, visible_width) in measurements.iter() {
            assert!(
                (width - 300.0).abs() < 0.5,
                "content width at {progress}: {width}"
            );
            assert!(
                (height - 350.0).abs() < 0.5,
                "content height at {progress}: {height}"
            );
            assert!(
                (visible_width + 2.0 - 300.0).abs() < 0.5,
                "surface width at {progress}: {visible_width}"
            );
        }
    }
}
