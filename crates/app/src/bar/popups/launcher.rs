use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    AnyElement, BoxShadow, ColorExt, Context, EventEmitter, FocusHandle, FontWeight, IntoElement,
    KeyDownEvent, Render, ScrollHandle, Size, Task, Window, div, point, px, svg,
};
use services::{AppState, Application, CapsuleAnimation, LauncherService};
use ui::components::{Input, InputState};
use ui::theme::Theme;

use super::{PopupEvent, animator::LauncherAnimator};
use crate::bar::style;

pub struct BarLauncher {
    pub search_input: InputState,
    pub query: String,
    pub apps: Vec<Application>,
    pub calculator: Option<String>,
    pub selected: usize,
    pub show_all: bool,
    pub scroll: ScrollHandle,
    pub service: LauncherService,
    pub animator: Option<LauncherAnimator>,
    pub is_open: bool,
    pub focus: FocusHandle,
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
                    if !launcher.query.is_empty() || launcher.show_all {
                        launcher.apps = launcher.service.search(&launcher.query);
                        cx.notify();
                    }
                });
            }
        });

        Self {
            search_input: InputState::new("").with_focused(true).with_max_len(120),
            query: String::new(),
            apps: Vec::new(),
            calculator: None,
            selected: 0,
            show_all: false,
            scroll: ScrollHandle::new(),
            service,
            animator: None,
            is_open: false,
            focus: cx.focus_handle(),
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
                    Duration::from_millis(cfg.ui.animation_duration_ms as u64),
                    cfg.ui.capsule_animation,
                    cfg.ui.spring_bounciness,
                    cfg.ui.squash_stretch,
                    cfg.ui.reduced_motion,
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
        self.apps.clear();
        self.calculator = None;
        self.selected = 0;
        self.show_all = false;
        self.scroll.scroll_to_item(0);

        let mut anim = LauncherAnimator::new(
            duration,
            animation,
            bounciness,
            squash_stretch,
            reduced_motion,
        );
        let target_size = Size {
            width: px(640.0),
            height: px(54.0),
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
        if query.is_empty() {
            self.show_all = false;
        }
        let is_expanded = !query.is_empty() || self.show_all;
        if !is_expanded {
            self.apps.clear();
            self.calculator = None;
        } else {
            self.calculator = services::launcher::calculator::evaluate(&query);
            self.apps = self.service.search(&query);
        }
        self.selected = 0;
        self.scroll.scroll_to_item(0);
        let target_size = if is_expanded {
            let count = self.count();
            let target_h = match count {
                0 => 230.0,
                n => (54.0 + 1.0 + 16.0 + n as f32 * 50.0 + (n.saturating_sub(1) as f32) * 2.0)
                    .clamp(121.0, 480.0),
            };
            Size {
                width: px(640.0),
                height: px(target_h),
            }
        } else {
            Size {
                width: px(640.0),
                height: px(54.0),
            }
        };
        if let Some(ref mut anim) = self.animator {
            anim.transition_to(target_size, Instant::now());
            cx.emit(PopupEvent::RequestAnimationFrame);
        }
        cx.notify();
    }

    pub fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.search_input.clear();
        self.show_all = false;
        self.update_search(String::new(), cx);
    }

    pub fn navigate(&mut self, delta: i32, cx: &mut Context<Self>) {
        if self.query.is_empty() && !self.show_all {
            self.show_all = true;
            self.apps = self.service.search("");
            self.selected = 0;
            let target_h = match self.count() {
                0 => 230.0,
                n => (54.0 + 1.0 + 16.0 + n as f32 * 50.0 + (n.saturating_sub(1) as f32) * 2.0)
                    .clamp(121.0, 480.0),
            };
            let target_size = Size {
                width: px(640.0),
                height: px(target_h),
            };
            if let Some(ref mut anim) = self.animator {
                anim.transition_to(target_size, Instant::now());
                cx.emit(PopupEvent::RequestAnimationFrame);
            }
            cx.notify();
            return;
        }
        let count = self.count();
        if count == 0 {
            return;
        }
        let current = self.selected as i32;
        let next = (current + delta).clamp(0, count as i32 - 1) as usize;
        self.selected = next;
        self.scroll.scroll_to_item(next);
        cx.notify();
    }

    pub fn activate_selected(&mut self, cx: &mut Context<Self>) {
        if self.selected == 0 && self.calculator.is_some() {
            if let Some(res) = &self.calculator {
                cx.write_to_clipboard(gpui::ClipboardItem::new_string(res.clone()));
            }
            self.close(cx);
            return;
        }

        let index = self
            .selected
            .saturating_sub(usize::from(self.calculator.is_some()));

        if let Some(app) = self.apps.get(index).cloned() {
            let _ = app.launch();
            self.close(cx);
        }
    }

    pub fn activate_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = index;
        self.activate_selected(cx);
    }

    fn key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();

        if key == "escape" || (key == "f4" && event.keystroke.modifiers.alt) {
            self.close(cx);
            cx.stop_propagation();
            return;
        }

        if key == "enter" || key == "return" {
            self.activate_selected(cx);
            cx.stop_propagation();
            return;
        }

        if key == "down" {
            self.navigate(1, cx);
            cx.stop_propagation();
            return;
        }

        if key == "up" {
            self.navigate(-1, cx);
            cx.stop_propagation();
            return;
        }

        if self.search_input.handle_key_down(event, cx) {
            let text = self.search_input.text();
            self.update_search(text, cx);
            cx.stop_propagation();
        }
    }

    fn render_calc_row(
        &self,
        result: &str,
        is_selected: bool,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (row_bg, border_color) = if is_selected {
            (
                style::surface(theme).opacity(0.85),
                style::border(theme).opacity(0.18),
            )
        } else {
            (gpui::transparent_black(), gpui::transparent_black())
        };

        let copy_hint = if cx.has_global::<AppState>() {
            cx.global::<AppState>().language.get("launcher.copy_hint")
        } else {
            "Enter para copiar".to_string()
        };

        div()
            .id("spotlight-calc-row")
            .w_full()
            .h(px(50.0))
            .flex_shrink_0()
            .px(px(8.0))
            .rounded(px(10.0))
            .bg(row_bg)
            .border_1()
            .border_color(border_color)
            .hover(move |s| {
                if !is_selected {
                    s.bg(style::surface(theme).opacity(0.35))
                } else {
                    s
                }
            })
            .flex()
            .items_center()
            .gap(px(10.0))
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.activate_item(0, cx);
            }))
            .child(
                div()
                    .w(px(3.5))
                    .h(px(20.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(if is_selected {
                        theme.accent()
                    } else {
                        gpui::transparent_black()
                    }),
            )
            .child(
                div()
                    .size(px(34.0))
                    .flex_shrink_0()
                    .rounded(px(8.0))
                    .bg(style::background(theme).opacity(0.55))
                    .border_1()
                    .border_color(style::border(theme).opacity(0.15))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .path("calculator.svg")
                            .size(px(18.0))
                            .text_color(theme.accent()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(1.5))
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.foreground())
                            .child(format!("= {result}")),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(theme.foreground_muted())
                            .child(copy_hint),
                    ),
            )
            .into_any_element()
    }

    fn render_app_row(
        &self,
        app: &Application,
        index: usize,
        is_selected: bool,
        theme: &Theme,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let (row_bg, border_color) = if is_selected {
            (
                style::surface(theme).opacity(0.85),
                style::border(theme).opacity(0.18),
            )
        } else {
            (gpui::transparent_black(), gpui::transparent_black())
        };

        let icon_inner = if let Some(icon_path) = &app.icon_path {
            gpui::img(icon_path.clone())
                .size(px(24.0))
                .rounded(px(5.0))
                .into_any_element()
        } else {
            svg()
                .path(if app.terminal {
                    "cpu.svg"
                } else {
                    "package.svg"
                })
                .size(px(18.0))
                .text_color(theme.foreground_muted())
                .into_any_element()
        };

        let icon_element = div()
            .size(px(34.0))
            .flex_shrink_0()
            .rounded(px(8.0))
            .bg(style::background(theme).opacity(0.55))
            .border_1()
            .border_color(style::border(theme).opacity(0.15))
            .flex()
            .items_center()
            .justify_center()
            .child(icon_inner);

        let name = app.name.clone();
        let comment = app.generic_name.as_ref().or(app.comment.as_ref()).cloned();

        div()
            .id(("spotlight-app-row", index))
            .w_full()
            .h(px(50.0))
            .flex_shrink_0()
            .px(px(8.0))
            .rounded(px(10.0))
            .bg(row_bg)
            .border_1()
            .border_color(border_color)
            .hover(move |s| {
                if !is_selected {
                    s.bg(style::surface(theme).opacity(0.35))
                } else {
                    s
                }
            })
            .flex()
            .items_center()
            .gap(px(10.0))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, _, cx| {
                this.activate_item(index, cx);
            }))
            .child(
                div()
                    .w(px(3.5))
                    .h(px(20.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(if is_selected {
                        theme.accent()
                    } else {
                        gpui::transparent_black()
                    }),
            )
            .child(icon_element)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(1.5))
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .line_clamp(1)
                            .child(name),
                    )
                    .when_some(comment, |el, c| {
                        el.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(theme.foreground_muted())
                                .line_clamp(1)
                                .child(c),
                        )
                    }),
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
                .get("launcher.spotlight_placeholder")
        } else {
            "Buscar aplicaciones o calcular...".to_string()
        };

        let size = anim.size();
        let corner_radius = anim.corner_radius();
        let cascade_opacity = anim.cascade_opacity();
        let anim_opacity = anim.opacity;
        let anim_offset_y = anim.offset_y;
        let expand_progress = anim.expand_progress();
        let blur_progress = (anim.backdrop_opacity / 0.40).clamp(0.0, 1.0);

        let mut spotlight_box = div()
            .id("spotlight-container")
            .w(size.width)
            .h(size.height)
            .rounded(corner_radius)
            .bg(style::background(&theme).opacity(0.92))
            .border_1()
            .border_color(style::border(&theme).opacity(0.18))
            .shadow(vec![BoxShadow {
                color: gpui::black().opacity(0.40).into(),
                offset: point(px(0.0), px(16.0)),
                blur_radius: px(36.0),
                spread_radius: px(0.0),
                inset: false,
            }])
            .flex()
            .flex_col()
            .overflow_hidden()
            .opacity(anim_opacity)
            .relative()
            .top(px(anim_offset_y))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down(gpui::MouseButton::Right, |_, _, cx| cx.stop_propagation())
            .on_click(|_, _, cx| cx.stop_propagation());

        let input_component = div()
            .id("spotlight-search-header")
            .w_full()
            .h(px(54.0))
            .flex_shrink_0()
            .px(px(16.0))
            .flex()
            .items_center()
            .child(
                Input::from_state("spotlight-input", &self.search_input)
                    .placeholder(placeholder)
                    .prefix(
                        svg()
                            .path("search.svg")
                            .size(px(18.0))
                            .text_color(theme.foreground_muted())
                            .into_any_element(),
                    )
                    .padding_x(px(0.0))
                    .gap(px(10.0))
                    .flex_1(true)
                    .h_px(44.0)
                    .text_size(px(15.5))
                    .bg(gpui::transparent_black())
                    .border_color(gpui::transparent_black())
                    .focused_border_color(gpui::transparent_black()),
            );

        spotlight_box = spotlight_box.child(input_component);

        if expand_progress > 0.05 {
            let divider = div()
                .id("spotlight-divider")
                .w_full()
                .h(px(1.0))
                .flex_shrink_0()
                .bg(style::border(&theme).opacity(0.16));

            spotlight_box = spotlight_box.child(divider);

            let mut cascade_list = div()
                .id("spotlight-cascade-list")
                .flex_1()
                .min_h_0()
                .w_full()
                .p(px(8.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .opacity(cascade_opacity);

            if let Some(calc_res) = self.calculator.clone() {
                let is_selected = self.selected == 0;
                let calc_row = self.render_calc_row(&calc_res, is_selected, &theme, cx);
                cascade_list = cascade_list.child(calc_row);
            }

            let calc_offset = usize::from(self.calculator.is_some());

            if self.apps.is_empty() && self.calculator.is_none() {
                let empty_text = if cx.has_global::<AppState>() {
                    cx.global::<AppState>().language.get("launcher.no_apps")
                } else {
                    "No se encontraron aplicaciones".to_string()
                };

                cascade_list = cascade_list.child(
                    div()
                        .w_full()
                        .flex_1()
                        .h_full()
                        .py(px(24.0))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(10.0))
                        .child(
                            svg()
                                .path("search.svg")
                                .size(px(28.0))
                                .text_color(theme.foreground_muted().opacity(0.35)),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground_muted())
                                .child(empty_text),
                        ),
                );
            } else {
                let apps = self.apps.clone();
                for (index, app) in apps.iter().enumerate() {
                    let global_index = index + calc_offset;
                    let is_selected = self.selected == global_index;
                    cascade_list = cascade_list.child(self.render_app_row(
                        app,
                        global_index,
                        is_selected,
                        &theme,
                        cx,
                    ));
                }
            }

            spotlight_box = spotlight_box.child(cascade_list);
        }

        div()
            .id("spotlight-backdrop")
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
                    .id("spotlight-backdrop-click")
                    .absolute()
                    .inset_0()
                    .backdrop_blur(px(20.0 * blur_progress))
                    .bg(gpui::black().opacity(0.20 * blur_progress))
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.close(cx);
                        }),
                    ),
            )
            .child(spotlight_box)
            .into_any_element()
    }
}
