use crate::github::{self, ReleaseInfo};
use crate::install;
use crate::system::{self, AurHelper};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub const ARCH_DEPENDENCIES: &[&str] = &[
    "vulkan-icd-loader",
    "libxkbcommon",
    "wayland",
    "wireplumber",
    "brightnessctl",
    "cliphist",
    "wl-clipboard",
    "qt5ct",
    "qt6ct",
    "gsettings-desktop-schemas",
    "zenity",
    "dconf",
    "nwg-look",
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "adw-gtk-theme",
    "git",
    "curl",
    "tar",
    "gpu-screen-recorder",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Welcome,
    Dependencies,
    Installing,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    InstallCapsule,
    InstallDeps,
}

pub enum WorkerMsg {
    Progress(f64, String),
    Log(String),
    Done(Result<(), String>),
}

pub struct App {
    pub step: Step,
    pub beta_mode: bool,
    pub target: &'static str,
    pub installed_version: Option<String>,
    pub target_version: Option<String>,
    pub release_info: Option<ReleaseInfo>,
    pub aur_helper: Option<AurHelper>,
    pub dependencies: Vec<(String, bool)>,
    pub wallpaper_daemon: Option<String>,
    pub capsule_was_running: bool,
    pub progress: f64,
    pub status_message: String,
    pub logs: Vec<String>,
    pub selected_index: usize,
    pub launch_on_finish: bool,
    pub should_quit: bool,
    pub error_message: Option<String>,
    pub worker_rx: Option<Receiver<WorkerMsg>>,

    // Stored credentials & modal state
    pub sudo_password: Option<String>,
    pub show_password_modal: bool,
    pub password_input: String,
    pub password_error: Option<String>,
    pub pending_action: Option<PendingAction>,
}

impl App {
    pub fn new(beta_mode: bool) -> Self {
        let target = system::detect_target();
        let aur_helper = system::detect_aur_helper();
        let installed_version = system::get_installed_capsule_version();
        let capsule_was_running = system::is_capsule_running();
        let wallpaper_daemon = system::check_wallpaper_daemon().map(|s| s.to_string());

        let mut app = Self {
            step: Step::Welcome,
            beta_mode,
            target,
            installed_version,
            target_version: None,
            release_info: None,
            aur_helper,
            dependencies: Vec::new(),
            wallpaper_daemon,
            capsule_was_running,
            progress: 0.0,
            status_message: "Ready".to_string(),
            logs: Vec::new(),
            selected_index: 0,
            launch_on_finish: true,
            should_quit: false,
            error_message: None,
            worker_rx: None,

            sudo_password: None,
            show_password_modal: false,
            password_input: String::new(),
            password_error: None,
            pending_action: None,
        };

        app.scan_dependencies();
        app.refresh_release_info();
        app
    }

    pub fn scan_dependencies(&mut self) {
        self.dependencies = ARCH_DEPENDENCIES
            .iter()
            .map(|&pkg| {
                let installed = system::check_package_installed(pkg, &self.aur_helper);
                (pkg.to_string(), installed)
            })
            .collect();
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

    pub fn toggle_beta(&mut self) {
        self.beta_mode = !self.beta_mode;
        self.refresh_release_info();
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
        if system::verify_sudo_password(&self.password_input) {
            let pass = self.password_input.clone();
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

    pub fn start_installation(&mut self) {
        self.step = Step::Installing;
        self.progress = 0.05;
        self.status_message = "Preparing installation...".to_string();
        self.logs.clear();
        self.logs.push("Starting installation process...".to_string());

        let (tx, rx) = channel::<WorkerMsg>();
        self.worker_rx = Some(rx);

        let target = self.target;
        let release_info = self.release_info.clone();
        let was_running = self.capsule_was_running;
        let sudo_pass = self.sudo_password.clone();

        thread::spawn(move || {
            let run = || -> Result<(), String> {
                if was_running {
                    let _ = tx.send(WorkerMsg::Progress(0.10, "Stopping existing Capsule process...".to_string()));
                    let _ = tx.send(WorkerMsg::Log("Stopping existing Capsule process...".to_string()));
                    system::stop_capsule_process();
                }

                let binary_path = if let Some(local_bin) = system::find_local_release_binary() {
                    let _ = tx.send(WorkerMsg::Progress(0.50, "Found local release binary in workspace.".to_string()));
                    let _ = tx.send(WorkerMsg::Log(format!("Using local binary: {}", local_bin.display())));
                    local_bin
                } else {
                    let download_url = match &release_info {
                        Some(info) => info.download_url.clone(),
                        None => format!(
                            "https://github.com/{}/releases/latest/download/capsule-{}.tar.gz",
                            github::REPO, target
                        ),
                    };

                    let _ = tx.send(WorkerMsg::Log(format!("Fetching asset from: {}", download_url)));
                    let tx_prog = tx.clone();
                    match github::download_and_extract_binary(&download_url, move |pct, msg| {
                        let _ = tx_prog.send(WorkerMsg::Progress(pct, msg.to_string()));
                    }) {
                        Ok(p) => p,
                        Err(e) => return Err(format!("Download failed: {}", e)),
                    }
                };

                let _ = tx.send(WorkerMsg::Progress(0.92, "Installing binary to /usr/local/bin...".to_string()));
                let _ = tx.send(WorkerMsg::Log("Copying capsule binary to /usr/local/bin/capsule...".to_string()));
                if let Err(e) = install::copy_binary_to_system(&binary_path, sudo_pass.as_deref()) {
                    return Err(format!("Failed to copy binary: {}", e));
                }

                let _ = tx.send(WorkerMsg::Progress(0.96, "Creating desktop launcher entry...".to_string()));
                let _ = tx.send(WorkerMsg::Log("Writing /usr/share/applications/capsule.desktop...".to_string()));
                if let Err(e) = install::create_desktop_entry(sudo_pass.as_deref()) {
                    return Err(format!("Failed to create desktop entry: {}", e));
                }

                let _ = tx.send(WorkerMsg::Progress(1.0, "Installation finished successfully!".to_string()));
                let _ = tx.send(WorkerMsg::Log("Installation complete.".to_string()));

                Ok(())
            };

            let res = run();
            let _ = tx.send(WorkerMsg::Done(res));
        });
    }

    pub fn poll_worker(&mut self) {
        if let Some(rx) = &self.worker_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    WorkerMsg::Progress(pct, status) => {
                        self.progress = pct;
                        self.status_message = status;
                    }
                    WorkerMsg::Log(line) => {
                        self.logs.push(line);
                    }
                    WorkerMsg::Done(res) => {
                        match res {
                            Ok(()) => {
                                self.installed_version = system::get_installed_capsule_version();
                                self.step = Step::Finished;
                            }
                            Err(e) => {
                                self.error_message = Some(e);
                                self.step = Step::Welcome;
                            }
                        }
                    }
                }
            }
        }
    }
}
