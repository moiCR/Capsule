mod capsule;

use assets::Assets;
use capsule::capsule::Capsule;
use gpui::{
    AppContext, Bounds, Size, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    point, px,
};
use gpui_platform::application;

#[tokio::main]
async fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("This application is only supported on Linux.");

    // Start D-Bus notification server in background
    tokio::spawn(async {
        if let Err(err) = services::start_notification_server().await {
            eprintln!("D-Bus Notification Server warning: {err}");
        }
    });

    let app = application().with_assets(Assets {});

    app.run(|cx| {
        let theme_manager = ui::theme::theme_manager::ThemeManager::new();
        cx.set_global(theme_manager.current_theme.clone());

        let max_w = 1200.0;
        let idle_h = 42.0 + 8.0; // 50.0px

        let options = WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.0), px(0.0)),
                size: Size::new(px(max_w), px(idle_h)),
            })),
            app_id: Some("capsule-panel".to_string()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP,
                margin: Some((px(8.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(px(idle_h)),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            ..Default::default()
        };

        if let Err(err) = cx.open_window(options, |_, cx| cx.new(Capsule::new)) {
            eprintln!("Failed to open layer shell window: {err}");
        }
    });
}
