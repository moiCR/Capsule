use crate::deps::PackageManager;
use crate::deps::debian::DebianPackageManager;
use std::process::Command;

pub const UBUNTU_DEPENDENCIES: &[&str] = &[
    "libvulkan1",
    "libxkbcommon0",
    "libwayland-client0",
    "wireplumber",
    "brightnessctl",
    "cliphist",
    "wl-clipboard",
    "qt5ct",
    "qt6ct",
    "gsettings-desktop-schemas",
    "zenity",
    "dconf-cli",
    "xdg-desktop-portal",
    "xdg-desktop-portal-gtk",
    "adw-gtk-theme",
    "git",
    "curl",
    "tar",
    "gpu-screen-recorder",
];

pub struct UbuntuPackageManager {
    distro_label: String,
}

impl Default for UbuntuPackageManager {
    fn default() -> Self {
        Self::new("Ubuntu")
    }
}

impl UbuntuPackageManager {
    pub fn new(distro_label: impl Into<String>) -> Self {
        Self {
            distro_label: distro_label.into(),
        }
    }
}

impl PackageManager for UbuntuPackageManager {
    fn distro_name(&self) -> &str {
        &self.distro_label
    }

    fn manager_name(&self) -> String {
        "apt".to_string()
    }

    fn dependencies(&self) -> &[&str] {
        UBUNTU_DEPENDENCIES
    }

    fn is_package_installed(&self, package: &str) -> bool {
        DebianPackageManager::check_debian_package(package)
    }

    fn check_lock(&self) -> Result<(), String> {
        DebianPackageManager::check_apt_locks()
    }

    fn build_install_command(&self, packages: &[String]) -> Command {
        DebianPackageManager::build_apt_install_command(packages)
    }
}
