mod audio;
mod connectivity;
mod tray;
use super::error;
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, Context, IntoElement, MotionDurationExt, canvas, div, ease_in_out, prelude::*, px,
    svg,
};
use ui::theme::Theme;
pub(crate) fn render_satellite(
    module: &DashboardModule,
    visual: &crate::new_capsule::satellite::SatelliteVisual,
    radius: f32,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    use crate::new_capsule::satellite::SatelliteId;
    let body = match &visual.id {
        SatelliteId::Calendar => crate::new_capsule::widgets::satellite::calendar::render(
            (visual.layout.content_size.height - px(2.0)).max(px(0.0)),
            cx,
        )
        .into_any_element(),
        id => {
            let content = match id {
                SatelliteId::Wifi => connectivity::wifi(module, theme, cx),
                SatelliteId::Bluetooth => connectivity::bluetooth(module, theme, cx),
                SatelliteId::Audio => audio::audio(module, theme, cx),
                SatelliteId::Tray { bus, path } => tray::tray(bus, path, module, theme, cx),
                SatelliteId::Calendar => div().into_any_element(),
            };
            let title = match id {
                SatelliteId::Tray { bus, path } => module
                    .snapshot
                    .tray
                    .iter()
                    .find(|item| item.bus_name == *bus && item.object_path == *path)
                    .map(|item| item.title.clone())
                    .unwrap_or_else(|| module.text("dashboard_new.tray", cx)),
                _ => module.text(id.title_key(), cx),
            };
            let icon = match id {
                SatelliteId::Audio => "volume-2.svg",
                SatelliteId::Wifi => "wifi.svg",
                SatelliteId::Bluetooth => "bluetooth.svg",
                SatelliteId::Calendar => "calendar-days.svg",
                SatelliteId::Tray { .. } => "sparkles.svg",
            };
            let mut panel = div()
                .w(visual.layout.content_size.width)
                .max_h((visual.layout.content_size.height - px(2.0)).max(px(0.0)))
                .flex()
                .flex_col()
                .p(px(16.0))
                .gap(px(12.0))
                .font_family(theme.font_family())
                .text_color(theme.foreground())
                .text_size(px(13.0))
                .child(
                    div()
                        .h(px(28.0))
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(svg().path(icon).size(px(16.0)).text_color(theme.accent()))
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(title),
                                ),
                        )
                        .child(super::icon_action(
                            "satellite-close",
                            "close.svg",
                            super::DashboardAction::CloseSatellite(visual.id.clone()),
                            true,
                            theme,
                            cx,
                        )),
                );
            if let Some(message) = &module.navigation.error {
                panel = panel.child(error(message.clone(), theme));
            }
            panel.child(content).into_any_element()
        }
    };
    let measurement_entity = cx.entity().downgrade();
    let measurement_id = visual.id.clone();
    let body = div().relative().w_full().child(body).child(
        canvas(
            move |bounds, window, cx| {
                let entity = measurement_entity.clone();
                let id = measurement_id.clone();
                window.defer(cx, move |_, cx| {
                    let _ = entity.update(cx, |module, cx| {
                        module.measure_satellite(&id, bounds.size.height + px(2.0), cx);
                    });
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0(),
    );
    let bounds_entity = cx.entity().downgrade();
    let bounds_id = visual.id.clone();
    let height = module
        .satellite_heights
        .iter()
        .find(|(id, _)| *id == visual.id)
        .map(|(_, height)| *height)
        .unwrap_or(px(72.0))
        .min(visual.layout.bounds.size.height);
    crate::new_capsule::widgets::satellite::surface::render(
        visual.layout,
        visual.progress,
        visual.open,
        radius,
        body,
        theme,
    )
    .h(height)
    .max_h(visual.layout.bounds.size.height)
    .transitions(|t| t.h(module.snapshot.duration.with_easing(ease_in_out)))
    .child(
        canvas(
            move |bounds, window, cx| {
                let entity = bounds_entity.clone();
                let id = bounds_id.clone();
                let bounds = gpui::Bounds {
                    origin: bounds.origin - gpui::point(px(1.0), px(1.0)),
                    size: bounds.size + gpui::size(px(2.0), px(2.0)),
                };
                window.defer(cx, move |_, cx| {
                    let _ = entity.update(cx, |module, cx| {
                        if module.satellites.iter().any(|visual| visual.id == id) {
                            cx.emit(
                                crate::new_capsule::module::CapsuleModuleEvent::SatelliteBounds(
                                    id, bounds,
                                ),
                            );
                        }
                    });
                });
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0(),
    )
    .into_any_element()
}
pub(crate) fn list(id: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex_1()
        .min_h_0()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .overflow_y_scroll()
}
