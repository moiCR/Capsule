use std::process::Stdio;

use gpui::Context;
use services::{AppState, IpcCommand, config::Defaults};

use crate::new_capsule::Capsule;

fn configured_command(command: &IpcCommand, defaults: &Defaults) -> Option<String> {
    let (application, terminal_apps): (&str, &[&str]) = match command {
        IpcCommand::Terminal => (&defaults.terminal, &[]),
        IpcCommand::Browser => (&defaults.browser, &[]),
        IpcCommand::Editor => (
            &defaults.editor,
            &["nvim", "vim", "vi", "nano", "hx", "helix", "micro"],
        ),
        IpcCommand::FileManager => (
            &defaults.file_manager,
            &["yazi", "ranger", "lf", "nnn", "mc", "vifm"],
        ),
        _ => return None,
    };
    let executable = application.split_whitespace().next()?;
    let name = std::path::Path::new(executable.trim_matches(['\'', '"']))
        .file_name()
        .and_then(|name| name.to_str());
    if name.is_some_and(|name| terminal_apps.contains(&name)) {
        Some(format!("{} -e {application}", defaults.terminal))
    } else {
        Some(application.to_string())
    }
}

pub(super) fn launch(command: IpcCommand, cx: &Context<Capsule>) {
    let config = cx.global::<AppState>().config.get();
    let Some(application) = configured_command(&command, &config.defaults) else {
        services::log_warn!("IPC", "No application configured for {:?}", command);
        return;
    };
    services::spawn_tokio(async move {
        let mut process = tokio::process::Command::new("sh");
        process
            .arg("-c")
            .arg(application)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(home) = dirs::home_dir() {
            process.current_dir(home);
        }
        match process.status().await {
            Ok(status) if status.success() => {}
            Ok(status) => services::log_error!(
                "IPC",
                "Application for {:?} exited with {}",
                command,
                status
            ),
            Err(error) => services::log_error!(
                "IPC",
                "Failed to launch application for {:?}: {}",
                command,
                error
            ),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Defaults {
        Defaults {
            terminal: "kitty --single-instance".to_string(),
            browser: "firefox --new-window".to_string(),
            editor: "/usr/bin/nvim notes.txt".to_string(),
            file_manager: "yazi".to_string(),
        }
    }

    #[test]
    fn configured_apps_preserve_arguments_and_wrap_cli_tools_in_terminal() {
        let defaults = defaults();
        assert_eq!(
            configured_command(&IpcCommand::Terminal, &defaults).as_deref(),
            Some("kitty --single-instance")
        );
        assert_eq!(
            configured_command(&IpcCommand::Browser, &defaults).as_deref(),
            Some("firefox --new-window")
        );
        assert_eq!(
            configured_command(&IpcCommand::Editor, &defaults).as_deref(),
            Some("kitty --single-instance -e /usr/bin/nvim notes.txt")
        );
        assert_eq!(
            configured_command(&IpcCommand::FileManager, &defaults).as_deref(),
            Some("kitty --single-instance -e yazi")
        );
    }

    #[test]
    fn gui_apps_run_directly_and_empty_configuration_does_not_launch() {
        let mut defaults = defaults();
        defaults.editor = "zed".to_string();
        defaults.file_manager = "thunar".to_string();
        assert_eq!(
            configured_command(&IpcCommand::Editor, &defaults).as_deref(),
            Some("zed")
        );
        assert_eq!(
            configured_command(&IpcCommand::FileManager, &defaults).as_deref(),
            Some("thunar")
        );
        defaults.terminal.clear();
        assert_eq!(configured_command(&IpcCommand::Terminal, &defaults), None);
        assert_eq!(
            configured_command(&IpcCommand::ShowSettings, &defaults),
            None
        );
    }
}
