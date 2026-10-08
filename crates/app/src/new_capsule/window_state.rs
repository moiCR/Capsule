use gpui::{AnyWindowHandle, Bounds, Context, Pixels, Size, Window, point, px};
use services::AppState;

use super::{Capsule, CapsuleLocation, module::CapsuleModuleId};

pub(super) struct WindowState {
    pub handle: AnyWindowHandle,
    pub margin: f32,
    pub radius: f32,
    region: Option<Bounds<Pixels>>,
    requested_size: Option<Size<Pixels>>,
    exclusive_zone: Option<Pixels>,
    current_module: Option<CapsuleModuleId>,
}

impl WindowState {
    pub fn new(window: &Window) -> Self {
        Self {
            handle: window.window_handle(),
            margin: 0.0,
            radius: 0.0,
            region: None,
            requested_size: None,
            exclusive_zone: None,
            current_module: None,
        }
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
    pub(super) fn sync_window(&mut self, cx: &mut Context<Self>) {
        let handle = self.window_state.handle;
        let _ = handle.update(cx, |_, window, cx| self.apply_window_geometry(window, cx));
    }

    pub(super) fn apply_window_geometry(&mut self, window: &mut Window, cx: &mut gpui::App) {
        let current = self.module_manager.current_id();
        if self.window_state.current_module != Some(current) {
            self.window_state.current_module = Some(current);
            let keyboard = if current != CapsuleModuleId::Default {
                gpui::layer_shell::KeyboardInteractivity::Exclusive
            } else {
                gpui::layer_shell::KeyboardInteractivity::OnDemand
            };
            window.set_keyboard_interactivity(keyboard);
            if current == CapsuleModuleId::Launcher {
                window.focus(&self.module_manager.launcher_focus_handle(cx), cx);
            } else if current == CapsuleModuleId::Dashboard {
                window.focus(&self.module_manager.dashboard_focus_handle(cx), cx);
            } else {
                window.blur(cx);
            }
        }
        let config = cx.global::<AppState>().config.get();
        let margin = config.ui.margin_top.max(0.0);
        let radius = config.ui.capsule_round;
        let exclusive_zone =
            super::widgets::default::module_size(config.ui.idle_height).height + px(margin);
        if self.window_state.exclusive_zone != Some(exclusive_zone) {
            self.window_state.exclusive_zone = Some(exclusive_zone);
            window.set_exclusive_zone(exclusive_zone);
        }
        let mut maximum = self.module_manager.maximum_size(cx);
        maximum.width *= 1.1;
        maximum.height *= 1.1;
        maximum.height += px(margin);
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
        if self.window_state.region != Some(region) {
            self.window_state.region = Some(region);
            window.set_input_region(Some(&[region]));
            window.refresh();
        }
        if self.window_state.margin != margin || self.window_state.radius != radius {
            self.window_state.margin = margin;
            self.window_state.radius = radius;
            window.refresh();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::size;

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
