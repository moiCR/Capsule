mod input;

use std::time::Duration;

use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, Pixels, Render, ScrollHandle, Size,
    Subscription, Window, div, prelude::*, px,
};
use services::{AppState, Application, LauncherService};
use ui::theme::Theme;

use super::{CapsuleModule, CapsuleModuleEvent};
use crate::new_capsule::widgets::launcher::{
    COLUMNS, GRID_GAP, HEIGHT, TILE_HEIGHT, WIDTH, next_selection, render_app_tile,
    render_calculator_tile, render_search,
};

pub struct LauncherModule {
    service: LauncherService,
    query: String,
    apps: Vec<Application>,
    calculator: Option<String>,
    selected: usize,
    pub(crate) mouse_moved: bool,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
    launch_error: Option<String>,
    _theme_subscription: Subscription,
}

impl LauncherModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let service = cx.global::<AppState>().launcher.clone();
        let apps = service.search("");
        Self {
            service,
            query: String::new(),
            apps,
            calculator: None,
            selected: 0,
            mouse_moved: false,
            focus_handle: cx.focus_handle(),
            scroll: ScrollHandle::new(),
            launch_error: None,
            _theme_subscription: cx.observe_global::<Theme>(|_, cx| cx.notify()),
        }
    }

    pub(crate) fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub(crate) fn reset_search(&mut self, cx: &mut Context<Self>) {
        self.update_search(String::new(), cx);
        self.mouse_moved = false;
    }

    fn update_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.calculator = services::launcher::calculator::evaluate(&query);
        self.apps = self.service.search(&query);
        self.query = query;
        self.selected = 0;
        self.launch_error = None;
        self.scroll.scroll_to_item(0);
        cx.notify();
    }

    fn count(&self) -> usize {
        self.apps.len() + usize::from(self.calculator.is_some())
    }

    pub(crate) fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.count() && self.selected != index {
            self.selected = index;
            self.scroll.scroll_to_item(index / COLUMNS);
            cx.notify();
        }
    }

    fn navigate(&mut self, horizontal: i32, vertical: i32, cx: &mut Context<Self>) {
        self.mouse_moved = false;
        let next = next_selection(self.selected, self.count(), horizontal, vertical);
        self.select(next, cx);
    }

    pub(crate) fn activate_selected(&mut self, cx: &mut Context<Self>) {
        if self.selected == 0
            && let Some(result) = &self.calculator
        {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(result.clone()));
            cx.emit(CapsuleModuleEvent::Close);
            return;
        }
        let index = self
            .selected
            .saturating_sub(usize::from(self.calculator.is_some()));
        let Some(app) = self.apps.get(index).cloned() else {
            return;
        };
        if let Err(e) = app.launch() {
            services::log_error!("Launcher", "Failed to launch application: {:?}", e);
            self.launch_error = Some(
                cx.global::<AppState>()
                    .language
                    .get("launcher.launch_failed"),
            );
            cx.notify();
            return;
        }
        cx.emit(CapsuleModuleEvent::Close);
    }
}

impl CapsuleModule for LauncherModule {
    fn size(&self) -> Size<Pixels> {
        gpui::size(px(WIDTH), px(HEIGHT))
    }
}

impl EventEmitter<CapsuleModuleEvent> for LauncherModule {}

impl Render for LauncherModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let state = cx.global::<AppState>();
        let duration = Duration::from_millis(state.config.get().ui.animation_duration_ms as u64);
        let no_apps = state.language.get("launcher.no_apps");
        let hints = format!(
            "{}    {}    {}",
            state.language.get("launcher.grid_navigate_hint"),
            state.language.get("launcher.open_hint"),
            state.language.get("launcher.close_hint")
        );
        let count = self.count();
        let mut grid = div()
            .id("launcher-grid")
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(GRID_GAP))
            .overflow_y_scroll()
            .track_scroll(&self.scroll);
        for row_index in 0..count.div_ceil(COLUMNS) {
            let mut row = div()
                .id(("launcher-grid-row", row_index))
                .w_full()
                .h(px(TILE_HEIGHT))
                .flex_shrink_0()
                .flex()
                .gap(px(GRID_GAP));
            for column in 0..COLUMNS {
                let index = row_index * COLUMNS + column;
                if index >= count {
                    row = row.child(div().flex_1().min_w_0());
                } else if index == 0
                    && let Some(result) = &self.calculator
                {
                    row = row.child(render_calculator_tile(
                        result,
                        self.selected == index,
                        duration,
                        &theme,
                        cx,
                    ));
                } else {
                    let app_index = index - usize::from(self.calculator.is_some());
                    row = row.child(render_app_tile(
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
        let content = if count == 0 {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(13.0))
                .text_color(theme.foreground_muted())
                .child(no_apps)
                .into_any_element()
        } else {
            grid.into_any_element()
        };
        div()
            .id("launcher-module")
            .size_full()
            .flex()
            .flex_col()
            .p(px(16.0))
            .gap(px(12.0))
            .font_family(theme.font_family())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(cx.listener(|this, _, _, _| {
                this.mouse_moved = true;
            }))
            .child(render_search(&self.query, duration, &theme, cx))
            .child(content)
            .child(
                div()
                    .h(px(20.0))
                    .flex_shrink_0()
                    .text_size(px(10.0))
                    .text_color(if self.launch_error.is_some() {
                        theme.red()
                    } else {
                        theme.foreground_muted()
                    })
                    .child(self.launch_error.clone().unwrap_or(hints)),
            )
    }
}
