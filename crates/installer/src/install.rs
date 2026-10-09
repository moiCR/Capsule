use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

const DESKTOP_ENTRY_CONTENT: &str = r#"[Desktop Entry]
Name=Capsule
Comment=Dynamic Island Shell for Linux
Exec=/usr/local/bin/capsule
Icon=capsule
Terminal=false
Type=Application
Path=$HOME
NoDisplay=true
Categories=Utility;System;
"#;

pub fn run_sudo_cmd(args: &[&str], password: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("sudo");
    if password.is_some() {
        cmd.args(["-S", "-p", ""]);
    }
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to run sudo with args {:?}", args))?;

    if let (Some(pass), Some(mut stdin)) = (password, child.stdin.take()) {
        let _ = writeln!(stdin, "{}", pass);
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("sudo command failed: {}", err.trim()));
    }

    Ok(())
}

pub fn copy_binary_to_system(source_bin: &Path, password: Option<&str>) -> Result<()> {
    let dest_dir = Path::new("/usr/local/bin");
    let dest_file = dest_dir.join("capsule");

    if !dest_dir.exists() {
        run_sudo_cmd(&["mkdir", "-p", "/usr/local/bin"], password)
            .context("Failed to create /usr/local/bin")?;
    }

    run_sudo_cmd(
        &["cp", source_bin.to_string_lossy().as_ref(), dest_file.to_string_lossy().as_ref()],
        password,
    )
    .context("Failed to copy binary to /usr/local/bin/capsule")?;

    run_sudo_cmd(
        &["chmod", "755", dest_file.to_string_lossy().as_ref()],
        password,
    )
    .context("Failed to chmod /usr/local/bin/capsule")?;

    Ok(())
}

pub fn create_desktop_entry(password: Option<&str>) -> Result<()> {
    let desktop_dir = Path::new("/usr/share/applications");
    let desktop_file = desktop_dir.join("capsule.desktop");

    if !desktop_dir.exists() {
        let _ = run_sudo_cmd(&["mkdir", "-p", "/usr/share/applications"], password);
    }

    let temp_desktop = std::env::temp_dir().join(format!("capsule-desktop-{}.desktop", std::process::id()));
    fs::write(&temp_desktop, DESKTOP_ENTRY_CONTENT)?;

    run_sudo_cmd(
        &["cp", temp_desktop.to_string_lossy().as_ref(), desktop_file.to_string_lossy().as_ref()],
        password,
    )?;

    run_sudo_cmd(
        &["chmod", "644", desktop_file.to_string_lossy().as_ref()],
        password,
    )?;

    let _ = fs::remove_file(temp_desktop);
    Ok(())
}

pub fn launch_capsule() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());

    if crate::system::command_exists("gtk-launch")
        && Path::new("/usr/share/applications/capsule.desktop").exists()
    {
        let status = Command::new("gtk-launch")
            .arg("capsule.desktop")
            .current_dir(&home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        if status.is_ok() {
            return Ok(());
        }
    }

    Command::new("/usr/local/bin/capsule")
        .current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn /usr/local/bin/capsule")?;

    Ok(())
}
