pub mod arch;
pub mod debian;
pub mod fedora;
pub mod ubuntu;

use crate::deps::{DependsManager, DistroKind};
use crate::system;
use arch::ArchCompositorBackend;
use debian::DebianCompositorBackend;
use fedora::FedoraCompositorBackend;
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use ubuntu::UbuntuCompositorBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compositor {
    Hyprland,
    Niri,
    MangoWm,
    KWin,
}

impl Compositor {
    pub const ALL: &'static [Compositor] = &[
        Compositor::Hyprland,
        Compositor::Niri,
        Compositor::MangoWm,
        Compositor::KWin,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Hyprland => "Hyprland",
            Self::Niri => "Niri",
            Self::MangoWm => "MangoWM",
            Self::KWin => "KWin (Plasma Wayland)",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Self::Hyprland => {
                "Dynamic tiling Wayland compositor with smooth animations, rounded corners & plugins"
            }
            Self::Niri => {
                "Scrollable-tiling Wayland compositor built in Rust with infinite horizontal strip"
            }
            Self::MangoWm => {
                "Lightweight wlroots & scenefx Wayland compositor with scroller & master-stack layouts"
            }
            Self::KWin => {
                "KDE's robust Wayland window manager & compositor, paired with Capsule shell"
            }
        }
    }

    pub fn binary_names(&self) -> &'static [&'static str] {
        match self {
            Self::Hyprland => &["Hyprland", "hyprland"],
            Self::Niri => &["niri"],
            Self::MangoWm => &["mango", "mangowm", "mangowc"],
            Self::KWin => &["kwin_wayland"],
        }
    }

    pub fn is_installed(&self) -> bool {
        self.binary_names()
            .iter()
            .any(|bin| system::command_exists(bin))
    }
}

pub trait CompositorBackend: Send + Sync {
    fn distro_name(&self) -> &str;
    fn build_compositor_install_command(&self, compositor: Compositor) -> Command;
}

#[derive(Clone)]
pub struct CompositorManager {
    backend: Arc<dyn CompositorBackend>,
}

impl Default for CompositorManager {
    fn default() -> Self {
        Self::detect()
    }
}

impl CompositorManager {
    pub fn new(backend: Arc<dyn CompositorBackend>) -> Self {
        Self { backend }
    }

    pub fn detect() -> Self {
        let os_release = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
            .unwrap_or_default();

        let backend: Arc<dyn CompositorBackend> =
            match DependsManager::parse_os_release(&os_release) {
                Some(DistroKind::ArchBased(_)) => Arc::new(ArchCompositorBackend::new()),
                Some(DistroKind::Debian(_)) => Arc::new(DebianCompositorBackend),
                Some(DistroKind::Ubuntu(_)) => Arc::new(UbuntuCompositorBackend),
                Some(DistroKind::Fedora(_)) => Arc::new(FedoraCompositorBackend),
                None => {
                    if system::command_exists("pacman") {
                        Arc::new(ArchCompositorBackend::new())
                    } else if system::command_exists("dnf") {
                        Arc::new(FedoraCompositorBackend)
                    } else if system::command_exists("apt-get") {
                        Arc::new(DebianCompositorBackend)
                    } else {
                        Arc::new(ArchCompositorBackend::new())
                    }
                }
            };

        Self::new(backend)
    }

    pub fn scan_compositors(&self) -> Vec<(Compositor, bool)> {
        Compositor::ALL
            .iter()
            .map(|&comp| (comp, comp.is_installed()))
            .collect()
    }

    pub fn install_compositor<F>(
        &self,
        compositor: Compositor,
        sudo_pass: Option<&str>,
        on_log: F,
    ) -> Result<(), String>
    where
        F: Fn(String) + Send + Sync + Clone + 'static,
    {
        if let Some(pass) = sudo_pass
            && !system::refresh_sudo_with_password(pass)
        {
            return Err("Failed to refresh sudo credentials with provided password.".to_string());
        }

        on_log(format!(
            "==> Installing {} compositor for {}...",
            compositor.name(),
            self.backend.distro_name()
        ));

        let mut cmd = self.backend.build_compositor_install_command(compositor);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn compositor installer: {}", e))?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let on_log_stdout = on_log.clone();
        let stdout_handle = thread::spawn(move || {
            if let Some(out) = stdout {
                let reader = BufReader::new(out);
                for line in reader.lines().map_while(Result::ok) {
                    on_log_stdout(line);
                }
            }
        });

        let on_log_stderr = on_log.clone();
        let stderr_handle = thread::spawn(move || {
            if let Some(err) = stderr {
                let reader = BufReader::new(err);
                for line in reader.lines().map_while(Result::ok) {
                    on_log_stderr(line);
                }
            }
        });

        let _ = stdout_handle.join();
        let _ = stderr_handle.join();

        let status = child
            .wait()
            .map_err(|e| format!("Failed waiting on compositor installation: {}", e))?;

        if status.success() {
            on_log(format!("==> {} installed successfully!", compositor.name()));
            Ok(())
        } else {
            Err(format!(
                "{} installation exited with error status: {:?}",
                compositor.name(),
                status.code()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_compositors_metadata() {
        assert_eq!(Compositor::ALL.len(), 4);
        assert_eq!(Compositor::Hyprland.name(), "Hyprland");
        assert_eq!(Compositor::Niri.name(), "Niri");
        assert_eq!(Compositor::MangoWm.name(), "MangoWM");
        assert_eq!(Compositor::KWin.name(), "KWin (Plasma Wayland)");
    }

    #[test]
    fn test_compositor_commands_for_all_distros() {
        let arch = ArchCompositorBackend::new();
        let fedora = FedoraCompositorBackend;
        let debian = DebianCompositorBackend;
        let ubuntu = UbuntuCompositorBackend;

        for &comp in Compositor::ALL {
            let a_cmd = arch.build_compositor_install_command(comp);
            assert!(!a_cmd.get_program().is_empty());

            let f_cmd = fedora.build_compositor_install_command(comp);
            let f_args: Vec<String> = f_cmd
                .get_args()
                .map(|a| a.to_string_lossy().to_string())
                .collect();
            let f_joined = f_args.join(" ");
            match comp {
                Compositor::Hyprland => assert!(f_joined.contains("solopasha/hyprland")),
                Compositor::Niri => assert!(f_joined.contains("yalter/niri")),
                Compositor::MangoWm => assert!(f_joined.contains("MangoWC")),
                Compositor::KWin => assert!(f_joined.contains("kwin-wayland")),
            }

            let d_cmd = debian.build_compositor_install_command(comp);
            assert_eq!(d_cmd.get_program(), "sudo");

            let u_cmd = ubuntu.build_compositor_install_command(comp);
            assert_eq!(u_cmd.get_program(), "sudo");
        }
    }
}
