mod app;
mod github;
mod install;
mod system;
mod ui;

use anyhow::Result;
use app::{App, Step};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
        eprintln!("Sudo password will be prompted within the TUI if needed.");
        std::process::exit(1);
    }

    // Shared password storage for background sudo keep-alive
    let shared_password: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let sp_clone = Arc::clone(&shared_password);
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = Arc::clone(&running);

    std::thread::spawn(move || {
        while r_clone.load(Ordering::Relaxed) {
            if let Ok(guard) = sp_clone.lock() {
                if let Some(pass) = guard.as_deref() {
                    system::refresh_sudo_with_password(pass);
                } else {
                    let _ = Command::new("sudo").arg("-n").arg("true").output();
                }
            }
            std::thread::sleep(Duration::from_secs(45));
        }
    });

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(beta_mode);

    let res = run_loop(&mut terminal, &mut app, &shared_password);

    running.store(false, Ordering::Relaxed);
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show)?;
    terminal.show_cursor()?;

    if let Err(e) = res {
        eprintln!("Installer error: {:#}", e);
    } else if app.step == Step::Finished && app.launch_on_finish {
        println!("==> Starting Capsule daemon in background...");
        let _ = install::launch_capsule();
        std::thread::sleep(Duration::from_millis(800));
        if system::is_capsule_running() {
            println!("==> Capsule is now running!");
        } else {
            println!("==> Note: Capsule can be started anytime via 'capsule' command.");
        }
    }

    Ok(())
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    shared_password: &Arc<Mutex<Option<String>>>,
) -> Result<()> {
    while !app.should_quit {
        app.poll_worker();

        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Handle password modal input first
                if app.show_password_modal {
                    match key.code {
                        KeyCode::Char(c) => {
                            app.password_input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.password_input.pop();
                        }
                        KeyCode::Enter => {
                            if app.submit_password() {
                                if let Ok(mut guard) = shared_password.lock() {
                                    *guard = app.sudo_password.clone();
                                }
                                if let Some(action) = app.pending_action.take() {
                                    match action {
                                        app::PendingAction::InstallCapsule => {
                                            app.start_installation();
                                        }
                                        app::PendingAction::InstallDeps => {
                                            run_deps_installation(terminal, app)?;
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.show_password_modal = false;
                            app.pending_action = None;
                            app.password_input.clear();
                            app.password_error = None;
                        }
                        _ => {}
                    }
                    continue;
                }

                match app.step {
                    Step::Welcome => handle_welcome_key(app, key.code),
                    Step::Dependencies => handle_deps_key(terminal, app, key.code)?,
                    Step::Installing => {
                        // Background installation running
                    }
                    Step::Finished => handle_finished_key(app, key.code),
                }
            }
        }
    }

    Ok(())
}

fn handle_welcome_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            } else {
                app.selected_index = 2;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < 2 {
                app.selected_index += 1;
            } else {
                app.selected_index = 0;
            }
        }
        KeyCode::Char('b') => {
            app.toggle_beta();
        }
        KeyCode::Enter => match app.selected_index {
            0 => {
                app.selected_index = 0;
                app.step = Step::Dependencies;
            }
            1 => {
                app.toggle_beta();
            }
            2 => {
                app.should_quit = true;
            }
            _ => {}
        },
        KeyCode::Esc | KeyCode::Char('q') => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_deps_key<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    code: KeyCode,
) -> Result<()> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            } else {
                app.selected_index = 2;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.selected_index < 2 {
                app.selected_index += 1;
            } else {
                app.selected_index = 0;
            }
        }
        KeyCode::Enter => match app.selected_index {
            0 => {
                let missing = app.missing_dependencies();
                if !missing.is_empty() {
                    if app.request_auth_or_proceed(app::PendingAction::InstallDeps) {
                        run_deps_installation(terminal, app)?;
                    }
                } else {
                    app.scan_dependencies();
                }
            }
            1 => {
                if app.request_auth_or_proceed(app::PendingAction::InstallCapsule) {
                    app.selected_index = 0;
                    app.start_installation();
                }
            }
            2 => {
                app.selected_index = 0;
                app.step = Step::Welcome;
            }
            _ => {}
        },
        KeyCode::Esc => {
            app.selected_index = 0;
            app.step = Step::Welcome;
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        _ => {}
    }
    Ok(())
}

fn run_deps_installation<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let missing = app.missing_dependencies();
    if missing.is_empty() {
        return Ok(());
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, crossterm::cursor::Show)?;

    println!("\n==> Installing missing packages: {}", missing.join(" "));

    if let Some(pass) = &app.sudo_password {
        system::refresh_sudo_with_password(pass);
    }

    let cmd_name = match &app.aur_helper {
        Some(system::AurHelper::Paru) => "paru",
        Some(system::AurHelper::Yay) => "yay",
        _ => "sudo",
    };

    let mut cmd = if cmd_name == "sudo" {
        let mut c = Command::new("sudo");
        c.args(["pacman", "-S", "--needed"]);
        c.args(&missing);
        c
    } else {
        let mut c = Command::new(cmd_name);
        c.args(["-S", "--needed"]);
        c.args(&missing);
        c
    };

    let status = cmd.status();
    if let Ok(s) = status {
        if !s.success() {
            println!("\nPackage installation reported an error.");
        }
    }

    println!("\nPress Enter to return to installer...");
    let mut dummy = String::new();
    let _ = std::io::stdin().read_line(&mut dummy);

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen, crossterm::cursor::Hide)?;
    terminal.clear()?;

    app.scan_dependencies();
    Ok(())
}

fn handle_finished_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Down | KeyCode::Char('k') | KeyCode::Char('j') => {
            app.selected_index = if app.selected_index == 0 { 1 } else { 0 };
        }
        KeyCode::Char(' ') => {
            app.launch_on_finish = !app.launch_on_finish;
        }
        KeyCode::Enter => {
            if app.selected_index == 0 {
                app.launch_on_finish = !app.launch_on_finish;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}
