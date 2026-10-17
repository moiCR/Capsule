use crate::deps::PackageManager;
use crate::system::{self, AurHelper};
use std::path::Path;
use std::process::{Command, Stdio};

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
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "adw-gtk-theme",
    "git",
    "curl",
    "tar",
    "gpu-screen-recorder",
];

pub struct ArchPackageManager {
    distro_label: String,
    helper: Option<AurHelper>,
}

impl Default for ArchPackageManager {
    fn default() -> Self {
        Self::new("Arch Linux")
    }
}

impl ArchPackageManager {
    pub fn new(distro_label: impl Into<String>) -> Self {
        let helper = system::detect_aur_helper();
        Self {
            distro_label: distro_label.into(),
            helper,
        }
    }
}

impl PackageManager for ArchPackageManager {
    fn distro_name(&self) -> &str {
        &self.distro_label
    }

    fn manager_name(&self) -> String {
        self.helper
            .as_ref()
            .map(|h| h.name().to_string())
            .unwrap_or_else(|| "pacman".to_string())
    }

    fn dependencies(&self) -> &[&str] {
        ARCH_DEPENDENCIES
    }

    fn is_package_installed(&self, package: &str) -> bool {
        let cmd_name = match &self.helper {
            Some(AurHelper::Paru) => "paru",
            Some(AurHelper::Yay) => "yay",
            _ => "pacman",
        };

        Command::new(cmd_name)
            .arg("-Qq")
            .arg(package)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn check_lock(&self) -> Result<(), String> {
        if Path::new("/var/lib/pacman/db.lck").exists() {
            return Err(
                "Pacman database is currently locked (/var/lib/pacman/db.lck). Another package manager is running."
                    .to_string(),
            );
        }
        Ok(())
    }

    fn build_install_command(&self, packages: &[String]) -> Command {
        match &self.helper {
            Some(AurHelper::Paru) => {
                let mut cmd = Command::new("paru");
                cmd.args(["-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
            Some(AurHelper::Yay) => {
                let mut cmd = Command::new("yay");
                cmd.args(["-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
            _ => {
                let mut cmd = Command::new("sudo");
                cmd.args(["-n", "pacman", "-S", "--needed", "--noconfirm"]);
                cmd.args(packages);
                cmd
            }
        }
    }
}
