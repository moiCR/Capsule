use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::{Capsule, widgets::shelf_drop as widgets};
use gpui::{Context, EventEmitter, IntoElement, Pixels, Render, Size, Task, Window, px};
use std::time::Duration;
use ui::theme::Theme;

pub(crate) struct ShelfDropModule {
    return_to: Option<CapsuleModuleId>,
    epoch: u64,
    monitor: Option<Task<()>>,
}

impl ShelfDropModule {
    pub fn new(_: &mut Context<Self>) -> Self {
        Self {
            return_to: None,
            epoch: 0,
            monitor: None,
        }
    }

    pub fn return_to(&self) -> Option<CapsuleModuleId> {
        self.return_to
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn open(&mut self, previous: CapsuleModuleId, cx: &mut Context<Self>) {
        if self.return_to.is_some()
            || !matches!(previous, CapsuleModuleId::Default | CapsuleModuleId::Shelf)
        {
            return;
        }
        self.epoch = self.epoch.wrapping_add(1);
        self.return_to = Some(previous);
        self.monitor = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(40))
                    .await;
                let running = this.update(cx, |module, cx| {
                    if !cx.has_active_drag() {
                        module.finish(cx);
                    }
                    module.return_to.is_some()
                });
                if !matches!(running, Ok(true)) {
                    break;
                }
            }
        }));
        cx.emit(CapsuleModuleEvent::Open(CapsuleModuleId::ShelfDrop));
    }

    pub fn finish(&mut self, cx: &mut Context<Self>) {
        let Some(return_to) = self.return_to.take() else {
            return;
        };
        self.monitor = None;
        cx.emit(CapsuleModuleEvent::ShelfDropFinished {
            return_to,
            epoch: self.epoch,
        });
    }

    pub fn cancel(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.return_to = None;
        self.monitor = None;
    }
}

impl EventEmitter<CapsuleModuleEvent> for ShelfDropModule {}

impl CapsuleModule for ShelfDropModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(px(widgets::WIDTH), px(widgets::HEIGHT))
    }
}

impl Render for ShelfDropModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        widgets::render(cx.global::<Theme>())
    }
}

impl Capsule {
    pub(in crate::new_capsule) fn set_shelf_drop_hovered(
        &mut self,
        hovered: bool,
        cx: &mut Context<Self>,
    ) {
        if self.module_manager.polkit.read(cx).has_request() {
            return;
        }
        let current = self.module_manager.current_id();
        self.module_manager.shelf_drop.update(cx, |module, cx| {
            if hovered {
                module.open(current, cx);
            } else {
                module.finish(cx);
            }
        });
    }

    pub(in crate::new_capsule) fn drop_on_shelf(
        &mut self,
        paths: &gpui::ExternalPaths,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        self.module_manager
            .shelf
            .update(cx, |module, _| module.add_paths(paths.paths().to_vec()));
        self.module_manager
            .shelf_drop
            .update(cx, |module, cx| module.finish(cx));
    }

    pub(in crate::new_capsule) fn drop_shelf_file(
        &mut self,
        file: &crate::new_capsule::widgets::shelf::DraggedFile,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        self.module_manager
            .shelf
            .update(cx, |module, _| module.add_paths(vec![file.path.clone()]));
        self.module_manager
            .shelf_drop
            .update(cx, |module, cx| module.finish(cx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        AppContext, Bounds, Entity, FileDropEvent, InputEvent, Subscription, div, point, prelude::*,
    };

    struct Preview {
        module: Entity<ShelfDropModule>,
        current: CapsuleModuleId,
        window_available: bool,
        shelf: services::ShelfService,
        _subscription: Subscription,
    }

    impl Render for Preview {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .id("shelf-drop-preview")
                .size_full()
                .on_drag_move::<gpui::ExternalPaths>(cx.listener(
                    |preview, event: &gpui::DragMoveEvent<gpui::ExternalPaths>, _, cx| {
                        let previous = preview.current;
                        preview.module.update(cx, |module, cx| {
                            if event.bounds.contains(&event.event.position) {
                                module.open(previous, cx);
                            } else {
                                module.finish(cx);
                            }
                        });
                    },
                ))
                .on_drop(cx.listener(|preview, _: &gpui::ExternalPaths, _, cx| {
                    preview.module.update(cx, |module, cx| module.finish(cx));
                }))
                .child(
                    div()
                        .size_full()
                        .occlude()
                        .on_drop(cx.listener(|preview, paths: &gpui::ExternalPaths, _, cx| {
                            preview.shelf.add_paths(paths.paths());
                            preview.module.update(cx, |module, cx| module.finish(cx));
                        }))
                        .child(self.module.clone()),
                )
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn drag_events_restore_the_previous_module_without_borrowing_the_window() {
        let shelf = services::ShelfService::new();
        let dropped_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        gpui_platform::headless()
            .with_assets(assets::Assets {})
            .run(|cx| {
                cx.set_global(Theme::default());
                let window = cx
                    .open_window(
                        gpui::WindowOptions {
                            window_bounds: Some(gpui::WindowBounds::Windowed(Bounds {
                                origin: point(px(0.0), px(0.0)),
                                size: gpui::size(px(560.0), px(104.0)),
                            })),
                            ..Default::default()
                        },
                        |window, cx| {
                            cx.new(|cx| {
                                let module = cx.new(ShelfDropModule::new);
                                let handle = window.window_handle();
                                let subscription = cx.subscribe(
                                    &module,
                                    move |preview: &mut Preview, _, event, cx| {
                                        preview.window_available =
                                            handle.update(cx, |_, _, _| {}).is_ok();
                                        match event {
                                            CapsuleModuleEvent::Open(
                                                CapsuleModuleId::ShelfDrop,
                                            ) => {
                                                preview.current = CapsuleModuleId::ShelfDrop;
                                            }
                                            CapsuleModuleEvent::ShelfDropFinished {
                                                return_to,
                                                epoch,
                                            } if preview.current == CapsuleModuleId::ShelfDrop
                                                && preview.module.read(cx).epoch() == *epoch =>
                                            {
                                                preview.current = *return_to;
                                            }
                                            _ => {}
                                        }
                                        cx.notify();
                                    },
                                );
                                Preview {
                                    module,
                                    current: CapsuleModuleId::Default,
                                    window_available: false,
                                    shelf: shelf.clone(),
                                    _subscription: subscription,
                                }
                            })
                        },
                    )
                    .expect("shelf drop preview");
                let executor = cx.background_executor().clone();
                let mut app = cx.to_async();
                cx.foreground_executor()
                    .spawn(async move {
                        for previous in [CapsuleModuleId::Default, CapsuleModuleId::Shelf] {
                            app.update_window(window.into(), |root, window, cx| {
                                let root = root.downcast::<Preview>().expect("preview");
                                root.update(cx, |preview, cx| {
                                    preview.current = previous;
                                    cx.notify();
                                });
                                window.draw(cx).clear(cx);
                                window.dispatch_event(
                                    FileDropEvent::Entered {
                                        position: point(px(30.0), px(30.0)),
                                        paths: gpui::ExternalPaths(
                                            [dropped_path.clone()].into_iter().collect(),
                                        ),
                                    }
                                    .to_platform_input(),
                                    cx,
                                );
                            })
                            .expect("drag enter");
                            app.update_window(window.into(), |root, window, cx| {
                                let root = root.downcast::<Preview>().expect("preview");
                                let preview = root.read(cx);
                                assert_eq!(preview.current, CapsuleModuleId::ShelfDrop);
                                assert_eq!(preview.module.read(cx).return_to(), Some(previous));
                                assert!(preview.window_available);
                                window.draw(cx).clear(cx);
                                window.dispatch_event(
                                    FileDropEvent::Submit {
                                        position: point(px(30.0), px(30.0)),
                                    }
                                    .to_platform_input(),
                                    cx,
                                );
                            })
                            .expect("drag submit");
                            app.update_window(window.into(), |root, _, cx| {
                                let root = root.downcast::<Preview>().expect("preview");
                                let preview = root.read(cx);
                                assert_eq!(preview.current, previous);
                                assert!(preview.module.read(cx).monitor.is_none());
                                assert!(preview.window_available);
                            })
                            .expect("restore module");
                            assert_eq!(shelf.count(), 1);
                            assert_eq!(shelf.get_items()[0].path, dropped_path);
                        }
                        app.update_window(window.into(), |root, window, cx| {
                            let root = root.downcast::<Preview>().expect("preview");
                            root.update(cx, |preview, cx| {
                                preview.current = CapsuleModuleId::Default;
                                cx.notify();
                            });
                            window.draw(cx).clear(cx);
                            window.dispatch_event(
                                FileDropEvent::Entered {
                                    position: point(px(30.0), px(30.0)),
                                    paths: gpui::ExternalPaths(
                                        ["/tmp/shelf-drop-test.txt".into()].into_iter().collect(),
                                    ),
                                }
                                .to_platform_input(),
                                cx,
                            );
                            window.dispatch_event(FileDropEvent::Exited.to_platform_input(), cx);
                        })
                        .expect("cancel drag");
                        executor.timer(Duration::from_millis(90)).await;
                        app.update_window(window.into(), |root, _, cx| {
                            let root = root.downcast::<Preview>().expect("preview");
                            let preview = root.read(cx);
                            assert_eq!(preview.current, CapsuleModuleId::Default);
                            assert!(preview.module.read(cx).monitor.is_none());
                        })
                        .expect("cancel restores default");
                        app.update(|cx| cx.quit());
                    })
                    .detach();
            });
    }
}
