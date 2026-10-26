use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, BoxShadow, ColorExt, Context, DurationWithEasing, EventEmitter, FocusHandle,
    FontWeight, IntoElement, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render,
    ScrollHandle, Size, Task, Window, div, point, px, svg,
};
use services::{AppState, Application, CapsuleAnimation, LauncherService};
use ui::components::{Input, InputState};
use ui::theme::Theme;

use super::{PopupEvent, animator::LauncherAnimator};
use crate::island::widgets::launcher::{COLUMNS, GRID_GAP, TILE_HEIGHT, next_selection};

pub const LAUNCHER_WIDTH: f32 = 540.0;
pub const LAUNCHER_HEIGHT: f32 = 460.0;

pub struct BarLauncher {
    pub search_input: InputState,
    pub query: String,
    pub apps: Vec<Application>,
    pub calculator: Option<String>,
    pub selected: usize,
    pub mouse_moved: bool,
    pub scroll: ScrollHandle,
    pub service: LauncherService,
    pub animator: Option<LauncherAnimator>,
    pub is_open: bool,
    pub focus: FocusHandle,
    pub launch_error: Option<String>,
    pub _watch_task: Option<Task<()>>,
}

impl EventEmitter<PopupEvent> for BarLauncher {}

impl BarLauncher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = if cx.has_global::<AppState>() {
            cx.global::<AppState>().launcher.clone()
        } else {
            LauncherService::new()
        };

        let mut watch_receiver = service.subscribe();
        let watch_task = cx.spawn(async move |this, cx| {
            while let Ok(()) = watch_receiver.recv().await {
                let _ = this.update(cx, |launcher, cx| {
                    launcher.apps = launcher.service.search(&launcher.query);
                    cx.notify();
                });
            }
        });

        Self {
            search_input: InputState::new("").with_focused(true).with_max_len(120),
            query: String::new(),
            apps: Vec::new(),
            calculator: None,
            selected: 0,
            mouse_moved: false,
            scroll: ScrollHandle::new(),
            service,
            animator: None,
            is_open: false,
            focus: cx.focus_handle(),
            launch_error: None,
            _watch_task: Some(watch_task),
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_visible(&self) -> bool {
        self.is_open
            || self
                .animator
                .as_ref()
                .map(|a| a.opacity > 0.0)
                .unwrap_or(false)
    }

    pub fn open(&mut self, cx: &mut Context<Self>) {
        let (duration, animation, bounciness, squash_stretch, reduced_motion) =
            if cx.has_global::<AppState>() {
                let cfg = cx.global::<AppState>().config.get();
                (
                    Duration::from_millis(cfg.ui.bar.animation_duration_ms as u64),
                    cfg.ui.bar.animation,
                    cfg.ui.bar.spring_bounciness,
                    cfg.ui.bar.squash_stretch,
                    cfg.ui.bar.reduced_motion || cfg.ui.reduced_motion,
                )
            } else {
                (
                    Duration::from_millis(220),
                    CapsuleAnimation::Fluid,
                    0.75,
                    false,
                    false,
                )
            };

        self.is_open = true;
        self.search_input.clear();
        self.query.clear();
        self.apps = self.service.search("");
        self.calculator = None;
        self.selected = 0;
        self.mouse_moved = false;
        self.launch_error = None;
        self.scroll.scroll_to_item(0);

        let mut anim = LauncherAnimator::new(
            duration,
            animation,
            bounciness,
            squash_stretch,
            reduced_motion,
        );
        let target_size = Size {
            width: px(LAUNCHER_WIDTH),
            height: px(LAUNCHER_HEIGHT),
        };
        anim.transition_to(target_size, Instant::now());
        anim.advance(Instant::now());
        self.animator = Some(anim);
        cx.emit(PopupEvent::Open);
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        if let Some(ref mut anim) = self.animator
            && !anim.is_closing()
        {
            anim.request_close();
            cx.emit(PopupEvent::Close);
            cx.notify();
        }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        if self.is_open {
            self.close(cx);
        } else {
            self.open(cx);
        }
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        if let Some(ref mut anim) = self.animator {
            let (running, closed) = anim.advance(now);
            if closed {
                self.is_open = false;
                self.animator = None;
            }
            (running, closed)
        } else {
            (false, false)
        }
    }

    pub fn count(&self) -> usize {
        self.apps.len() + usize::from(self.calculator.is_some())
    }

    pub fn update_search(&mut self, query: String, cx: &mut Context<Self>) {
        if self.search_input.text() != query {
            self.search_input.set_text(&query);
        }
        self.query = query.clone();
        self.calculator = services::launcher::calculator::evaluate(&query);
        self.apps = self.service.search(&query);
        self.selected = 0;
        self.mouse_moved = false;
        self.launch_error = None;
        self.scroll.scroll_to_item(0);
        cx.notify();
    }

    pub fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.search_input.clear();
        self.update_search(String::new(), cx);
        self.mouse_moved = false;

        let service = self.service.clone();
        cx.spawn(async move |this, cx| {
            if service.refresh().await.is_ok() {
                let _ = this.update(cx, |this, cx| {
                    if this.query.is_empty() {
                        this.apps = this.service.search("");
                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    pub fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.count() && self.selected != index {
            self.selected = index;
            self.scroll.scroll_to_item(index / COLUMNS);
            cx.notify();
        }
    }

    pub fn navigate(&mut self, horizontal: i32, vertical: i32, cx: &mut Context<Self>) {
        self.mouse_moved = false;
        let next = next_selection(self.selected, self.count(), horizontal, vertical);
        self.select(next, cx);
    }

    pub fn activate_selected(&mut self, cx: &mut Context<Self>) {
        if self.selected == 0
            && let Some(res) = &self.calculator
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(res.clone()));
            self.close(cx);
            return;
        }

        let index = self
            .selected
            .saturating_sub(usize::from(self.calculator.is_some()));

        let Some(app) = self.apps.get(index).cloned() else {
            return;
        };

        if let Err(e) = app.launch() {
            services::log_error!("BarLauncher", "Failed to launch application: {:?}", e);
            self.launch_error = Some(if cx.has_global::<AppState>() {
                cx.global::<AppState>().language.get("launcher.launch_failed")
            } else {
                "Error al abrir la aplicación".to_string()
            });
            cx.notify();
            return;
        }
        self.close(cx);
    }

    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let control = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let shift = event.keystroke.modifiers.shift;

        if key == "escape" {
            if !self.search_input.is_empty() {
                self.reset_search(cx);
            } else {
                self.close(cx);
            }
            cx.stop_propagation();
            return;
        }

        if key == "f4" && event.keystroke.modifiers.alt {
            self.close(cx);
            cx.stop_propagation();
            return;
        }

        if key == "enter" || key == "return" {
            self.activate_selected(cx);
            cx.stop_propagation();
            return;
        }

        if (key == "up" || key == "down") && !shift && !control {
            self.navigate(0, if key == "up" { -1 } else { 1 }, cx);
            cx.stop_propagation();
            return;
        }

        if (key == "left" || key == "right") && !shift && !control && self.search_input.is_empty() {
            self.navigate(if key == "left" { -1 } else { 1 }, 0, cx);
            cx.stop_propagation();
            return;
        }

        if key == "tab" && !control {
            self.navigate(if shift { -1 } else { 1 }, 0, cx);
            cx.stop_propagation();
            return;
        }

        if self.search_input.handle_key_down(event, cx) {
            let text = self.search_input.text();
            self.update_search(text, cx);
            cx.stop_propagation();
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.handle_key_down(event, cx);
    }

    fn render_calculator_tile(
        &self,
        result: &str,
        is_selected: bool,
        duration: Duration,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let background = if is_selected {
            theme.primary_container()
        } else {
            theme.surface_container()
        };
        let border_color = if is_selected {
            theme.primary()
        } else {
            theme.outline_variant().opacity(0.35)
        };
        let hover_bg = if is_selected {
            theme.primary_hover()
        } else {
            theme.surface_container_highest()
        };

        div()
            .id("bar-launcher-calc-tile")
            .flex_1()
            .min_w_0()
            .h(px(TILE_HEIGHT))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10.0))
            .px(px(8.0))
            .rounded(px(16.0))
            .bg(background)
            .border_1()
            .border_color(border_color)
            .cursor_pointer()
            .transitions(|t| t.bg(duration.with_easing(gpui::ease_in_out)))
            .hover(move |s| s.bg(hover_bg))
            .on_hover(cx.listener(|this, &hovered, _, cx| {
                if hovered && this.mouse_moved {
                    this.select(0, cx);
                }
            }))
            .on_click(cx.listener(|this, _, _, cx| {
                this.select(0, cx);
                this.activate_selected(cx);
            }))
            .child(
                div()
                    .h(px(44.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(26.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.primary())
                    .child("="),
            )
            .child(
                div()
                    .w_full()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(if is_selected {
                        theme.on_primary_container()
                    } else {
                        theme.foreground()
                    })
                    .text_center()
                    .text_ellipsis()
                    .child(result.to_string()),
            )
            .into_any_element()
    }

    fn render_app_tile(
        &self,
        app: &Application,
        index: usize,
        is_selected: bool,
        duration: Duration,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let background = if is_selected {
            theme.primary_container()
        } else {
            theme.surface_container()
        };
        let border_color = if is_selected {
            theme.primary()
        } else {
            theme.outline_variant().opacity(0.35)
        };
        let hover_bg = if is_selected {
            theme.primary_hover()
        } else {
            theme.surface_container_highest()
        };

        let icon = match &app.icon_path {
            Some(path) => gpui::img(path.clone())
                .size(px(44.0))
                .rounded(px(10.0))
                .into_any_element(),
            None => svg()
                .path(if app.terminal {
                    "cpu.svg"
                } else {
                    "package.svg"
                })
                .size(px(40.0))
                .text_color(theme.primary())
                .into_any_element(),
        };

        div()
            .id(("bar-launcher-app-tile", index))
            .flex_1()
            .min_w_0()
            .h(px(TILE_HEIGHT))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(10.0))
            .px(px(8.0))
            .rounded(px(16.0))
            .bg(background)
            .border_1()
            .border_color(border_color)
            .cursor_pointer()
            .transitions(|t| t.bg(duration.with_easing(gpui::ease_in_out)))
            .hover(move |s| s.bg(hover_bg))
            .on_hover(cx.listener(move |this, &hovered, _, cx| {
                if hovered && this.mouse_moved {
                    this.select(index, cx);
                }
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.select(index, cx);
                this.activate_selected(cx);
            }))
            .child(icon)
            .child(
                div()
                    .w_full()
                    .text_size(px(12.0))
                    .font_weight(if is_selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_color(if is_selected {
                        theme.on_primary_container()
                    } else {
                        theme.foreground()
                    })
                    .text_center()
                    .text_ellipsis()
                    .child(app.name.clone()),
            )
            .into_any_element()
    }
}

impl Render for BarLauncher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(ref anim) = self.animator else {
            return div().into_any_element();
        };

        let theme = cx.global::<Theme>().clone();
        let placeholder = if cx.has_global::<AppState>() {
            cx.global::<AppState>()
                .language
                .get("launcher.search_placeholder")
        } else {
            "Buscar aplicaciones...".to_string()
        };

        let hints = if cx.has_global::<AppState>() {
            let state = cx.global::<AppState>();
            format!(
                "{}    {}    {}",
                state.language.get("launcher.grid_navigate_hint"),
                state.language.get("launcher.open_hint"),
                state.language.get("launcher.close_hint")
            )
        } else {
            "←↑↓→ navegar    ↵ abrir    esc cerrar".to_string()
        };

        let no_apps = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("launcher.no_apps")
        } else {
            "No se encontraron aplicaciones".to_string()
        };

        let anim_opacity = anim.opacity;
        let anim_offset_y = anim.offset_y;
        let blur_progress = (anim.backdrop_opacity / 0.40).clamp(0.0, 1.0);
        let duration = Duration::from_millis(200);

        let search_bar = Input::from_state("bar-launcher-search", &self.search_input)
            .placeholder(placeholder)
            .cleanable(true)
            .w_full()
            .h_px(44.0)
            .rounded_full()
            .bg(theme.surface_container())
            .border_color(theme.outline_variant().opacity(0.40))
            .focused_border_color(theme.primary())
            .prefix(
                svg()
                    .path("search.svg")
                    .size(px(17.0))
                    .text_color(theme.on_surface_variant()),
            )
            .on_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                let theme = cx.global::<Theme>().clone();
                this.search_input.handle_mouse_down(event, window, &theme);
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                if this.search_input.is_dragging {
                    let theme = cx.global::<Theme>().clone();
                    this.search_input.handle_mouse_move(event, window, &theme);
                    cx.notify();
                }
            }))
            .on_mouse_up(cx.listener(|this, event: &MouseUpEvent, _, cx| {
                this.search_input.handle_mouse_up(event);
                cx.notify();
            }))
            .on_clear(cx.listener(|this, _, _, cx| this.reset_search(cx)));

        let count = self.count();
        let content = if count == 0 {
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(12.0))
                .child(
                    svg()
                        .path("search.svg")
                        .size(px(36.0))
                        .text_color(theme.foreground_muted().opacity(0.40)),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.foreground_muted())
                        .child(no_apps),
                )
                .into_any_element()
        } else {
            let mut grid = div()
                .id("bar-launcher-grid")
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(GRID_GAP))
                .overflow_y_scroll()
                .track_scroll(&self.scroll);

            let row_count = count.div_ceil(COLUMNS);
            for row_index in 0..row_count {
                let mut row = div()
                    .id(("bar-launcher-grid-row", row_index))
                    .w_full()
                    .h(px(TILE_HEIGHT))
                    .flex_shrink_0()
                    .flex()
                    .gap(px(GRID_GAP));

                for col in 0..COLUMNS {
                    let index = row_index * COLUMNS + col;
                    if index >= count {
                        row = row.child(div().flex_1().min_w_0());
                    } else if index == 0 && let Some(result) = &self.calculator {
                        row = row.child(self.render_calculator_tile(
                            result,
                            self.selected == index,
                            duration,
                            &theme,
                            cx,
                        ));
                    } else {
                        let app_index = index - usize::from(self.calculator.is_some());
                        row = row.child(self.render_app_tile(
                            &self.apps[app_index],
                            index,
                            self.selected == index,
                            duration,
                            &theme,
                            cx,
                        ));
                    }
                }
                grid = grid.child(row);
            }
            grid.into_any_element()
        };

        let footer = div()
            .id("bar-launcher-footer")
            .h(px(20.0))
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(if self.launch_error.is_some() {
                        theme.red()
                    } else {
                        theme.foreground_muted()
                    })
                    .child(self.launch_error.clone().unwrap_or(hints)),
            )
            .child(
                div()
                    .text_size(px(10.5))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child(if self.apps.len() == 1 {
                        "1 aplicación".to_string()
                    } else {
                        format!("{} aplicaciones", self.apps.len())
                    }),
            );

        let dialog_box = div()
            .id("bar-launcher-dialog")
            .w(px(LAUNCHER_WIDTH))
            .h(px(LAUNCHER_HEIGHT))
            .rounded(px(24.0))
            .bg(theme.surface_container_high())
            .border_1()
            .border_color(theme.outline_variant().opacity(0.40))
            .shadow(vec![
                BoxShadow {
                    color: gpui::black().opacity(0.35).into(),
                    offset: point(px(0.0), px(20.0)),
                    blur_radius: px(48.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                BoxShadow {
                    color: gpui::black().opacity(0.15).into(),
                    offset: point(px(0.0), px(4.0)),
                    blur_radius: px(12.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
            ])
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(12.0))
            .font_family(theme.font_family())
            .overflow_hidden()
            .opacity(anim_opacity)
            .relative()
            .top(px(anim_offset_y))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down(gpui::MouseButton::Right, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation())
            .on_mouse_move(cx.listener(|this, _, _, _| {
                this.mouse_moved = true;
            }))
            .child(search_bar)
            .child(content)
            .child(footer);

        div()
            .id("bar-launcher-backdrop")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .absolute()
            .inset_0()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .id("bar-launcher-backdrop-dim")
                    .absolute()
                    .inset_0()
                    .backdrop_blur(px(16.0 * blur_progress))
                    .bg(gpui::black().opacity(0.35 * blur_progress))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.close(cx);
                        }),
                    ),
            )
            .child(dialog_box)
            .into_any_element()
    }
}
