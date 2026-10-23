use crate::compositor::{Compositor, CompositorManager};
use crate::deps::DependsManager;
use crate::github::{self, ReleaseInfo};
use crate::install;
use crate::system;
use gpui::{Context, FocusHandle, ScrollHandle};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use ui::components::InputState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Dependencies,
    Compositor,
    Installing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    InstallCapsule,
    InstallDeps,
    InstallCompositor,
}

pub enum WorkerMsg {
    Progress(f32, String),
    Log(String),
    Done(Result<(), String>),
    DepsDone(Result<(), String>),
    CompositorDone(Result<(), String>),
}

pub struct InstallerApp {
    pub step: Step,
    pub beta_mode: bool,
    pub target: &'static str,
    pub installed_version: Option<String>,
    pub target_version: Option<String>,
    pub release_info: Option<ReleaseInfo>,
    pub depends_manager: DependsManager,
    pub dependencies: Vec<(String, bool)>,
    pub compositor_manager: CompositorManager,
    pub compositors: Vec<(Compositor, bool)>,
    pub selected_compositor: Compositor,
    pub capsule_was_running: bool,
    pub progress: f32,
    pub status_message: String,
    pub logs: Vec<String>,
    pub launch_on_finish: bool,
    pub error_message: Option<String>,

    pub sudo_password: Option<String>,
    pub show_password_modal: bool,
    pub password_input: InputState,
    pub password_error: Option<String>,
    pub pending_action: Option<PendingAction>,

    pub focus_handle: FocusHandle,
    pub logs_scroll_handle: ScrollHandle,
    pub deps_scroll_handle: ScrollHandle,

    pub worker_tx: Sender<WorkerMsg>,
    pub worker_rx: Receiver<WorkerMsg>,
}

impl InstallerApp {
    pub fn new(beta_mode: bool, cx: &mut Context<Self>) -> Self {
        let target = system::detect_target();
        let depends_manager = DependsManager::detect();
        let compositor_manager = CompositorManager::detect();
        let installed_version = system::get_installed_capsule_version();
        let capsule_was_running = system::is_capsule_running();
        let focus_handle = cx.focus_handle();
        let logs_scroll_handle = ScrollHandle::new();
        let deps_scroll_handle = ScrollHandle::new();
        let (worker_tx, worker_rx) = channel::<WorkerMsg>();

        let mut app = Self {
            step: Step::Welcome,
            beta_mode,
            target,
            installed_version,
            target_version: None,
            release_info: None,
            depends_manager,
            dependencies: Vec::new(),
            compositor_manager,
            compositors: Vec::new(),
            selected_compositor: Compositor::Hyprland,
            capsule_was_running,
            progress: 0.0,
            status_message: "Ready".to_string(),
            logs: Vec::new(),
            launch_on_finish: true,
            error_message: None,

            sudo_password: None,
            show_password_modal: false,
            password_input: InputState::new("").with_masked(true).with_focused(true),
            password_error: None,
            pending_action: None,

            focus_handle,
            logs_scroll_handle,
            deps_scroll_handle,
            worker_tx,
            worker_rx,
        };

        app.scan_dependencies();
        app.scan_compositors();
        if let Some((comp, _)) = app.compositors.iter().find(|(_, inst)| *inst) {
            app.selected_compositor = *comp;
        }
        app.refresh_release_info();
        app
    }

    pub fn scan_dependencies(&mut self) {
        self.dependencies = self.depends_manager.scan_dependencies();
    }

    pub fn scan_compositors(&mut self) {
        self.compositors = self.compositor_manager.scan_compositors();
    }

    pub fn is_compositor_installed(&self, comp: Compositor) -> bool {
        self.compositors
            .iter()
            .find(|(c, _)| *c == comp)
            .map(|(_, inst)| *inst)
            .unwrap_or_else(|| comp.is_installed())
    }

    pub fn missing_dependencies_count(&self) -> usize {
        self.dependencies.iter().filter(|(_, inst)| !*inst).count()
    }

    pub fn missing_dependencies(&self) -> Vec<String> {
        self.dependencies
            .iter()
            .filter(|(_, inst)| !*inst)
            .map(|(pkg, _)| pkg.clone())
            .collect()
    }

    pub fn refresh_release_info(&mut self) {
        match github::fetch_release_info(self.target, self.beta_mode) {
            Ok(info) => {
                self.target_version = Some(info.version.clone());
                self.release_info = Some(info);
                self.error_message = None;
            }
            Err(e) => {
                self.logs.push(format!("Notice: {}", e));
                self.target_version = Some("latest".to_string());
            }
        }
    }

    pub fn toggle_beta(&mut self, cx: &mut Context<Self>) {
        self.beta_mode = !self.beta_mode;
        self.refresh_release_info();
        cx.notify();
    }

    pub fn request_auth_or_proceed(&mut self, action: PendingAction) -> bool {
        if self.sudo_password.is_some() || system::check_sudo() {
            true
        } else {
            self.pending_action = Some(action);
            self.show_password_modal = true;
            self.password_error = None;
            self.password_input.clear();
            false
        }
    }

    pub fn submit_password(&mut self) -> bool {
        let pass = self.password_input.text();
        if system::verify_sudo_password(&pass) {
            system::refresh_sudo_with_password(&pass);
            self.sudo_password = Some(pass);
            self.show_password_modal = false;
            self.password_error = None;
            self.password_input.clear();
            true
        } else {
            self.password_error = Some("Contraseña incorrecta. Inténtalo de nuevo.".to_string());
            self.password_input.clear();
            false
        }
    }

    pub fn cancel_password_modal(&mut self, cx: &mut Context<Self>) {
        self.show_password_modal = false;
        self.pending_action = None;
        self.password_input.clear();
        self.password_error = None;
        cx.notify();
    }

    pub fn execute_pending_action(&mut self, cx: &mut Context<Self>) {
        if let Some(action) = self.pending_action.take() {
            match action {
                PendingAction::InstallCapsule => {
                    self.start_installation(cx);
                }
                PendingAction::InstallDeps => {
                    self.start_deps_installation(cx);
                }
                PendingAction::InstallCompositor => {
                    self.start_compositor_installation(cx);
                }
            }
        }
    }

    pub fn start_deps_installation(&mut self, cx: &mut Context<Self>) {
        let missing = self.missing_dependencies();
        if missing.is_empty() {
            return;
        }

        self.step = Step::Installing;
        self.progress = 0.1;
        self.error_message = None;
        self.status_message = format!("Installing {} missing dependencies...", missing.len());
        self.logs.clear();
        self.logs.push(format!(
            "==> Installing missing packages: {}",
            missing.join(" ")
        ));

        let tx = self.worker_tx.clone();
        let depends = self.depends_manager.clone();
        let sudo_pass = self.sudo_password.clone();

        thread::spawn(move || {
            let tx_log = tx.clone();
            let res =
                depends.install_missing_packages(&missing, sudo_pass.as_deref(), move |line| {
                    let _ = tx_log.send(WorkerMsg::Log(line));
                });
            let _ = tx.send(WorkerMsg::DepsDone(res));
        });

        cx.notify();
    }

    pub fn start_compositor_installation(&mut self, cx: &mut Context<Self>) {
        let comp = self.selected_compositor;
        self.step = Step::Installing;
        self.progress = 0.15;
        self.error_message = None;
        self.status_message = format!("Installing {} compositor...", comp.name());
        self.logs.clear();
        self.logs
            .push(format!("==> Starting {} installation...", comp.name()));

        let tx = self.worker_tx.clone();
        let comp_mgr = self.compositor_manager.clone();
        let sudo_pass = self.sudo_password.clone();

        thread::spawn(move || {
            let tx_log = tx.clone();
            let res = comp_mgr.install_compositor(comp, sudo_pass.as_deref(), move |line| {
                let _ = tx_log.send(WorkerMsg::Log(line));
            });
            let _ = tx.send(WorkerMsg::CompositorDone(res));
        });

        cx.notify();
    }

    pub fn start_installation(&mut self, cx: &mut Context<Self>) {
        self.step = Step::Installing;
        self.progress = 0.05;
        self.status_message = "Preparing installation...".to_string();
        self.logs.clear();
        self.logs
            .push("Starting installation process...".to_string());

        let tx = self.worker_tx.clone();
        let target = self.target;
        let release_info = self.release_info.clone();
        let was_running = self.capsule_was_running;
        let sudo_pass = self.sudo_password.clone();
        let selected_comp = self.selected_compositor;
        let comp_needed = !self.is_compositor_installed(selected_comp);
        let comp_mgr = self.compositor_manager.clone();

        thread::spawn(move || {
            let run = || -> Result<(), String> {
                if comp_needed {
                    let _ = tx.send(WorkerMsg::Progress(
                        0.08,
                        format!("Installing {} compositor...", selected_comp.name()),
                    ));
                    let tx_log = tx.clone();
                    comp_mgr.install_compositor(
                        selected_comp,
                        sudo_pass.as_deref(),
                        move |line| {
                            let _ = tx_log.send(WorkerMsg::Log(line));
                        },
                    )?;
                }

                if was_running {
                    let _ = tx.send(WorkerMsg::Progress(
                        0.15,
                        "Stopping existing Capsule process...".to_string(),
                    ));
                    let _ = tx.send(WorkerMsg::Log(
                        "Stopping existing Capsule process...".to_string(),
                    ));
                    system::stop_capsule_process();
                }

                let binary_path = if let Some(local_bin) = system::find_local_release_binary() {
                    let _ = tx.send(WorkerMsg::Progress(
                        0.50,
                        "Found local release binary in workspace.".to_string(),
                    ));
                    let _ = tx.send(WorkerMsg::Log(format!(
                        "Using local binary: {}",
                        local_bin.display()
                    )));
                    local_bin
                } else {
                    let download_url = match &release_info {
                        Some(info) => info.download_url.clone(),
                        None => format!(
                            "https://github.com/{}/releases/latest/download/capsule-{}.tar.gz",
                            github::REPO,
                            target
                        ),
                    };

                    let _ = tx.send(WorkerMsg::Log(format!(
                        "Fetching asset from: {}",
                        download_url
                    )));
                    let tx_prog = tx.clone();
                    match github::download_and_extract_binary(&download_url, move |pct, msg| {
                        let _ = tx_prog.send(WorkerMsg::Progress(pct as f32, msg.to_string()));
                    }) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Download failed: {}", e)),
                    }
                };

                let _ = tx.send(WorkerMsg::Progress(
                    0.92,
                    "Installing binary to /usr/local/bin...".to_string(),
                ));
                let _ = tx.send(WorkerMsg::Log(
                    "Copying capsule binary to /usr/local/bin/capsule...".to_string(),
                ));
                if let Err(e) = install::copy_binary_to_system(&binary_path, sudo_pass.as_deref()) {
                    return Err(format!("Failed to copy binary: {}", e));
                }

                let _ = tx.send(WorkerMsg::Progress(
                    0.96,
                    "Creating desktop launcher entry...".to_string(),
                ));
                let _ = tx.send(WorkerMsg::Log(
                    "Writing /usr/share/applications/capsule.desktop...".to_string(),
                ));
                if let Err(e) = install::create_desktop_entry(sudo_pass.as_deref()) {
                    return Err(format!("Failed to create desktop entry: {}", e));
                }

                let _ = tx.send(WorkerMsg::Progress(
                    0.98,
                    "Setting up KWin + Capsule session for login greeters...".to_string(),
                ));
                let _ = tx.send(WorkerMsg::Log(
                    "Registering /usr/share/wayland-sessions/kwin-capsule.desktop...".to_string(),
                ));
                if let Err(e) = install::create_kwin_capsule_session(sudo_pass.as_deref()) {
                    let _ = tx.send(WorkerMsg::Log(format!(
                        "Note: Could not register wayland-session: {}",
                        e
                    )));
                }

                let _ = tx.send(WorkerMsg::Progress(
                    1.0,
                    "Installation finished successfully!".to_string(),
                ));
                let _ = tx.send(WorkerMsg::Log("Installation complete.".to_string()));

                Ok(())
            };

            let res = run();
            let _ = tx.send(WorkerMsg::Done(res));
        });

        cx.notify();
    }

    pub fn poll_worker(&mut self, cx: &mut Context<Self>) {
        let mut updated = false;
        while let Ok(msg) = self.worker_rx.try_recv() {
            updated = true;
            match msg {
                WorkerMsg::Progress(pct, status) => {
                    self.progress = pct;
                    self.status_message = status;
                }
                WorkerMsg::Log(line) => {
                    self.logs.push(line);
                }
                WorkerMsg::Done(res) => match res {
                    Ok(()) => {
                        self.installed_version = system::get_installed_capsule_version();
                        self.scan_compositors();
                        self.step = Step::Finished;
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                        self.step = Step::Welcome;
                    }
                },
                WorkerMsg::DepsDone(res) => {
                    self.scan_dependencies();
                    match res {
                        Ok(()) => {
                            self.step = Step::Dependencies;
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                            self.step = Step::Dependencies;
                        }
                    }
                }
                WorkerMsg::CompositorDone(res) => {
                    self.scan_compositors();
                    match res {
                        Ok(()) => {
                            self.step = Step::Compositor;
                        }
                        Err(e) => {
                            self.error_message = Some(e);
                            self.step = Step::Compositor;
                        }
                    }
                }
            }
        }
        if updated {
            cx.notify();
        }
    }

    pub fn finish_and_exit(&mut self, cx: &mut Context<Self>) {
        if self.launch_on_finish {
            let _ = install::launch_capsule();
        }
        install::cleanup_self();
        cx.quit();
    }

    pub fn cancel_and_exit(&mut self, cx: &mut Context<Self>) {
        install::cleanup_self();
        cx.quit();
    }
}
