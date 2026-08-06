use crate::dbus_util::get_shared_system_conn;
use arc_swap::ArcSwap;
use std::sync::Arc;
use tokio::time::Duration;
use zbus::fdo::PropertiesProxy;
use zbus::zvariant::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerProfile {
    Performance,
    Balanced,
    PowerSaver,
}

impl PowerProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            PowerProfile::Performance => "performance",
            PowerProfile::Balanced => "balanced",
            PowerProfile::PowerSaver => "power-saver",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "performance" => PowerProfile::Performance,
            "power-saver" => PowerProfile::PowerSaver,
            _ => PowerProfile::Balanced,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PowerProfile::Performance => "Performance",
            PowerProfile::Balanced => "Balanced",
            PowerProfile::PowerSaver => "Power Saver",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            PowerProfile::Performance => "zap.svg",
            PowerProfile::Balanced => "scale.svg",
            PowerProfile::PowerSaver => "leaf.svg",
        }
    }
}

#[derive(Clone)]
pub struct PowerService {
    active_profile: Arc<ArcSwap<PowerProfile>>,
}

impl Default for PowerService {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerService {
    pub fn new() -> Self {
        let service = Self {
            active_profile: Arc::new(ArcSwap::from_pointee(PowerProfile::Balanced)),
        };

        let svc_clone = service.clone();
        tokio::spawn(async move {
            svc_clone.start_polling().await;
        });

        service
    }

    pub fn get_active_profile(&self) -> PowerProfile {
        (**self.active_profile.load()).clone()
    }

    pub fn set_active_profile(&self, profile: PowerProfile) {
        self.active_profile.store(Arc::new(profile.clone()));

        tokio::spawn(async move {
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(proxy) = PropertiesProxy::builder(&conn)
                    .destination("net.hadess.PowerProfiles")
                    .unwrap()
                    .path("/net/hadess/PowerProfiles")
                    .unwrap()
                    .build()
                    .await
                {
                    if let Ok(iface) = "net.hadess.PowerProfiles".try_into() {
                        let _ = proxy
                            .set(iface, "ActiveProfile", Value::from(profile.as_str()))
                            .await;
                    }
                }
            }
        });
    }

    async fn start_polling(&self) {
        loop {
            if let Some(conn) = get_shared_system_conn().await {
                if let Ok(proxy) = PropertiesProxy::builder(&conn)
                    .destination("net.hadess.PowerProfiles")
                    .unwrap()
                    .path("/net/hadess/PowerProfiles")
                    .unwrap()
                    .build()
                    .await
                {
                    if let Ok(iface) = "net.hadess.PowerProfiles".try_into() {
                        if let Ok(val) = proxy.get(iface, "ActiveProfile").await {
                            if let Ok(str_val) = String::try_from(val) {
                                self.active_profile
                                    .store(Arc::new(PowerProfile::from_str(&str_val)));
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
}
