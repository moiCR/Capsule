use gpui::{
    App, Bounds, LinuxWindowBackground, TitlebarOptions, WindowBounds, WindowDecorations,
    WindowOptions, px, size,
};

pub struct InstallerPanel;

impl InstallerPanel {
    pub fn window_options(cx: &App) -> WindowOptions {
        let window_size = size(px(880.0), px(580.0));
        let bounds = Bounds::centered(None, window_size, cx);

        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some("Capsule Installer".into()),
                appears_transparent: false,
                ..Default::default()
            }),
            window_decorations: Some(WindowDecorations::Server),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            app_id: Some("capsule-installer".to_string()),
            linux_window_background: LinuxWindowBackground::Opaque,
            is_resizable: true,
            is_minimizable: true,
            is_movable: true,
            window_min_size: Some(size(px(800.0), px(520.0))),
            ..Default::default()
        }
    }
}
