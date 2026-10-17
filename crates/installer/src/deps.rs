use crate::system::{self, AurHelper};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
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

pub fn scan_dependencies(helper: &Option<AurHelper>) -> Vec<(String, bool)> {
    ARCH_DEPENDENCIES
        .iter()
        .map(|&pkg| {
            let installed = system::check_package_installed(pkg, helper);
            (pkg.to_string(), installed)
        })
        .collect()
}

pub fn install_missing_packages<F>(
    packages: &[String],
    helper: &Option<AurHelper>,
    sudo_pass: Option<&str>,
    on_log: F,
) -> Result<(), String>
where
    F: Fn(String) + Send + Sync + Clone + 'static,
{
    if packages.is_empty() {
        return Ok(());
    }

    if Path::new("/var/lib/pacman/db.lck").exists() {
        return Err(
            "Pacman database is currently locked (/var/lib/pacman/db.lck). Another package manager is running.".to_string(),
        );
    }

    if let Some(pass) = sudo_pass
        && !system::refresh_sudo_with_password(pass)
    {
        return Err("Failed to refresh sudo credentials with provided password.".to_string());
    }

    let cmd_name = match helper {
        Some(AurHelper::Paru) => "paru",
        Some(AurHelper::Yay) => "yay",
        _ => "sudo",
    };

    on_log(format!(
        "==> Executing package installation via {} for {} packages...",
        cmd_name,
        packages.len()
    ));

    let mut cmd = if cmd_name == "sudo" {
        let mut c = Command::new("sudo");
        c.args(["-n", "pacman", "-S", "--needed", "--noconfirm"]);
        c.args(packages);
        c
    } else {
        let mut c = Command::new(cmd_name);
        c.args(["-S", "--needed", "--noconfirm"]);
        c.args(packages);
        c
    };

    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", cmd_name, e))?;

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
