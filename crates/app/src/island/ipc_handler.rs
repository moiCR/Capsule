pub(crate) mod launch_apps;

use gpui::{Context, Task};
use services::{AppState, IpcCommand};

use super::{
    Capsule,
    module::{CapsuleModuleEvent, CapsuleModuleId},
};

impl Capsule {
    pub(super) fn start_ipc(_cx: &mut Context<Self>) -> Task<()> {
        Task::ready(())
    }

    pub fn handle_ipc_command(&mut self, command: IpcCommand, cx: &mut Context<Self>) {
        if crate::panel::SettingsPanel::is_open() {
            match command {
                IpcCommand::ToggleSettings | IpcCommand::Hide => {
                    crate::panel::SettingsPanel::close(cx);
                }
                IpcCommand::Lock => {
                    crate::panel::LockScreenPanel::open_all(cx);
                }
                IpcCommand::Quit => cx.quit(),
                IpcCommand::Ping => {}
                _ => {}
            }
            return;
        }

        match command {
            command @ (IpcCommand::Terminal
            | IpcCommand::Browser
            | IpcCommand::Editor
            | IpcCommand::FileManager) => {
                launch_apps::launch(command, cx.global::<AppState>());
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

            command @ (IpcCommand::ShowNotification | IpcCommand::ToggleNotification) => {
                if command == IpcCommand::ToggleNotification
                    && self.module_manager.current_id() == CapsuleModuleId::Notification
                {
                    self.handle_module_event(&CapsuleModuleEvent::Close, cx);
                } else {
                    self.module_manager
                        .notification
                        .update(cx, |module, cx| module.open(true, cx));
                    self.handle_module_event(
                        &CapsuleModuleEvent::Open(CapsuleModuleId::Notification),
                        cx,
                    );
                }
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
            command @ (IpcCommand::ShowEmoji | IpcCommand::ToggleEmoji) => {
                let event = if command == IpcCommand::ToggleEmoji
                    && self.module_manager.current_id() == CapsuleModuleId::Emoji
                {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(CapsuleModuleId::Emoji)
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
            command @ (IpcCommand::ToggleSelectTheme | IpcCommand::ShowThemes) => {
                let id = CapsuleModuleId::Themes;
                let show = command == IpcCommand::ShowThemes;
                let event = if !show && self.module_manager.current_id() == id {
                    CapsuleModuleEvent::Close
                } else {
                    CapsuleModuleEvent::Open(id)
                };
                self.handle_module_event(&event, cx);
            }
            command @ (IpcCommand::ToggleSelectWallpaper | IpcCommand::ShowWallpapers) => {
                self.handle_module_event(&CapsuleModuleEvent::Close, cx);
                if command == IpcCommand::ShowWallpapers {
                    crate::panel::SettingsPanel::open(cx);
                } else {
                    crate::panel::SettingsPanel::toggle(cx);
                }
            }
            IpcCommand::SetWallpaper(path) => {
                let state = cx.global::<services::AppState>().clone();
                let _ = state.config.update(|cfg| {
                    cfg.ui.wallpaper_path = Some(path.clone());
                });
                state.wallpaper.set_wallpaper(path);
            }
            IpcCommand::Hide | IpcCommand::Default => {
                self.handle_module_event(&CapsuleModuleEvent::Close, cx);
            }
            command @ (IpcCommand::ShowSettings | IpcCommand::ToggleSettings) => {
                self.handle_module_event(&CapsuleModuleEvent::Close, cx);
                if command == IpcCommand::ShowSettings {
                    crate::panel::SettingsPanel::open(cx);
                } else {
                    crate::panel::SettingsPanel::toggle(cx);
                }
            }
            IpcCommand::Lock => {
                crate::panel::LockScreenPanel::open_all(cx);
            }
            IpcCommand::Quit => cx.quit(),
            IpcCommand::Ping => {}
        }
    }
}
