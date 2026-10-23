pub mod arch;
pub mod debian;
pub mod fedora;
pub mod ubuntu;

use crate::system;
use arch::ArchPackageManager;
use debian::DebianPackageManager;
use fedora::FedoraPackageManager;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;
use ubuntu::UbuntuPackageManager;

pub trait PackageManager: Send + Sync {
    fn distro_name(&self) -> &str;
    fn manager_name(&self) -> String;
    fn dependencies(&self) -> &[&str];
    fn is_package_installed(&self, package: &str) -> bool;
    fn check_lock(&self) -> Result<(), String>;
    fn build_install_command(&self, packages: &[String]) -> Command;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistroKind {
    ArchBased(String),
    Debian(String),
    Ubuntu(String),
    Fedora(String),
}

#[derive(Clone)]
pub struct DependsManager {
    backend: Arc<dyn PackageManager>,
}

impl Default for DependsManager {
    fn default() -> Self {
        Self::detect()
    }
}

impl DependsManager {
    pub fn new(backend: Arc<dyn PackageManager>) -> Self {
        Self { backend }
    }

    pub fn detect() -> Self {
        let os_release = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
            .unwrap_or_default();

        let kind = Self::parse_os_release(&os_release).unwrap_or_else(Self::fallback_detect);

        let backend: Arc<dyn PackageManager> = match kind {
            DistroKind::ArchBased(name) => Arc::new(ArchPackageManager::new(name)),
            DistroKind::Debian(name) => Arc::new(DebianPackageManager::new(name)),
            DistroKind::Ubuntu(name) => Arc::new(UbuntuPackageManager::new(name)),
            DistroKind::Fedora(name) => Arc::new(FedoraPackageManager::new(name)),
        };

        Self::new(backend)
    }

    pub fn parse_os_release(content: &str) -> Option<DistroKind> {
        let mut id = String::new();
        let mut id_like = String::new();
        let mut pretty_name = String::new();
        let mut name = String::new();

        for line in content.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("ID=") {
                id = rest.trim_matches('"').trim_matches('\'').to_lowercase();
            } else if let Some(rest) = line.strip_prefix("ID_LIKE=") {
                id_like = rest.trim_matches('"').trim_matches('\'').to_lowercase();
            } else if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                pretty_name = rest.trim_matches('"').trim_matches('\'').to_string();
            } else if let Some(rest) = line.strip_prefix("NAME=") {
                name = rest.trim_matches('"').trim_matches('\'').to_string();
            }
        }

        if id.is_empty() && id_like.is_empty() {
            return None;
        }

        let display_label = if !pretty_name.is_empty() {
            pretty_name
        } else if !name.is_empty() {
            name
        } else {
            id.clone()
        };

        let like_tokens: Vec<&str> = id_like.split_whitespace().collect();

        if id == "ubuntu" || like_tokens.contains(&"ubuntu") {
            Some(DistroKind::Ubuntu(display_label))
        } else if id == "debian" || like_tokens.contains(&"debian") {
            Some(DistroKind::Debian(display_label))
        } else if id == "fedora" || like_tokens.contains(&"fedora") || like_tokens.contains(&"rhel")
        {
            Some(DistroKind::Fedora(display_label))
        } else if id == "arch"
            || like_tokens.contains(&"arch")
            || matches!(
                id.as_str(),
                "cachyos" | "endeavouros" | "manjaro" | "garuda" | "artix" | "omarchy"
            )
        {
            Some(DistroKind::ArchBased(display_label))
        } else {
            None
        }
    }

    fn fallback_detect() -> DistroKind {
        if Path::new("/etc/arch-release").exists() || system::command_exists("pacman") {
            DistroKind::ArchBased("Arch Linux".to_string())
        } else if Path::new("/etc/fedora-release").exists() || system::command_exists("dnf") {
            DistroKind::Fedora("Fedora".to_string())
        } else if Path::new("/etc/debian_version").exists() || system::command_exists("apt-get") {
            DistroKind::Debian("Debian".to_string())
        } else {
            DistroKind::ArchBased("Arch Linux".to_string())
        }
    }

    pub fn distro_name(&self) -> &str {
        self.backend.distro_name()
    }

    pub fn manager_name(&self) -> String {
        self.backend.manager_name()
    }

    #[allow(dead_code)]
    pub fn dependencies(&self) -> &[&str] {
        self.backend.dependencies()
    }

    pub fn scan_dependencies(&self) -> Vec<(String, bool)> {
        self.backend
            .dependencies()
            .iter()
            .map(|&pkg| {
                let installed = self.backend.is_package_installed(pkg);
                (pkg.to_string(), installed)
            })
            .collect()
    }

    pub fn install_missing_packages<F>(
        &self,
        packages: &[String],
        sudo_pass: Option<&str>,
        on_log: F,
    ) -> Result<(), String>
    where
        F: Fn(String) + Send + Sync + Clone + 'static,
    {
        if packages.is_empty() {
            return Ok(());
        }

        self.backend.check_lock()?;

        if let Some(pass) = sudo_pass
            && !system::refresh_sudo_with_password(pass)
        {
            return Err("Failed to refresh sudo credentials with provided password.".to_string());
        }

        let mgr_name = self.backend.manager_name();
        on_log(format!(
            "==> Executing package installation via {} ({}) for {} packages...",
            mgr_name,
            self.backend.distro_name(),
            packages.len()
        ));

        let mut cmd = self.backend.build_install_command(packages);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", mgr_name, e))?;

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
            .map_err(|e| format!("Failed waiting on package manager: {}", e))?;

        if status.success() {
            on_log("==> Package installation completed successfully.".to_string());
            Ok(())
        } else {
            Err(format!(
                "Package manager exited with error status: {:?}",
                status.code()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_os_release_arch_cachyos() {
        let os_release = r#"
NAME="CachyOS Linux"
PRETTY_NAME="CachyOS"
ID=cachyos
ID_LIKE=arch
BUILD_ID=rolling
"#;
        let kind = DependsManager::parse_os_release(os_release);
        assert_eq!(kind, Some(DistroKind::ArchBased("CachyOS".to_string())));
    }

    #[test]
    fn test_parse_os_release_ubuntu() {
        let os_release = r#"
PRETTY_NAME="Ubuntu 24.04.1 LTS"
NAME="Ubuntu"
VERSION_ID="24.04"
ID=ubuntu
ID_LIKE=debian
"#;
        let kind = DependsManager::parse_os_release(os_release);
        assert_eq!(
            kind,
            Some(DistroKind::Ubuntu("Ubuntu 24.04.1 LTS".to_string()))
        );
    }

    #[test]
    fn test_parse_os_release_debian() {
        let os_release = r#"
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
ID=debian
"#;
        let kind = DependsManager::parse_os_release(os_release);
        assert_eq!(
            kind,
            Some(DistroKind::Debian(
                "Debian GNU/Linux 12 (bookworm)".to_string()
            ))
        );
    }

    #[test]
    fn test_parse_os_release_fedora() {
        let os_release = r#"
NAME="Fedora Linux"
VERSION="41 (Workstation Edition)"
ID=fedora
PRETTY_NAME="Fedora Linux 41 (Workstation Edition)"
"#;
        let kind = DependsManager::parse_os_release(os_release);
        assert_eq!(
            kind,
            Some(DistroKind::Fedora(
                "Fedora Linux 41 (Workstation Edition)".to_string()
            ))
        );
    }

    #[test]
    fn test_all_distro_backends_provide_dependencies() {
        let arch = ArchPackageManager::default();
        let debian = DebianPackageManager::default();
        let ubuntu = UbuntuPackageManager::default();
        let fedora = FedoraPackageManager::default();

        assert_eq!(arch.dependencies().len(), 19);
        assert_eq!(debian.dependencies().len(), 19);
        assert_eq!(ubuntu.dependencies().len(), 19);
        assert_eq!(fedora.dependencies().len(), 19);

        assert!(arch.dependencies().contains(&"vulkan-icd-loader"));
        assert!(arch.dependencies().contains(&"adw-gtk-theme"));
        assert!(arch.dependencies().contains(&"gpu-screen-recorder"));

        assert!(debian.dependencies().contains(&"libvulkan1"));
        assert!(debian.dependencies().contains(&"dconf-cli"));
        assert!(debian.dependencies().contains(&"adw-gtk-theme"));
        assert!(debian.dependencies().contains(&"gpu-screen-recorder"));

        assert!(ubuntu.dependencies().contains(&"libvulkan1"));
        assert!(ubuntu.dependencies().contains(&"adw-gtk-theme"));
        assert!(ubuntu.dependencies().contains(&"gpu-screen-recorder"));

        assert!(fedora.dependencies().contains(&"vulkan-loader"));
        assert!(fedora.dependencies().contains(&"adw-gtk3-theme"));
        assert!(fedora.dependencies().contains(&"gpu-screen-recorder"));
    }

    #[test]
    fn test_debian_and_fedora_special_package_install_commands() {
        let debian = DebianPackageManager::default();
        let cmd = debian.build_install_command(&[
            "adw-gtk-theme".to_string(),
            "gpu-screen-recorder".to_string(),
        ]);
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        let joined = args.join(" ");
        assert!(joined.contains("adw-gtk3"));
        assert!(joined.contains("deb.griffo.io"));
        assert!(joined.contains("gpu-screen-recorder-cli"));

        let fedora = FedoraPackageManager::default();
        let f_cmd = fedora.build_install_command(&["gpu-screen-recorder".to_string()]);
        let f_args: Vec<String> = f_cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        let f_joined = f_args.join(" ");
        assert!(f_joined.contains("copr enable -y brycensranch/gpu-screen-recorder-git"));
        assert!(f_joined.contains("dnf install -y gpu-screen-recorder"));
    }
}
