use anyhow::{Context, Result, anyhow};
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
        &[
            "cp",
            source_bin.to_string_lossy().as_ref(),
            dest_file.to_string_lossy().as_ref(),
        ],
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

    let temp_desktop =
        std::env::temp_dir().join(format!("capsule-desktop-{}.desktop", std::process::id()));
    fs::write(&temp_desktop, DESKTOP_ENTRY_CONTENT)?;

    run_sudo_cmd(
        &[
            "cp",
            temp_desktop.to_string_lossy().as_ref(),
            desktop_file.to_string_lossy().as_ref(),
        ],
        password,
    )?;

    run_sudo_cmd(
        &["chmod", "644", desktop_file.to_string_lossy().as_ref()],
        password,
    )?;

    let _ = fs::remove_file(temp_desktop);
    Ok(())
}

const KWIN_CAPSULE_SESSION_CONTENT: &str = r#"#!/usr/bin/env bash
export XDG_CURRENT_DESKTOP="KDE:Capsule"
export XDG_SESSION_DESKTOP="KDE"
export KDE_FULL_SESSION=true
if [[ -z "${KDE_SESSION_VERSION:-}" ]]; then
    if command -v kwin_wayland &>/dev/null && kwin_wayland --version 2>&1 | grep -q " 6\."; then
        export KDE_SESSION_VERSION=6
    else
        export KDE_SESSION_VERSION=5
    fi
fi

# Synchronize cursor theme & size
if [[ -z "${XCURSOR_THEME:-}" || "${XCURSOR_THEME}" == "default" ]]; then
    DETECTED_CURSOR=$(kreadconfig6 --file kcminputrc --group Mouse --key cursorTheme 2>/dev/null)
    if [[ -z "$DETECTED_CURSOR" && -f "$HOME/.config/gtk-3.0/settings.ini" ]]; then
        DETECTED_CURSOR=$(grep -E '^gtk-cursor-theme-name=' "$HOME/.config/gtk-3.0/settings.ini" | cut -d'=' -f2 | tr -d ' "')
    fi
    export XCURSOR_THEME="${DETECTED_CURSOR:-breeze_cursors}"
fi
export XCURSOR_SIZE="${XCURSOR_SIZE:-24}"

if command -v dbus-update-activation-environment &>/dev/null; then
    dbus-update-activation-environment --systemd --all
    dbus-update-activation-environment --systemd XCURSOR_THEME XCURSOR_SIZE
fi

exec kwin_wayland --xwayland --exit-with-session /usr/local/bin/capsule
"#;

const WAYLAND_SESSION_DESKTOP_CONTENT: &str = r#"[Desktop Entry]
Name=KWin + Capsule
Comment=KWin Wayland Compositor with Capsule Shell
Exec=/usr/local/bin/capsule-session-kwin
Type=Application
DesktopNames=KDE;Capsule;
"#;

pub fn create_kwin_capsule_session(password: Option<&str>) -> Result<()> {
    let script_dest = Path::new("/usr/local/bin/capsule-session-kwin");
    let session_dir = Path::new("/usr/share/wayland-sessions");
    let session_file = session_dir.join("kwin-capsule.desktop");

    let temp_script =
        std::env::temp_dir().join(format!("capsule-kwin-session-{}.sh", std::process::id()));
    fs::write(&temp_script, KWIN_CAPSULE_SESSION_CONTENT)?;

    let _ = run_sudo_cmd(&["mkdir", "-p", "/usr/local/bin"], password);
    run_sudo_cmd(
        &[
            "cp",
            temp_script.to_string_lossy().as_ref(),
            script_dest.to_string_lossy().as_ref(),
        ],
        password,
    )?;
    run_sudo_cmd(
        &["chmod", "755", script_dest.to_string_lossy().as_ref()],
        password,
    )?;
    let _ = fs::remove_file(temp_script);

    if !session_dir.exists() {
        let _ = run_sudo_cmd(&["mkdir", "-p", "/usr/share/wayland-sessions"], password);
    }

    let temp_desktop =
        std::env::temp_dir().join(format!("kwin-capsule-{}.desktop", std::process::id()));
    fs::write(&temp_desktop, WAYLAND_SESSION_DESKTOP_CONTENT)?;

    run_sudo_cmd(
        &[
            "cp",
            temp_desktop.to_string_lossy().as_ref(),
            session_file.to_string_lossy().as_ref(),
        ],
        password,
    )?;
    run_sudo_cmd(
        &["chmod", "644", session_file.to_string_lossy().as_ref()],
        password,
    )?;
    let _ = fs::remove_file(temp_desktop);

    let _ = configure_kwin_rules();

    Ok(())
}

pub fn configure_kwin_rules() -> Result<()> {
    if let Some(home) =
        dirs::home_dir().or_else(|| std::env::var("HOME").ok().map(std::path::PathBuf::from))
    {
        let config_dir = home.join(".config");
        let _ = fs::create_dir_all(&config_dir);
        let rules_path = config_dir.join("kwinrulesrc");
        let content = if rules_path.exists() {
            fs::read_to_string(&rules_path).unwrap_or_default()
        } else {
            String::new()
        };

        let mut new_content = content;
        if !new_content.contains("[General]") {
            new_content.push_str("[General]\nOrder=capsule\n\n");
        } else if !new_content.contains("Order=") {
            new_content = new_content.replace("[General]", "[General]\nOrder=capsule");
        } else if !new_content.contains("capsule") {
            if let Some(idx) = new_content.find("Order=") {
                let order_end = new_content[idx..]
                    .find('\n')
                    .map(|i| idx + i)
                    .unwrap_or(new_content.len());
                let cur_order = &new_content[idx + 6..order_end];
                let updated = format!("Order=capsule,{}", cur_order.trim_start_matches(','));
                new_content.replace_range(idx..order_end, &updated);
            }
        }

        // Clean out legacy rule sections if present
        if new_content.contains("[capsule]") {
            let start = new_content.find("[capsule]").unwrap_or(0);
            let next_sec = new_content[start + 9..]
                .find("\n[")
                .map(|i| start + 9 + i)
                .unwrap_or(new_content.len());
            new_content.replace_range(start..next_sec, "");
        }
        if new_content.contains("[capsule-lockscreen]") {
            let start = new_content.find("[capsule-lockscreen]").unwrap_or(0);
            let next_sec = new_content[start + 20..]
                .find("\n[")
                .map(|i| start + 20 + i)
                .unwrap_or(new_content.len());
            new_content.replace_range(start..next_sec, "");
        }

        new_content.push_str("\n[capsule]\nDescription=Capsule Shell\nwmclass=capsule\nwmclassmatch=1\ncloseable=false\ncloseablerule=2\nskipswitcher=true\nskipswitcherrule=2\nskiptaskbar=true\nskiptaskbarrule=2\nskippager=true\nskippagerrule=2\ndesktopsrule=2\n");
        let _ = fs::write(&rules_path, new_content);

        // Ensure ~/.icons/default/index.theme exists so client apps fallback to the same cursor
        let icons_default = home.join(".icons/default");
        let index_theme = icons_default.join("index.theme");
        if !index_theme.exists() {
            let _ = fs::create_dir_all(&icons_default);
            let _ = fs::write(
                &index_theme,
                "[Icon Theme]\nName=Default\nComment=Default Cursor Theme\nInherits=breeze_cursors\n",
            );
        }

        // Ensure ~/.config/kcminputrc has [Mouse] cursorTheme=breeze_cursors
        let kcminput_path = config_dir.join("kcminputrc");
        let kcminput_content = fs::read_to_string(&kcminput_path).unwrap_or_default();
        if !kcminput_content.contains("cursorTheme") {
            let mut updated_kcminput = kcminput_content;
            if !updated_kcminput.contains("[Mouse]") {
                updated_kcminput.push_str("\n[Mouse]\ncursorTheme=breeze_cursors\ncursorSize=24\n");
            } else {
                updated_kcminput = updated_kcminput.replace("[Mouse]", "[Mouse]\ncursorTheme=breeze_cursors\ncursorSize=24");
            }
            let _ = fs::write(&kcminput_path, updated_kcminput);
        }

        // Tell KWin to reload rules
        let _ = std::process::Command::new("qdbus6")
            .args(["org.kde.KWin", "/KWin", "reconfigure"])
            .spawn()
            .or_else(|_| {
                std::process::Command::new("qdbus")
                    .args(["org.kde.KWin", "/KWin", "reconfigure"])
                    .spawn()
            });
    }
    Ok(())
}

pub fn launch_capsule() -> Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let bin_path = if Path::new("/usr/local/bin/capsule").exists() {
        "/usr/local/bin/capsule"
    } else if Path::new("/usr/bin/capsule").exists() {
        "/usr/bin/capsule"
    } else {
        "capsule"
    };

    if crate::system::command_exists("setsid") {
        let mut cmd = Command::new("setsid");
        cmd.arg(bin_path)
            .current_dir(&home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if cmd.spawn().is_ok() {
            return Ok(());
        }
    }

    Command::new(bin_path)
        .current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .with_context(|| format!("Failed to spawn {}", bin_path))?;

    Ok(())
}

pub fn cleanup_self() {
    if let Ok(exe_path) = std::env::current_exe() {
        let is_temp = exe_path.starts_with(std::env::temp_dir())
            || exe_path.to_string_lossy().contains("/tmp/");
        let is_target = exe_path.to_string_lossy().contains("/target/");
        if is_temp && !is_target {
            let _ = std::fs::remove_file(exe_path);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cleanup_self_ignores_target() {
        let target_path =
            std::path::PathBuf::from("/home/user/Capsule/target/release/capsule-installer");
        let is_temp = target_path.starts_with(std::env::temp_dir())
            || target_path.to_string_lossy().contains("/tmp/");
        let is_target = target_path.to_string_lossy().contains("/target/");
        assert!(is_target);
        assert!(!(is_temp && !is_target));
    }

    #[test]
    fn test_cleanup_self_detects_tmp() {
        let tmp_path = std::path::PathBuf::from("/tmp/capsule-installer");
        let is_temp = tmp_path.starts_with(std::env::temp_dir())
            || tmp_path.to_string_lossy().contains("/tmp/");
        let is_target = tmp_path.to_string_lossy().contains("/target/");
        assert!(is_temp);
        assert!(!is_target);
        assert!(is_temp && !is_target);
    }
}
