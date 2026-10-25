pub mod animator;
pub mod bar;
pub mod island;
pub use island as capsule;
pub mod lockscreen;
pub mod panel;
pub mod settings;
pub mod wallpaper;

use assets::Assets;
use gpui_platform::application;

fn main() -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    unsafe {
        // Limit glibc per-thread memory arenas to 2 (M_ARENA_MAX = -8 in glibc)
        // This eliminates multi-threaded allocator bloat when background threads decode images.
        libc::mallopt(-8, 2);
    }
    run_with_runtime(run_application)
}

fn run_with_runtime(run: impl FnOnce()) -> std::io::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _runtime_context = runtime.enter();
    run();
    Ok(())
}

fn run_application() {
    services::init_tokio_handle(tokio::runtime::Handle::current());

    #[cfg(not(target_os = "linux"))]
    compile_error!("This application is only supported on Linux.");

    if let Some(home) = dirs::home_dir() {
        let _ = std::env::set_current_dir(home);
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
        if normalized == "--version" || normalized == "-v" || normalized == "version" {
            println!("Capsule {}", env!("CARGO_PKG_VERSION"));
            return;
        }

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

    // Synchronize cursor theme & size so GPUI and all launched apps match compositor theme
    if std::env::var("XCURSOR_THEME")
        .map(|t| t.is_empty() || t == "default")
        .unwrap_or(true)
    {
        if let Some(home) = dirs::home_dir() {
            let mut detected = None;
            let gtk_ini = home.join(".config/gtk-3.0/settings.ini");
            if let Ok(content) = std::fs::read_to_string(&gtk_ini) {
                for line in content.lines() {
                    if let Some(theme) = line.strip_prefix("gtk-cursor-theme-name=") {
                        let trimmed = theme.trim().trim_matches('"');
                        if !trimmed.is_empty() {
                            detected = Some(trimmed.to_string());
                            break;
                        }
                    }
                }
            }

            if detected.is_none() {
                let kcminput = home.join(".config/kcminputrc");
                if let Ok(content) = std::fs::read_to_string(&kcminput) {
                    for line in content.lines() {
                        if let Some(theme) = line.strip_prefix("cursorTheme=") {
                            let trimmed = theme.trim().trim_matches('"');
                            if !trimmed.is_empty() {
                                detected = Some(trimmed.to_string());
                                break;
                            }
                        }
                    }
                }
            }

            let theme = detected.unwrap_or_else(|| "breeze_cursors".to_string());

            unsafe {
                std::env::set_var("XCURSOR_THEME", &theme);
            }
        }
    }
    if std::env::var("XCURSOR_SIZE").is_err() {
        unsafe {
            std::env::set_var("XCURSOR_SIZE", "24");
        }
    }

    if let Some(home) = dirs::home_dir() {
        let default_icons = home.join(".icons/default");
        let index_theme = default_icons.join("index.theme");
        if !index_theme.exists() {
            let _ = std::fs::create_dir_all(&default_icons);
            let theme =
                std::env::var("XCURSOR_THEME").unwrap_or_else(|_| "breeze_cursors".to_string());
            let _ = std::fs::write(
                &index_theme,
                format!(
                    "[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits={theme}\n"
                ),
            );
        }
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        let _ = std::process::Command::new("dbus-update-activation-environment")
            .args([
                "--systemd",
                "WAYLAND_DISPLAY",
                "DISPLAY",
                "XAUTHORITY",
                "XDG_CURRENT_DESKTOP",
                "XDG_SESSION_TYPE",
                "XCURSOR_THEME",
                "XCURSOR_SIZE",
            ])
            .spawn();
    }

    services::spawn_tokio(async {
        if let Err(err) = services::start_notification_server().await {
            eprintln!("D-Bus Notification Server warning: {err}");
        }
    });

    let app = application().with_assets(Assets {});

    app.run(|cx| {
        cx.set_quit_mode(gpui::QuitMode::Explicit);
        let font_data = assets::load_fonts();
        if let Err(err) = cx.text_system().add_fonts(font_data) {
            eprintln!("Failed to load fonts: {err}");
        }

        let app_state = services::AppState::new();
        cx.set_global(app_state);

        let mut theme_manager = ui::theme::theme_manager::ThemeManager::new();
        theme_manager.current_theme = theme_manager
            .selected_theme
            .clone()
            .with_mode(cx.global::<services::AppState>().config.get().ui.dark_mode);
        cx.set_global(theme_manager.current_theme.clone());
        cx.set_global(theme_manager);
        settings::appearance::initialize(cx);

        let mut ipc_subscriber = ipc_subscriber;
        ipc_subscriber.start_listener();

        wallpaper::WallpaperPanel::open(cx);
        panel::CapsulePanel::open(cx);
    });
}

const CONTENT: &str = include_str!("command_usage.txt");

fn print_help() {
    println!("{CONTENT}");
}
