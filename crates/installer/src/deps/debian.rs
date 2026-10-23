use crate::deps::PackageManager;
use crate::system;
use std::path::Path;
use std::process::Command;

pub const DEBIAN_DEPENDENCIES: &[&str] = &[
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

pub const ADW_GTK3_TARBALL_URL: &str =
    "https://github.com/lassekongo83/adw-gtk3/releases/download/v6.5/adw-gtk3v6.5.tar.xz";

pub struct DebianPackageManager {
    distro_label: String,
}

impl Default for DebianPackageManager {
    fn default() -> Self {
        Self::new("Debian")
    }
}

impl DebianPackageManager {
    pub fn new(distro_label: impl Into<String>) -> Self {
        Self {
            distro_label: distro_label.into(),
        }
    }

    pub fn dpkg_is_installed(package: &str) -> bool {
        if let Ok(output) = Command::new("dpkg-query")
            .args(["-W", "-f=${Status}", package])
            .output()
            && output.status.success()
        {
            let status = String::from_utf8_lossy(&output.stdout);
            return status.contains("install ok installed");
        }
        false
    }

    pub fn is_adw_gtk3_installed() -> bool {
        if Path::new("/usr/share/themes/adw-gtk3").exists()
            && Path::new("/usr/share/themes/adw-gtk3-dark").exists()
        {
            return true;
        }
        if let Some(home) = std::env::var_os("HOME") {
            let local_themes = Path::new(&home).join(".local/share/themes");
            if local_themes.join("adw-gtk3").exists() && local_themes.join("adw-gtk3-dark").exists()
            {
                return true;
            }
        }
        Self::dpkg_is_installed("adw-gtk-theme") || Self::dpkg_is_installed("adw-gtk3")
    }

    pub fn is_gpu_screen_recorder_installed() -> bool {
        system::command_exists("gpu-screen-recorder")
            || Self::dpkg_is_installed("gpu-screen-recorder-cli")
            || Self::dpkg_is_installed("gpu-screen-recorder")
    }

    pub fn check_debian_package(package: &str) -> bool {
        match package {
            "adw-gtk-theme" => Self::is_adw_gtk3_installed(),
            "gpu-screen-recorder" => Self::is_gpu_screen_recorder_installed(),
            other => Self::dpkg_is_installed(other),
        }
    }

    pub fn check_apt_locks() -> Result<(), String> {
        let locks = [
            "/var/lib/dpkg/lock-frontend",
            "/var/lib/dpkg/lock",
            "/var/lib/apt/lists/lock",
        ];
        for lock in locks {
            if Path::new(lock).exists()
                && let Ok(out) = Command::new("fuser").arg(lock).output()
                && out.status.success()
            {
                return Err(format!(
                    "APT database is currently locked ({}). Another package manager is running.",
                    lock
                ));
            }
        }
        Ok(())
    }

    pub fn build_apt_install_command(packages: &[String]) -> Command {
        let mut apt_pkgs: Vec<&str> = Vec::new();
        let mut need_adw = false;
        let mut need_gsr = false;

        for pkg in packages {
            match pkg.as_str() {
                "adw-gtk-theme" => need_adw = true,
                "gpu-screen-recorder" => need_gsr = true,
                other => apt_pkgs.push(other),
            }
        }

        if !need_adw && !need_gsr {
            let mut cmd = Command::new("sudo");
            cmd.args([
                "-n",
                "env",
                "DEBIAN_FRONTEND=noninteractive",
                "apt-get",
                "install",
                "-y",
                "--no-install-recommends",
            ]);
            cmd.args(apt_pkgs);
            return cmd;
        }

        let mut script_steps: Vec<String> = Vec::new();
        script_steps.push("set -e".to_string());
        script_steps.push("export DEBIAN_FRONTEND=noninteractive".to_string());

        if need_adw || need_gsr {
            for helper_pkg in ["curl", "ca-certificates", "tar", "xz-utils", "gpg"] {
                if !apt_pkgs.contains(&helper_pkg) {
                    apt_pkgs.push(helper_pkg);
                }
            }
        }

        if !apt_pkgs.is_empty() {
            script_steps.push("apt-get update -y".to_string());
            script_steps.push(format!(
                "apt-get install -y --no-install-recommends {}",
                apt_pkgs.join(" ")
            ));
        }

        if need_adw {
            script_steps.push(
                "echo '==> Downloading and installing adw-gtk3 theme to /usr/share/themes...'"
                    .to_string(),
            );
            script_steps.push("mkdir -p /usr/share/themes".to_string());
            script_steps.push(format!(
                "curl -fsSL '{}' | tar -xJ -C /usr/share/themes",
                ADW_GTK3_TARBALL_URL
            ));
        }

        if need_gsr {
            script_steps.push(
                "echo '==> Configuring repository and installing gpu-screen-recorder...'"
                    .to_string(),
            );
            script_steps.push("install -d -m 0755 /etc/apt/keyrings".to_string());
            script_steps.push(
                "curl -fsSL https://deb.griffo.io/EA0F721D231FDD3A0A17B9AC7808B4DD62C41256.asc | gpg --dearmor --yes -o /etc/apt/keyrings/deb.griffo.io.gpg"
                    .to_string(),
            );
            script_steps.push(
                "CODENAME=$(. /etc/os-release && echo \"${VERSION_CODENAME:-bookworm}\")"
                    .to_string(),
            );
            script_steps.push(
                "echo \"deb [signed-by=/etc/apt/keyrings/deb.griffo.io.gpg] https://deb.griffo.io/apt $CODENAME main\" > /etc/apt/sources.list.d/deb.griffo.io.list"
                    .to_string(),
            );
            script_steps.push("apt-get update -y".to_string());
            script_steps.push("apt-get install -y gpu-screen-recorder-cli".to_string());
        }

        let full_script = script_steps.join(" && ");
        let mut cmd = Command::new("sudo");
        cmd.args(["-n", "sh", "-c", &full_script]);
        cmd
    }
}

impl PackageManager for DebianPackageManager {
    fn distro_name(&self) -> &str {
        &self.distro_label
    }

    fn manager_name(&self) -> String {
        "apt".to_string()
    }

    fn dependencies(&self) -> &[&str] {
        DEBIAN_DEPENDENCIES
    }

    fn is_package_installed(&self, package: &str) -> bool {
        Self::check_debian_package(package)
    }

    fn check_lock(&self) -> Result<(), String> {
        Self::check_apt_locks()
    }

    fn build_install_command(&self, packages: &[String]) -> Command {
        Self::build_apt_install_command(packages)
    }
}
