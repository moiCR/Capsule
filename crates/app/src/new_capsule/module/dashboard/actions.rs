use super::{DashboardModule, Snapshot};
use crate::new_capsule::{module::CapsuleModuleEvent, widgets::dashboard::DashboardAction};
use gpui::Context;
use services::{AppState, NotificationStore};
impl DashboardModule {
    pub fn dispatch(&mut self, action: DashboardAction, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>().clone();
        match action {
            DashboardAction::Satellite(id) => {
                self.navigation.reset();
                cx.emit(CapsuleModuleEvent::ToggleSatellite(id));
            }
            DashboardAction::Close => cx.emit(CapsuleModuleEvent::Close),
            DashboardAction::CloseSatellite(id) => {
                cx.emit(CapsuleModuleEvent::CloseSatelliteId(id))
            }
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
            DashboardAction::ClearNotifications => {
                NotificationStore::global().clear_all_notifications()
            }
            DashboardAction::RemoveNotification(id) => {
                NotificationStore::global().remove_notification(id)
            }
            DashboardAction::Notification(id, key) => {
                NotificationStore::global().invoke_action(id, key)
            }
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
            let result: Result<(), String> = match action {
                DashboardAction::Brightness(value) => state
                    .system
                    .set_brightness(value)
                    .await
                    .map_err(|error| error.to_string()),
                DashboardAction::Mute => state
                    .system
                    .toggle_mute()
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                DashboardAction::Sink(name) => state
                    .system
                    .set_default_sink(&name)
                    .await
                    .map(|_| ())
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
                        Ok(())
                    } else {
                        Err("dashboard_new.media_error".into())
                    }
                }
                DashboardAction::Seek(seconds) => {
                    if let Some(bus) = bus {
                        if services::MprisService::seek_to(&bus, seconds).await {
                            Ok(())
                        } else {
                            Err("dashboard_new.media_error".into())
                        }
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            };
            let _ = sender.send(result);
        });
        self.action_task = Some(cx.spawn(async move |this, cx| {
            let result = receiver.await;
            let _ = this.update(cx, |module, cx| {
                module.pending = false;
                match result {
                    Ok(Ok(())) => {}
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
