use super::{ORB_SIZE, OrbitId, Presentation, USAGE_PANEL_WIDTH};
use crate::capsule::{Capsule, widgets::style};
use gpui::{Context, IntoElement, div, prelude::*, px, svg};
use services::RecordStatus;
use ui::theme::Theme;

const CONTENT_INSET: f32 = 11.0;
const ICON_SLOT: f32 = 20.0;
const COLUMN_GAP: f32 = 8.0;
const ACTION_PADDING: f32 = 4.0;

pub(super) fn render(
    presentation: Presentation,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement + use<> {
    let Presentation {
        state,
        visuals,
        capsule,
        content_opacity,
        power_opacity,
        power: power_status,
    } = presentation;
    div()
        .absolute()
        .top_0()
        .left_0()
        .w(capsule.size.width)
        .h(capsule.size.height)
        .font_family(theme.font_family())
        .children(visuals.into_iter().map(|visual| {
            let id = visual.id;
            if id == OrbitId::Power {
                return power(visual, capsule, power_opacity, &power_status, theme, cx)
                    .into_any_element();
            }
            if id == OrbitId::Usage {
                return usage(visual, capsule, state.usage, content_opacity, theme, cx)
                    .into_any_element();
            }
            let indicator = match id {
                OrbitId::Power => svg()
                    .path("power.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground())
                    .into_any_element(),
                OrbitId::Usage => svg()
                    .path("cpu.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground())
                    .into_any_element(),
                OrbitId::Shelf => svg()
                    .path("pin-tilted.svg")
                    .size(px(13.0))
                    .text_color(theme.foreground())
                    .into_any_element(),
                OrbitId::Recording if state.record_status == RecordStatus::Paused => svg()
                    .path("pause.svg")
                    .size(px(13.0))
                    .text_color(theme.accent())
                    .into_any_element(),
                OrbitId::Recording => div()
                    .size(px(8.0))
                    .rounded_full()
                    .bg(theme.red())
                    .into_any_element(),
            };
            div()
                .id(match id {
                    OrbitId::Shelf => "orbit-shelf",
                    OrbitId::Recording => "orbit-recording",
                    OrbitId::Usage => "orbit-usage",
                    OrbitId::Power => "orbit-power",
                })
                .absolute()
                .left(visual.bounds.origin.x - capsule.origin.x)
                .top(visual.bounds.origin.y - capsule.origin.y)
                .size(px(ORB_SIZE))
                .rounded_full()
                .bg(style::background(theme))
                .border_1()
                .border_color(style::border(theme))
                .opacity(visual.opacity)
                .flex()
                .items_center()
                .justify_center()
                .child(indicator)
                .when(id == OrbitId::Shelf, |element| {
                    element.child(
                        gpui::deferred(
                            div()
                                .absolute()
                                .bottom(px(-3.0))
                                .right(px(-3.0))
                                .min_w(px(14.0))
                                .h(px(14.0))
                                .px(px(2.0))
                                .rounded_full()
                                .bg(theme.accent())
                                .text_color(style::on_accent(theme))
                                .text_size(px(8.0))
                                .line_height(px(8.0))
                                .text_center()
                                .font_weight(gpui::FontWeight::BOLD)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(if state.shelf_count > 99 {
                                    "99+".to_owned()
                                } else {
                                    state.shelf_count.to_string()
                                }),
                        )
                        .priority(10),
                    )
                })
                .when(visual.interactive, |element| {
                    let element = element
                        .cursor_pointer()
                        .hover(|s| s.border_color(theme.accent()))
                        .on_click(cx.listener(move |capsule, _, _, cx| {
                            cx.stop_propagation();
                            capsule.open_orbit(id, cx);
                        }));
                    element.when(id == OrbitId::Shelf, |element| {
                        element
                            .on_drop(cx.listener(Capsule::drop_on_shelf))
                            .on_drop(cx.listener(Capsule::drop_shelf_file))
                    })
                })
                .into_any_element()
        }))
}

fn toggle(
    id: OrbitId,
    progress: f32,
    bottom: bool,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> gpui::Stateful<gpui::Div> {
    let left = id == OrbitId::Power;
    div()
        .id(if left {
            "orbit-power-toggle"
        } else {
            "orbit-usage-toggle"
        })
        .absolute()
        .when(left, |s| s.right_0())
        .when(!left, |s| s.left_0())
        .when(bottom, |s| s.bottom_0())
        .when(!bottom, |s| s.top_0())
        .size(px(ORB_SIZE - 2.0))
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .hover(|s| s.bg(style::hover(theme)))
        .on_click(cx.listener(move |capsule, _, _, cx| {
            cx.stop_propagation();
            capsule.open_orbit(id, cx);
        }))
        .child(
            div()
                .relative()
                .size(px(13.0))
                .child(
                    svg()
                        .absolute()
                        .top_0()
                        .left_0()
                        .path(if left { "power.svg" } else { "cpu.svg" })
                        .size_full()
                        .opacity(1.0 - progress)
                        .text_color(theme.foreground()),
                )
                .child(
                    svg()
                        .absolute()
                        .top_0()
                        .left_0()
                        .path("close.svg")
                        .size_full()
                        .opacity(progress)
                        .text_color(theme.foreground()),
                ),
        )
}

fn usage(
    visual: super::OrbitVisual,
    capsule: gpui::Bounds<gpui::Pixels>,
    status: services::system::UsageStatus,
    content_opacity: f32,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement + use<> {
    let percent = |value: Option<u8>| {
        value
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "—".to_owned())
    };
    let memory_percent = (status.memory_total_kib > 0).then(|| {
        ((status.memory_used_kib as f64 / status.memory_total_kib as f64 * 100.0).round() as u8)
            .min(100)
    });
    let memory = if status.memory_total_kib > 0 {
        format!(
            "{:.1} / {:.1} GiB",
            status.memory_used_kib as f64 / 1_048_576.0,
            status.memory_total_kib as f64 / 1_048_576.0
        )
    } else {
        "—".to_owned()
    };
    let bottom = cx
        .global::<services::AppState>()
        .config
        .get()
        .ui
        .capsule_location
        == services::CapsuleLocation::Bottom;
    div()
        .id("orbit-usage")
        .absolute()
        .left(visual.bounds.origin.x - capsule.origin.x)
        .top(visual.bounds.origin.y - capsule.origin.y)
        .w(visual.bounds.size.width)
        .h(visual.bounds.size.height)
        .rounded(px((f32::from(visual.bounds.size.height) * 0.5).min(16.0)))
        .bg(style::background(theme))
        .border_1()
        .border_color(style::border(theme))
        .overflow_hidden()
        .text_color(theme.foreground())
        .occlude()
        .child(toggle(OrbitId::Usage, content_opacity, bottom, theme, cx))
        .when(content_opacity > 0.0, |element| {
            element.child(
                div()
                    .absolute()
                    .when(bottom, |s| s.bottom(px(32.0)))
                    .when(!bottom, |s| s.top(px(32.0)))
                    .left(px(CONTENT_INSET))
                    .right(px(CONTENT_INSET))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .opacity(content_opacity)
                    .children(
                        [
                            ("cpu.svg", "CPU", percent(status.cpu), status.cpu),
                            ("memory-stick.svg", "RAM", memory, memory_percent),
                            ("gpu.svg", "GPU", percent(status.gpu), status.gpu),
                        ]
                        .into_iter()
                        .map(|(icon, label, value, amount)| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .h(px(20.0))
                                        .flex_shrink_0()
                                        .flex()
                                        .items_center()
                                        .gap(px(COLUMN_GAP))
                                        .child(
                                            div()
                                                .size(px(ICON_SLOT))
                                                .flex_shrink_0()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    svg()
                                                        .path(icon)
                                                        .size(px(16.0))
                                                        .flex_shrink_0()
                                                        .text_color(theme.foreground()),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .line_height(px(14.0))
                                                .child(label),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_right()
                                                .text_size(px(11.0))
                                                .line_height(px(14.0))
                                                .child(value),
                                        ),
                                )
                                .child(
                                    div()
                                        .h(px(4.0))
                                        .flex_shrink_0()
                                        .rounded_full()
                                        .bg(style::surface(theme))
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .h_full()
                                                .w(px((USAGE_PANEL_WIDTH - 24.0)
                                                    * f32::from(amount.unwrap_or(0))
                                                    / 100.0))
                                                .rounded_full()
                                                .bg(theme.accent()),
                                        ),
                                )
                        }),
                    ),
            )
        })
}

fn power(
    visual: super::OrbitVisual,
    capsule: gpui::Bounds<gpui::Pixels>,
    opacity: f32,
    status: &super::PowerStatus,
    theme: &Theme,
    cx: &mut Context<Capsule>,
) -> impl IntoElement + use<> {
    let language = &cx.global::<services::AppState>().language;
    let actions = [
        (
            super::PowerAction::Poweroff,
            "power.svg",
            language.get("orbit.poweroff"),
        ),
        (
            super::PowerAction::Reboot,
            "rotate-ccw.svg",
            language.get("orbit.reboot"),
        ),
        (
            super::PowerAction::Logout,
            "log-out.svg",
            language.get("orbit.logout"),
        ),
        (
            super::PowerAction::Lock,
            "lock.svg",
            language.get("orbit.lock"),
        ),
    ];
    let pending = status.pending;
    let bottom = cx
        .global::<services::AppState>()
        .config
        .get()
        .ui
        .capsule_location
        == services::CapsuleLocation::Bottom;
    div()
        .id("orbit-power")
        .absolute()
        .left(visual.bounds.origin.x - capsule.origin.x)
        .top(visual.bounds.origin.y - capsule.origin.y)
        .w(visual.bounds.size.width)
        .h(visual.bounds.size.height)
        .rounded(px((f32::from(visual.bounds.size.height) * 0.5).min(16.0)))
        .bg(style::background(theme))
        .border_1()
        .border_color(style::border(theme))
        .overflow_hidden()
        .text_color(theme.foreground())
        .occlude()
        .child(toggle(OrbitId::Power, opacity, bottom, theme, cx))
        .when(opacity > 0.0, |element| {
            element
                .child(
                    div()
                        .absolute()
                        .left(px(CONTENT_INSET - ACTION_PADDING))
                        .right(px(CONTENT_INSET - ACTION_PADDING))
                        .when(bottom, |s| s.bottom(px(32.0)))
                        .when(!bottom, |s| s.top(px(32.0)))
                        .opacity(opacity)
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .children(actions.into_iter().enumerate().map(
                            |(index, (action, icon, label))| {
                                div()
                                    .id(("orbit-power-action", index))
                                    .h(px(30.0))
                                    .px(px(ACTION_PADDING))
                                    .rounded(px(8.0))
                                    .flex()
                                    .items_center()
                                    .gap(px(COLUMN_GAP))
                                    .text_size(px(12.0))
                                    .opacity(if pending { 0.5 } else { 1.0 })
                                    .when(!pending, |s| {
                                        s.cursor_pointer()
                                            .hover(|s| s.bg(style::hover(theme)))
                                            .on_click(cx.listener(move |capsule, _, _, cx| {
                                                cx.stop_propagation();
                                                capsule.execute_power(action, cx);
                                            }))
                                    })
                                    .child(
                                        div()
                                            .size(px(ICON_SLOT))
                                            .flex_shrink_0()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                svg()
                                                    .path(icon)
                                                    .size(px(16.0))
                                                    .flex_shrink_0()
                                                    .text_color(theme.foreground()),
                                            ),
                                    )
                                    .child(label)
                            },
                        )),
                )
                .when_some(status.error.clone(), |element, error| {
                    element.child(
                        div()
                            .absolute()
                            .bottom(px(2.0))
                            .left(px(8.0))
                            .w(px(USAGE_PANEL_WIDTH - 16.0))
                            .text_size(px(9.0))
                            .text_color(theme.red())
                            .text_ellipsis()
                            .child(error),
                    )
                })
        })
}

pub(super) fn preview(
    state: &super::state::State,
    size: gpui::Size<gpui::Pixels>,
    gap: gpui::Pixels,
    bottom: bool,
    theme: &Theme,
) -> impl IntoElement + use<> {
    let scale = 0.5;
    let capsule = gpui::Bounds {
        origin: gpui::point(px(1000.0), px(1000.0)),
        size: gpui::size(size.width / scale, size.height / scale),
    };
    let viewport = gpui::size(px(3000.0), px(3000.0));
    let location = if bottom {
        crate::capsule::CapsuleLocation::BOTTOM
    } else {
        crate::capsule::CapsuleLocation::TOP
    };
    div()
        .absolute()
        .top_0()
        .left_0()
        .w(size.width)
        .h(size.height)
        .children(
            state
                .layout(viewport, capsule, gap / scale, location)
                .map(|visual| {
                    let left = visual.id == OrbitId::Power;
                    let icon = match visual.id {
                        OrbitId::Power => "power.svg",
                        OrbitId::Usage => "cpu.svg",
                        OrbitId::Shelf => "pin-tilted.svg",
                        OrbitId::Recording => "circle.svg",
                    };
                    let opacity = match visual.id {
                        OrbitId::Power => state.power_expanded.content_opacity(),
                        OrbitId::Usage => state.expanded.content_opacity(),
                        _ => 0.0,
                    };
                    div()
                        .absolute()
                        .left((visual.bounds.origin.x - capsule.origin.x) * scale)
                        .top((visual.bounds.origin.y - capsule.origin.y) * scale)
                        .w(visual.bounds.size.width * scale)
                        .h(visual.bounds.size.height * scale)
                        .rounded(px(8.0))
                        .bg(style::raised(theme))
                        .border_1()
                        .border_color(style::border(theme))
                        .overflow_hidden()
                        .opacity(visual.opacity)
                        .child(
                            div()
                                .absolute()
                                .when(left, |s| s.right_0())
                                .when(!left, |s| s.left_0())
                                .when(bottom, |s| s.bottom_0())
                                .when(!bottom, |s| s.top_0())
                                .size(px(ORB_SIZE * scale))
                                .flex()
                                .items_center()
                                .justify_center()
                                .when(visual.id != OrbitId::Recording, |s| {
                                    s.child(
                                        svg()
                                            .path(icon)
                                            .size(px(8.0))
                                            .text_color(theme.foreground()),
                                    )
                                })
                                .when(visual.id == OrbitId::Recording, |s| {
                                    s.child(div().size(px(4.0)).rounded_full().bg(theme.red()))
                                }),
                        )
                        .when(opacity > 0.0, |s| {
                            s.child(
                                div()
                                    .absolute()
                                    .left(px(6.0))
                                    .when(bottom, |s| s.bottom(px(18.0)))
                                    .when(!bottom, |s| s.top(px(18.0)))
                                    .w(px(USAGE_PANEL_WIDTH * scale - 12.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(5.0))
                                    .opacity(opacity)
                                    .children(
                                        if left {
                                            [
                                                "power.svg",
                                                "rotate-ccw.svg",
                                                "log-out.svg",
                                                "lock.svg",
                                            ]
                                        } else {
                                            ["cpu.svg", "memory-stick.svg", "gpu.svg", ""]
                                        }
                                        .into_iter()
                                        .filter(|icon| !icon.is_empty())
                                        .map(|icon| {
                                            div()
                                                .h(px(12.0))
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .child(
                                                    svg()
                                                        .path(icon)
                                                        .size(px(10.0))
                                                        .text_color(theme.foreground()),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .h(px(3.0))
                                                        .rounded_full()
                                                        .bg(theme.accent()),
                                                )
                                        }),
                                    ),
                            )
                        })
                }),
        )
}
