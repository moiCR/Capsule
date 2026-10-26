pub mod actions;
pub mod home;
pub mod input;
pub mod subviews;

use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    App, BoxShadow, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, MouseButton,
    MouseMoveEvent, Pixels, Render, Size, Task, Window, div, point, px,
};
use services::AppState;
use ui::components::InputState;
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardSubView {
    #[default]
    Home,
    Wifi,
    Bluetooth,
    AudioOutput,
    AudioInput,
}

pub struct DashboardModule {
    pub view: DashboardSubView,
    pub overlay_view: Option<DashboardSubView>,
    pub overlay_progress: f32,
    pub overlay_anim_start: Option<Instant>,
    pub overlay_closing: bool,
    pub overlay_task: Option<Task<()>>,
    pub selected_ssid: Option<String>,
    pub password_input: InputState,
    pub volume_bounds: DimensionTracker,
    pub input_volume_bounds: DimensionTracker,
    pub brightness_bounds: DimensionTracker,
    pub brightness_preview: Option<u32>,
    pub dragging_volume: bool,
    pub dragging_input_volume: bool,
    pub dragging_brightness: bool,
    focus: FocusHandle,
}

pub type DashboardState = DashboardModule;

impl DashboardModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            view: DashboardSubView::Home,
            overlay_view: None,
            overlay_progress: 0.0,
            overlay_anim_start: None,
            overlay_closing: false,
            overlay_task: None,
            selected_ssid: None,
            password_input: InputState::new("").with_max_len(128),
            volume_bounds: DimensionTracker::new(),
            input_volume_bounds: DimensionTracker::new(),
            brightness_bounds: DimensionTracker::new(),
            brightness_preview: None,
            dragging_volume: false,
            dragging_input_volume: false,
            dragging_brightness: false,
            focus: cx.focus_handle(),
        }
    }

    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.view = DashboardSubView::Home;
        self.overlay_view = None;
        self.overlay_progress = 0.0;
        self.overlay_anim_start = None;
        self.overlay_closing = false;
        self.overlay_task = None;
        self.selected_ssid = None;
        self.password_input.clear();
        self.dragging_volume = false;
        self.dragging_input_volume = false;
        self.dragging_brightness = false;
        self.brightness_preview = None;
        cx.notify();
    }

    pub fn advance(&mut self, now: Instant) -> bool {
        let Some(start) = self.overlay_anim_start else {
            return false;
        };
        let duration = Duration::from_millis(180);
        let raw =
            (now.duration_since(start).as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - raw).powi(3);

        if self.overlay_closing {
            self.overlay_progress = (1.0 - eased).clamp(0.0, 1.0);
            if raw >= 1.0 {
                self.overlay_progress = 0.0;
                self.overlay_view = None;
                self.overlay_anim_start = None;
                self.overlay_closing = false;
                return false;
            }
            true
        } else {
            self.overlay_progress = eased.clamp(0.0, 1.0);
            if raw >= 1.0 {
                self.overlay_progress = 1.0;
                self.overlay_anim_start = None;
                return false;
            }
            true
        }
    }
}

pub fn slider_fraction(x: f32, left: f32, width: f32) -> Option<f32> {
    (width > 0.0).then(|| ((x - left) / width).clamp(0.0, 1.0))
}

impl EventEmitter<SharedPanelEvent> for DashboardModule {}

impl SharedPanelItem for DashboardModule {
    fn size(&self, cx: &App) -> Size<Pixels> {
        let has_battery = cx
            .try_global::<AppState>()
            .and_then(|s| s.power.get_battery())
            .is_some();
        Size {
            width: px(440.0),
            height: if has_battery { px(414.0) } else { px(346.0) },
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }

    fn needs_exclusive_keyboard(&self) -> bool {
        self.selected_ssid.is_some()
    }
}

impl Render for DashboardModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let home = home::render_home(self, &theme, cx);

        let mut root = div()
            .id("shared-panel-dashboard")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _, cx| {
                if this.dragging_volume {
                    this.volume_at(f32::from(event.position.x), cx);
                }
                if this.dragging_input_volume {
                    this.input_volume_at(f32::from(event.position.x), cx);
                }
                if this.dragging_brightness {
                    this.brightness_at(f32::from(event.position.x), cx);
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.finish_sliders(cx);
                }),
            )
            .on_mouse_exit(cx.listener(|this, _, _, cx| {
                this.finish_sliders(cx);
            }))
            .size_full()
            .relative()
            .p(px(16.0))
            .font_family(theme.font_family())
            .text_color(theme.foreground())
            .child(home);

        if let Some(subview) = self.overlay_view {
            let progress = self.overlay_progress.clamp(0.0, 1.0);
            let offset_y = (1.0 - progress) * 18.0;
            let sub_content = match subview {
                DashboardSubView::Wifi => subviews::render_wifi(self, &theme, cx),
                DashboardSubView::Bluetooth => subviews::render_bluetooth(&theme, cx),
                DashboardSubView::AudioOutput => subviews::render_audio_output(&theme, cx),
                DashboardSubView::AudioInput => subviews::render_audio_input(&theme, cx),
                DashboardSubView::Home => div().into_any_element(),
            };

            let overlay = div()
                .id("bar-dash-subview-backdrop")
                .absolute()
                .inset_0()
                .backdrop_blur(px(16.0 * progress))
                .bg(gpui::black().opacity(0.25 * progress))
                .p(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.close_subview(cx);
                    }),
                )
                .child(
                    div()
                        .id("bar-dash-subview-card")
                        .w_full()
                        .h_full()
                        .relative()
                        .top(px(offset_y))
                        .opacity(progress)
                        .rounded(px(16.0))
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
                        .p(px(14.0))
                        .overflow_hidden()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                            cx.stop_propagation();
                        })
                        .on_click(|_, _, cx| {
                            cx.stop_propagation();
                        })
                        .child(sub_content),
                );

            root = root.child(overlay);
        }

        root.into_any_element()
    }
}
