mod capsule;

use assets::Assets;
use capsule::capsule::Capsule;
use gpui::{
    AppContext, Bounds, Size, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    layer_shell::{Anchor, KeyboardInteractivity, Layer, LayerShellOptions},
    point, px,
};
use gpui_platform::application;

unsafe fn daemonize() {
    unsafe {
        // Create a new session so the process is no longer associated with
        // any controlling terminal. This makes it immune to SIGHUP and the
        // SIGTERM sent when the terminal session ends.
        libc::setsid();

        // Also ignore SIGHUP and SIGPIPE defensively.
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

#[tokio::main]
async fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("This application is only supported on Linux.");

    // Detach from the controlling terminal so `Capsule &` survives
    // when the launching shell/terminal exits.
    unsafe {
        daemonize();
    }

    services::init_logger();

    let args: Vec<String> = std::env::args().collect();
    let cmd_arg = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    if let Some(ref raw) = cmd_arg {
        let normalized = raw.trim().to_lowercase();
        if normalized == "--help" || normalized == "-h" || normalized == "help" {
            print_help();
            return;
        }

        if services::decode_command(raw).is_none() {
            eprintln!("Error: Unknown command '{raw}'.");
            eprintln!("Run 'Capsule --help' for a list of available commands.");
            std::process::exit(1);
        }
    }

    // Initialize IPC subscriber (single-instance handling).
    // If a primary instance is already running, this signals it synchronously and returns None (exiting cleanly).
    let ipc_subscriber = match services::IpcSubscriber::init(cmd_arg.as_deref()) {
        Some(sub) => sub,
        None => return,
    };

    tokio::spawn(async {
        if let Err(err) = services::start_notification_server().await {
            eprintln!("D-Bus Notification Server warning: {err}");
        }
    });

    let app = application().with_assets(Assets {});

    app.run(|cx| {
        // Initialize all singleton services once and register as GPUI globals.
        let app_state = services::AppState::new();
        cx.set_global(app_state);

        let theme_manager = ui::theme::theme_manager::ThemeManager::new();
        cx.set_global(theme_manager.current_theme.clone());
        cx.set_global(theme_manager);

        let max_w = 3840.0;
        let max_h = 2160.0;
        let idle_h = 42.0 + 8.0;

        let options = WindowOptions {
            titlebar: None,
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.0), px(0.0)),
                size: Size::new(px(max_w), px(max_h)),
            })),
            app_id: Some("capsule-panel".to_string()),
            window_background: WindowBackgroundAppearance::Transparent,
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "capsule-panel".to_string(),
                layer: Layer::Top,
                anchor: Anchor::TOP,
                margin: Some((px(8.0), px(0.0), px(0.0), px(0.0))),
                exclusive_zone: Some(px(idle_h)),
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                ..Default::default()
            }),
            ..Default::default()
        };

        let window = match cx.open_window(options, |_, cx| cx.new(Capsule::new)) {
            Ok(w) => w,
            Err(err) => {
                eprintln!("Failed to open layer shell window: {err}");
                return;
            }
        };

        if let Ok(capsule_handle) = window.entity(cx) {
            ipc_subscriber.start(cx, capsule_handle, Capsule::handle_ipc_command);
        }
    });
}

fn print_help() {
    println!(
        r#"Capsule - Dynamic Island & Dashboard for Wayland

USAGE:
    Capsule [COMMAND]

COMMANDS:
    (no args)           Start Capsule daemon process
    toggle-launcher     Toggle application launcher (alias: launcher)
    toggle-dashboard    Toggle main dashboard panel (alias: dashboard)
    toggle-notification Toggle notification panel (alias: notifications)
    show-launcher       Show application launcher
    show-dashboard      Show main dashboard panel
    show-notification   Show notification panel
    hide                Hide panels and return to compact pill (alias: close)
    quit                Stop running Capsule daemon (alias: exit)
    ping                Check if Capsule daemon is running
    help, --help, -h    Print this help message"#
    );
}
