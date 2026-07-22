use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use zbus::proxy;
use zbus::zvariant::Value;
use zbus::Connection;

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

#[proxy(
    interface = "org.mpris.MediaPlayer2.Player",
    default_path = "/org/mpris/MediaPlayer2"
)]
pub trait MediaPlayer2Player {
    fn next(&self) -> zbus::Result<()>;
    fn previous(&self) -> zbus::Result<()>;
    fn pause(&self) -> zbus::Result<()>;
    fn play(&self) -> zbus::Result<()>;
    fn play_pause(&self) -> zbus::Result<()>;
    fn stop(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn playback_status(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn metadata(&self) -> zbus::Result<HashMap<String, Value<'static>>>;

    #[zbus(property)]
    fn position(&self) -> zbus::Result<i64>;

    #[zbus(property)]
    fn can_go_next(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn can_go_previous(&self) -> zbus::Result<bool>;
}

pub struct MprisService {
    current_track: Arc<Mutex<MediaTrack>>,
}

impl MprisService {
    pub fn new() -> Self {
        Self {
            current_track: Arc::new(Mutex::new(MediaTrack::default())),
        }
    }

    pub fn get_current_track(&self) -> MediaTrack {
        self.current_track
            .lock()
            .map(|t| t.clone())
            .unwrap_or_default()
    }

    pub async fn fetch_all_players() -> Vec<MediaTrack> {
        let connection = match Connection::session().await.ok() {
            Some(conn) => conn,
            None => return Vec::new(),
        };

        let reply = match connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .await
            .ok()
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

        let mut players = Vec::new();

        for mpris_name in mpris_names {
            if let Ok(builder) =
                MediaPlayer2PlayerProxy::builder(&connection).destination(mpris_name.as_str())
            {
                if let Ok(proxy) = builder.build().await {
                    let status = proxy.playback_status().await.unwrap_or_default();
                    let is_playing = status == "Playing";
                    let metadata = proxy.metadata().await.unwrap_or_default();
                    let (title, artist, album, art_url, length_micros) = parse_metadata(&metadata);

                    let position_micros = proxy.position().await.ok();

                    let has_media = !title.is_empty() && title != "Silence";

                    let raw_name = mpris_name
                        .trim_start_matches("org.mpris.MediaPlayer2.")
                        .split('.')
                        .next()
                        .unwrap_or("Player");

                    let clean_name = match raw_name.to_lowercase().as_str() {
                        "spotify" => "Spotify",
                        "firefox" => "Firefox",
                        "chromium" | "chrome" | "brave" => "Browser",
                        "mpv" => "mpv",
                        "vlc" => "VLC",
                        _ => raw_name,
                    }
                    .to_string();

                    let can_next = proxy.can_go_next().await.unwrap_or(true);
                    let can_prev = proxy.can_go_previous().await.unwrap_or(true);

                    let local_art_path = if let Some(url) = &art_url {
                        resolve_art_url(url).await
                    } else {
                        None
                    };

                    players.push(MediaTrack {
                        bus_name: mpris_name,
                        title,
                        artist,
                        album,
                        art_url,
                        local_art_path,
                        is_playing,
                        can_go_next: can_next,
                        can_go_previous: can_prev,
                        player_name: clean_name,
                        has_media,
                        position_micros,
                        length_micros,
                    });
                }
            }
        }

        // Sort playing players first
        players.sort_by_key(|p| !p.is_playing);

        players
    }

    pub async fn play_pause_bus(bus_name: &str) -> bool {
        if let Some(conn) = Connection::session().await.ok() {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return proxy.play_pause().await.is_ok();
                }
            }
        }
        false
    }

    pub async fn next_bus(bus_name: &str) -> bool {
        if let Some(conn) = Connection::session().await.ok() {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return proxy.next().await.is_ok();
                }
            }
        }
        false
    }

    pub async fn previous_bus(bus_name: &str) -> bool {
        if let Some(conn) = Connection::session().await.ok() {
            if let Ok(builder) = MediaPlayer2PlayerProxy::builder(&conn).destination(bus_name) {
                if let Ok(proxy) = builder.build().await {
                    return proxy.previous().await.is_ok();
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

        if let Ok(resp) = reqwest::get(url).await {
            if let Ok(bytes) = resp.bytes().await {
                if tokio::fs::write(&cache_path, &bytes).await.is_ok() {
                    return Some(cache_path);
                }
            }
        }
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
