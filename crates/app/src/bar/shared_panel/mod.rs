pub mod animator;
pub mod calendar;
pub mod clipboard;
pub mod dashboard;
pub mod emoji;
pub mod media;
pub mod notification_popup;
pub mod power;
pub mod tray_menu;
pub mod view;

use std::time::{Duration, Instant};

use gpui::{AnyElement, App, Context, IntoElement, Pixels, Size, Window, div, px};
use services::{AppState, CapsuleAnimation};

use self::animator::SharedPanelAnimator;
use crate::bar::Bar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SharedPanelModule {
    Calendar,
    Dashboard,
    Tray(usize),
    Media,
    Power,
    NotificationPopup(u32),
    Clipboard,
    Emoji,
}

impl SharedPanelModule {
    pub fn default_size(&self, cx: &App) -> Size<Pixels> {
        match self {
            Self::Calendar => Size {
                width: px(720.0),
                height: px(450.0),
            },
            Self::Dashboard => {
                let has_battery = cx
                    .try_global::<AppState>()
                    .and_then(|s| s.power.get_battery())
                    .is_some();
                Size {
                    width: px(440.0),
                    height: if has_battery { px(414.0) } else { px(346.0) },
                }
            }
            Self::Tray(idx) => {
                let items_count = if let Some(state) = cx.try_global::<AppState>() {
                    let items = state.sni_host.items_arc();
                    items.get(*idx).map(|i| i.menu_items.len()).unwrap_or(0)
                } else {
                    0
                };
                let h = (52.0 + items_count as f32 * 38.0).clamp(150.0, 440.0);
                Size {
                    width: px(310.0),
                    height: px(h),
                }
            }
            Self::Media => Size {
                width: px(430.0),
                height: px(250.0),
            },
            Self::Power => Size {
                width: px(380.0),
                height: px(172.0),
            },
            Self::NotificationPopup(id) => {
                let notif_opt = services::NotificationStore::global()
                    .get_latest_active_notification()
                    .filter(|n| n.id == *id)
                    .or_else(|| {
                        services::NotificationStore::global()
                            .get_all_notifications()
                            .into_iter()
                            .find(|n| n.id == *id)
                    });

                let (has_body, has_actions) = if let Some(ref item) = notif_opt {
                    (!item.body.trim().is_empty(), !item.actions.is_empty())
                } else {
                    (false, false)
                };

                let mut h = 108.0;
                if has_body {
                    h += 46.0;
                }
                if has_actions {
                    h += 38.0;
                }

                Size {
                    width: px(440.0),
                    height: px(h),
                }
            }
            Self::Clipboard => Size {
                width: px(580.0),
                height: px(540.0),
            },
            Self::Emoji => Size {
                width: px(540.0),
                height: px(500.0),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Lock,
    Suspend,
    Logout,
    Reboot,
    Poweroff,
}

#[derive(Debug, Clone)]
pub struct PowerHold {
    pub action: PowerAction,
    pub start: Instant,
    pub duration: Duration,
    pub progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockEdge {
    #[default]
    Center,
    Left,
    Right,
}

pub fn compute_target_position(
    anchor_x: Pixels,
    width: Pixels,
    screen_width: Pixels,
    is_notification_popup: bool,
) -> (Pixels, DockEdge) {
    if is_notification_popup {
        let x = (screen_width - width).max(px(0.0));
        return (x, DockEdge::Right);
    }

    let ideal = f32::from(anchor_x) - f32::from(width) / 2.0;
    let max_x = (f32::from(screen_width) - f32::from(width)).max(0.0);

    if ideal >= max_x - 32.0 {
        (px(max_x), DockEdge::Right)
    } else if ideal <= 32.0 {
        (px(0.0), DockEdge::Left)
    } else {
        (px(ideal.clamp(0.0, max_x)), DockEdge::Center)
    }
}

pub struct SharedPanel {
    pub active_module: Option<SharedPanelModule>,
    pub old_module: Option<SharedPanelModule>,
    pub animator: Option<SharedPanelAnimator>,
    pub is_open: bool,
    pub screen_width: Pixels,
    pub screen_height: Pixels,
    pub power_hold: Option<PowerHold>,
    pub dashboard_state: dashboard::DashboardState,
    pub clipboard_state: clipboard::ClipboardState,
    pub emoji_state: emoji::EmojiState,
    pub focus_handle: Option<gpui::FocusHandle>,
    pub animation_frame_pending: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Default for SharedPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedPanel {
    pub fn new() -> Self {
        Self {
            active_module: None,
            old_module: None,
            animator: None,
            is_open: false,
            screen_width: px(1920.0),
            screen_height: px(1080.0),
            power_hold: None,
            dashboard_state: dashboard::DashboardState::default(),
            clipboard_state: clipboard::ClipboardState::default(),
            emoji_state: emoji::EmojiState::default(),
            focus_handle: None,
            animation_frame_pending: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
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

    pub fn sync_display_bounds(&mut self, window: &Window, cx: &App) {
        if let Some(display) = window.display(cx) {
            let bounds = display.bounds();
            self.screen_width = bounds.size.width;
            self.screen_height = bounds.size.height;
        }
    }

    pub fn open(
        &mut self,
        module: SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        entity: gpui::WeakEntity<Bar>,
        cx: &App,
    ) {
        self.sync_display_bounds(window, cx);

        if self.is_open && self.active_module.as_ref() == Some(&module) {
            return;
        }

        if module == SharedPanelModule::Dashboard {
            self.dashboard_state.reset();
        }

        let is_notif = matches!(module, SharedPanelModule::NotificationPopup(_));
        if self.is_open {
            self.old_module = self.active_module.clone();
            self.active_module = Some(module.clone());
            let target_size = module.default_size(cx);
            let (target_x, dock_edge) =
                compute_target_position(anchor_x, target_size.width, self.screen_width, is_notif);
            if let Some(ref mut anim) = self.animator {
                anim.switch_to(target_x, target_size, dock_edge, Instant::now());
            }
            Bar::queue_animation_frame_checked(window, entity, &self.animation_frame_pending);
            return;
        }

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

        let target_size = module.default_size(cx);
        let (initial_x, dock_edge) =
            compute_target_position(anchor_x, target_size.width, self.screen_width, is_notif);

        self.animator = Some(SharedPanelAnimator::new(
            initial_x,
            target_size,
            dock_edge,
            duration,
            animation,
            bounciness,
            squash_stretch,
            reduced_motion,
        ));
        self.is_open = true;
        self.active_module = Some(module);
        self.old_module = None;

        Bar::queue_animation_frame_checked(window, entity, &self.animation_frame_pending);
    }

    pub fn toggle(
        &mut self,
        module: SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Bar>,
    ) {
        if self.is_open && self.active_module.as_ref() == Some(&module) {
            self.close(window, cx);
            return;
        }
        self.open(module, anchor_x, window, cx.entity().downgrade(), cx);
        cx.notify();
    }

    pub fn request_close(&mut self, window: &mut Window, entity: gpui::WeakEntity<Bar>) {
        if !self.is_open && self.animator.is_none() {
            return;
        }
        self.is_open = false;
        self.power_hold = None;
        self.dashboard_state.dragging_volume = false;
        self.dashboard_state.dragging_input_volume = false;
        self.dashboard_state.dragging_brightness = false;
        if let Some(ref mut anim) = self.animator {
            anim.request_close();
            Bar::queue_animation_frame_checked(window, entity, &self.animation_frame_pending);
        }
    }

    pub fn close(&mut self, window: &mut Window, cx: &mut Context<Bar>) {
        self.request_close(window, cx.entity().downgrade());
        cx.notify();
    }

    pub fn start_power_hold(
        &mut self,
        action: PowerAction,
        window: &Window,
        entity: gpui::WeakEntity<Bar>,
    ) {
        self.power_hold = Some(PowerHold {
            action,
            start: Instant::now(),
            duration: Duration::from_millis(750),
            progress: 0.0,
        });
        Bar::queue_animation_frame_checked(window, entity, &self.animation_frame_pending);
    }

    pub fn cancel_power_hold(&mut self) {
        self.power_hold = None;
    }

    pub fn execute_power_action(action: PowerAction) {
        services::spawn_tokio(async move {
            let _ = match action {
                PowerAction::Lock => services::system::power::lock().await,
                PowerAction::Suspend => services::system::power::suspend().await,
                PowerAction::Logout => services::system::power::logout().await,
                PowerAction::Reboot => services::system::power::reboot().await,
                PowerAction::Poweroff => services::system::power::poweroff().await,
            };
        });
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        let mut triggered = None;
        let mut holding = false;
        if let Some(ref mut hold) = self.power_hold {
            let elapsed = now.duration_since(hold.start);
            let t = (elapsed.as_secs_f32() / hold.duration.as_secs_f32()).clamp(0.0, 1.0);
            hold.progress = t;
            if t >= 1.0 {
                triggered = Some(hold.action);
            } else {
                holding = true;
            }
        }
        if let Some(action) = triggered {
            self.power_hold = None;
            Self::execute_power_action(action);
            return (true, true);
        }

        if let Some(ref mut anim) = self.animator {
            let (running, closed) = anim.advance(now);
            let sub_running = self.dashboard_state.advance(now);
            if closed {
                self.animator = None;
                self.active_module = None;
                self.old_module = None;
                self.is_open = false;
                self.power_hold = None;
                return (false, true);
            }
            if anim.content_progress >= 1.0 {
                self.old_module = None;
            }
            return (
                running || anim.is_closing() || holding || sub_running,
                false,
            );
        }
        (holding, false)
    }

    pub fn render_content(&self, window: &mut Window, cx: &mut Context<Bar>) -> AnyElement {
        let Some(ref module) = self.active_module else {
            return div().into_any_element();
        };

        match module {
            SharedPanelModule::Calendar => calendar::render(window, cx),
            SharedPanelModule::Dashboard => dashboard::render(&self.dashboard_state, window, cx),
            SharedPanelModule::Tray(idx) => tray_menu::render(*idx, window, cx),
            SharedPanelModule::Media => media::render(window, cx),
            SharedPanelModule::Power => power::render(self.power_hold.as_ref(), window, cx),
            SharedPanelModule::NotificationPopup(id) => notification_popup::render(*id, window, cx),
            SharedPanelModule::Clipboard => clipboard::render(&self.clipboard_state, window, cx),
            SharedPanelModule::Emoji => emoji::render(&self.emoji_state, window, cx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_target_position_notification_popup_always_right() {
        let (x, dock) = compute_target_position(px(100.0), px(380.0), px(1920.0), true);
        assert_eq!(x, px(1540.0));
        assert_eq!(dock, DockEdge::Right);
    }

    #[test]
    fn test_compute_target_position_near_right_edge() {
        let (x, dock) = compute_target_position(px(1900.0), px(280.0), px(1920.0), false);
        assert_eq!(x, px(1640.0));
        assert_eq!(dock, DockEdge::Right);
    }

    #[test]
    fn test_compute_target_position_near_left_edge() {
        let (x, dock) = compute_target_position(px(50.0), px(250.0), px(1920.0), false);
        assert_eq!(x, px(0.0));
        assert_eq!(dock, DockEdge::Left);
    }

    #[test]
    fn test_compute_target_position_center() {
        let (x, dock) = compute_target_position(px(960.0), px(320.0), px(1920.0), false);
        assert_eq!(x, px(800.0));
        assert_eq!(dock, DockEdge::Center);
    }

    #[test]
    fn test_compute_target_position_clipboard_centered() {
        let (x, dock) = compute_target_position(px(960.0), px(480.0), px(1920.0), false);
        assert_eq!(x, px(720.0));
        assert_eq!(dock, DockEdge::Center);
    }

    #[test]
    fn test_compute_target_position_emoji_centered() {
        let (x, dock) = compute_target_position(px(960.0), px(440.0), px(1920.0), false);
        assert_eq!(x, px(740.0));
        assert_eq!(dock, DockEdge::Center);
    }
}
