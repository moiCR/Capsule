use gpui::{
    App, LinuxWindowBackground, WindowBounds, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    px,
};

pub struct InstallerPanel;

impl InstallerPanel {
    pub fn window_options(cx: &App) -> WindowOptions {
        let displays = cx.displays();
        let display = displays.first();
        let display_id = display.map(|d| d.id());

        WindowOptions {
            display_id,
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::default())),
            app_id: Some("capsule-installer".to_string()),
            inactive_frame_interval: None,
            linux_window_background: LinuxWindowBackground::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-installer".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
                exclusive_zone: Some(px(-1.0)),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn fallback_window_options() -> WindowOptions {
        WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds {
                origin: gpui::Point::default(),
                size: gpui::size(px(680.0), px(540.0)),
            })),
            app_id: Some("capsule-installer".to_string()),
            linux_window_background: LinuxWindowBackground::Transparent,
            ..Default::default()
        }
    }
}
