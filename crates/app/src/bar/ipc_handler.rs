use gpui::{Context, Window};
use services::IpcCommand;

use crate::bar::Bar;

impl Bar {
    pub fn handle_ipc_command(
        &mut self,
        command: IpcCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match command {
            IpcCommand::ToggleLauncher => self.toggle_launcher(window, cx),
            IpcCommand::ShowLauncher => self.open_launcher(window, cx),
            IpcCommand::ToggleDashboard => self.toggle_dashboard(window, cx),
            IpcCommand::ShowDashboard => self.show_dashboard(window, cx),
            IpcCommand::ToggleNotification => self.toggle_notifications(window, cx),
            IpcCommand::ShowNotification => self.show_notifications(window, cx),
            IpcCommand::ToggleClipboard => self.toggle_clipboard(window, cx),
            IpcCommand::ShowClipboard => self.show_clipboard(window, cx),
            IpcCommand::ToggleEmoji => self.toggle_emoji(window, cx),
            IpcCommand::ShowEmoji => self.show_emoji(window, cx),
            IpcCommand::ToggleRecord => self.toggle_record(window, cx),
            IpcCommand::ShowRecord => self.show_record(window, cx),
            IpcCommand::Hide => {
                self.close_shared_panel(window, cx);
                self.close_launcher(window, cx);
                self.close_polkit(window, cx);
            }
            _ => {}
        }
    }
}
