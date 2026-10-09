mod launch_apps;

use gpui::{Context, Task};
use services::IpcCommand;

use super::{
    Capsule,
    module::{CapsuleModuleEvent, CapsuleModuleId},
};

impl Capsule {
    pub(super) fn start_ipc(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                services::wait_for_ipc_command().await;
                if this
                    .update(cx, |capsule, cx| {
                        let mut received = false;
                        while let Some(command) = services::pop_ipc_command() {
                            received = true;
                            capsule.handle_ipc_command(command, cx);
                        }
                        if received {
                            capsule.sync_window(cx);
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
    }

    pub fn handle_ipc_command(&mut self, command: IpcCommand, cx: &mut Context<Self>) {
        match command {
            command @ (IpcCommand::Terminal
            | IpcCommand::Browser
            | IpcCommand::Editor
            | IpcCommand::FileManager) => {
                launch_apps::launch(command, cx);
            }

            IpcCommand::ShowDashboard => {
                self.handle_module_event(&CapsuleModuleEvent::Open(CapsuleModuleId::Dashboard), cx)
            }

            IpcCommand::ToggleDashboard => {
                let event = if self.module_manager.current_id() == CapsuleModuleId::Dashboard {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(CapsuleModuleId::Dashboard)
                };
                self.handle_module_event(&event, cx);
            }

            IpcCommand::ShowLauncher => {
                self.handle_module_event(&CapsuleModuleEvent::Open(CapsuleModuleId::Launcher), cx)
            }

            IpcCommand::ToggleLauncher => {
                let event = if self.module_manager.current_id() == CapsuleModuleId::Launcher {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(CapsuleModuleId::Launcher)
                };
                self.handle_module_event(&event, cx);
            }

            IpcCommand::ShowClipboard => {
                self.handle_module_event(&CapsuleModuleEvent::Open(CapsuleModuleId::Clipboard), cx)
            }
            IpcCommand::ToggleClipboard => {
                let event = if self.module_manager.current_id() == CapsuleModuleId::Clipboard {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(CapsuleModuleId::Clipboard)
                };
                self.handle_module_event(&event, cx);
            }
            IpcCommand::ShowShelf => {
                self.handle_module_event(&CapsuleModuleEvent::Open(CapsuleModuleId::Shelf), cx)
            }
            IpcCommand::ShowRecord => {
                self.handle_module_event(&CapsuleModuleEvent::Open(CapsuleModuleId::Record), cx)
            }
            command @ (IpcCommand::ToggleShelf | IpcCommand::ToggleRecord) => {
                let id = if command == IpcCommand::ToggleShelf {
                    CapsuleModuleId::Shelf
                } else {
                    CapsuleModuleId::Record
                };
                let event = if self.module_manager.current_id() == id {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(id)
                };
                self.handle_module_event(&event, cx);
            }
            command @ (IpcCommand::ToggleSelectTheme
            | IpcCommand::ToggleSelectWallpaper
            | IpcCommand::ShowThemes
            | IpcCommand::ShowWallpapers) => {
                let id = if matches!(
                    command,
                    IpcCommand::ToggleSelectTheme | IpcCommand::ShowThemes
                ) {
                    CapsuleModuleId::Themes
                } else {
                    CapsuleModuleId::Wallpapers
                };
                let show = matches!(command, IpcCommand::ShowThemes | IpcCommand::ShowWallpapers);
                let event = if !show && self.module_manager.current_id() == id {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(id)
                };
                self.handle_module_event(&event, cx);
            }
            IpcCommand::Hide | IpcCommand::Default => {
                self.handle_module_event(&CapsuleModuleEvent::Close, cx);
            }
            IpcCommand::ShowSettings => {
                crate::panel::SettingsPanel::open(cx);
            }
            IpcCommand::ToggleSettings => crate::panel::SettingsPanel::toggle(cx),
            IpcCommand::Lock => {
                crate::panel::LockScreenPanel::open_all(cx);
            }
            IpcCommand::Quit => cx.quit(),
            IpcCommand::Ping => {}
            other => {
                services::log_warn!("IPC", "Command not implemented in new Capsule: {:?}", other)
            }
        }
    }
}
