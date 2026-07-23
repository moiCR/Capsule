use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use zbus::interface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub received_at: Instant,
    pub timeout: Duration,
}

#[derive(Clone, Default)]
pub struct NotificationStore {
    items: Arc<Mutex<Vec<NotificationItem>>>,
    latest_notification: Arc<Mutex<Option<NotificationItem>>>,
    counter: Arc<Mutex<u32>>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            latest_notification: Arc::new(Mutex::new(None)),
            counter: Arc::new(Mutex::new(1)),
        }
    }

    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::OnceLock<NotificationStore> = std::sync::OnceLock::new();
        INSTANCE.get_or_init(NotificationStore::new)
    }

    pub fn add_notification(
        &self,
        app_name: String,
        app_icon: String,
        summary: String,
        body: String,
        timeout_ms: i32,
    ) -> u32 {
        let mut cnt_guard = self.counter.lock().unwrap();
        let id = *cnt_guard;
        *cnt_guard = cnt_guard.wrapping_add(1).max(1);

        let timeout_duration = if timeout_ms > 0 {
            Duration::from_millis(timeout_ms as u64)
        } else {
            Duration::from_secs(5)
        };

        let item = NotificationItem {
            id,
            app_name,
            app_icon,
            summary,
            body,
            received_at: Instant::now(),
            timeout: timeout_duration,
        };

        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.push(item.clone());
        }

        if let Ok(mut latest_guard) = self.latest_notification.lock() {
            *latest_guard = Some(item);
        }

        id
    }

    pub fn get_latest_active_notification(&self) -> Option<NotificationItem> {
        if let Ok(guard) = self.latest_notification.lock() {
            if let Some(item) = guard.as_ref() {
                if item.received_at.elapsed() < item.timeout {
                    return Some(item.clone());
                }
            }
        }
        None
    }

    pub fn get_all_notifications(&self) -> Vec<NotificationItem> {
        self.items.lock().map(|i| i.clone()).unwrap_or_default()
    }

    pub fn remove_notification(&self, id: u32) {
        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.retain(|item| item.id != id);
        }
        if let Ok(mut latest_guard) = self.latest_notification.lock() {
            if let Some(item) = latest_guard.as_ref() {
                if item.id == id {
                    *latest_guard = None;
                }
            }
        }
    }

    pub fn clear_all_notifications(&self) {
        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.clear();
        }
        if let Ok(mut latest_guard) = self.latest_notification.lock() {
            *latest_guard = None;
        }
    }
}

pub struct NotificationServer {
    store: NotificationStore,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    async fn notify(
        &self,
        app_name: String,
        _replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        _hints: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        self.store
            .add_notification(app_name, app_icon, summary, body, expire_timeout)
    }

    async fn close_notification(&self, _id: u32) {}

    async fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "body-markup".to_string(),
            "actions".to_string(),
            "icon-static".to_string(),
        ]
    }

    async fn get_server_information(&self) -> (String, String, String, String) {
        (
            "Capsule Notifications".to_string(),
            "Capsule".to_string(),
            "0.1.0".to_string(),
            "1.2".to_string(),
        )
    }
}

pub async fn start_notification_server() -> anyhow::Result<()> {
    let store = NotificationStore::global().clone();
    let server = NotificationServer { store };

    let _connection = zbus::connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", server)?
        .build()
        .await?;

    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
    }
}
