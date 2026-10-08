use super::{DashboardModule, Snapshot};
use crate::new_capsule::{
    module::CapsuleModuleEvent,
    widgets::dashboard::{DashboardAction, DashboardView},
};
use gpui::Context;
use services::{AppState, NotificationStore};
use ui::theme::{Theme, theme_manager::ThemeManager};
impl DashboardModule {
    pub fn dispatch(&mut self, action: DashboardAction, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>().clone();
        match action {
            DashboardAction::View(view) => {
                self.navigation.open(view.clone());
                match &view {
                    DashboardView::Wifi => state.network.rescan_wifi(),
                    DashboardView::Bluetooth => state.network.start_bluetooth_scan(),
                    _ => {}
                }
            }
            DashboardAction::Back => {
                self.navigation.open(DashboardView::Home);
            }
            DashboardAction::Close => cx.emit(CapsuleModuleEvent::Close),
            DashboardAction::Wifi => state.network.toggle_wifi(),
            DashboardAction::Bluetooth => state.network.toggle_bluetooth(),
            DashboardAction::Dnd => {
                NotificationStore::global().toggle_dnd();
            }
            DashboardAction::ScanWifi => state.network.rescan_wifi(),
            DashboardAction::ScanBluetooth => state.network.start_bluetooth_scan(),
            DashboardAction::SelectWifi(ssid) => {
                if let Some(ap) = self
                    .snapshot
                    .network
                    .wifi_ap_list
                    .iter()
                    .find(|ap| ap.ssid == ssid)
                {
                    if ap.is_connected {
                        state.network.disconnect_wifi(&ssid);
                    } else if ap.security.is_empty() || ap.security == "--" || ap.is_saved {
                        state.network.connect_wifi(&ssid, None);
                    } else {
                        self.navigation.selected_ssid = Some(ssid);
                        self.navigation.password.clear();
                    }
                }
            }
            DashboardAction::ConnectWifi => {
                if let Some(ssid) = &self.navigation.selected_ssid {
                    state
                        .network
                        .connect_wifi(ssid, Some(&self.navigation.password));
                    self.navigation.password.clear();
                }
            }
            DashboardAction::BluetoothDevice(mac) => {
                if self
                    .snapshot
                    .network
                    .bluetooth_device_list
                    .iter()
                    .any(|d| d.mac == mac && d.is_connected)
                {
                    state.network.disconnect_bluetooth(&mac);
                } else {
                    state.network.connect_bluetooth(&mac);
                }
            }
            DashboardAction::Month(offset) => match offset {
                -1 => state.calendar.prev_month(),
                1 => state.calendar.next_month(),
                _ => state.calendar.reset_to_today(),
            },
            DashboardAction::Settings => {
                cx.emit(CapsuleModuleEvent::Close);
                crate::panel::SettingsPanel::open(cx);
            }
            DashboardAction::Lock => {
                cx.emit(CapsuleModuleEvent::Close);
                crate::panel::LockScreenPanel::open_all(cx);
            }
            DashboardAction::ClearNotifications => {
                NotificationStore::global().clear_all_notifications()
            }
            DashboardAction::RemoveNotification(id) => {
                NotificationStore::global().remove_notification(id)
            }
            DashboardAction::Notification(id, key) => {
                NotificationStore::global().invoke_action(id, key)
            }
            DashboardAction::Player(bus) => self.player_bus = Some(bus),
            DashboardAction::TrayActivate { bus, path } => {
                if let Some(index) = state
                    .sni_host
                    .get_items()
                    .iter()
                    .position(|item| item.bus_name == bus && item.object_path == path)
                {
                    state.sni_host.activate_item(index);
                }
            }
            DashboardAction::TrayMenu(bus, path, id) => state.sni_host.trigger_menu(bus, path, id),
            action => self.run_action(action, state, cx),
        }
        self.snapshot = Snapshot::read(cx);
        cx.notify();
    }

    fn run_action(&mut self, action: DashboardAction, state: AppState, cx: &mut Context<Self>) {
        if self.pending {
            return;
        }
        if let DashboardAction::Media(direction) = &action
            && self.player().is_none_or(|player| {
                (*direction == -1 && !player.can_go_previous)
                    || (*direction == 1 && !player.can_go_next)
            })
        {
            return;
        }
        let bus = self.player_bus.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.pending = true;
        self.navigation.error = None;
        services::spawn_tokio(async move {
            let result: Result<Option<Theme>, String> = match action {
                DashboardAction::Mute => state
                    .system
                    .toggle_mute()
                    .await
                    .map(|_| None)
                    .map_err(|e| e.to_string()),
                DashboardAction::Sink(name) => state
                    .system
                    .set_default_sink(&name)
                    .await
                    .map(|_| None)
                    .map_err(|e| e.to_string()),
                DashboardAction::Media(command) => {
                    let ok = if let Some(bus) = bus {
                        match command {
                            -1 => services::MprisService::previous_bus(&bus).await,
                            1 => services::MprisService::next_bus(&bus).await,
                            _ => services::MprisService::play_pause_bus(&bus).await,
                        }
                    } else {
                        false
                    };
                    if ok {
                        Ok(None)
                    } else {
                        Err("dashboard_new.media_error".into())
                    }
                }
                DashboardAction::Seek(seconds) => {
                    if let Some(bus) = bus {
                        if services::MprisService::seek_to(&bus, seconds).await {
                            Ok(None)
                        } else {
                            Err("dashboard_new.media_error".into())
                        }
                    } else {
                        Ok(None)
                    }
                }
                DashboardAction::Theme(theme) => tokio::task::spawn_blocking(move || {
                    ThemeManager::save_current_theme(&theme).map(|_| Some(theme))
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|result| result),
                DashboardAction::Wallpaper(path) => tokio::task::spawn_blocking(move || {
                    if state.wallpaper.set_wallpaper(path) {
                        Ok(None)
                    } else {
                        Err("dashboard_new.wallpaper_error".into())
                    }
                })
                .await
                .map_err(|e| e.to_string())
                .and_then(|result| result),
                _ => Ok(None),
            };
            let _ = sender.send(result);
        });
        self.action_task = Some(cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |module, cx| {
                module.pending = false;
                match result {
                    Ok(Ok(Some(theme))) => {
                        if cx.has_global::<ThemeManager>() {
                            let manager = cx.global_mut::<ThemeManager>();
                            manager.current_theme = theme.clone();
                            manager.apply_theme_to_apps();
                        }
                        cx.set_global(theme);
                    }
                    Ok(Ok(None)) => {}
                    Ok(Err(error)) => {
                        module.navigation.error = Some(if error.starts_with("dashboard_new.") {
                            module.text(&error, cx)
                        } else {
                            error
                        })
                    }
                    Err(_) => {
                        module.navigation.error =
                            Some(module.text("dashboard_new.action_error", cx))
                    }
                }
                module.snapshot = Snapshot::read(cx);
                cx.notify();
            });
        }));
    }
}
