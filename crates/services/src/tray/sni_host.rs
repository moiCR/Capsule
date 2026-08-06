use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use zbus::interface;

use super::dbus_menu::{DBusMenuItem, fetch_dbus_menu, trigger_dbus_menu_item};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniItem {
    pub service: String,
    pub bus_name: String,
    pub object_path: String,
    pub id: String,
    pub title: String,
    pub icon_name: String,
    pub icon_file_path: Option<String>,
    pub menu_path: Option<String>,
    pub tooltip: String,
    pub menu_items: Vec<DBusMenuItem>,
}

#[derive(Clone)]
pub struct SniHostService {
    items: Arc<Mutex<Vec<SniItem>>>,
    raw_services: Arc<Mutex<Vec<String>>>,
    selected_tray_idx: Arc<Mutex<Option<usize>>>,
    notify: Arc<tokio::sync::Notify>,
}

impl SniHostService {
    pub fn new() -> Self {
        let _ = std::fs::create_dir_all("/tmp/capsule_tray_icons");
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            raw_services: Arc::new(Mutex::new(Vec::new())),
            selected_tray_idx: Arc::new(Mutex::new(None)),
            notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub fn get_items(&self) -> Vec<SniItem> {
        self.items.lock().map(|g| g.clone()).unwrap_or_default()
    }

    fn set_items(&self, new: Vec<SniItem>) {
        if let Ok(mut g) = self.items.lock() {
            *g = new;
        }
    }

    pub fn get_selected_idx(&self) -> Option<usize> {
        self.selected_tray_idx.lock().ok().and_then(|g| *g)
    }

    pub fn set_selected_idx(&self, idx: Option<usize>) {
        if let Ok(mut g) = self.selected_tray_idx.lock() {
            *g = idx;
        }
    }

    pub fn activate_item(&self, idx: usize) {
        let items = self.get_items();
        if let Some(item) = items.get(idx) {
            let bus = item.bus_name.clone();
            let path = item.object_path.clone();
            tokio::spawn(async move {
                let Some(conn) = crate::dbus_util::get_shared_session_conn().await else {
                    return;
                };
                let res1 = conn
                    .call_method(
                        Some(bus.as_str()),
                        path.as_str(),
                        Some("org.kde.StatusNotifierItem"),
                        "Activate",
                        &(0i32, 0i32),
                    )
                    .await;
                if res1.is_err() {
                    let _ = conn
                        .call_method(
                            Some(bus.as_str()),
                            path.as_str(),
                            Some("org.kde.StatusNotifierItem"),
                            "SecondaryActivate",
                            &(0i32, 0i32),
                        )
                        .await;
                }
            });
        }
    }

    pub fn trigger_menu(&self, bus_name: String, menu_path: String, item_id: i32) {
        tokio::spawn(async move {
            let Some(conn) = crate::dbus_util::get_shared_session_conn().await else {
                return;
            };
            let _ = trigger_dbus_menu_item(&conn, &bus_name, &menu_path, item_id).await;
        });
    }

    pub fn start(&self) {
        let svc = self.clone();
        tokio::spawn(async move {
            if let Err(e) = run_sni_service(svc).await {
                eprintln!("[SNI HOST] Error in SNI service: {e}");
            }
        });
    }
}

// ── D-Bus Watcher Server ─────────────────────────────────────────────────────

struct StatusNotifierWatcherServer {
    raw_services: Arc<Mutex<Vec<String>>>,
    notify: Arc<tokio::sync::Notify>,
}

#[interface(name = "org.kde.StatusNotifierWatcher")]
impl StatusNotifierWatcherServer {
    async fn register_status_notifier_item(
        &self,
        service: String,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) {
        let sender = header
            .sender()
            .map(|s| s.as_str().to_string())
            .unwrap_or_default();

        let item_path = if service.starts_with('/') {
            format!("{}{}", sender, service)
        } else if service.contains('/') {
            service
        } else if service.starts_with(':')
            || service.contains("StatusNotifierItem")
            || service.is_empty()
        {
            format!("{}/StatusNotifierItem", sender)
        } else {
            format!("{}/StatusNotifierItem", service)
        };

        if let Ok(mut items) = self.raw_services.lock() {
            if !items.contains(&item_path) {
                items.push(item_path);
                self.notify.notify_one();
            }
        }
    }

    async fn register_status_notifier_host(&self, _service: String) {}

    #[zbus(property)]
    async fn registered_status_notifier_items(&self) -> Vec<String> {
        self.raw_services
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    #[zbus(property)]
    async fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    async fn protocol_version(&self) -> i32 {
        0
    }
}

async fn run_sni_service(host: SniHostService) -> anyhow::Result<()> {
    let server = StatusNotifierWatcherServer {
        raw_services: host.raw_services.clone(),
        notify: host.notify.clone(),
    };

    let connection_res = zbus::connection::Builder::session()?
        .name("org.kde.StatusNotifierWatcher")?
        .serve_at("/StatusNotifierWatcher", server)?
        .build()
        .await;

    let conn = match connection_res {
        Ok(c) => c,
        Err(_) => zbus::Connection::session().await?,
    };

    let mut item_cache: std::collections::HashMap<String, (std::time::Instant, SniItem)> =
        std::collections::HashMap::new();

    loop {
        let services = fetch_registered_services(&conn, &host).await;
        let mut detailed_items = Vec::new();
        let now = std::time::Instant::now();

        item_cache.retain(|svc, _| services.contains(svc));

        for svc in &services {
            let (bus_name, obj_path) = if let Some(slash_pos) = svc.find('/') {
                (svc[..slash_pos].to_string(), svc[slash_pos..].to_string())
            } else {
                (svc.clone(), "/StatusNotifierItem".to_string())
            };

            let is_alive = tokio::time::timeout(
                std::time::Duration::from_millis(300),
                conn.call_method(
                    Some("org.freedesktop.DBus"),
                    "/org/freedesktop/DBus",
                    Some("org.freedesktop.DBus"),
                    "NameHasOwner",
                    &(bus_name.as_str(),),
                ),
            )
            .await
            .ok()
            .and_then(|r| r.ok())
            .and_then(|m| m.body().deserialize::<bool>().ok())
            .unwrap_or(false);

            if !is_alive {
                if let Ok(mut raw) = host.raw_services.lock() {
                    raw.retain(|s| s != svc);
                }
                item_cache.remove(svc);
                continue;
            }

            if let Some((last_fetch, cached_item)) = item_cache.get(svc) {
                if now.duration_since(*last_fetch) < std::time::Duration::from_secs(30) {
                    detailed_items.push(cached_item.clone());
                    continue;
                }
            }

            let id = get_string_prop(&conn, &bus_name, &obj_path, "Id")
                .await
                .unwrap_or_default();
            let title = get_string_prop(&conn, &bus_name, &obj_path, "Title")
                .await
                .unwrap_or_else(|| id.clone());
            let icon_name = get_string_prop(&conn, &bus_name, &obj_path, "IconName")
                .await
                .unwrap_or_default();
            let icon_theme_path =
                get_string_prop(&conn, &bus_name, &obj_path, "IconThemePath").await;
            let menu_path = get_object_path_prop(&conn, &bus_name, &obj_path, "Menu").await;
            let tooltip = get_tooltip_title(&conn, &bus_name, &obj_path)
                .await
                .unwrap_or_default();

            let mut icon_file_path =
                resolve_icon_file(&icon_name, icon_theme_path.as_deref(), &id, &title);
            if icon_file_path.is_none() {
                if let Some(saved) = save_pixmap_to_file(&conn, &bus_name, &obj_path, &id).await {
                    icon_file_path = Some(saved);
                }
            }

            let mut menu_items = Vec::new();
            if let Some(ref mp) = menu_path {
                if let Ok(fetched) = tokio::time::timeout(
                    std::time::Duration::from_millis(1000),
                    fetch_dbus_menu(&conn, &bus_name, mp),
                )
                .await
                {
                    menu_items = fetched;
                }
            }

            let display = if !title.is_empty() {
                title.clone()
            } else if !id.is_empty() {
                id.clone()
            } else {
                bus_name.clone()
            };

            let item = SniItem {
                service: svc.clone(),
                bus_name,
                object_path: obj_path,
                id,
                title: display,
                icon_name,
                icon_file_path,
                menu_path,
                tooltip,
                menu_items,
            };

            item_cache.insert(svc.clone(), (now, item.clone()));
            detailed_items.push(item);
        }

        if host.get_items() != detailed_items {
            host.set_items(detailed_items);
        }

        let _ =
            tokio::time::timeout(std::time::Duration::from_secs(15), host.notify.notified()).await;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}

fn resolve_icon_file(
    icon_name: &str,
    theme_path: Option<&str>,
    item_id: &str,
    title: &str,
) -> Option<String> {
    let name_candidates = [icon_name, item_id, title];

    for name in &name_candidates {
        if name.is_empty() {
            continue;
        }

        // 1. Absolute path check
        if Path::new(name).is_absolute() && Path::new(name).exists() {
            return Some(name.to_string());
        }

        // 2. IconThemePath check
        if let Some(tp) = theme_path {
            for ext in &["png", "svg", "xpm"] {
                let p = PathBuf::from(tp).join(format!("{name}.{ext}"));
                if p.exists() {
                    return Some(p.to_string_lossy().to_string());
                }
            }
        }

        // 3. XDG Icon Directory Search
        let home = std::env::var("HOME").unwrap_or_default();
        let search_dirs = [
            format!("{home}/.local/share/icons/hicolor/128x128/apps"),
            format!("{home}/.local/share/icons/hicolor/64x64/apps"),
            format!("{home}/.local/share/icons/hicolor/48x48/apps"),
            format!("{home}/.local/share/icons/hicolor/scalable/apps"),
            "/usr/share/icons/hicolor/128x128/apps".to_string(),
            "/usr/share/icons/hicolor/64x64/apps".to_string(),
            "/usr/share/icons/hicolor/48x48/apps".to_string(),
            "/usr/share/icons/hicolor/32x32/apps".to_string(),
            "/usr/share/icons/hicolor/scalable/apps".to_string(),
            "/usr/share/icons/Adwaita/scalable/apps".to_string(),
            "/usr/share/pixmaps".to_string(),
        ];

        let clean_name = name.to_lowercase().replace(' ', "-");
        for dir in &search_dirs {
            for ext in &["png", "svg", "xpm"] {
                let p1 = PathBuf::from(dir).join(format!("{name}.{ext}"));
                if p1.exists() {
                    return Some(p1.to_string_lossy().to_string());
                }
                let p2 = PathBuf::from(dir).join(format!("{clean_name}.{ext}"));
                if p2.exists() {
                    return Some(p2.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

async fn save_pixmap_to_file(
    conn: &zbus::Connection,
    bus_name: &str,
    obj_path: &str,
    item_id: &str,
) -> Option<String> {
    let msg = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        conn.call_method(
            Some(bus_name),
            obj_path,
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.kde.StatusNotifierItem", "IconPixmap"),
        ),
    )
    .await
    .ok()?
    .ok()?;

    let body = msg.body();
    let val: zbus::zvariant::Value = body.deserialize().ok()?;
    let arr = match val {
        zbus::zvariant::Value::Array(a) => a,
        _ => return None,
    };

    let first_struct = match arr.iter().next() {
        Some(zbus::zvariant::Value::Structure(s)) => s,
        _ => return None,
    };

    let fields = first_struct.fields();
    if fields.len() < 3 {
        return None;
    }

    let width = match fields[0] {
        zbus::zvariant::Value::I32(w) => w as u32,
        _ => return None,
    };
    let height = match fields[1] {
        zbus::zvariant::Value::I32(h) => h as u32,
        _ => return None,
    };
    let bytes = match &fields[2] {
        zbus::zvariant::Value::Array(b) => {
            let mut buf = Vec::new();
            for u in b.iter() {
                if let zbus::zvariant::Value::U8(byte) = u {
                    buf.push(*byte);
                }
            }
            buf
        }
        _ => return None,
    };

    if width == 0 || height == 0 || bytes.len() < (width * height * 4) as usize {
        return None;
    }

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for chunk in bytes.chunks_exact(4) {
        let a = chunk[0];
        let r = chunk[1];
        let g = chunk[2];
        let b = chunk[3];
        rgba.push(r);
        rgba.push(g);
        rgba.push(b);
        rgba.push(a);
    }

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(&rgba, &mut hasher);
    let hash = std::hash::Hasher::finish(&hasher);

    let file_path = format!("/tmp/capsule_tray_icons/{item_id}_{hash}.png");
    if !std::path::Path::new(&file_path).exists() {
        let img_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            image::ImageBuffer::from_raw(width, height, rgba)?;
        img_buf.save(&file_path).ok()?;
    }

    Some(file_path)
}

async fn fetch_registered_services(_conn: &zbus::Connection, host: &SniHostService) -> Vec<String> {
    host.raw_services
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

fn unwrap_val<'a, 'b>(val: &'a zbus::zvariant::Value<'b>) -> &'a zbus::zvariant::Value<'b> {
    let mut curr = val;
    while let zbus::zvariant::Value::Value(inner) = curr {
        curr = inner;
    }
    curr
}

async fn get_string_prop(
    conn: &zbus::Connection,
    bus_name: &str,
    obj_path: &str,
    prop: &str,
) -> Option<String> {
    let interfaces = [
        "org.kde.StatusNotifierItem",
        "org.freedesktop.StatusNotifierItem",
    ];
    for iface in interfaces {
        if let Ok(Ok(msg)) = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            conn.call_method(
                Some(bus_name),
                obj_path,
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &(iface, prop),
            ),
        )
        .await
        {
            if let Ok(val) = msg.body().deserialize::<zbus::zvariant::Value>() {
                match unwrap_val(&val) {
                    zbus::zvariant::Value::Str(s) => return Some(s.to_string()),
                    _ => {}
                }
            }
        }
    }
    None
}

async fn get_object_path_prop(
    conn: &zbus::Connection,
    bus_name: &str,
    obj_path: &str,
    prop: &str,
) -> Option<String> {
    let interfaces = [
        "org.kde.StatusNotifierItem",
        "org.freedesktop.StatusNotifierItem",
    ];
    for iface in interfaces {
        if let Ok(Ok(msg)) = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            conn.call_method(
                Some(bus_name),
                obj_path,
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &(iface, prop),
            ),
        )
        .await
        {
            if let Ok(val) = msg.body().deserialize::<zbus::zvariant::Value>() {
                match unwrap_val(&val) {
                    zbus::zvariant::Value::ObjectPath(p) => return Some(p.to_string()),
                    zbus::zvariant::Value::Str(s) => return Some(s.to_string()),
                    _ => {}
                }
            }
        }
    }
    None
}

async fn get_tooltip_title(
    conn: &zbus::Connection,
    bus_name: &str,
    obj_path: &str,
) -> Option<String> {
    let interfaces = [
        "org.kde.StatusNotifierItem",
        "org.freedesktop.StatusNotifierItem",
    ];
    for iface in interfaces {
        if let Ok(Ok(msg)) = tokio::time::timeout(
            std::time::Duration::from_millis(300),
            conn.call_method(
                Some(bus_name),
                obj_path,
                Some("org.freedesktop.DBus.Properties"),
                "Get",
                &(iface, "ToolTip"),
            ),
        )
        .await
        {
            if let Ok(val) = msg.body().deserialize::<zbus::zvariant::Value>() {
                if let zbus::zvariant::Value::Structure(s) = unwrap_val(&val) {
                    let fields = s.fields();
                    if fields.len() >= 3 {
                        if let zbus::zvariant::Value::Str(title) = unwrap_val(&fields[2]) {
                            return Some(title.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
