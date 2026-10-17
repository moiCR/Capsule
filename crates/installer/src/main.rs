mod app;
mod deps;
mod github;
mod install;
mod panel;
mod style;
mod system;
mod views;

use anyhow::Result;
use app::InstallerApp;
use assets::Assets;
use gpui::{AppContext, Context, Entity, IntoElement, Render, Window};
use gpui_platform::application;
use panel::InstallerPanel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ui::theme::Theme;

struct InstallerRoot {
    app_entity: Entity<InstallerApp>,
}

impl Render for InstallerRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.app_entity.update(cx, |installer, cx| {
            views::render_installer(installer, window, cx)
        })
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut beta_mode = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "--beta" => beta_mode = true,
            "--help" | "-h" => {
                println!("Capsule Installer");
                println!("Usage: capsule-installer [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --beta       Install pre-release beta version");
                println!("  -h, --help   Print this help information");
                return Ok(());
            }
            other => {
                eprintln!("Unknown option: {}", other);
                eprintln!("Usage: capsule-installer [--beta]");
                std::process::exit(1);
            }
        }
    }

    if system::is_root() {
        eprintln!("Error: Please run capsule-installer as regular user, not root or sudo.");
        eprintln!("Sudo privileges will be requested when needed.");
        std::process::exit(1);
    }

    let shared_password: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sp_clone = Arc::clone(&shared_password);
    let keep_alive_running = Arc::new(AtomicBool::new(true));
    let r_clone = Arc::clone(&keep_alive_running);

    std::thread::spawn(move || {
        while r_clone.load(Ordering::Relaxed) {
            if let Ok(guard) = sp_clone.lock() {
                if let Some(pass) = guard.as_deref() {
                    system::refresh_sudo_with_password(pass);
                } else {
                    let _ = std::process::Command::new("sudo")
                        .arg("-n")
                        .arg("true")
                        .output();
                }
            }
            std::thread::sleep(Duration::from_secs(45));
        }
    });

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _runtime_context = runtime.enter();

    let app = application().with_assets(Assets {});

    app.run(move |cx| {
        let font_data = assets::load_fonts();
        if let Err(err) = cx.text_system().add_fonts(font_data) {
            eprintln!("Failed to load fonts: {err}");
        }

        let theme = Theme::default();
        cx.set_global(theme);

        let installer_entity = cx.new(|cx| InstallerApp::new(beta_mode, cx));

        let poll_entity = installer_entity.clone();
        let pass_ref = Arc::clone(&shared_password);
        cx.spawn(async move |cx| {
            loop {
                tokio::time::sleep(Duration::from_millis(50)).await;
                cx.update(|cx| {
                    poll_entity.update(cx, |app, cx| {
                        app.poll_worker(cx);
                        if let Some(pass) = &app.sudo_password
                            && let Ok(mut guard) = pass_ref.lock()
                            && guard.as_deref() != Some(pass.as_str())
                        {
                            *guard = Some(pass.clone());
                        }
                    });
                });
            }
        })
        .detach();

        let options = InstallerPanel::window_options(cx);
        let open_res = cx.open_window(options, |window, cx| {
            installer_entity.update(cx, |this, cx| {
                window.focus(&this.focus_handle, cx);
            });
            cx.new(|_| InstallerRoot {
                app_entity: installer_entity.clone(),
            })
        });

        if let Err(err) = open_res {
            eprintln!("LayerShell open failed ({err}), falling back to standard windowed mode...");
            let fallback_options = InstallerPanel::fallback_window_options();
            let _ = cx.open_window(fallback_options, |window, cx| {
                installer_entity.update(cx, |this, cx| {
                    window.focus(&this.focus_handle, cx);
                });
                cx.new(|_| InstallerRoot {
                    app_entity: installer_entity,
                })
            });
        }
    });

    keep_alive_running.store(false, Ordering::Relaxed);
    Ok(())
}
