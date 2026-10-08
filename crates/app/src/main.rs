mod capsule;
pub mod lockscreen;
pub mod new_capsule;
pub mod panel;
pub mod settings;

use assets::Assets;
use gpui_platform::application;

fn main() -> std::io::Result<()> {
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

    services::spawn_tokio(async {
        if let Err(err) = services::start_notification_server().await {
            eprintln!("D-Bus Notification Server warning: {err}");
        }
    });

    let app = application().with_assets(Assets {});

    app.run(|cx| {
        let font_data = assets::load_fonts();
        if let Err(err) = cx.text_system().add_fonts(font_data) {
            eprintln!("Failed to load fonts: {err}");
        }

        let app_state = services::AppState::new();
        cx.set_global(app_state);

        let theme_manager = ui::theme::theme_manager::ThemeManager::new();
        cx.set_global(theme_manager.current_theme.clone());
        cx.set_global(theme_manager);

        if cx
            .global::<services::AppState>()
            .config
            .get()
            .ui
            .use_new_capsule
        {
            panel::CapsulePanel::open_new(cx, ipc_subscriber);
        } else {
            panel::CapsulePanel::open(cx, ipc_subscriber);
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
    toggle-clipboard    Toggle clipboard history manager (alias: clipboard, clip)
    show-launcher       Show application launcher
    show-dashboard      Show main dashboard panel
    show-notification   Show notification panel
    show-clipboard      Show clipboard history manager
    lock                Lock screen (alias: lockscreen)
    terminal            Launch default terminal (alias: term)
    browser             Launch default browser (alias: web)
    editor              Launch default editor (alias: edit)
    settings            Open settings application (alias: config, preferences)
    toggle-record       Toggle screen recording module (alias: record)
    hide                Hide panels and return to compact pill (alias: close)
    quit                Stop running Capsule daemon (alias: exit)
    ping                Check if Capsule daemon is running
    help, --help, -h    Print this help message"#
    );
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::task::{Context, Poll, Wake};

    #[derive(Default)]
    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn poll_foreign_executor_channel() -> (usize, usize) {
        let counter = Arc::new(WakeCounter::default());
        let waker = counter.clone().into();
        let mut context = Context::from_waker(&waker);
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        for value in 0..512 {
            sender.send(value).expect("enqueue message");
        }

        let mut received = 0;
        for _ in 0..1024 {
            if let Poll::Ready(Some(_)) = receiver.poll_recv(&mut context) {
                received += 1;
            }
        }
        (received, counter.0.load(Ordering::Relaxed))
    }

    #[test]
    fn nested_event_loop_exhausts_tokio_budget_and_self_wakes() {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("runtime");
        let (received, wakes) = runtime.block_on(async { poll_foreign_executor_channel() });
        assert!(
            received < 512,
            "nested executor unexpectedly drained its channel"
        );
        assert!(
            wakes > 512,
            "expected repeated wakes without channel progress"
        );
        eprintln!("nested event loop: {received} messages received, {wakes} self-wakes");
    }

    #[test]
    fn application_runtime_keeps_channels_idle_without_starving_services() {
        super::run_with_runtime(|| {
            let (received, wakes) = poll_foreign_executor_channel();
            assert_eq!(received, 512);
            assert_eq!(wakes, 0, "an idle receiver must not reschedule itself");

            let (sender, receiver) = std::sync::mpsc::channel();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                sender.send(()).expect("report timer completion");
            });
            receiver
                .recv_timeout(std::time::Duration::from_secs(2))
                .expect("services must progress while the main thread runs GPUI");
        })
        .expect("application runtime");
    }
}
