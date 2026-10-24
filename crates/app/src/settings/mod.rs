pub(crate) mod animator;
pub(crate) mod appearance;
pub(crate) mod header;
pub(crate) mod input;
pub(crate) mod layout;
pub(crate) mod sidebar;
pub mod views;

use std::time::{Duration, Instant};

use gpui::{Context, FocusHandle, IntoElement, Render, ScrollHandle, Window};
use ui::components::InputState;

use crate::panel::SettingsPanel;
use animator::SettingsAnimator;
use sidebar::{SettingsTab, Sidebar};
use views::{
    SettingsView, appearance::AppearanceView, apps::AppsView, bar::BarView, island::IslandView,
    lockscreen::LockScreenView, media::MediaView, record::RecordView, system::SystemView,
};

pub const WIDTH: f32 = 1120.0;
pub const HEIGHT: f32 = 780.0;

pub struct Settings {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) animator: SettingsAnimator,
    pub(crate) handle: gpui::AnyWindowHandle,
    pub(crate) frame_pending: bool,
    pub(crate) active_tab: SettingsTab,
    pub(crate) sidebar: Sidebar,
    pub(crate) views: Vec<(SettingsTab, Box<dyn SettingsView>)>,
    pub search_input: InputState,
    pub(crate) scroll_handle: ScrollHandle,
}

impl Settings {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let animation_duration_ms = if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .config
                .get()
                .ui
                .animation_duration_ms as u64
        } else {
            250
        };

        let views: Vec<(SettingsTab, Box<dyn SettingsView>)> = vec![
            (SettingsTab::Appearance, Box::new(AppearanceView::new())),
            (SettingsTab::Island, Box::new(IslandView::new())),
            (SettingsTab::Bar, Box::new(BarView::new())),
            (SettingsTab::Apps, Box::new(AppsView::new())),
            (SettingsTab::Media, Box::new(MediaView::new())),
            (SettingsTab::LockScreen, Box::new(LockScreenView::new())),
            (SettingsTab::System, Box::new(SystemView::new())),
            (SettingsTab::Record, Box::new(RecordView::new())),
        ];

        let settings = Self {
            focus_handle,
            animator: SettingsAnimator::new(animation_duration_ms),
            handle: window.window_handle(),
            frame_pending: true,
            active_tab: SettingsTab::Appearance,
            sidebar: Sidebar::new(),
            views,
            search_input: InputState::new("").with_max_len(120),
            scroll_handle: ScrollHandle::new(),
        };

        window.focus(&settings.focus_handle, cx);
        Self::queue_frame(window, cx.entity().downgrade());
        settings
    }

    pub fn search_query(&self) -> String {
        self.search_input.text()
    }

    pub fn is_searching(&self) -> bool {
        self.search_input.focused
    }

    pub fn view_as_mut<T: 'static>(&mut self) -> Option<&mut T> {
        for (_, view) in &mut self.views {
            if let Some(v) = view.as_any_mut().downcast_mut::<T>() {
                return Some(v);
            }
        }
        None
    }

    pub fn request_close(&mut self, cx: &mut Context<Self>) {
        if self.animator.is_closing() {
            return;
        }
        self.animator.request_close();
        if !self.frame_pending {
            self.frame_pending = true;
            let entity = cx.entity().downgrade();
            let handle = self.handle;
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_, window, _| Self::queue_frame(window, entity));
            });
        }
        cx.notify();
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        if self.active_tab != tab {
            self.active_tab = tab;
            self.search_input.clear();
            self.search_input.focused = false;
            self.scroll_handle.scroll_to_item(0);

            let duration = if cx.has_global::<services::AppState>() {
                Duration::from_millis(
                    cx.global::<services::AppState>()
                        .config
                        .get()
                        .ui
                        .animation_duration_ms as u64,
                )
            } else {
                Duration::from_millis(200)
            };
            self.animator.start_section_transition(duration);
            if !self.frame_pending {
                self.frame_pending = true;
                let entity = cx.entity().downgrade();
                let handle = self.handle;
                cx.defer(move |cx| {
                    let _ = handle.update(cx, |_, window, _| Self::queue_frame(window, entity));
                });
            }
            cx.notify();
        }
    }

    pub(crate) fn queue_frame(window: &Window, entity: gpui::WeakEntity<Self>) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |this, cx| {
                let (running, closed) = this.animator.advance(Instant::now());
                if closed {
                    window.remove_window();
                    return;
                }
                this.frame_pending = running;
                cx.notify();
                if running {
                    Self::queue_frame(window, cx.entity().downgrade());
                }
            });
        });
    }
}

impl Drop for Settings {
    fn drop(&mut self) {
        SettingsPanel::mark_closed();
    }
}

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        layout::render_settings(self, window, cx)
    }
}
