mod capsule;
pub mod panel;
pub mod lockscreen;

use assets::Assets;
use gpui_platform::application;

unsafe fn daemonize() {
    unsafe {
        libc::setsid();
        libc::signal(libc::SIGHUP, libc::SIG_IGN);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

#[tokio::main]
async fn main() {
    #[cfg(not(target_os = "linux"))]
    compile_error!("This application is only supported on Linux.");
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
        let font_data = assets::load_fonts();
        if let Err(err) = cx.text_system().add_fonts(font_data) {
            eprintln!("Failed to load Geist fonts: {err}");
        }

        let app_state = services::AppState::new();
        cx.set_global(app_state);

        let theme_manager = ui::theme::theme_manager::ThemeManager::new();
        cx.set_global(theme_manager.current_theme.clone());
        cx.set_global(theme_manager);

        let lang_manager = ui::language::language_manager::LanguageManager::new();
        cx.set_global(lang_manager.current_language.clone());
        cx.set_global(lang_manager);

        panel::CapsulePanel::open(cx, ipc_subscriber);
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
    toggle-clipboard    Toggle clipboard history manager (alias: clipboard, clip)
    show-launcher       Show application launcher
    show-dashboard      Show main dashboard panel
    show-notification   Show notification panel
    show-clipboard      Show clipboard history manager
    lock                Lock screen (alias: lockscreen)
    hide                Hide panels and return to compact pill (alias: close)
    quit                Stop running Capsule daemon (alias: exit)
    ping                Check if Capsule daemon is running
    help, --help, -h    Print this help message"#
    );
}
