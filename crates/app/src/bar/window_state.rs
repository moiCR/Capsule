use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use gpui::{
    AnyWindowHandle, App, Bounds, Entity, FocusHandle, Pixels, WeakEntity, Window,
    layer_shell::KeyboardInteractivity, point, px, size,
};

use crate::bar::{
    Bar, popups::launcher::BarLauncher, popups::polkit::BarPolkit, shared_panel::SharedPanelManager,
};

pub struct WindowState {
    pub handle: AnyWindowHandle,
    pub height: Pixels,
    pub screen_width: Pixels,
    pub screen_height: Pixels,
    pub animation_frame_pending: Rc<Cell<bool>>,
    last_input_regions: Option<Vec<Bounds<Pixels>>>,
    last_keyboard: Option<KeyboardInteractivity>,
}

impl WindowState {
    pub fn new(handle: AnyWindowHandle, height: Pixels) -> Self {
        Self {
            handle,
            height,
            screen_width: px(1920.0),
            screen_height: px(1080.0),
            animation_frame_pending: Rc::new(Cell::new(false)),
            last_input_regions: None,
            last_keyboard: None,
        }
    }

    pub fn sync_display_bounds(&mut self, window: &Window, cx: &App) {
        if let Some(display) = window.display(cx) {
            let bounds = display.bounds();
            self.screen_width = bounds.size.width;
            self.screen_height = bounds.size.height;
        }
    }

    pub fn needs_exclusive_keyboard(
        &self,
        shared_panel: &SharedPanelManager,
        launcher: &Entity<BarLauncher>,
        polkit: &Entity<BarPolkit>,
        cx: &App,
    ) -> bool {
        if launcher.read(cx).is_open() || polkit.read(cx).is_open() {
            return true;
        }
        if !shared_panel.is_open() {
            return false;
        }
        shared_panel.needs_exclusive_keyboard(cx)
    }

    pub fn sync_window(
        &mut self,
        shared_panel: &SharedPanelManager,
        launcher: &Entity<BarLauncher>,
        polkit: &Entity<BarPolkit>,
        focus_handle: &FocusHandle,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.sync_display_bounds(window, cx);
        let viewport = window.viewport_size();
        let width = if viewport.width > px(0.0) {
            viewport.width
        } else {
            self.screen_width
        };
        let height = if viewport.height > px(0.0) {
            viewport.height
        } else {
            self.screen_height
        };

        let is_popup_open = launcher.read(cx).is_open() || polkit.read(cx).is_open();
        let has_popup = launcher.read(cx).is_visible() || polkit.read(cx).is_visible();
        let is_shared_open = shared_panel.is_open();
        let has_panel = is_shared_open || shared_panel.animator.is_some();

        let regions = if is_popup_open || has_popup || has_panel {
            vec![Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(width, height),
            }]
        } else {
            vec![Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(width, self.height),
            }]
        };

        if self.last_input_regions.as_ref() != Some(&regions) {
            self.last_input_regions = Some(regions.clone());
            window.set_input_region(Some(&regions));
        }

        let desired_keyboard = if self.needs_exclusive_keyboard(shared_panel, launcher, polkit, cx)
        {
            KeyboardInteractivity::Exclusive
        } else {
            KeyboardInteractivity::None
        };

        if self.last_keyboard != Some(desired_keyboard) {
            self.last_keyboard = Some(desired_keyboard);
            window.set_keyboard_interactivity(desired_keyboard);
            if desired_keyboard == KeyboardInteractivity::Exclusive {
                window.focus(focus_handle, cx);
            } else {
                window.blur(cx);
            }
        }
    }

    pub fn queue_animation_frame_checked(
        window: &Window,
        entity: WeakEntity<Bar>,
        pending: &Rc<Cell<bool>>,
    ) {
        if pending.get() {
            return;
        }
        pending.set(true);
        Self::schedule_animation_tick(window, entity);
    }

    pub fn queue_animation_frame(
        window: &Window,
        entity: WeakEntity<Bar>,
        pending: &Rc<Cell<bool>>,
    ) {
        let pending = pending.clone();
        window.on_next_frame(move |window, cx| {
            if pending.get() {
                return;
            }
            Self::run_animation_tick(window, entity, cx);
        });
    }

    pub fn schedule_animation_tick(window: &Window, entity: WeakEntity<Bar>) {
        window.on_next_frame(move |window, cx| {
            Self::run_animation_tick(window, entity, cx);
        });
    }

    pub fn run_animation_tick(window: &mut Window, entity: WeakEntity<Bar>, cx: &mut App) {
        let entity_clone = entity.clone();
        let _ = entity.update(cx, |this, cx| {
            this.window_state.animation_frame_pending.set(false);
            let now = Instant::now();
            let mut needs_frame = false;

            let (sp_running, sp_closed) = this.shared_panel.advance(now);
            if sp_closed && this.shared_panel.is_open() {
                this.shared_panel.request_close(cx);
            }
            if sp_running {
                needs_frame = true;
            }

            let (l_running, l_closed) = this.launcher.update(cx, |l, _| l.advance(now));
            if l_closed {
                this.launcher.update(cx, |l, _| l.is_open = false);
            }
            if l_running {
                needs_frame = true;
            }

            let (p_running, p_closed) = this.polkit.update(cx, |p, _| p.advance(now));
            if p_closed {
                this.polkit.update(cx, |p, _| p.is_open = false);
            }
            if p_running {
                needs_frame = true;
            }

            if sp_closed || l_closed || p_closed || !needs_frame {
                this.sync_window_state(window, cx);
            }

            this.window_state.animation_frame_pending.set(needs_frame);
            cx.notify();
            if needs_frame {
                Self::schedule_animation_tick(window, entity_clone);
            }
        });
    }
}
