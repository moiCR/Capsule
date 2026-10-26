use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    App, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render,
    Size, Task, Window, div, px, svg,
};
use services::AppState;
use ui::theme::Theme;

use super::{PowerAction, PowerHold, SharedPanelEvent, SharedPanelItem};

pub struct PowerModule {
    pub power_hold: Option<PowerHold>,
    hold_task: Option<Task<()>>,
    focus: FocusHandle,
}

impl PowerModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            power_hold: None,
            hold_task: None,
            focus: cx.focus_handle(),
        }
    }

    pub fn start_hold(&mut self, action: PowerAction, cx: &mut Context<Self>) {
        let start = Instant::now();
        let duration = Duration::from_millis(750);
        self.power_hold = Some(PowerHold {
            action,
            start,
            duration,
            progress: 0.0,
        });
        self.hold_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let mut done = false;
                let updated = this.update(cx, |module, cx| {
                    if let Some(ref mut hold) = module.power_hold {
                        let elapsed = Instant::now().duration_since(hold.start);
                        let t =
                            (elapsed.as_secs_f32() / hold.duration.as_secs_f32()).clamp(0.0, 1.0);
                        hold.progress = t;
                        if t >= 1.0 {
                            let act = hold.action;
                            module.power_hold = None;
                            Self::execute_action(act);
                            cx.emit(SharedPanelEvent::Close);
                            done = true;
                        }
                        cx.notify();
                    } else {
                        done = true;
                    }
                });
                if updated.is_err() || done {
                    break;
                }
            }
        }));
    }

    pub fn cancel_hold(&mut self) {
        self.power_hold = None;
        self.hold_task = None;
    }

    pub fn execute_action(action: PowerAction) {
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
}

impl EventEmitter<SharedPanelEvent> for PowerModule {}

impl SharedPanelItem for PowerModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        Size {
            width: px(380.0),
            height: px(172.0),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

impl Render for PowerModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let current_hold = self.power_hold.clone();

        let (lang_lock, lang_suspend, lang_logout, lang_reboot, lang_poweroff, idle_hint) =
            if let Some(state) = cx.try_global::<AppState>() {
                let l = &state.language;
                let hint = if l.current_target() == "es" {
                    "Mantén presionado para ejecutar"
                } else {
                    "Hold to execute"
                };
                (
                    l.get("orbit.lock"),
                    l.get("orbit.suspend"),
                    l.get("orbit.logout"),
                    l.get("orbit.reboot"),
                    l.get("orbit.poweroff"),
                    hint.to_string(),
                )
            } else {
                (
                    "Bloquear".to_string(),
                    "Suspender".to_string(),
                    "Cerrar sesión".to_string(),
                    "Reiniciar".to_string(),
                    "Apagar".to_string(),
                    "Mantén presionado para ejecutar".to_string(),
                )
            };

        let buttons = [
            (PowerAction::Lock, "shared-power-lock", "lock.svg", false),
            (
                PowerAction::Suspend,
                "shared-power-suspend",
                "moon.svg",
                false,
            ),
            (
                PowerAction::Logout,
                "shared-power-logout",
                "log-out.svg",
                false,
            ),
            (
                PowerAction::Reboot,
                "shared-power-reboot",
                "rotate-ccw.svg",
                false,
            ),
            (
                PowerAction::Poweroff,
                "shared-power-poweroff",
                "power.svg",
                true,
            ),
        ];

        let mut grid = div()
            .id("power-buttons-grid")
            .flex()
            .items_center()
            .gap(px(8.0))
            .w_full();

        let btn_hover_bg = theme.surface_container_high();

        for (action, btn_id, icon, is_danger) in buttons {
            let is_held = current_hold
                .as_ref()
                .map(|h| h.action == action)
                .unwrap_or(false);
            let progress = if is_held {
                current_hold.as_ref().map(|h| h.progress).unwrap_or(0.0)
            } else {
                0.0
            };

            let bar_color = if is_danger {
                theme.red()
            } else {
                theme.primary()
            };
            let sweep_color = if is_danger {
                theme.red().opacity(0.28)
            } else {
                theme.primary_container()
            };
            let icon_color = if is_danger {
                theme.red()
            } else if is_held {
                theme.primary()
            } else {
                theme.foreground()
            };

            let mut btn = div()
                .id(btn_id)
                .flex_1()
                .h(px(58.0))
                .relative()
                .overflow_hidden()
                .rounded(px(16.0))
                .bg(if is_held {
                    theme.primary_container().opacity(0.5)
                } else {
                    theme.surface_container_low()
                })
                .border_1()
                .border_color(if is_held {
                    bar_color
                } else {
                    theme.outline_variant()
                })
                .cursor_pointer()
                .hover(move |s| s.bg(btn_hover_bg));

            if progress > 0.0 {
                btn = btn.child(
                    div()
                        .id(format!("{btn_id}-sweep"))
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(gpui::relative(progress))
                        .bg(sweep_color),
                );
            }

            btn = btn.child(
                div()
                    .id(format!("{btn_id}-track"))
                    .absolute()
                    .bottom_0()
                    .left_0()
                    .right_0()
                    .h(px(4.0))
                    .bg(if is_held {
                        theme.foreground_muted().opacity(0.15)
                    } else {
                        gpui::transparent_black()
                    }),
            );

            btn = btn.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(svg().path(icon).size(px(22.0)).text_color(icon_color)),
            );

            btn = btn
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, cx| {
                        this.start_hold(action, cx);
                        cx.notify();
                    }),
                )
                .on_mouse_up(
                    gpui::MouseButton::Left,
                    cx.listener(|this, _, _, cx| {
                        this.cancel_hold();
                        cx.notify();
                    }),
                );

            grid = grid.child(btn);
        }

        let (hint_text, hint_color) = if let Some(ref hold) = current_hold {
            let text = match hold.action {
                PowerAction::Lock => format!("{lang_lock}..."),
                PowerAction::Suspend => format!("{lang_suspend}..."),
                PowerAction::Logout => format!("{lang_logout}..."),
                PowerAction::Reboot => format!("{lang_reboot}..."),
                PowerAction::Poweroff => format!("{lang_poweroff}..."),
            };
            let color = if hold.action == PowerAction::Poweroff {
                theme.red()
            } else {
                theme.accent()
            };
            (text, color)
        } else {
            (idle_hint, theme.foreground_muted().opacity(0.75))
        };

        let hint_row = div()
            .flex()
            .items_center()
            .justify_center()
            .h(px(20.0))
            .text_size(px(12.5))
            .font_weight(if current_hold.is_some() {
                gpui::FontWeight::SEMIBOLD
            } else {
                gpui::FontWeight::NORMAL
            })
            .text_color(hint_color)
            .child(hint_text);

        div()
            .id("shared-panel-power")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SharedPanelEvent::Close);
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .justify_between()
            .px(px(16.0))
            .py(px(14.0))
            .gap(px(10.0))
            .font_family(theme.font_family())
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.cancel_hold();
                    cx.notify();
                }),
            )
            .on_mouse_exit(cx.listener(|this, _, _, cx| {
                if this.power_hold.is_some() {
                    this.cancel_hold();
                    cx.notify();
                }
            }))
            .child(grid)
            .child(hint_row)
    }
}
