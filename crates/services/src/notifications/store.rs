use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::item::{LatestNotification, NotificationItem};
use super::server::NotificationServer;

#[derive(Clone)]
pub struct NotificationStore {
    pub(crate) items: Arc<Mutex<Vec<NotificationItem>>>,
    pub(crate) latest_notification: Arc<Mutex<LatestNotification>>,
    pub(crate) connection: Arc<OnceLock<zbus::Connection>>,
    pub(crate) senders: Arc<Mutex<HashMap<u32, zbus::names::OwnedUniqueName>>>,
    counter: Arc<Mutex<u32>>,
    dnd: Arc<AtomicBool>,
    changes: tokio::sync::watch::Sender<()>,
}

impl Default for NotificationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationStore {
    pub fn subscribe(&self) -> tokio::sync::watch::Receiver<()> {
        self.changes.subscribe()
    }

    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            latest_notification: Arc::new(Mutex::new(LatestNotification::default())),
            connection: Arc::new(OnceLock::new()),
            senders: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(Mutex::new(1)),
            dnd: Arc::new(AtomicBool::new(false)),
            changes: tokio::sync::watch::channel(()).0,
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
        self.add_notification_with_actions(
            (app_name, app_icon, summary, body),
            timeout_ms,
            Vec::new(),
            None,
        )
    }

    pub(crate) fn add_notification_with_actions(
        &self,
        content: (String, String, String, String),
        timeout_ms: i32,
        actions: Vec<String>,
        sender: Option<zbus::names::OwnedUniqueName>,
    ) -> u32 {
        let mut senders = self
            .senders
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (app_name, app_icon, summary, body) = content;
        let mut cnt_guard = self
            .counter
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let id = *cnt_guard;
        *cnt_guard = cnt_guard.wrapping_add(1).max(1);
        if let Some(sender) = sender {
            senders.insert(id, sender);
        } else {
            senders.remove(&id);
        }

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
            actions: actions
                .as_chunks::<2>()
                .0
                .iter()
                .map(|[key, label]| (key.clone(), label.clone()))
                .collect(),
            received_at: Instant::now(),
            timeout: timeout_duration,
        };

        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.push(item.clone());
        }

        if !self.is_dnd_enabled() {
            if let Ok(mut latest_guard) = self.latest_notification.lock() {
                *latest_guard = LatestNotification {
                    item: Some(item),
                    hovered_at: None,
                };
            }
        }

        self.changes.send_replace(());
        id
    }

    pub fn is_dnd_enabled(&self) -> bool {
        self.dnd.load(Ordering::SeqCst)
    }

    pub fn set_dnd(&self, enabled: bool) {
        self.dnd.store(enabled, Ordering::SeqCst);
        self.changes.send_replace(());
    }

    pub fn toggle_dnd(&self) -> bool {
        let prev = self.dnd.fetch_xor(true, Ordering::SeqCst);
        self.changes.send_replace(());
        !prev
    }

    pub fn get_latest_active_notification(&self) -> Option<NotificationItem> {
        if self.is_dnd_enabled() {
            return None;
        }
        if let Ok(guard) = self.latest_notification.lock() {
            if let Some(item) = guard.item.as_ref() {
                let is_hover_active = guard
                    .hovered_at
                    .as_ref()
                    .is_some_and(|hovered_at| hovered_at.elapsed() < Duration::from_secs(60));
                if is_hover_active || item.received_at.elapsed() < item.timeout {
                    return Some(item.clone());
                }
            }
        }
        None
    }

    pub fn is_hovered(&self) -> bool {
        if let Ok(guard) = self.latest_notification.lock() {
            guard.hovered_at.is_some()
        } else {
            false
        }
    }

    pub fn set_hovered(&self, hovered: bool) {
        self.set_hovered_at(hovered, Instant::now());
        self.changes.send_replace(());
    }

    pub(crate) fn set_hovered_at(&self, hovered: bool, now: Instant) {
        let Ok(mut latest) = self.latest_notification.lock() else {
            return;
        };
        if hovered {
            if latest.hovered_at.is_none()
                && latest.item.as_ref().is_some_and(|item| {
                    now.saturating_duration_since(item.received_at) < item.timeout
                })
            {
                latest.hovered_at = Some(now);
            }
        } else if let Some(hovered_at) = latest.hovered_at.take() {
            if let Some(item) = latest.item.as_mut() {
                let paused_duration = now
                    .saturating_duration_since(hovered_at)
                    .min(Duration::from_secs(60));
                if let Some(received_at) = item.received_at.checked_add(paused_duration) {
                    item.received_at = received_at;
                }
            }
        }
    }

    pub fn invoke_action(&self, id: u32, key: String) {
        let valid = self.items.lock().is_ok_and(|items| {
            items
                .iter()
                .any(|item| item.id == id && item.actions.iter().any(|(action, _)| action == &key))
        });
        if !valid {
            return;
        }
        let Some(connection) = self.connection.get().cloned() else {
            return;
        };
        let store = self.clone();
        crate::spawn_tokio(async move {
            if connection
                .emit_signal(
                    None::<&str>,
                    "/org/freedesktop/Notifications",
                    "org.freedesktop.Notifications",
                    "ActionInvoked",
                    &(id, key),
                )
                .await
                .is_ok()
            {
                store.remove_notification(id);
            }
        });
    }

    pub async fn reply(&self, id: u32, text: String) -> anyhow::Result<()> {
        anyhow::ensure!(!text.trim().is_empty(), "Reply text must not be blank");
        {
            let items = self
                .items
                .lock()
                .map_err(|_| anyhow::anyhow!("Notification store lock is poisoned"))?;
            let item = items
                .iter()
                .find(|item| item.id == id)
                .ok_or_else(|| anyhow::anyhow!("Notification {id} was not found"))?;
            anyhow::ensure!(
                item.actions.iter().any(|(key, _)| key == "inline-reply"),
                "Notification {id} does not advertise inline-reply"
            );
        }
        let sender = self
            .senders
            .lock()
            .map_err(|_| anyhow::anyhow!("Notification sender map lock is poisoned"))?
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Notification {id} has no recorded sender"))?;
        let connection = self
            .connection
            .get()
            .ok_or_else(|| anyhow::anyhow!("Notification server connection is not initialized"))?;
        let emitter =
            zbus::object_server::SignalEmitter::new(connection, "/org/freedesktop/Notifications")?
                .set_destination(sender.into());
        NotificationServer::notification_replied(&emitter, id, text).await?;
        self.remove_notification(id);
        Ok(())
    }

    pub fn contains_notification(&self, id: u32) -> bool {
        self.items
            .lock()
            .is_ok_and(|items| items.iter().any(|item| item.id == id))
    }

    pub fn get_all_notifications(&self) -> Vec<NotificationItem> {
        self.items.lock().map(|i| i.clone()).unwrap_or_default()
    }

    pub fn remove_notification(&self, id: u32) {
        let mut senders = self
            .senders
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        senders.remove(&id);
        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.retain(|item| item.id != id);
        }
        if let Ok(mut latest_guard) = self.latest_notification.lock() {
            if let Some(item) = latest_guard.item.as_ref() {
                if item.id == id {
                    *latest_guard = LatestNotification::default();
                }
            }
        }
        self.changes.send_replace(());
    }

    pub fn clear_all_notifications(&self) {
        let mut senders = self
            .senders
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        senders.clear();
        if let Ok(mut items_guard) = self.items.lock() {
            items_guard.clear();
        }
        if let Ok(mut latest_guard) = self.latest_notification.lock() {
            *latest_guard = LatestNotification::default();
        }
        self.changes.send_replace(());
    }
}
