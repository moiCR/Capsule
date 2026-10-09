use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AurHelper {
    Paru,
    Yay,
    Pacman,
}

impl AurHelper {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Paru => "paru",
            Self::Yay => "yay",
            Self::Pacman => "pacman",
        }
    }
}

pub fn detect_target() -> &'static str {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => "x86_64-unknown-linux-gnu",
    }
}

use std::io::Write;

pub fn is_root() -> bool {
    let output = Command::new("id").arg("-u").output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).trim() == "0"
    } else {
        false
    }
}

pub fn check_sudo() -> bool {
    Command::new("sudo")
        .arg("-n")
        .arg("true")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn verify_sudo_password(password: &str) -> bool {
    let mut child = match Command::new("sudo")
        .args(["-k", "-S", "-p", "", "true"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", password);
    }

    child.wait().map(|s| s.success()).unwrap_or(false)
}

pub fn refresh_sudo_with_password(password: &str) -> bool {
    let mut child = match Command::new("sudo")
        .args(["-S", "-p", "", "-v"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "{}", password);
    }

    child.wait().map(|s| s.success()).unwrap_or(false)
}

pub fn detect_aur_helper() -> Option<AurHelper> {
    if command_exists("paru") {
        Some(AurHelper::Paru)
    } else if command_exists("yay") {
        Some(AurHelper::Yay)
    } else if command_exists("pacman") {
        Some(AurHelper::Pacman)
    } else {
        None
    }
}

pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn check_package_installed(pkg: &str, helper: &Option<AurHelper>) -> bool {
    let cmd_name = match helper {
        Some(AurHelper::Paru) => "paru",
        Some(AurHelper::Yay) => "yay",
        _ => "pacman",
    };

    Command::new(cmd_name)
        .arg("-Qq")
        .arg(pkg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn get_installed_capsule_version() -> Option<String> {
    let paths = ["capsule", "/usr/local/bin/capsule", "/usr/bin/capsule"];
    for p in paths {
        if let Ok(out) = Command::new(p).arg("--version").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                if let Some(ver) = text.split_whitespace().last() {
                    return Some(ver.trim().to_string());
                }
            }
        }
    }
    None
}

pub fn is_capsule_running() -> bool {
    let pgrep_capsule = Command::new("pgrep")
        .arg("-x")
        .arg("capsule")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if pgrep_capsule {
        return true;
    }

    Command::new("pgrep")
        .arg("-x")
        .arg("Capsule")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn stop_capsule_process() {
    let _ = Command::new("capsule")
        .arg("quit")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    for _ in 0..15 {
        if !is_capsule_running() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    let _ = Command::new("pkill").args(["-x", "capsule"]).status();
    let _ = Command::new("pkill").args(["-x", "Capsule"]).status();
    std::thread::sleep(std::time::Duration::from_millis(300));

    if is_capsule_running() {
        let _ = Command::new("pkill").args(["-9", "-x", "capsule"]).status();
        let _ = Command::new("pkill").args(["-9", "-x", "Capsule"]).status();
    }
}

pub fn check_wallpaper_daemon() -> Option<&'static str> {
    if command_exists("awww") {
        Some("awww")
    } else if command_exists("swww") {
        Some("swww")
    } else {
        None
    }
}

pub fn find_local_release_binary() -> Option<std::path::PathBuf> {
    let candidates = [
        "target/release/capsule",
        "target/release/Capsule",
        "../target/release/capsule",
        "../../target/release/capsule",
    ];

    for c in candidates {
        let path = Path::new(c);
        if path.exists() && path.is_file() {
            if let Ok(abs) = path.canonicalize() {
                return Some(abs);
            }
        }
    }
    None
}
