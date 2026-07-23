pub mod dbus;

use arc_swap::ArcSwap;
use dbus::MediaPlayer2PlayerProxy;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::zvariant::Value;

async fn get_session_conn() -> Option<Connection> {
    Connection::session().await.ok()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaTrack {
    pub bus_name: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub art_url: Option<String>,
    pub local_art_path: Option<String>,
    pub is_playing: bool,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub player_name: String,
    pub has_media: bool,
    pub position_micros: Option<i64>,
    pub length_micros: Option<i64>,
}

#[derive(Clone)]
pub struct MprisService {
    players: Arc<ArcSwap<Vec<MediaTrack>>>,
}

impl MprisService {
    pub fn new() -> Self {
        let players = Arc::new(ArcSwap::from_pointee(Vec::new()));
        let players_clone = players.clone();

        tokio::spawn(async move {
            run_mpris_listener(players_clone).await;
        });

        Self { players }
    }

    pub fn get_current_track(&self) -> MediaTrack {
        let apps = self.players.load();
        apps.first().cloned().unwrap_or_default()
    }

    pub fn get_all_players(&self) -> Arc<Vec<MediaTrack>> {
        self.players.load_full()
    }

    pub async fn fetch_all_players() -> Vec<MediaTrack> {
        poll_all_players_dbus().await
    }

    pub async fn play_pause_bus(bus_name: &str) -> bool {
        if let Some(conn) = get_session_conn().await {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return tokio::time::timeout(Duration::from_millis(300), proxy.play_pause())
                        .await
                        .ok()
                        .and_then(|r| r.ok())
                        .is_some();
                }
            }
        }
        false
    }

    pub async fn next_bus(bus_name: &str) -> bool {
        if let Some(conn) = get_session_conn().await {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return tokio::time::timeout(Duration::from_millis(300), proxy.next())
                        .await
                        .ok()
                        .and_then(|r| r.ok())
                        .is_some();
                }
            }
        }
        false
    }

    pub async fn previous_bus(bus_name: &str) -> bool {
        if let Some(conn) = get_session_conn().await {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return tokio::time::timeout(Duration::from_millis(300), proxy.previous())
                        .await
                        .ok()
                        .and_then(|r| r.ok())
                        .is_some();
                }
            }
        }
        false
    }
}

impl Default for MprisService {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_mpris_listener(players: Arc<ArcSwap<Vec<MediaTrack>>>) {
    loop {
        let initial_players = poll_all_players_dbus().await;
        players.store(Arc::new(initial_players));

        let conn = match get_session_conn().await {
            Some(c) => c,
            None => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        if let Ok(dbus) = DBusProxy::new(&conn).await {
            if let Ok(mut name_changed) = dbus.receive_name_owner_changed().await {
                let _ = tokio::time::timeout(Duration::from_secs(1), async {
                    use zbus::export::ordered_stream::OrderedStreamExt;
                    while let Some(signal) = name_changed.next().await {
                        if let Ok(args) = signal.args() {
                            if args.name.as_str().starts_with("org.mpris.MediaPlayer2.") {
                                break;
                            }
                        }
                    }
                })
                .await;
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn poll_single_player(connection: &Connection, mpris_name: String) -> Option<MediaTrack> {
    let builder = MediaPlayer2PlayerProxy::builder(connection)
        .destination(mpris_name.as_str())
        .ok()?;
    let proxy = tokio::time::timeout(Duration::from_millis(150), builder.build())
        .await
        .ok()?
        .ok()?;

    let status = tokio::time::timeout(Duration::from_millis(100), proxy.playback_status())
        .await
        .ok()?
        .ok()
        .unwrap_or_default();

    let is_playing = status == "Playing";

    let metadata = tokio::time::timeout(Duration::from_millis(100), proxy.metadata())
        .await
        .ok()?
        .ok()
        .unwrap_or_default();

    let (title, artist, album, art_url, length_micros) = parse_metadata(&metadata);

    let position_micros = tokio::time::timeout(Duration::from_millis(50), proxy.position())
        .await
        .ok()?
        .ok();

    let can_go_next = tokio::time::timeout(Duration::from_millis(50), proxy.can_go_next())
        .await
        .ok()?
        .ok()
        .unwrap_or(true);

    let can_go_previous = tokio::time::timeout(Duration::from_millis(50), proxy.can_go_previous())
        .await
        .ok()?
        .ok()
        .unwrap_or(true);

    let has_media = !title.is_empty() && title != "Silence";

    let clean_name = {
        let raw_name = mpris_name
            .trim_start_matches("org.mpris.MediaPlayer2.")
            .split('.')
            .next()
            .unwrap_or("Player");

        match raw_name.to_lowercase().as_str() {
            "spotify" => "Spotify".to_string(),
            "firefox" => "Firefox".to_string(),
            "chromium" => "Chromium".to_string(),
            "chrome" => "Chrome".to_string(),
            "vlc" => "VLC".to_string(),
            "mpv" => "mpv".to_string(),
            "rhythmbox" => "Rhythmbox".to_string(),
            "cider" => "Cider".to_string(),
            "amberol" => "Amberol".to_string(),
            _ => raw_name.to_string(),
        }
    };

    let local_art_path = if let Some(url) = &art_url {
        resolve_art_url(url).await
    } else {
        None
    };

    Some(MediaTrack {
        bus_name: mpris_name,
        title,
        artist,
        album,
        art_url,
        local_art_path,
        is_playing,
        can_go_next,
        can_go_previous,
        player_name: clean_name,
        has_media,
        position_micros,
        length_micros,
    })
}

async fn poll_all_players_dbus() -> Vec<MediaTrack> {
    let connection = match get_session_conn().await {
        Some(conn) => conn,
        None => return Vec::new(),
    };

    let list_call = connection.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "ListNames",
        &(),
    );

    let reply = match tokio::time::timeout(Duration::from_millis(200), list_call)
        .await
        .ok()
        .and_then(|r| r.ok())
    {
        Some(rep) => rep,
        None => return Vec::new(),
    };

    let names: Vec<String> = match reply.body().deserialize().ok() {
        Some(n) => n,
        None => return Vec::new(),
    };

    let mpris_names: Vec<String> = names
        .into_iter()
        .filter(|name| {
            name.starts_with("org.mpris.MediaPlayer2.") && !name.ends_with(".playerctld")
        })
        .collect();

    let futures = mpris_names
        .into_iter()
        .map(|name| poll_single_player(&connection, name));

    let results = futures::future::join_all(futures).await;
    let mut players: Vec<MediaTrack> = results.into_iter().flatten().collect();

    players.sort_by_key(|p| !p.is_playing);
    players
}

async fn resolve_art_url(url: &str) -> Option<String> {
    if url.starts_with("file://") {
        let path = url.trim_start_matches("file://").to_string();
        if Path::new(&path).exists() {
            return Some(path);
        }
    } else if url.starts_with("http://") || url.starts_with("https://") {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let cache_path = format!("/tmp/capsule_art_{}.jpg", &hash[..16]);

        if Path::new(&cache_path).exists() {
            return Some(cache_path);
        }

        let url_owned = url.to_string();
        tokio::spawn(async move {
            if let Ok(client) = reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
            {
                if let Ok(resp) = client.get(&url_owned).send().await {
                    if let Ok(bytes) = resp.bytes().await {
                        let _ = tokio::fs::write(&cache_path, &bytes).await;
                    }
                }
            }
        });
    }
    None
}

fn extract_string(v: &Value<'static>) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.as_str().to_string()),
        _ => v
            .downcast_ref::<String>()
            .ok()
            .or_else(|| v.downcast_ref::<&str>().ok().map(|s| s.to_string())),
    }
}

fn parse_metadata(
    meta: &HashMap<String, Value<'static>>,
) -> (String, String, String, Option<String>, Option<i64>) {
    let title = meta
        .get("xesam:title")
        .and_then(extract_string)
        .unwrap_or_else(|| "Silence".to_string());

    let artist = meta
        .get("xesam:artist")
        .and_then(|v| match v {
            Value::Array(arr) => {
                let artists: Vec<String> = arr.iter().filter_map(extract_string).collect();
                if artists.is_empty() {
                    None
                } else {
                    Some(artists.join(", "))
                }
            }
            _ => extract_string(v),
        })
        .unwrap_or_else(|| "No media playing".to_string());

    let album = meta
        .get("xesam:album")
        .and_then(extract_string)
        .unwrap_or_default();

    let art_url = meta.get("mpris:artUrl").and_then(extract_string);

    let length_micros = meta.get("mpris:length").and_then(extract_i64);

    (title, artist, album, art_url, length_micros)
}

fn extract_i64(v: &Value<'static>) -> Option<i64> {
    match v {
        Value::I64(i) => Some(*i),
        Value::U64(u) => Some(*u as i64),
        Value::I32(i) => Some(*i as i64),
        Value::U32(u) => Some(*u as i64),
        _ => v
            .downcast_ref::<i64>()
            .ok()
            .or_else(|| v.downcast_ref::<u64>().ok().map(|u| u as i64)),
    }
}
