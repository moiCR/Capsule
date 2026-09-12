use crate::dbus_util::get_shared_system_conn;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, sleep, timeout};
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;
use zbus::proxy;
use zbus::zvariant::{ObjectPath, OwnedValue};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub is_connected: bool,
    pub is_saved: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BluetoothDeviceItem {
    pub mac: String,
    pub name: String,
    pub is_connected: bool,
    pub is_paired: bool,
}

#[derive(Clone, Debug, Default)]
pub struct NetworkStatus {
    pub wifi_enabled: bool,
    pub wifi_ssid: String,
    pub wifi_signal: u8,
    pub ethernet_connected: bool,
    pub ethernet_name: String,
    pub wifi_ap_list: Vec<WifiAccessPoint>,
    pub bluetooth_enabled: bool,
    pub bluetooth_device_name: String,
    pub bluetooth_device_list: Vec<BluetoothDeviceItem>,
    pub is_scanning_wifi: bool,
    pub is_scanning_bluetooth: bool,
    pub connecting_wifi_ssid: Option<String>,
    pub connecting_bluetooth_mac: Option<String>,
    pub wifi_error: Option<String>,
    pub bluetooth_error: Option<String>,
}

#[proxy(
    interface = "org.freedesktop.NetworkManager",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager"
)]
pub trait NetworkManager {
    #[zbus(property)]
    fn wireless_enabled(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_wireless_enabled(&self, value: bool) -> zbus::Result<()>;

    fn get_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    fn activate_connection(
        &self,
        connection: &zbus::zvariant::ObjectPath<'_>,
        device: &zbus::zvariant::ObjectPath<'_>,
        specific_object: &zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMWireless {
    fn get_all_access_points(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings",
    default_service = "org.freedesktop.NetworkManager",
    default_path = "/org/freedesktop/NetworkManager/Settings"
)]
pub trait NMSettings {
    fn list_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Settings.Connection",
    default_service = "org.freedesktop.NetworkManager"
)]
pub trait NMSettingsConnection {
    fn get_settings(
        &self,
    ) -> zbus::Result<HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>>;
}

#[proxy(
    interface = "org.bluez.Adapter1",
    default_service = "org.bluez",
    default_path = "/org/bluez/hci0"
)]
pub trait BluezAdapter {
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_powered(&self, value: bool) -> zbus::Result<()>;

    fn start_discovery(&self) -> zbus::Result<()>;
    fn stop_discovery(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn discovering(&self) -> zbus::Result<bool>;
}

#[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
pub trait BluezDevice {
    fn connect(&self) -> zbus::Result<()>;
    fn disconnect(&self) -> zbus::Result<()>;
    fn pair(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn paired(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;
}

pub type BluezObjectsMap = HashMap<
    zbus::zvariant::OwnedObjectPath,
    HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>,
>;

#[proxy(
    interface = "org.freedesktop.DBus.ObjectManager",
    default_service = "org.bluez",
    default_path = "/"
)]
pub trait BluezObjectManager {
    fn get_managed_objects(&self) -> zbus::Result<BluezObjectsMap>;
}

#[derive(Clone)]
pub struct NetworkService {
    status: Arc<Mutex<NetworkStatus>>,
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkService {
    pub fn new() -> Self {
        let initial = NetworkStatus::default();
        let status = Arc::new(Mutex::new(initial));
        let service = Self { status };
        service.start_polling();
        service
    }

    pub fn get_status(&self) -> NetworkStatus {
        if let Ok(guard) = self.status.lock() {
            guard.clone()
        } else {
            NetworkStatus::default()
        }
    }

    pub fn toggle_wifi(&self) {
        let is_on = self.get_status().wifi_enabled;
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("nmcli")
                .arg("radio")
                .arg("wifi")
                .arg(if is_on { "off" } else { "on" })
                .output()
                .await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(nm) = NetworkManagerProxy::new(&conn).await
            {
                let _ = nm.set_wireless_enabled(!is_on).await;
            }
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
            }
        });
    }

    pub fn toggle_bluetooth(&self) {
        let is_on = self.get_status().bluetooth_enabled;
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("bluetoothctl")
                .arg("power")
                .arg(if is_on { "off" } else { "on" })
                .output()
                .await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(adapter) = BluezAdapterProxy::new(&conn).await
            {
                let _ = adapter.set_powered(!is_on).await;
            }
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
            }
        });
    }

    pub fn rescan_wifi(&self) {
        let status_arc = self.status.clone();
        if let Ok(mut lock) = status_arc.lock() {
            lock.is_scanning_wifi = true;
        }
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("nmcli")
                .arg("dev")
                .arg("wifi")
                .arg("rescan")
                .output()
                .await;
            sleep(Duration::from_millis(1500)).await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(3), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
                lock.is_scanning_wifi = false;
                return;
            }
            if let Ok(mut lock) = status_arc.lock() {
                lock.is_scanning_wifi = false;
            }
        });
    }

    pub fn connect_wifi(&self, ssid: &str, password: Option<&str>) {
        let ssid_string = ssid.to_string();
        let pwd_string = password.map(|p| p.to_string());
        let status_arc = self.status.clone();
        if let Ok(mut lock) = status_arc.lock() {
            lock.connecting_wifi_ssid = Some(ssid_string.clone());
            lock.wifi_error = None;
        }
        tokio::spawn(async move {
            let mut cmd = tokio::process::Command::new("nmcli");
            cmd.arg("dev").arg("wifi").arg("connect").arg(&ssid_string);
            if let Some(ref pwd) = pwd_string
                && !pwd.is_empty()
            {
                cmd.arg("password").arg(pwd);
            }
            let res = cmd.output().await;
            let mut error_msg = None;
            match res {
                Ok(output) => {
                    if !output.status.success() {
                        let err_text = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        let out_text = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let combined = if !err_text.is_empty() {
                            err_text
                        } else if !out_text.is_empty() {
                            out_text
                        } else {
                            "Error al conectar".to_string()
                        };
                        error_msg = Some(combined);
                    }
                }
                Err(e) => {
                    error_msg = Some(e.to_string());
                }
            }

            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(3), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
                lock.connecting_wifi_ssid = None;
                lock.wifi_error = error_msg;
                return;
            }
            if let Ok(mut lock) = status_arc.lock() {
                lock.connecting_wifi_ssid = None;
                lock.wifi_error = error_msg;
            }
        });
    }

    pub fn disconnect_wifi(&self, ssid: &str) {
        let ssid_string = ssid.to_string();
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("nmcli")
                .arg("con")
                .arg("down")
                .arg("id")
                .arg(&ssid_string)
                .output()
                .await;
            sleep(Duration::from_millis(500)).await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
            }
        });
    }

    pub fn start_bluetooth_scan(&self) {
        let status_arc = self.status.clone();
        if let Ok(mut lock) = status_arc.lock() {
            lock.is_scanning_bluetooth = true;
        }
        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(adapter) = BluezAdapterProxy::new(&conn).await
            {
                let _ = adapter.start_discovery().await;
            }
            let _ = tokio::process::Command::new("bluetoothctl")
                .arg("--timeout")
                .arg("12")
                .arg("scan")
                .arg("on")
                .output()
                .await;
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(adapter) = BluezAdapterProxy::new(&conn).await {
                    let _ = adapter.stop_discovery().await;
                }
                if let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                    && let Ok(mut lock) = status_arc.lock()
                {
                    *lock = new_status;
                    lock.is_scanning_bluetooth = false;
                    return;
                }
            }
            if let Ok(mut lock) = status_arc.lock() {
                lock.is_scanning_bluetooth = false;
            }
        });
    }

    pub fn connect_bluetooth(&self, mac: &str) {
        let mac_string = mac.to_string();
        let status_arc = self.status.clone();
        if let Ok(mut lock) = status_arc.lock() {
            lock.connecting_bluetooth_mac = Some(mac_string.clone());
            lock.bluetooth_error = None;
        }
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("bluetoothctl")
                .arg("pair")
                .arg(&mac_string)
                .output()
                .await;
            let conn_out = tokio::process::Command::new("bluetoothctl")
                .arg("connect")
                .arg(&mac_string)
                .output()
                .await;
            let mut error_msg = None;
            if let Ok(out) = conn_out
                && !out.status.success()
            {
                let err_text = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let out_text = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let combined = if !err_text.is_empty() {
                    err_text
                } else if !out_text.is_empty() {
                    out_text
                } else {
                    "Error al conectar".to_string()
                };
                error_msg = Some(combined);
            }
            sleep(Duration::from_millis(500)).await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
                lock.connecting_bluetooth_mac = None;
                lock.bluetooth_error = error_msg;
                return;
            }
            if let Ok(mut lock) = status_arc.lock() {
                lock.connecting_bluetooth_mac = None;
                lock.bluetooth_error = error_msg;
            }
        });
    }

    pub fn disconnect_bluetooth(&self, mac: &str) {
        let mac_string = mac.to_string();
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("bluetoothctl")
                .arg("disconnect")
                .arg(&mac_string)
                .output()
                .await;
            sleep(Duration::from_millis(500)).await;
            if let Some(conn) = get_shared_system_conn().await
                && let Ok(new_status) =
                    timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                && let Ok(mut lock) = status_arc.lock()
            {
                *lock = new_status;
            }
        });
    }

    async fn fetch_saved_ssids(conn: &zbus::Connection) -> Vec<String> {
        let mut saved_ssids = Vec::new();
        let Ok(settings) = NMSettingsProxy::new(conn).await else {
            return saved_ssids;
        };
        let Ok(conns) = settings.list_connections().await else {
            return saved_ssids;
        };
        for c_path in conns {
            let Ok(builder) = NMSettingsConnectionProxy::builder(conn).path(&c_path) else {
                continue;
            };
            let Ok(c_proxy) = builder.build().await else {
                continue;
            };
            let Ok(dict) = c_proxy.get_settings().await else {
                continue;
            };
            if let Some(wifi_settings) = dict.get("802-11-wireless")
                && let Some(ssid_val) = wifi_settings.get("ssid")
                && let Ok(ssid_bytes) = <Vec<u8>>::try_from((**ssid_val).clone())
            {
                saved_ssids.push(String::from_utf8_lossy(&ssid_bytes).to_string());
            }
        }
        saved_ssids
    }

    async fn fetch_active_connection_id(
        conn: &zbus::Connection,
        dict: &HashMap<String, OwnedValue>,
    ) -> Option<String> {
        let ac_val = dict.get("ActiveConnection")?;
        let ac_path = <&ObjectPath>::try_from(&**ac_val).ok()?;
        let builder = PropertiesProxy::builder(conn)
            .destination("org.freedesktop.NetworkManager")
            .ok()?
            .path(ac_path.clone())
            .ok()?;
        let ac_props = builder.build().await.ok()?;
        let iface_active =
            InterfaceName::try_from("org.freedesktop.NetworkManager.Connection.Active").ok()?;
        let ac_id_val = ac_props.get(iface_active, "Id").await.ok()?;
        let ac_id = <&str>::try_from(&*ac_id_val).ok()?;
        Some(ac_id.to_string())
    }

    async fn fetch_status_async(conn: &zbus::Connection) -> NetworkStatus {
        let mut status = NetworkStatus::default();
        let saved_ssids = Self::fetch_saved_ssids(conn).await;

        if let Ok(nm) = NetworkManagerProxy::new(conn).await {
            status.wifi_enabled = nm.wireless_enabled().await.unwrap_or(false);

            if let Ok(devices) = nm.get_devices().await {
                for dev_path in devices {
                    let Ok(builder) = PropertiesProxy::builder(conn)
                        .destination("org.freedesktop.NetworkManager")
                    else {
                        continue;
                    };
                    let Ok(builder) = builder.path(&dev_path) else {
                        continue;
                    };
                    let Ok(props) = builder.build().await else {
                        continue;
                    };

                    let Ok(iface_dev) =
                        InterfaceName::try_from("org.freedesktop.NetworkManager.Device")
                    else {
                        continue;
                    };
                    let Ok(dict) = props.get_all(iface_dev).await else {
                        continue;
                    };

                    let dev_type = dict
                        .get("DeviceType")
                        .and_then(|v| <u32>::try_from(&**v).ok())
                        .unwrap_or(0);
                    let state = dict
                        .get("State")
                        .and_then(|v| <u32>::try_from(&**v).ok())
                        .unwrap_or(0);
                    let is_conn = state == 100;

                    if dev_type == 1 && is_conn {
                        status.ethernet_connected = true;
                        if let Some(id) = Self::fetch_active_connection_id(conn, &dict).await {
                            status.ethernet_name = id;
                        }
                    } else if dev_type == 2 {
                        if is_conn
                            && let Some(id) = Self::fetch_active_connection_id(conn, &dict).await
                        {
                            status.wifi_ssid = id;
                        }

                        if status.wifi_enabled
                            && let Ok(builder) = NMWirelessProxy::builder(conn).path(&dev_path)
                            && let Ok(wireless) = builder.build().await
                            && let Ok(aps) = wireless.get_all_access_points().await
                        {
                            for ap_path in aps.into_iter().take(35) {
                                let Ok(builder) = PropertiesProxy::builder(conn)
                                    .destination("org.freedesktop.NetworkManager")
                                else {
                                    continue;
                                };
                                let Ok(builder) = builder.path(&ap_path) else {
                                    continue;
                                };
                                let Ok(ap_props) = builder.build().await else {
                                    continue;
                                };
                                let Ok(iface_ap) = InterfaceName::try_from(
                                    "org.freedesktop.NetworkManager.AccessPoint",
                                ) else {
                                    continue;
                                };
                                if let Ok(ap_dict) = ap_props.get_all(iface_ap).await {
                                    let mut ssid_str = String::new();
                                    if let Some(ssid_val) = ap_dict.get("Ssid")
                                        && let Ok(ssid_bytes) =
                                            <Vec<u8>>::try_from((**ssid_val).clone())
                                    {
                                        ssid_str = String::from_utf8_lossy(&ssid_bytes).to_string();
                                    }
                                    if ssid_str.is_empty() {
                                        continue;
                                    }
                                    let strength = ap_dict
                                        .get("Strength")
                                        .and_then(|v| <u8>::try_from(&**v).ok())
                                        .unwrap_or(0);
                                    let rsn_flags = ap_dict
                                        .get("RsnFlags")
                                        .and_then(|v| <u32>::try_from(&**v).ok())
                                        .unwrap_or(0);
                                    let wpa_flags = ap_dict
                                        .get("WpaFlags")
                                        .and_then(|v| <u32>::try_from(&**v).ok())
                                        .unwrap_or(0);

                                    let security = if rsn_flags != 0 || wpa_flags != 0 {
                                        "WPA".to_string()
                                    } else {
                                        "".to_string()
                                    };

                                    let is_active = status.wifi_ssid == ssid_str;
                                    if is_active {
                                        status.wifi_signal = strength;
                                    }
                                    let is_saved = saved_ssids.iter().any(|s| s == &ssid_str);
                                    if !status.wifi_ap_list.iter().any(|ap| ap.ssid == ssid_str) {
                                        status.wifi_ap_list.push(WifiAccessPoint {
                                            ssid: ssid_str,
                                            signal: strength,
                                            security,
                                            is_connected: is_active,
                                            is_saved,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Ok(builder) = BluezObjectManagerProxy::builder(conn)
            .destination("org.bluez")
            .and_then(|b| b.path("/"))
            && let Ok(om) = builder.build().await
            && let Ok(objects) = om.get_managed_objects().await
        {
            for (_path, ifaces) in objects {
                if let Some(adapter_props) = ifaces.get("org.bluez.Adapter1") {
                    if let Some(powered_val) = adapter_props.get("Powered")
                        && let Ok(powered) = <bool>::try_from(&**powered_val)
                    {
                        status.bluetooth_enabled = powered;
                    }
                    if let Some(discovering_val) = adapter_props.get("Discovering")
                        && let Ok(discovering) = <bool>::try_from(&**discovering_val)
                    {
                        status.is_scanning_bluetooth = discovering;
                    }
                }
                if let Some(device_props) = ifaces.get("org.bluez.Device1") {
                    let name = device_props
                        .get("Name")
                        .and_then(|v| <&str>::try_from(&**v).ok())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let address = device_props
                        .get("Address")
                        .and_then(|v| <&str>::try_from(&**v).ok())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let connected = device_props
                        .get("Connected")
                        .and_then(|v| <bool>::try_from(&**v).ok())
                        .unwrap_or(false);
                    let paired = device_props
                        .get("Paired")
                        .and_then(|v| <bool>::try_from(&**v).ok())
                        .unwrap_or(false);

                    if connected && status.bluetooth_device_name.is_empty() {
                        status.bluetooth_device_name = name.clone();
                    }
                    status.bluetooth_device_list.push(BluetoothDeviceItem {
                        mac: address,
                        name,
                        is_connected: connected,
                        is_paired: paired,
                    });
                }
            }
        }

        status
    }

    fn start_polling(&self) {
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            loop {
                if let Some(conn) = get_shared_system_conn().await
                    && let Ok(mut status) =
                        timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                {
                    if let Ok(guard) = status_arc.lock() {
                        if status.connecting_wifi_ssid.is_none() {
                            status.connecting_wifi_ssid = guard.connecting_wifi_ssid.clone();
                        }
                        if status.connecting_bluetooth_mac.is_none() {
                            status.connecting_bluetooth_mac =
                                guard.connecting_bluetooth_mac.clone();
                        }
                        if status.wifi_error.is_none() {
                            status.wifi_error = guard.wifi_error.clone();
                        }
                        if status.bluetooth_error.is_none() {
                            status.bluetooth_error = guard.bluetooth_error.clone();
                        }
                        if guard.is_scanning_wifi {
                            status.is_scanning_wifi = true;
                        }
                        if guard.is_scanning_bluetooth {
                            status.is_scanning_bluetooth = true;
                        }
                    }
                    if let Ok(mut guard) = status_arc.lock() {
                        *guard = status;
                    }
                }
                sleep(Duration::from_secs(3)).await;
            }
        });
    }
}
