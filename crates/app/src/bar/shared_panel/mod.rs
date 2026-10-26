pub mod animator;
pub mod calendar;
pub mod clipboard;
pub mod dashboard;
pub mod emoji;
pub mod media;
pub mod notification_popup;
pub mod power;
pub mod record;
pub mod tray_menu;
pub mod view;

use std::time::{Duration, Instant};

use gpui::{
    AnyView, App, AppContext, Context, Entity, EventEmitter, FocusHandle, KeyDownEvent, Pixels,
    Render, Size, Subscription, Window, px,
};
use services::{AppState, CapsuleAnimation};

use self::animator::SharedPanelAnimator;
use crate::bar::Bar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedPanelModule {
    Calendar,
    Dashboard,
    Tray(usize),
    Media,
    Power,
    NotificationPopup(u32),
    Clipboard,
    Emoji,
    Record,
}

pub type SharedPanelModuleId = SharedPanelModule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedPanelEvent {
    Close,
    Open {
        id: SharedPanelModule,
        anchor_x: Option<Pixels>,
    },
    SizeChanged(SharedPanelModule),
    NotificationChanged,
    SyncWindow,
}

pub trait SharedPanelItem: Render + EventEmitter<SharedPanelEvent> {
    fn size(&self, cx: &App) -> Size<Pixels>;
    fn focus_handle(&self) -> Option<FocusHandle> {
        None
    }
    fn needs_exclusive_keyboard(&self) -> bool {
        false
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

pub struct SharedPanelManager {
    pub active_module: Option<SharedPanelModule>,
    pub old_module: Option<SharedPanelModule>,
    pub animator: Option<SharedPanelAnimator>,
    pub is_open: bool,
    pub screen_width: Pixels,
    pub screen_height: Pixels,
    pub calendar: Entity<calendar::CalendarModule>,
    pub dashboard: Entity<dashboard::DashboardModule>,
    pub tray: Entity<tray_menu::TrayModule>,
    pub media: Entity<media::MediaModule>,
    pub power: Entity<power::PowerModule>,
    pub notification_popup: Entity<notification_popup::NotificationPopupModule>,
    pub clipboard: Entity<clipboard::ClipboardModule>,
    pub emoji: Entity<emoji::EmojiModule>,
    pub record: Entity<record::RecordModule>,
}

pub type SharedPanel = SharedPanelManager;

impl SharedPanelManager {
    pub fn new(cx: &mut impl AppContext) -> Self {
        Self {
            active_module: None,
            old_module: None,
            animator: None,
            is_open: false,
            screen_width: px(1920.0),
            screen_height: px(1080.0),
            calendar: cx.new(calendar::CalendarModule::new),
            dashboard: cx.new(dashboard::DashboardModule::new),
            tray: cx.new(tray_menu::TrayModule::new),
            media: cx.new(media::MediaModule::new),
            power: cx.new(power::PowerModule::new),
            notification_popup: cx.new(notification_popup::NotificationPopupModule::new),
            clipboard: cx.new(clipboard::ClipboardModule::new),
            emoji: cx.new(emoji::EmojiModule::new),
            record: cx.new(record::RecordModule::new),
        }
    }

    pub fn subscribe(&self, cx: &mut Context<Bar>) -> Vec<Subscription> {
        vec![
            cx.subscribe(&self.calendar, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.dashboard, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.tray, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.media, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.power, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.notification_popup, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.clipboard, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.emoji, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
            cx.subscribe(&self.record, |bar, _, event, cx| {
                bar.handle_shared_panel_event(*event, cx);
            }),
        ]
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

    pub fn current(&self) -> Option<AnyView> {
        Some(match self.active_module? {
            SharedPanelModule::Calendar => self.calendar.clone().into(),
            SharedPanelModule::Dashboard => self.dashboard.clone().into(),
            SharedPanelModule::Tray(_) => self.tray.clone().into(),
            SharedPanelModule::Media => self.media.clone().into(),
            SharedPanelModule::Power => self.power.clone().into(),
            SharedPanelModule::NotificationPopup(_) => self.notification_popup.clone().into(),
            SharedPanelModule::Clipboard => self.clipboard.clone().into(),
            SharedPanelModule::Emoji => self.emoji.clone().into(),
            SharedPanelModule::Record => self.record.clone().into(),
        })
    }

    pub fn module_size(&self, module: SharedPanelModule, cx: &App) -> Size<Pixels> {
        match module {
            SharedPanelModule::Calendar => self.calendar.read(cx).size(cx),
            SharedPanelModule::Dashboard => self.dashboard.read(cx).size(cx),
            SharedPanelModule::Tray(_) => self.tray.read(cx).size(cx),
            SharedPanelModule::Media => self.media.read(cx).size(cx),
            SharedPanelModule::Power => self.power.read(cx).size(cx),
            SharedPanelModule::NotificationPopup(_) => self.notification_popup.read(cx).size(cx),
            SharedPanelModule::Clipboard => self.clipboard.read(cx).size(cx),
            SharedPanelModule::Emoji => self.emoji.read(cx).size(cx),
            SharedPanelModule::Record => self.record.read(cx).size(cx),
        }
    }

    pub fn current_size(&self, cx: &App) -> Size<Pixels> {
        if let Some(module) = self.active_module {
            self.module_size(module, cx)
        } else {
            Size {
                width: px(0.0),
                height: px(0.0),
            }
        }
    }

    pub fn current_focus_handle(&self, cx: &App) -> Option<FocusHandle> {
        match self.active_module? {
            SharedPanelModule::Calendar => self.calendar.read(cx).focus_handle(),
            SharedPanelModule::Dashboard => self.dashboard.read(cx).focus_handle(),
            SharedPanelModule::Tray(_) => self.tray.read(cx).focus_handle(),
            SharedPanelModule::Media => self.media.read(cx).focus_handle(),
            SharedPanelModule::Power => self.power.read(cx).focus_handle(),
            SharedPanelModule::NotificationPopup(_) => {
                self.notification_popup.read(cx).focus_handle()
            }
            SharedPanelModule::Clipboard => self.clipboard.read(cx).focus_handle(),
            SharedPanelModule::Emoji => self.emoji.read(cx).focus_handle(),
            SharedPanelModule::Record => self.record.read(cx).focus_handle(),
        }
    }

    pub fn needs_exclusive_keyboard(&self, cx: &App) -> bool {
        if !self.is_open {
            return false;
        }
        match self.active_module {
            Some(SharedPanelModule::Calendar) => self.calendar.read(cx).needs_exclusive_keyboard(),
            Some(SharedPanelModule::Dashboard) => {
                self.dashboard.read(cx).needs_exclusive_keyboard()
            }
            Some(SharedPanelModule::Tray(_)) => self.tray.read(cx).needs_exclusive_keyboard(),
            Some(SharedPanelModule::Media) => self.media.read(cx).needs_exclusive_keyboard(),
            Some(SharedPanelModule::Power) => self.power.read(cx).needs_exclusive_keyboard(),
            Some(SharedPanelModule::NotificationPopup(_)) => {
                self.notification_popup.read(cx).needs_exclusive_keyboard()
            }
            Some(SharedPanelModule::Clipboard) => {
                self.clipboard.read(cx).needs_exclusive_keyboard()
            }
            Some(SharedPanelModule::Emoji) => self.emoji.read(cx).needs_exclusive_keyboard(),
            Some(SharedPanelModule::Record) => self.record.read(cx).needs_exclusive_keyboard(),
            None => false,
        }
    }

    pub fn prepare_module(&mut self, module: SharedPanelModule, cx: &mut App) {
        match module {
            SharedPanelModule::Dashboard => {
                self.dashboard.update(cx, |m, cx| m.reset(cx));
            }
            SharedPanelModule::Tray(idx) => {
                self.tray.update(cx, |m, _| m.index = idx);
            }
            SharedPanelModule::Clipboard => {
                self.clipboard.update(cx, |m, cx| m.open(cx));
            }
            SharedPanelModule::Emoji => {
                self.emoji.update(cx, |m, cx| m.open(cx));
            }
            SharedPanelModule::Record => {
                self.record.update(cx, |m, cx| m.open(cx));
            }
            SharedPanelModule::Power => {
                self.power.update(cx, |m, _| m.cancel_hold());
            }
            SharedPanelModule::NotificationPopup(id) => {
                let timeout = services::NotificationStore::global()
                    .get_latest_active_notification()
                    .filter(|n| n.id == id)
                    .map(|n| n.timeout)
                    .unwrap_or(Duration::from_secs(5));
                self.notification_popup
                    .update(cx, |m, cx| m.open(id, timeout, cx));
            }
            SharedPanelModule::Calendar | SharedPanelModule::Media => {}
        }
    }

    pub fn open(
        &mut self,
        module: SharedPanelModule,
        anchor_x: Pixels,
        window: &mut Window,
        cx: &mut Context<Bar>,
    ) {
        self.sync_display_bounds(window, cx);

        if self.is_open && self.active_module == Some(module) {
            return;
        }

        self.prepare_module(module, cx);

        let is_notif = matches!(module, SharedPanelModule::NotificationPopup(_));
        let target_size = self.module_size(module, cx);
        let (target_x, dock_edge) =
            compute_target_position(anchor_x, target_size.width, self.screen_width, is_notif);

        if self.is_open {
            self.old_module = self.active_module;
            self.active_module = Some(module);
            if let Some(ref mut anim) = self.animator {
                anim.switch_to(target_x, target_size, dock_edge, anchor_x, Instant::now());
            }
            return;
        }

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

        self.animator = Some(SharedPanelAnimator::new(
            anchor_x,
            target_x,
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
    }

    pub fn request_close(&mut self, cx: &mut App) {
        if !self.is_open && self.animator.is_none() {
            return;
        }
        self.is_open = false;
        self.power.update(cx, |m, _| m.cancel_hold());
        self.dashboard.update(cx, |m, _| {
            m.dragging_volume = false;
            m.dragging_input_volume = false;
            m.dragging_brightness = false;
        });
        if let Some(ref mut anim) = self.animator {
            anim.request_close();
        }
    }

    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut App) {
        match self.active_module {
            Some(SharedPanelModule::Dashboard) => {
                self.dashboard
                    .update(cx, |m, cx| m.handle_key_down(event, cx));
            }
            Some(SharedPanelModule::Clipboard) => {
                self.clipboard
                    .update(cx, |m, cx| m.handle_key_down(event, cx));
            }
            Some(SharedPanelModule::Emoji) => {
                self.emoji.update(cx, |m, cx| m.handle_key_down(event, cx));
            }
            _ => {
                if event.keystroke.key == "escape" {
                    self.request_close(cx);
                }
            }
        }
    }

    pub fn advance(&mut self, now: Instant) -> (bool, bool) {
        if let Some(ref mut anim) = self.animator {
            let (running, closed) = anim.advance(now);
            if closed {
                self.animator = None;
                self.active_module = None;
                self.old_module = None;
                self.is_open = false;
                return (false, true);
            }
            if anim.content_progress >= 1.0 {
                self.old_module = None;
            }
            return (running || anim.is_closing(), false);
        }
        (false, false)
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
