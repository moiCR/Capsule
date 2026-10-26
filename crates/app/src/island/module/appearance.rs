mod carousel;
mod catalog;

use super::{CapsuleModule, CapsuleModuleEvent, CapsuleModuleId};
use crate::island::widgets::appearance as widgets;
use carousel::Carousel;
use catalog::Catalog;
use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, Pixels, Render, Size, Subscription, Task,
    Window, prelude::*,
};
use services::AppState;
use std::time::{Duration, Instant};
use ui::theme::{
    Theme,
    theme_manager::{ThemeItem, ThemeManager},
};

pub(crate) struct AppearanceModule {
    pub themes: Vec<ThemeItem>,
    pub ready: bool,
    pub pending: bool,
    pub error: Option<String>,
    carousel: Carousel,
    focus: FocusHandle,
    active: bool,
    animation: Option<Task<()>>,
    action: Option<Task<()>>,
    _catalog: Task<()>,
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
                            .get(module.carousel.selected)
                            .map(|item| item.path.clone());
                        module.ready = true;
                        module.themes = catalog.themes;
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
                        module.carousel.reset(
                            theme_index.unwrap_or(module.carousel.selected),
                            module.themes.len(),
                        );
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            themes: Vec::new(),
            ready: false,
            pending: false,
            error: None,
            carousel: Carousel::default(),
            focus: cx.focus_handle(),
            active: false,
            animation: None,
            action: None,
            _catalog: catalog,
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
        &self.carousel
    }
    fn carousel_mut(&mut self) -> &mut Carousel {
        &mut self.carousel
    }
    pub fn count(&self) -> usize {
        self.themes.len()
    }
    pub fn open(&mut self, cx: &mut Context<Self>) {
        self.active = true;
        self.error = None;
        self.animation = None;
        let selected = self
            .themes
            .iter()
            .position(|item| item.theme == *cx.global::<Theme>())
            .unwrap_or(self.carousel.selected);
        let count = self.count();
        self.carousel.reset(selected, count);
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
        if self.themes_locked(cx) || self.pending {
            return;
        }
        let selected = self.carousel().selected;
        let Some(theme) = self.themes.get(selected).map(|item| item.theme.clone()) else {
            return;
        };
        if theme
            .clone()
            .with_mode(cx.global::<AppState>().config.get().ui.dark_mode)
            == *cx.global::<Theme>()
        {
            return;
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.pending = true;
        self.error = None;
        services::spawn_tokio(async move {
            let result = tokio::task::spawn_blocking(move || {
                ThemeManager::save_selected_theme(&theme).map(|_| theme)
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
                    Ok(Ok(theme)) => {
                        cx.global_mut::<ThemeManager>().selected_theme = theme.clone();
                        if module.themes_locked(cx) {
                            cx.notify();
                            return;
                        }
                        let selected = theme.clone();
                        let theme =
                            theme.with_mode(cx.global::<AppState>().config.get().ui.dark_mode);
                        if cx.has_global::<ThemeManager>() {
                            let manager = cx.global_mut::<ThemeManager>();
                            manager.selected_theme = selected;
                            manager.current_theme = theme.clone();
                            manager.apply_theme_to_apps();
                        }
                        crate::settings::appearance::persist_theme(theme.clone(), cx);
                        crate::settings::appearance::transition_theme(theme, cx);
                    }
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
                cx.notify();
            });
        }));
        cx.notify();
    }
    pub fn themes_locked(&self, cx: &gpui::App) -> bool {
        cx.global::<AppState>().config.get().ui.dynamic_colors
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
            "f4" if event.keystroke.modifiers.alt => self.back(cx),
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
