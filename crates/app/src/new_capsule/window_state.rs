use gpui::{AnyWindowHandle, Bounds, Context, Pixels, Size, Window, point, px};
use services::{AppState, CapsuleStyle};

use super::{Capsule, CapsuleLocation, module::CapsuleModuleId};

pub(super) struct WindowState {
    pub handle: AnyWindowHandle,
    pub margin: f32,
    pub radius: f32,
    pub style: CapsuleStyle,
    region: Option<Vec<Bounds<Pixels>>>,
    requested_size: Option<Size<Pixels>>,
    exclusive_zone: Option<Pixels>,
    current_module: Option<CapsuleModuleId>,
    satellite_open: bool,
    notification_popup: bool,
    pub satellite_bounds: Vec<(super::satellite::SatelliteId, Bounds<Pixels>)>,
}

impl WindowState {
    pub fn new(window: &Window) -> Self {
        Self {
            handle: window.window_handle(),
            margin: 0.0,
            radius: 0.0,
            style: CapsuleStyle::Normal,
            region: None,
            requested_size: None,
            exclusive_zone: None,
            current_module: None,
            satellite_open: false,
            notification_popup: false,
            satellite_bounds: Vec::new(),
        }
    }
}

fn effective_margin(style: CapsuleStyle, margin: f32) -> f32 {
    if style == CapsuleStyle::Concave {
        0.0
    } else {
        margin.max(0.0)
    }
}

pub(super) fn visible_bounds(
    viewport: Size<Pixels>,
    size: Size<Pixels>,
    margin: Pixels,
    location: &CapsuleLocation,
) -> Bounds<Pixels> {
    let y = match location {
        CapsuleLocation::TOP => margin,
        CapsuleLocation::BOTTOM => viewport.height - margin - size.height,
    };
    Bounds {
        origin: point((viewport.width - size.width) / 2.0, y),
        size,
    }
}

impl Capsule {
    pub(crate) fn maximum_window_size(
        module_manager: &super::module::CapsuleModuleManager,
        available: Option<Size<Pixels>>,
        cx: &gpui::App,
    ) -> Size<Pixels> {
        let config = cx.global::<AppState>().config.get();
        let margin = px(effective_margin(
            config.ui.capsule_style,
            config.ui.margin_top,
        ));
        let gap = super::orbit::Orbit::satellite_gap(px(config.ui.gap));
        let mut maximum = module_manager.maximum_size(cx);
        maximum.width *= 1.1;
        if config.ui.capsule_style == CapsuleStyle::Concave {
            maximum.width += px(24.0);
        }
        maximum.height *= 1.1;
        maximum.height += margin;
        let satellite = super::satellite::Satellite::maximum_size();
        maximum.width += (satellite.width + gap) * 2.0;
        maximum.height = maximum.height.max(
            satellite.height
                + margin
                + super::widgets::default::module_size(config.ui.idle_height).height
                + gap,
        );
        if let Some(available) = available {
            maximum.width = maximum.width.min(available.width);
            if maximum.width < module_manager.maximum_size(cx).width + (satellite.width + gap) * 2.0
            {
                maximum.height += satellite.height + gap;
            }
            maximum.height = maximum.height.min(available.height);
        }
        maximum
    }

    pub(super) fn sync_window(&mut self, cx: &mut Context<Self>) {
        let handle = self.window_state.handle;
        let _ = handle.update(cx, |_, window, cx| self.apply_window_geometry(window, cx));
    }

    pub(super) fn apply_window_geometry(&mut self, window: &mut Window, cx: &mut gpui::App) {
        let current = self.module_manager.current_id();
        let satellite_open = self.satellite.open();
        let passive_notification = current == CapsuleModuleId::Notification
            && !self.module_manager.notification.read(cx).history;
        if self.window_state.current_module != Some(current)
            || self.window_state.satellite_open != satellite_open
            || self.window_state.notification_popup != passive_notification
        {
            self.window_state.current_module = Some(current);
            self.window_state.satellite_open = satellite_open;
            self.window_state.notification_popup = passive_notification;
            let keyboard = if !passive_notification
                && !matches!(
                    current,
                    CapsuleModuleId::Default | CapsuleModuleId::Shelf | CapsuleModuleId::ShelfDrop
                )
                || satellite_open
            {
                gpui::layer_shell::KeyboardInteractivity::Exclusive
            } else {
                gpui::layer_shell::KeyboardInteractivity::OnDemand
            };

            window.set_keyboard_interactivity(keyboard);
            if satellite_open {
                window.focus(&self.module_manager.dashboard_focus_handle(cx), cx);
            } else if current == CapsuleModuleId::Launcher {
                window.focus(&self.module_manager.launcher_focus_handle(cx), cx);
            } else if current == CapsuleModuleId::Dashboard {
                window.focus(&self.module_manager.dashboard_focus_handle(cx), cx);
            } else if current == CapsuleModuleId::Notification {
                let focus = self.module_manager.notification.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if current == CapsuleModuleId::Polkit {
                let focus = self.module_manager.polkit.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if current == CapsuleModuleId::Shelf {
                let focus = self.module_manager.shelf.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if current == CapsuleModuleId::Record {
                let focus = self.module_manager.record.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if current == CapsuleModuleId::Clipboard {
                let focus = self.module_manager.clipboard.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if current == CapsuleModuleId::Emoji {
                let focus = self.module_manager.emoji.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else if matches!(
                current,
                CapsuleModuleId::Themes | CapsuleModuleId::Wallpapers
            ) {
                let focus = self.module_manager.appearance.read(cx).focus_handle();
                window.focus(&focus, cx);
            } else {
                window.blur(cx);
            }
        }

        let config = cx.global::<AppState>().config.get();
        let style = config.ui.capsule_style;
        let margin = effective_margin(style, config.ui.margin_top);
        let radius = config.ui.capsule_round;

        let exclusive_zone =
            super::widgets::default::module_size(config.ui.idle_height).height + px(margin);
        if self.window_state.exclusive_zone != Some(exclusive_zone) {
            self.window_state.exclusive_zone = Some(exclusive_zone);
            window.set_exclusive_zone(exclusive_zone);
        }

        let available = window.display(cx).map(|display| display.bounds().size);
        let maximum = Self::maximum_window_size(&self.module_manager, available, cx);

        if self.window_state.requested_size != Some(maximum) {
            self.window_state.requested_size = Some(maximum);
            window.resize(maximum);
        }

        let region = visible_bounds(
            window.viewport_size(),
            self.animator.size(),
            px(margin),
            &self.location,
        );

        let mut regions = if style == CapsuleStyle::Concave {
            super::widgets::style::concave_input_regions(
                region,
                radius,
                matches!(self.location, CapsuleLocation::BOTTOM),
            )
        } else {
            vec![region]
        };

        regions.extend(
            self.orbit
                .read(cx)
                .layout(window.viewport_size(), region, px(config.ui.gap))
                .filter(|visual| visual.interactive)
                .map(|visual| visual.bounds),
        );

        let mut visual = if current == CapsuleModuleId::Dashboard {
            self.satellite.layout(
                window.viewport_size(),
                region,
                super::orbit::Orbit::satellite_gap(px(config.ui.gap)),
                &self.window_state.satellite_bounds,
            )
        } else {
            Vec::new()
        };

        self.window_state
            .satellite_bounds
            .retain(|(id, _)| self.satellite.contains(id));
        for visual in &mut visual {
            regions.push(
                self.window_state
                    .satellite_bounds
                    .iter()
                    .find(|(id, _)| *id == visual.id)
                    .map(|(_, bounds)| *bounds)
                    .unwrap_or(visual.layout.bounds),
            );
            visual.layout.bounds.origin -= region.origin;
        }

        self.module_manager.notification.update(cx, |module, _| {
            module.set_active(current == CapsuleModuleId::Notification);
        });

        self.module_manager.shelf.update(cx, |module, _| {
            module.set_active(current == CapsuleModuleId::Shelf)
        });

        self.module_manager.record.update(cx, |module, _| {
            module.set_active(current == CapsuleModuleId::Record)
        });
        self.module_manager.clipboard.update(cx, |module, _| {
            module.set_active(current == CapsuleModuleId::Clipboard);
        });
        self.module_manager.appearance.update(cx, |module, _| {
            module.set_active(matches!(
                current,
                CapsuleModuleId::Themes | CapsuleModuleId::Wallpapers
            ));
        });
        self.module_manager.dashboard.update(cx, |dashboard, cx| {
            dashboard.set_active(current == CapsuleModuleId::Dashboard, region.size, cx);
            dashboard.set_satellites(visual, cx);
        });
        if self.window_state.region.as_ref() != Some(&regions) {
            window.set_input_region(Some(&regions));
            self.window_state.region = Some(regions);
            window.refresh();
        }
        if self.window_state.margin != margin
            || self.window_state.radius != radius
            || self.window_state.style != style
        {
            self.window_state.margin = margin;
            self.window_state.radius = radius;
            self.window_state.style = style;
            window.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

    #[test]
    fn concave_attaches_to_the_edge_without_losing_normal_margin() {
        assert_eq!(effective_margin(CapsuleStyle::Concave, 24.0), 0.0);
        assert_eq!(effective_margin(CapsuleStyle::Normal, 24.0), 24.0);
        assert_eq!(effective_margin(CapsuleStyle::Normal, -24.0), 0.0);
    }

    #[test]
    fn centered_bounds_keep_the_screen_edge_fixed_during_growth() {
        let viewport = size(px(400.0), px(308.0));
        for height in [16.0, 25.0, 60.0, 300.0] {
            let dimensions = size(px(172.0), px(height));
            let top = visible_bounds(viewport, dimensions, px(8.0), &CapsuleLocation::TOP);
            assert_eq!(top.origin, point(px(114.0), px(8.0)));
            let bottom = visible_bounds(viewport, dimensions, px(8.0), &CapsuleLocation::BOTTOM);
            assert_eq!(bottom.origin.y + bottom.size.height, px(300.0));
        }
    }
}
