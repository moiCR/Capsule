use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use zbus::interface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationItem {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
    pub received_at: Instant,
    pub timeout: Duration,
}

#[derive(Default)]
struct LatestNotification {
    item: Option<NotificationItem>,
    hovered_at: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct NotificationStore {
    items: Arc<Mutex<Vec<NotificationItem>>>,
    latest_notification: Arc<Mutex<LatestNotification>>,
    connection: Arc<OnceLock<zbus::Connection>>,
    senders: Arc<Mutex<HashMap<u32, zbus::names::OwnedUniqueName>>>,
    counter: Arc<Mutex<u32>>,
    dnd: Arc<AtomicBool>,
}

impl NotificationStore {
    pub fn new() -> Self {
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            latest_notification: Arc::new(Mutex::new(LatestNotification::default())),
            connection: Arc::new(OnceLock::new()),
            senders: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(Mutex::new(1)),
            dnd: Arc::new(AtomicBool::new(false)),
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

    fn add_notification_with_actions(
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

        id
    }

    pub fn is_dnd_enabled(&self) -> bool {
        self.dnd.load(Ordering::SeqCst)
    }

    pub fn set_dnd(&self, enabled: bool) {
        self.dnd.store(enabled, Ordering::SeqCst);
    }

    pub fn toggle_dnd(&self) -> bool {
        let prev = self.dnd.fetch_xor(true, Ordering::SeqCst);
        !prev
    }

    pub fn get_latest_active_notification(&self) -> Option<NotificationItem> {
        if self.is_dnd_enabled() {
            return None;
        }
        if let Ok(guard) = self.latest_notification.lock() {
            if let Some(item) = guard.item.as_ref() {
                if guard.hovered_at.is_some() || item.received_at.elapsed() < item.timeout {
                    return Some(item.clone());
                }
            }
        }
        None
    }

    pub fn set_hovered(&self, hovered: bool) {
        self.set_hovered_at(hovered, Instant::now());
    }

    fn set_hovered_at(&self, hovered: bool, now: Instant) {
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
        } else if let Some(hovered_at) = latest.hovered_at.take()
            && let Some(item) = latest.item.as_mut()
            && let Some(received_at) = item
                .received_at
                .checked_add(now.saturating_duration_since(hovered_at))
        {
            item.received_at = received_at;
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
        tokio::spawn(async move {
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
        actions: Vec<String>,
        _hints: std::collections::HashMap<String, zbus::zvariant::Value<'_>>,
        expire_timeout: i32,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> u32 {
        self.store.add_notification_with_actions(
            (app_name, app_icon, summary, body),
            expire_timeout,
            actions,
            header.sender().map(|sender| sender.to_owned().into()),
        )
    }

    async fn close_notification(&self, id: u32) {
        self.store.remove_notification(id);
    }

    #[zbus(signal)]
    async fn notification_replied(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        id: u32,
        text: String,
    ) -> zbus::Result<()>;

    async fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_string(),
            "body-markup".to_string(),
            "actions".to_string(),
            "inline-reply".to_string(),
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
    let server = NotificationServer {
        store: store.clone(),
    };

    let connection = zbus::connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", server)?
        .build()
        .await?;
    store
        .connection
        .set(connection)
        .map_err(|_| anyhow::anyhow!("Notification server connection is already initialized"))?;

    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add_test_notification(store: &NotificationStore) -> u32 {
        store.add_notification(
            "TestApp".to_string(),
            String::new(),
            "Title".to_string(),
            String::new(),
            5000,
        )
    }

    fn add_reply_notification(store: &NotificationStore, key: &str, label: &str) -> u32 {
        store.add_notification_with_actions(
            (String::new(), String::new(), String::new(), String::new()),
            5000,
            vec![key.into(), label.into()],
            None,
        )
    }

    #[tokio::test]
    async fn reply_validation_preserves_notifications() {
        let store = NotificationStore::new();
        let id = add_reply_notification(&store, "inline-reply", "Répondre");
        store.set_hovered(true);
        let original = store.get_all_notifications();
        for text in ["", " \t\n", "\u{2003}"] {
            let error = store.reply(id, text.into()).await.unwrap_err();
            assert!(error.to_string().contains("must not be blank"));
        }
        let error = store.reply(0, "Hello".into()).await.unwrap_err();
        assert!(error.to_string().contains("was not found"));
        let error = store.reply(id, "Hello".into()).await.unwrap_err();
        assert!(error.to_string().contains("has no recorded sender"));
        store
            .senders
            .lock()
            .unwrap()
            .insert(id, ":1.42".try_into().unwrap());
        let error = store.reply(id, "Hello".into()).await.unwrap_err();
        assert!(error.to_string().contains("connection is not initialized"));
        assert_eq!(store.get_all_notifications(), original);
        assert_eq!(
            store.get_latest_active_notification(),
            original.first().cloned()
        );
        assert!(
            store
                .latest_notification
                .lock()
                .unwrap()
                .hovered_at
                .is_some()
        );
    }

    #[tokio::test]
    async fn reply_requires_exact_action_key_not_label() {
        let store = NotificationStore::new();
        for key in ["reply", "Inline-Reply", "inline-reply ", "default"] {
            let id = add_reply_notification(&store, key, "inline-reply");
            let original = store.get_all_notifications();
            let error = store.reply(id, "Hello".into()).await.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("does not advertise inline-reply")
            );
            assert_eq!(store.get_all_notifications(), original);
        }
        let id = add_test_notification(&store);
        assert!(store.reply(id, "Hello".into()).await.is_err());
    }

    #[tokio::test]
    async fn contains_notification_tracks_close_and_clear_not_latest() {
        let store = NotificationStore::new();
        let first = add_test_notification(&store);
        let second = add_test_notification(&store);
        {
            let mut senders = store.senders.lock().unwrap();
            senders.insert(first, ":1.42".try_into().unwrap());
            senders.insert(second, ":1.43".try_into().unwrap());
        }
        assert!(store.contains_notification(first));
        assert!(store.contains_notification(second));
        assert!(!store.contains_notification(0));
        assert_eq!(store.get_latest_active_notification().unwrap().id, second);
        store.set_dnd(true);
        assert!(store.contains_notification(first));
        let server = NotificationServer {
            store: store.clone(),
        };
        server.close_notification(first).await;
        assert!(!store.contains_notification(first));
        assert!(store.contains_notification(second));
        assert!(!store.senders.lock().unwrap().contains_key(&first));
        assert!(store.senders.lock().unwrap().contains_key(&second));
        store.clear_all_notifications();
        assert!(!store.contains_notification(second));
        assert!(store.senders.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn inline_reply_capability_is_advertised() {
        let server = NotificationServer {
            store: NotificationStore::new(),
        };
        assert!(
            server
                .get_capabilities()
                .await
                .iter()
                .any(|capability| capability == "inline-reply")
        );
    }

    #[tokio::test]
    #[ignore = "Requires dbus-daemon; launches a private bus, never connects to the session bus"]
    async fn reply_on_private_bus() -> anyhow::Result<()> {
        use futures::StreamExt;
        use std::process::Stdio;
        use tokio::io::{AsyncBufReadExt, BufReader};

        let mut daemon = tokio::process::Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--nopidfile", "--print-address=1"])
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let stdout = daemon
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Missing daemon stdout"))?;
        let mut lines = BufReader::new(stdout).lines();
        let address = tokio::time::timeout(Duration::from_secs(5), lines.next_line())
            .await??
            .ok_or_else(|| anyhow::anyhow!("Missing private bus address"))?;
        tokio::time::timeout(Duration::from_secs(10), async {
            let store = NotificationStore::new();
            let connection = zbus::connection::Builder::address(address.as_str())?
                .name("org.freedesktop.Notifications")?
                .serve_at(
                    "/org/freedesktop/Notifications",
                    NotificationServer {
                        store: store.clone(),
                    },
                )?
                .build()
                .await?;
            store
                .connection
                .set(connection.clone())
                .map_err(|_| anyhow::anyhow!("Connection already set"))?;
            let client = zbus::connection::Builder::address(address.as_str())?
                .build()
                .await?;
            let proxy = zbus::Proxy::new(
                &client,
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
            )
            .await?;
            let introspection = zbus::fdo::IntrospectableProxy::builder(&client)
                .destination("org.freedesktop.Notifications")?
                .path("/org/freedesktop/Notifications")?
                .build()
                .await?
                .introspect()
                .await?;
            assert!(introspection.contains("<signal name=\"NotificationReplied\">"));
            let mut signals = proxy.receive_all_signals().await?;
            let observer = zbus::connection::Builder::address(address.as_str())?
                .build()
                .await?;
            let observer_proxy = zbus::Proxy::new(
                &observer,
                "org.freedesktop.Notifications",
                "/org/freedesktop/Notifications",
                "org.freedesktop.Notifications",
            )
            .await?;
            let mut observer_signals = observer_proxy.receive_all_signals().await?;
            let notification = (
                "TestApp",
                0_u32,
                "",
                "Title",
                "Body",
                vec!["inline-reply", "Répondre"],
                HashMap::<String, zbus::zvariant::Value<'_>>::new(),
                5000_i32,
            );
            let missing_sender = add_reply_notification(&store, "inline-reply", "Reply");
            let error = store
                .reply(missing_sender, "Private".into())
                .await
                .unwrap_err();
            assert!(error.to_string().contains("has no recorded sender"));
            assert!(store.contains_notification(missing_sender));
            store.remove_notification(missing_sender);
            let id: u32 = proxy.call("Notify", &notification).await?;
            assert_eq!(
                store
                    .senders
                    .lock()
                    .unwrap()
                    .get(&id)
                    .map(|sender| sender.as_str()),
                client.unique_name().map(|name| name.as_str())
            );
            store.set_hovered(true);
            let text = "  Bonjour 🌍\n".to_string();
            store.reply(id, text.clone()).await?;
            let message = signals
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("Signal stream ended"))?;
            assert_eq!(
                message.header().member().map(|member| member.as_str()),
                Some("NotificationReplied")
            );
            assert_eq!(message.body().deserialize::<(u32, String)>()?, (id, text));
            assert_eq!(
                message.header().sender().map(|name| name.as_str()),
                connection.unique_name().map(|name| name.as_str())
            );
            assert_eq!(
                message.header().destination().map(|name| name.as_str()),
                client.unique_name().map(|name| name.as_str())
            );
            assert!(!store.contains_notification(id));
            assert!(store.senders.lock().unwrap().is_empty());
            assert!(store.get_all_notifications().is_empty());
            assert!(store.get_latest_active_notification().is_none());
            assert!(
                store
                    .latest_notification
                    .lock()
                    .unwrap()
                    .hovered_at
                    .is_none()
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(100), signals.next())
                    .await
                    .is_err()
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(100), observer_signals.next())
                    .await
                    .is_err()
            );
            let closed_id: u32 = proxy.call("Notify", &notification).await?;
            let id: u32 = proxy.call("Notify", &notification).await?;
            assert!(store.contains_notification(closed_id));
            assert!(store.contains_notification(id));
            let _: () = proxy.call("CloseNotification", &(closed_id,)).await?;
            assert!(!store.contains_notification(closed_id));
            assert!(!store.senders.lock().unwrap().contains_key(&closed_id));
            assert!(store.contains_notification(id));
            store.set_hovered(true);
            let original = store.get_all_notifications();
            connection.close().await?;
            assert!(store.reply(id, "Not sent".into()).await.is_err());
            assert!(store.senders.lock().unwrap().contains_key(&id));
            assert_eq!(store.get_all_notifications(), original);
            assert_eq!(
                store.get_latest_active_notification(),
                original.first().cloned()
            );
            assert!(
                store
                    .latest_notification
                    .lock()
                    .unwrap()
                    .hovered_at
                    .is_some()
            );
            Ok::<(), anyhow::Error>(())
        })
        .await??;
        daemon.kill().await?;
        daemon.wait().await?;
        Ok(())
    }

    #[test]
    fn actions_are_published_as_pairs() {
        let store = NotificationStore::new();
        let id = store.add_notification_with_actions(
            (String::new(), String::new(), String::new(), String::new()),
            5000,
            vec![
                "default".into(),
                "Open".into(),
                "reply".into(),
                "Reply".into(),
                "unpaired".into(),
            ],
            None,
        );
        let latest = store.get_latest_active_notification().unwrap();
        assert_eq!(latest.id, id);
        assert_eq!(
            latest.actions,
            vec![
                ("default".into(), "Open".into()),
                ("reply".into(), "Reply".into())
            ]
        );
        assert_eq!(store.get_all_notifications(), vec![latest]);
        store.invoke_action(id, "missing".into());
        store.invoke_action(id, "default".into());
        assert_eq!(store.get_all_notifications().len(), 1);
        let id = add_test_notification(&store);
        assert_eq!(store.get_latest_active_notification().unwrap().id, id);
        assert!(
            store
                .get_latest_active_notification()
                .unwrap()
                .actions
                .is_empty()
        );
    }

    #[test]
    fn hover_preserves_remaining_timeout() {
        let store = NotificationStore::new();
        add_test_notification(&store);
        let received_at = Instant::now() - Duration::from_secs(20);
        store
            .latest_notification
            .lock()
            .unwrap()
            .item
            .as_mut()
            .unwrap()
            .received_at = received_at;
        let entered_at = received_at + Duration::from_secs(2);
        store.set_hovered_at(true, entered_at);
        store.set_hovered_at(true, entered_at + Duration::from_secs(1));
        assert!(store.get_latest_active_notification().is_some());
        let exited_at = entered_at + Duration::from_secs(10);
        store.set_hovered_at(false, exited_at);
        let latest = store.latest_notification.lock().unwrap();
        let item = latest.item.as_ref().unwrap();
        assert_eq!(item.received_at, received_at + Duration::from_secs(10));
        assert_eq!(
            item.timeout - exited_at.duration_since(item.received_at),
            Duration::from_secs(3)
        );
        assert!(latest.hovered_at.is_none());
        drop(latest);
        assert!(store.get_latest_active_notification().is_none());
    }

    #[test]
    fn replacement_does_not_inherit_hover_and_expired_items_stay_expired() {
        let store = NotificationStore::new();
        add_test_notification(&store);
        store.set_hovered(true);
        let id = add_test_notification(&store);
        {
            let mut latest = store.latest_notification.lock().unwrap();
            assert!(latest.hovered_at.is_none());
            assert_eq!(latest.item.as_ref().unwrap().id, id);
            latest.item.as_mut().unwrap().received_at = Instant::now() - Duration::from_secs(10);
        }
        store.set_hovered(true);
        assert!(store.get_latest_active_notification().is_none());
        assert!(
            store
                .latest_notification
                .lock()
                .unwrap()
                .hovered_at
                .is_none()
        );
    }

    #[tokio::test]
    async fn close_and_clear_reset_hover() {
        let store = NotificationStore::new();
        let first = add_test_notification(&store);
        let second = add_test_notification(&store);
        store.set_hovered(true);
        let server = NotificationServer {
            store: store.clone(),
        };
        server.close_notification(first).await;
        assert_eq!(store.get_latest_active_notification().unwrap().id, second);
        assert!(
            store
                .latest_notification
                .lock()
                .unwrap()
                .hovered_at
                .is_some()
        );
        server.close_notification(second).await;
        assert!(store.get_all_notifications().is_empty());
        assert!(store.get_latest_active_notification().is_none());
        assert!(
            store
                .latest_notification
                .lock()
                .unwrap()
                .hovered_at
                .is_none()
        );
        add_test_notification(&store);
        store.set_hovered(true);
        store.clear_all_notifications();
        assert!(store.get_all_notifications().is_empty());
        assert!(store.get_latest_active_notification().is_none());
        assert!(
            store
                .latest_notification
                .lock()
                .unwrap()
                .hovered_at
                .is_none()
        );
    }

    #[test]
    fn test_dnd_toggle_and_suppression() {
        let store = NotificationStore::new();
        assert!(!store.is_dnd_enabled());

        store.add_notification(
            "TestApp".to_string(),
            "".to_string(),
            "Title".to_string(),
            "Body".to_string(),
            5000,
        );
        assert!(store.get_latest_active_notification().is_some());
        assert_eq!(store.get_all_notifications().len(), 1);

        store.toggle_dnd();
        assert!(store.is_dnd_enabled());
        assert!(store.get_latest_active_notification().is_none());

        store.add_notification(
            "TestApp2".to_string(),
            "".to_string(),
            "Title2".to_string(),
            "Body2".to_string(),
            5000,
        );
        assert!(store.get_latest_active_notification().is_none());
        assert_eq!(store.get_all_notifications().len(), 2);

        store.toggle_dnd();
        assert!(!store.is_dnd_enabled());
    }
}
