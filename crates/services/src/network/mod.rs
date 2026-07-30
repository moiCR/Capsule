use crate::dbus_util::get_shared_system_conn;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, timeout, Duration};
use zbus::fdo::PropertiesProxy;
use zbus::names::InterfaceName;
use zbus::proxy;
use zbus::zvariant::ObjectPath;

#[derive(Clone, Debug, Default)]
pub struct WifiAccessPoint {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub is_connected: bool,
}

#[derive(Clone, Debug, Default)]
pub struct BluetoothDeviceItem {
    pub mac: String,
    pub name: String,
    pub is_connected: bool,
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
}

#[proxy(interface = "org.bluez.Device1", default_service = "org.bluez")]
pub trait BluezDevice {
    fn connect(&self) -> zbus::Result<()>;
    fn disconnect(&self) -> zbus::Result<()>;
}

#[proxy(
    interface = "org.freedesktop.DBus.ObjectManager",
    default_service = "org.bluez",
    default_path = "/"
)]
pub trait BluezObjectManager {
    fn get_managed_objects(
        &self,
    ) -> zbus::Result<
        HashMap<
            zbus::zvariant::OwnedObjectPath,
            HashMap<String, HashMap<String, zbus::zvariant::OwnedValue>>,
        >,
    >;
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
        self.status.lock().unwrap().clone()
    }

    pub fn toggle_wifi(&self) {
        let is_on = self.get_status().wifi_enabled;
        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(nm) = NetworkManagerProxy::new(&conn).await {
                    let _ = nm.set_wireless_enabled(!is_on).await;
                }
            }
        });
    }

    pub fn toggle_bluetooth(&self) {
        let is_on = self.get_status().bluetooth_enabled;
        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(adapter) = BluezAdapterProxy::new(&conn).await {
                    let _ = adapter.set_powered(!is_on).await;
                }
            }
        });
    }

    pub fn connect_wifi(&self, ssid: &str) {
        let ssid = ssid.to_string();
        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await {
                let _ = Self::activate_wifi_connection(&conn, &ssid).await;
            }
        });
    }

    async fn activate_wifi_connection(
        conn: &zbus::Connection,
        ssid: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let nm = NetworkManagerProxy::new(conn).await?;
        let settings = NMSettingsProxy::new(conn).await?;

        // Find existing connection with this SSID
        let conns = settings.list_connections().await?;
        let mut target_conn = None;
        for c_path in conns {
            let c_proxy = NMSettingsConnectionProxy::builder(conn)
                .path(&c_path).unwrap()
                .build()
                .await?;
            if let Ok(dict) = c_proxy.get_settings().await {
                if let Some(wifi_settings) = dict.get("802-11-wireless") {
                    if let Some(ssid_val) = wifi_settings.get("ssid") {
                        if let Ok(ssid_bytes) = <Vec<u8>>::try_from((**ssid_val).clone()) {
                            if String::from_utf8_lossy(&ssid_bytes) == ssid {
                                target_conn = Some(c_path.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let devices = nm.get_devices().await?;
        let mut target_dev = None;
        let mut target_ap = None;

        for dev_path in devices {
            let props = PropertiesProxy::builder(conn)
                .destination("org.freedesktop.NetworkManager").unwrap()
                .path(&dev_path).unwrap()
                .build()
                .await?;

            let dev_type = props
                .get(
                    InterfaceName::try_from("org.freedesktop.NetworkManager.Device").unwrap(),
                    "DeviceType"
                )
                .await
                .and_then(|v| Ok(<u32>::try_from(&*v).unwrap_or(0)))
                .unwrap_or(0);

            if dev_type == 2 {
                // Wi-Fi
                target_dev = Some(dev_path.clone());
                // Find AP
                if let Ok(wireless) = NMWirelessProxy::builder(conn).path(&dev_path).unwrap().build().await {
                    if let Ok(aps) = wireless.get_all_access_points().await {
                        for ap_path in aps {
                            let ap_props = PropertiesProxy::builder(conn)
                                .destination("org.freedesktop.NetworkManager").unwrap()
                                .path(&ap_path).unwrap()
                                .build()
                                .await?;
                            if let Ok(ap_ssid_val) = ap_props
                                .get(
                                    InterfaceName::try_from("org.freedesktop.NetworkManager.AccessPoint").unwrap(),
                                    "Ssid"
                                )
                                .await
                            {
                                if let Ok(ap_ssid_bytes) = <Vec<u8>>::try_from((*ap_ssid_val).clone()) {
                                    if String::from_utf8_lossy(&ap_ssid_bytes) == ssid {
                                        target_ap = Some(ap_path.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                break; // Just use first wifi device
            }
        }

        if let (Some(c), Some(d), Some(a)) = (target_conn, target_dev, target_ap) {
            let _ = nm.activate_connection(&c, &d, &a).await?;
        }
        Ok(())
    }

    pub fn connect_bluetooth(&self, mac: &str) {
        let mac = mac.to_string();
        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(om) = BluezObjectManagerProxy::builder(&conn)
                    .destination("org.bluez").unwrap()
                    .path("/").unwrap()
                    .build()
                    .await
                {
                    if let Ok(objects) = om.get_managed_objects().await {
                        for (path, ifaces) in objects {
                            if let Some(device_props) = ifaces.get("org.bluez.Device1") {
                                if let Some(addr_val) = device_props.get("Address") {
                                    if let Ok(addr) = <&str>::try_from(&**addr_val) {
                                        if addr == mac {
                                            if let Ok(dev_proxy) = BluezDeviceProxy::builder(&conn)
                                                .path(&path).unwrap()
                                                .build()
                                                .await
                                            {
                                                let connected = device_props
                                                    .get("Connected")
                                                    .and_then(|v| <bool>::try_from(&**v).ok())
                                                    .unwrap_or(false);

                                                if connected {
                                                    let _ = dev_proxy.disconnect().await;
                                                } else {
                                                    let _ = dev_proxy.connect().await;
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    async fn fetch_status_async(conn: &zbus::Connection) -> NetworkStatus {
        let mut status = NetworkStatus::default();

        // 1. Check NetworkManager
        if let Ok(nm) = NetworkManagerProxy::new(conn).await {
            status.wifi_enabled = nm.wireless_enabled().await.unwrap_or(false);

            if let Ok(devices) = nm.get_devices().await {
                for dev_path in devices {
                    let props = match PropertiesProxy::builder(conn)
                        .destination("org.freedesktop.NetworkManager").unwrap()
                        .path(&dev_path).unwrap()
                        .build()
                        .await
                    {
                        Ok(p) => p,
                        Err(_) => continue,
                    };

                    let dict = match props.get_all(InterfaceName::try_from("org.freedesktop.NetworkManager.Device").unwrap()).await {
                        Ok(d) => d,
                        Err(_) => continue,
                    };

                    let dev_type = dict
                        .get("DeviceType")
                        .and_then(|v| <u32>::try_from(&**v).ok())
                        .unwrap_or(0);
                    let state = dict
                        .get("State")
                        .and_then(|v| <u32>::try_from(&**v).ok())
                        .unwrap_or(0);
                    let is_conn = state == 100; // Activated

                    if dev_type == 1 && is_conn {
                        // Ethernet
                        status.ethernet_connected = true;
                        if let Some(ac_val) = dict.get("ActiveConnection") {
                            if let Ok(ac_path) = <&ObjectPath>::try_from(&**ac_val) {
                                if let Ok(ac_props) = PropertiesProxy::builder(conn)
                                    .destination("org.freedesktop.NetworkManager").unwrap()
                                    .path(ac_path.clone()).unwrap()
                                    .build()
                                    .await
                                {
                                    if let Ok(ac_id_val) = ac_props
                                        .get(
                                            InterfaceName::try_from("org.freedesktop.NetworkManager.Connection.Active").unwrap(), 
                                            "Id"
                                        )
                                        .await
                                    {
                                        if let Ok(ac_id) = <&str>::try_from(&*ac_id_val) {
                                            status.ethernet_name = ac_id.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    } else if dev_type == 2 {
                        // Wi-Fi
                        if is_conn {
                            if let Some(ac_val) = dict.get("ActiveConnection") {
                                if let Ok(ac_path) = <&ObjectPath>::try_from(&**ac_val) {
                                    if let Ok(ac_props) = PropertiesProxy::builder(conn)
                                        .destination("org.freedesktop.NetworkManager").unwrap()
                                        .path(ac_path.clone()).unwrap()
                                        .build()
                                        .await
                                    {
                                        if let Ok(ac_id_val) = ac_props
                                            .get(
                                                InterfaceName::try_from("org.freedesktop.NetworkManager.Connection.Active").unwrap(),
                                                "Id",
                                            )
                                            .await
                                        {
                                            if let Ok(ac_id) = <&str>::try_from(&*ac_id_val) {
                                                status.wifi_ssid = ac_id.to_string();
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if status.wifi_enabled {
                            if let Ok(wireless) =
                                NMWirelessProxy::builder(conn).path(&dev_path).unwrap().build().await
                            {
                                if let Ok(aps) = wireless.get_all_access_points().await {
                                    // Limit to 30 APs to avoid bus congestion
                                    for ap_path in aps.into_iter().take(30) {
                                        if let Ok(ap_props) = PropertiesProxy::builder(conn)
                                            .destination("org.freedesktop.NetworkManager").unwrap()
                                            .path(&ap_path).unwrap()
                                            .build()
                                            .await
                                        {
                                            if let Ok(ap_dict) = ap_props
                                                .get_all(
                                                    InterfaceName::try_from("org.freedesktop.NetworkManager.AccessPoint").unwrap()
                                                )
                                                .await
                                            {
                                                let mut ssid_str = String::new();
                                                if let Some(ssid_val) = ap_dict.get("Ssid") {
                                                    if let Ok(ssid_bytes) = <Vec<u8>>::try_from((**ssid_val).clone())
                                                    {
                                                        ssid_str =
                                                            String::from_utf8_lossy(&ssid_bytes)
                                                                .to_string();
                                                    }
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
                                                if !status.wifi_ap_list
                                                    .iter()
                                                    .any(|ap| ap.ssid == ssid_str)
                                                {
                                                    status.wifi_ap_list.push(WifiAccessPoint {
                                                        ssid: ssid_str,
                                                        signal: strength,
                                                        security,
                                                        is_connected: is_active,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. Check BlueZ
        if let Ok(om) = BluezObjectManagerProxy::builder(conn)
            .destination("org.bluez").unwrap()
            .path("/").unwrap()
            .build()
            .await
        {
            if let Ok(objects) = om.get_managed_objects().await {
                for (_path, ifaces) in objects {
                    if let Some(adapter_props) = ifaces.get("org.bluez.Adapter1") {
                        if let Some(powered_val) = adapter_props.get("Powered") {
                            if let Ok(powered) = <bool>::try_from(&**powered_val) {
                                status.bluetooth_enabled = powered;
                            }
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

                        if connected {
                            if status.bluetooth_device_name.is_empty() {
                                status.bluetooth_device_name = name.clone();
                            }
                        }
                        status.bluetooth_device_list.push(BluetoothDeviceItem {
                            mac: address,
                            name,
                            is_connected: connected,
                        });
                    }
                }
            }
        }

        status
    }

    fn start_polling(&self) {
        let status_arc = self.status.clone();
        tokio::spawn(async move {
            loop {
                if let Some(conn) = get_shared_system_conn().await {
                    if let Ok(status) =
                        timeout(Duration::from_secs(2), Self::fetch_status_async(&conn)).await
                    {
                        *status_arc.lock().unwrap() = status;
                    }
                }
                sleep(Duration::from_secs(3)).await;
            }
        });
    }
}
