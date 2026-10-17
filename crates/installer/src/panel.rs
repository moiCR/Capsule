use gpui::{App, Bounds, LinuxWindowBackground, WindowBounds, WindowOptions, px, size};

pub struct InstallerPanel;

impl InstallerPanel {
    pub fn window_options(cx: &App) -> WindowOptions {
        let window_size = size(px(720.0), px(560.0));
        let bounds = Bounds::centered(None, window_size, cx);

        WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            app_id: Some("capsule-installer".to_string()),
            linux_window_background: LinuxWindowBackground::Opaque,
            ..Default::default()
        }
    }
}
