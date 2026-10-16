pub mod animator;
pub mod portal;
pub mod view;

use futures_util::FutureExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use animator::WallpaperAnimator;
use gpui::{
    App, AppContext, Context, IntoElement, LinuxWindowBackground, Render, Task, Window,
    WindowBounds, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
};
use services::{AppState, WallpaperTransition};

pub struct Wallpaper {
    pub animator: WallpaperAnimator,
    animation_task: Option<Task<()>>,
    _watch_task: Option<Task<()>>,
}

impl Wallpaper {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let initial = cx
            .global::<AppState>()
            .wallpaper
            .get_current_wallpaper()
            .or_else(|| {
                cx.global::<AppState>()
                    .config
                    .get()
                    .ui
                    .wallpaper_path
                    .clone()
                    .filter(|p| p.exists())
            });

        let mut watch_receiver = cx.global::<AppState>().wallpaper.subscribe();

        let watch_task = cx.spawn(async move |this, cx| {
            while let Ok(path) = watch_receiver.recv().await {
                let _ = this.update(cx, |wallpaper, cx| {
                    let config = cx.global::<AppState>().config.get();
                    let transition = config.ui.wallpaper_transition;
                    let duration = Duration::from_millis(if config.ui.reduced_motion {
                        0
                    } else {
                        (config.ui.animation_duration_ms as u64 * 2).clamp(400, 750)
                    });
                    wallpaper.set_wallpaper(path, transition, duration, cx);
                });
            }
        });

        Self {
            animator: WallpaperAnimator::new(initial),
            animation_task: None,
            _watch_task: Some(watch_task),
        }
    }

    pub fn set_wallpaper(
        &mut self,
        path: PathBuf,
        transition: WallpaperTransition,
        duration: Duration,
        cx: &mut Context<Self>,
    ) {
        self.animator
            .set_wallpaper(path, transition, duration, Instant::now());
        self.evict_completed_previous(cx);
        if self.animator.animating && self.animation_task.is_none() {
            let compositor = cx.global::<AppState>().compositor.clone();
            self.animation_task = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(
                            compositor
                                .get_frame_duration()
                                .max(Duration::from_millis(2)),
                        )
                        .await;
                    let running = this.update(cx, |wallpaper, cx| {
                        let running = wallpaper.animator.advance(Instant::now());
                        if !running {
                            wallpaper.animation_task = None;
                            wallpaper.evict_completed_previous(cx);
                        }
                        cx.notify();
                        running
                    });
                    match running {
                        Ok(true) => {}
                        Ok(false) | Err(_) => break,
                    }
                }
            }));
        }
        cx.notify();
    }

    fn evict_completed_previous(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.animator.take_completed_previous() {
            if self.animator.current.as_ref() != Some(&prev) {
                services::log_info!(
                    "WALLPAPER",
                    "Queuing eviction of previous wallpaper: {:?}",
                    prev
                );
                cx.spawn(async move |_this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(150))
                        .await;
                    let _ = cx.update(|cx| {
                        if cx.has_global::<AppState>() {
                            if let Some(curr) =
                                cx.global::<AppState>().wallpaper.get_current_wallpaper()
                            {
                                if curr == prev {
                                    services::log_info!(
                                        "WALLPAPER",
                                        "Skipping eviction: {:?} is still current",
                                        prev
                                    );
                                    return;
                                }
                            }
                        }
                        Self::evict_wallpaper_from_cache(&prev, cx);
                    });
                })
                .detach();
            }
        }
    }

    pub fn evict_wallpaper_from_cache(path: &std::path::Path, cx: &mut App) {
        let resource = gpui::Resource::from(path.to_path_buf());
        let (task, is_first) = cx.fetch_asset::<gpui::ImgResourceLoader>(&resource);
        if !is_first {
            if let Some(Ok(render_image)) = task.now_or_never() {
                services::log_info!(
                    "WALLPAPER",
                    "Successfully dropped GPU texture from sprite atlas for: {:?}",
                    path
                );
                cx.drop_image(render_image, None);
            }
        }
        services::log_info!(
            "WALLPAPER",
            "Removed asset from loading_assets cache for: {:?}",
            path
        );
        cx.remove_asset::<gpui::ImgResourceLoader>(&resource);

        #[cfg(target_os = "linux")]
        unsafe {
            libc::malloc_trim(0);
        }
    }
}

impl Render for Wallpaper {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        view::render_wallpaper(&self.animator, window)
    }
}

pub struct WallpaperPanel;

impl WallpaperPanel {
    pub fn open(cx: &mut App) {
        cx.spawn(async move |cx| {
            let mut attempts = 0;
            loop {
                let opened = cx.update(|cx| {
                    let displays = cx.displays();
                    if displays.is_empty() {
                        return false;
                    }
                    Self::open_displays(displays, cx);
                    true
                });

                if opened {
                    break;
                } else {
                    attempts += 1;
                    if attempts >= 20 {
                        cx.update(|cx| {
                            Self::open_default(cx);
                        });
                        break;
                    }
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(25))
                        .await;
                }
            }
        })
        .detach();
    }

    fn open_displays(displays: Vec<std::rc::Rc<dyn gpui::PlatformDisplay>>, cx: &mut App) {
        for display in displays {
            let options = WindowOptions {
                display_id: Some(display.id()),
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::default())),
                app_id: Some("capsule-wallpaper".to_string()),
                linux_window_background: LinuxWindowBackground::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "capsule-wallpaper".to_string(),
                    layer: Layer::Background,
                    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                    exclusive_zone: Some(gpui::px(-1.0)),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            };

            match cx.open_window(options, |window, cx| cx.new(|cx| Wallpaper::new(window, cx))) {
                Ok(_) => {
                    services::log_info!(
                        "WALLPAPER",
                        "Created wallpaper surface on display {:?}",
                        display.id()
                    );
                }
                Err(err) => {
                    services::log_error!(
                        "WALLPAPER",
                        "Failed to open wallpaper window on display {:?}: {err}",
                        display.id()
                    );
                }
            }
        }
    }

    fn open_default(cx: &mut App) {
        let options = WindowOptions {
            display_id: None,
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::default())),
            app_id: Some("capsule-wallpaper".to_string()),
            linux_window_background: LinuxWindowBackground::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-wallpaper".to_string(),
                layer: Layer::Background,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(gpui::px(-1.0)),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        };

        match cx.open_window(options, |window, cx| cx.new(|cx| Wallpaper::new(window, cx))) {
            Ok(_) => {
                services::log_info!("WALLPAPER", "Created default fallback wallpaper surface");
            }
            Err(err) => {
                services::log_error!("WALLPAPER", "Failed to open default wallpaper window: {err}");
            }
        }
    }
}
