mod carousel;
mod catalog;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::new_capsule::widgets::appearance as widgets;
use carousel::Carousel;
use catalog::{Catalog, WallpaperEntry};
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, Pixels, Render, Size, Subscription, Task,
    Window, prelude::*,
};
use services::AppState;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use ui::theme::{
    Theme,
    theme_manager::{ThemeItem, ThemeManager},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppearanceKind {
    Themes,
    Wallpapers,
}

pub(crate) struct AppearanceModule {
    pub kind: AppearanceKind,
    pub themes: Vec<ThemeItem>,
    pub wallpapers: Vec<WallpaperEntry>,
    pub ready: bool,
    pub pending: bool,
    pub error: Option<String>,
    pub current_wallpaper: Option<PathBuf>,
    themes_carousel: Carousel,
    wallpapers_carousel: Carousel,
    focus: FocusHandle,
    active: bool,
    animation: Option<Task<()>>,
    action: Option<Task<()>>,
    _catalog: Task<()>,
    _refresh: Task<()>,
    _theme: Subscription,
    worker: tokio::task::JoinHandle<()>,
}

impl AppearanceModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        let worker = services::spawn_tokio(async move {
            loop {
                let Ok(catalog) = tokio::task::spawn_blocking(Catalog::load).await else {
                    break;
                };
                if sender.send(catalog).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
        let catalog = cx.spawn(async move |this, cx| {
            while let Some(catalog) = receiver.recv().await {
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let first = !module.ready;
                        let changed = first
                            || module.wallpapers != catalog.wallpapers
                            || module.themes.len() != catalog.themes.len()
                            || module
                                .themes
                                .iter()
                                .zip(&catalog.themes)
                                .any(|(a, b)| a.path != b.path || a.theme != b.theme);
                        if !changed {
                            return;
                        }
                        let selected_theme = module
                            .themes
                            .get(module.themes_carousel.selected)
                            .map(|item| item.path.clone());
                        let selected_wallpaper = module
                            .wallpapers
                            .get(module.wallpapers_carousel.selected)
                            .map(|item| item.path.clone());
                        module.ready = true;
                        module.themes = catalog.themes;
                        module.wallpapers = catalog.wallpapers;
                        let theme_index = if first {
                            module
                                .themes
                                .iter()
                                .position(|item| item.theme == *cx.global::<Theme>())
                        } else {
                            module
                                .themes
                                .iter()
                                .position(|item| Some(&item.path) == selected_theme.as_ref())
                        };
                        let wallpaper_index = module.wallpapers.iter().position(|item| {
                            Some(&item.path)
                                == if first {
                                    module.current_wallpaper.as_ref()
                                } else {
                                    selected_wallpaper.as_ref()
                                }
                        });
                        module.themes_carousel.reset(
                            theme_index.unwrap_or(module.themes_carousel.selected),
                            module.themes.len(),
                        );
                        module.wallpapers_carousel.reset(
                            wallpaper_index.unwrap_or(module.wallpapers_carousel.selected),
                            module.wallpapers.len(),
                        );
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let refresh = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this
                    .update(cx, |module: &mut Self, cx| {
                        let current = cx.global::<AppState>().wallpaper.get_current();
                        if current != module.current_wallpaper {
                            module.current_wallpaper = current;
                            if module.active {
                                cx.notify();
                            }
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            kind: AppearanceKind::Themes,
            themes: Vec::new(),
            wallpapers: Vec::new(),
            ready: false,
            pending: false,
            error: None,
            current_wallpaper: cx.global::<AppState>().wallpaper.get_current(),
            themes_carousel: Carousel::default(),
            wallpapers_carousel: Carousel::default(),
            focus: cx.focus_handle(),
            active: false,
            animation: None,
            action: None,
            _catalog: catalog,
            _refresh: refresh,
            _theme: cx.observe_global::<Theme>(|module, cx| {
                if module.active {
                    cx.notify();
                }
            }),
            worker,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
        if !active {
            self.animation = None;
        }
    }
    pub fn carousel(&self) -> &Carousel {
        match self.kind {
            AppearanceKind::Themes => &self.themes_carousel,
            AppearanceKind::Wallpapers => &self.wallpapers_carousel,
        }
    }
    fn carousel_mut(&mut self) -> &mut Carousel {
        match self.kind {
            AppearanceKind::Themes => &mut self.themes_carousel,
            AppearanceKind::Wallpapers => &mut self.wallpapers_carousel,
        }
    }
    pub fn count(&self) -> usize {
        match self.kind {
            AppearanceKind::Themes => self.themes.len(),
            AppearanceKind::Wallpapers => self.wallpapers.len(),
        }
    }
    pub fn open(&mut self, kind: AppearanceKind, cx: &mut Context<Self>) {
        self.kind = kind;
        self.active = true;
        self.error = None;
        self.animation = None;
        let selected = match kind {
            AppearanceKind::Themes => self
                .themes
                .iter()
                .position(|item| item.theme == *cx.global::<Theme>()),
            AppearanceKind::Wallpapers => self
                .wallpapers
                .iter()
                .position(|item| Some(&item.path) == self.current_wallpaper.as_ref()),
        }
        .unwrap_or(self.carousel().selected);
        let count = self.count();
        self.carousel_mut().reset(selected, count);
        cx.notify();
    }
    pub fn navigate(&mut self, selected: usize, cx: &mut Context<Self>) {
        let duration = Duration::from_millis(
            cx.global::<AppState>()
                .config
                .get()
                .ui
                .animation_duration_ms as u64,
        );
        let count = self.count();
        if !self
            .carousel_mut()
            .navigate(selected, count, duration, Instant::now())
        {
            return;
        }
        cx.notify();
        if self.animation.is_some() || !self.carousel().animating() {
            return;
        }
        let compositor = cx.global::<AppState>().compositor.clone();
        self.animation = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(
                        compositor
                            .get_frame_duration()
                            .max(Duration::from_millis(2)),
                    )
                    .await;
                match this.update(cx, |module, cx| {
                    let running = module.carousel_mut().advance(Instant::now());
                    if !running {
                        module.animation = None;
                    }
                    cx.notify();
                    running
                }) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => break,
                }
            }
        }));
    }
    pub fn step(&mut self, direction: i32, cx: &mut Context<Self>) {
        let selected = (self.carousel().selected as i64 + direction as i64).max(0) as usize;
        self.navigate(selected, cx);
    }
    pub fn choose(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.carousel().selected != index {
            self.navigate(index, cx);
            return;
        }
        self.apply(cx);
    }
    pub fn apply(&mut self, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        let selected = self.carousel().selected;
        let theme = (self.kind == AppearanceKind::Themes)
            .then(|| self.themes.get(selected).map(|item| item.theme.clone()))
            .flatten();
        let path = (self.kind == AppearanceKind::Wallpapers)
            .then(|| self.wallpapers.get(selected).map(|item| item.path.clone()))
            .flatten();
        if theme.is_none() && path.is_none() {
            return;
        }
        if theme
            .as_ref()
            .is_some_and(|theme| theme == cx.global::<Theme>())
            || path
                .as_ref()
                .is_some_and(|path| Some(path) == self.current_wallpaper.as_ref())
        {
            return;
        }
        let state = cx.global::<AppState>().clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.pending = true;
        self.error = None;
        services::spawn_tokio(async move {
            let result = tokio::task::spawn_blocking(move || {
                if let Some(theme) = theme {
                    ThemeManager::save_current_theme(&theme).map(|_| Some(theme))
                } else if let Some(path) = path {
                    if state.wallpaper.set_wallpaper(path) {
                        Ok(None)
                    } else {
                        Err("dashboard_new.wallpaper_error".to_string())
                    }
                } else {
                    Ok(None)
                }
            })
            .await
            .map_err(|error| error.to_string())
            .and_then(|result| result);
            let _ = sender.send(result);
        });
        self.action = Some(cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |module, cx| {
                module.pending = false;
                match result {
                    Ok(Ok(Some(theme))) => {
                        if cx.has_global::<ThemeManager>() {
                            let manager = cx.global_mut::<ThemeManager>();
                            manager.current_theme = theme.clone();
                            manager.apply_theme_to_apps();
                        }
                        cx.set_global(theme);
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        module.error = Some(if error.starts_with("dashboard_new.") {
                            cx.global::<AppState>().language.get(&error)
                        } else {
                            error
                        })
                    }
                    Err(_) => {
                        module.error = Some(
                            cx.global::<AppState>()
                                .language
                                .get("dashboard_new.action_error"),
                        )
                    }
                }
                module.current_wallpaper = cx.global::<AppState>().wallpaper.get_current();
                cx.notify();
            });
        }));
        cx.notify();
    }
    pub fn text(&self, key: &str, cx: &gpui::App) -> String {
        cx.global::<AppState>().language.get(key)
    }
    pub fn back(&mut self, cx: &mut Context<Self>) {
        cx.emit(CapsuleModuleEvent::Open(CapsuleModuleId::Dashboard));
    }
    fn key_down(&mut self, event: &gpui::KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        match event.keystroke.key.as_str() {
            "left" => self.step(-1, cx),
            "right" => self.step(1, cx),
            "home" => self.navigate(0, cx),
            "end" => self.navigate(self.count().saturating_sub(1), cx),
            "enter" => self.apply(cx),
            "escape" => self.back(cx),
            _ => {}
        }
    }
}
impl Drop for AppearanceModule {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
impl EventEmitter<CapsuleModuleEvent> for AppearanceModule {}
impl CapsuleModule for AppearanceModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(gpui::px(widgets::WIDTH), gpui::px(widgets::HEIGHT))
    }
}
impl Render for AppearanceModule {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        widgets::render(self, &theme, cx)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
    }
}
