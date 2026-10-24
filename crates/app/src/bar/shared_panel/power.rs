use gpui::prelude::*;
use gpui::{AnyElement, ColorExt, Context, IntoElement, Window, div, px, svg};
use services::AppState;
use ui::theme::Theme;

use super::{PowerAction, PowerHold};
use crate::bar::{Bar, style};

pub fn render(
    power_hold: Option<&PowerHold>,
    _window: &mut Window,
    cx: &mut Context<Bar>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let current_hold = power_hold.cloned();

    let (battery_pct, is_charging, battery_icon) = if let Some(state) = cx.try_global::<AppState>()
    {
        match state.power.get_battery() {
            Some(b) => {
                let icon = if b.is_charging {
                    "battery-charging.svg"
                } else if b.percentage <= 15 {
                    "battery-low.svg"
                } else if b.percentage <= 40 {
                    "battery-medium.svg"
                } else {
                    "battery-full.svg"
                };
                (Some(b.percentage), b.is_charging, icon)
            }
            None => (None, false, "battery-full.svg"),
        }
    } else {
        (None, false, "battery-full.svg")
    };

    let battery_header = if let Some(pct) = battery_pct {
        let charge_status = if is_charging { "Cargando" } else { "Batería" };
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(38.0))
            .px(px(14.0))
            .rounded_xl()
            .bg(style::surface(&theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(svg().path(battery_icon).size(px(18.0)).text_color(
                        if pct <= 15 && !is_charging {
                            theme.red()
                        } else {
                            theme.foreground()
                        },
                    ))
                    .child(
                        div()
                            .text_size(px(13.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted())
                            .child(charge_status),
                    ),
            )
            .child(
                div()
                    .text_size(px(14.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.foreground())
                    .child(format!("{pct}%")),
            )
            .into_any_element()
    } else {
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(38.0))
            .px(px(14.0))
            .rounded_xl()
            .bg(style::surface(&theme))
            .child(
                div()
                    .text_size(px(13.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child("Alimentación AC"),
            )
            .into_any_element()
    };

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

    let hover_bg = style::hover(&theme);

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
            theme.accent()
        };
        let sweep_color = if is_danger {
            theme.red().opacity(0.22)
        } else {
            theme.accent().opacity(0.22)
        };
        let icon_color = if is_danger {
            theme.red()
        } else if is_held {
            theme.accent()
        } else {
            theme.foreground()
        };

        let mut btn = div()
            .id(btn_id)
            .flex_1()
            .h(px(62.0))
            .relative()
            .overflow_hidden()
            .rounded_2xl()
            .bg(style::surface(&theme))
            .border_1()
            .border_color(if is_held {
                bar_color
            } else {
                style::border(&theme).opacity(0.35)
            })
            .cursor_pointer()
            .hover(move |s| s.bg(hover_bg));

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
                })
                .child(
                    div()
                        .id(format!("{btn_id}-prog"))
                        .absolute()
                        .top_0()
                        .bottom_0()
                        .left_0()
                        .w(gpui::relative(progress))
                        .bg(bar_color)
                        .rounded_full(),
                ),
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
                cx.listener(move |bar, _, window, cx| {
                    let entity = cx.entity().downgrade();
                    bar.shared_panel.start_power_hold(action, window, entity);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|bar, _, _, cx| {
                    bar.shared_panel.cancel_power_hold();
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
            cx.listener(|bar, _, _, cx| {
                bar.shared_panel.cancel_power_hold();
                cx.notify();
            }),
        )
        .child(battery_header)
        .child(grid)
        .child(hint_row)
        .into_any_element()
}
