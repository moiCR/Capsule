mod audio;
mod calendar;
mod connectivity;
mod tray;
use super::{BODY, DashboardView as View, error};
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{
    AnyElement, Context, IntoElement, MotionDurationExt, canvas, div, ease_in_out, prelude::*, px,
};
use ui::theme::Theme;
pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = div()
        .id("dashboard-detail")
        .h(px(BODY))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .overflow_hidden();
    if let Some(message) = &module.navigation.error {
        content = content.child(error(message.clone(), theme));
    }
    if module.pending {
        content = content.child(empty(module.text("dashboard_new.pending", cx), theme));
    }
    let body = match &module.navigation.view {
        View::Wifi => connectivity::wifi(module, theme, cx),
        View::Bluetooth => connectivity::bluetooth(module, theme, cx),
        View::Audio => audio::audio(module, theme, cx),
        View::Calendar => calendar::calendar(module, theme, cx),
        View::Media => crate::new_capsule::widgets::satellite::media::render(module, theme, cx),
        View::Tray { bus, path } => tray::tray(bus, path, module, theme, cx),
        View::Themes => div().into_any_element(),
        View::Wallpapers => div().into_any_element(),
        View::Home => div().into_any_element(),
    };
    content.child(body).into_any_element()
}

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
                SatelliteId::Media => {
                    crate::new_capsule::widgets::satellite::media::render(module, theme, cx)
                }
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
                _ => module.text(id.view().title_key(), cx),
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
                        .child(div().flex_1().min_w_0().text_ellipsis().child(title))
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
