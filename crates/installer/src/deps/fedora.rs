use crate::deps::PackageManager;
use crate::system;
use std::path::Path;
use std::process::{Command, Stdio};

pub const FEDORA_DEPENDENCIES: &[&str] = &[
    "vulkan-loader",
    "libxkbcommon",
    "libwayland-client",
    "wireplumber",
    "brightnessctl",
    "cliphist",
    "wl-clipboard",
    "qt5ct",
    "qt6ct",
    "gsettings-desktop-schemas",
    "zenity",
    "dconf",
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "adw-gtk3-theme",
    "git",
    "curl",
    "tar",
    "gpu-screen-recorder",
];

pub struct FedoraPackageManager {
    distro_label: String,
}

impl Default for FedoraPackageManager {
    fn default() -> Self {
        Self::new("Fedora")
    }
}

impl FedoraPackageManager {
    pub fn new(distro_label: impl Into<String>) -> Self {
        Self {
            distro_label: distro_label.into(),
        }
    }

    fn rpm_is_installed(package: &str) -> bool {
        Command::new("rpm")
            .args(["-q", package])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

impl PackageManager for FedoraPackageManager {
    fn distro_name(&self) -> &str {
        &self.distro_label
    }

    fn manager_name(&self) -> String {
        "dnf".to_string()
    }

    fn dependencies(&self) -> &[&str] {
        FEDORA_DEPENDENCIES
    }

    fn is_package_installed(&self, package: &str) -> bool {
        if package == "gpu-screen-recorder" {
            return system::command_exists("gpu-screen-recorder")
                || Self::rpm_is_installed("gpu-screen-recorder");
        }
        Self::rpm_is_installed(package)
    }

    fn check_lock(&self) -> Result<(), String> {
        let locks = ["/var/run/dnf.pid", "/var/lib/dnf/lock"];
        for lock in locks {
            if Path::new(lock).exists() {
                return Err(format!(
                    "DNF database is currently locked ({}). Another package manager is running.",
                    lock
                ));
            }
        }
        Ok(())
    }

    fn build_install_command(&self, packages: &[String]) -> Command {
        if packages.iter().any(|p| p == "gpu-screen-recorder") {
            let pkgs_joined = packages.join(" ");
            let script = format!(
                "set -e && dnf copr enable -y brycensranch/gpu-screen-recorder-git && dnf install -y {}",
                pkgs_joined
            );
            let mut cmd = Command::new("sudo");
            cmd.args(["-n", "sh", "-c", &script]);
            cmd
        } else {
            let mut cmd = Command::new("sudo");
            cmd.args(["-n", "dnf", "install", "-y"]);
            cmd.args(packages);
            cmd
        }
    }
}
