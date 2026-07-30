pub mod dbus;

use arc_swap::ArcSwap;
use dbus::MediaPlayer2PlayerProxy;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use zbus::Connection;
use zbus::zvariant::Value;

async fn get_session_conn() -> Option<Connection> {
    crate::dbus_util::get_shared_session_conn().await
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
        call_mpris_method(bus_name, "PlayPause").await
    }

    pub async fn next_bus(bus_name: &str) -> bool {
        call_mpris_method(bus_name, "Next").await
    }

    pub async fn previous_bus(bus_name: &str) -> bool {
        call_mpris_method(bus_name, "Previous").await
    }
}

async fn run_playerctl_fallback(requested_bus: &str, method: &str) -> bool {
    let clean_player = if requested_bus.to_lowercase().contains("spotify") {
        "spotify"
    } else {
        "player"
    };
    let action = match method {
        "PlayPause" => "play-pause",
        "Next" => "next",
        "Previous" => "previous",
        _ => "play-pause",
    };

    crate::log_info!(
        "MPRIS",
        "Executing playerctl fallback: playerctl -p {} {}",
        clean_player,
        action
    );

    let cmd_fut = tokio::process::Command::new("playerctl")
        .args(["-p", clean_player, action])
        .status();

    match tokio::time::timeout(Duration::from_millis(300), cmd_fut).await {
        Ok(Ok(st)) => {
            crate::log_info!("MPRIS", "playerctl completed with status: {:?}", st);
            st.success()
        }
        Ok(Err(e)) => {
            crate::log_warn!("MPRIS", "playerctl execution error: {e}");
            false
        }
        Err(_) => {
            crate::log_warn!("MPRIS", "playerctl execution timed out after 300ms!");
            false
        }
    }
}

async fn call_mpris_method(requested_bus: &str, method: &str) -> bool {
    crate::log_info!(
        "MPRIS",
        "call_mpris_method requested: bus='{}', method='{}'",
        requested_bus,
        method
    );

    // 1. Try D-Bus first (returns immediately on success)
    if let Some(conn) = get_session_conn().await {
        let target = if requested_bus.trim().is_empty() {
            "org.mpris.MediaPlayer2.spotify"
        } else {
            requested_bus
        };

        let direct_fut = conn.call_method(
            Some(target),
            "/org/mpris/MediaPlayer2",
            Some("org.mpris.MediaPlayer2.Player"),
            method,
            &(),
        );

        if let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(150), direct_fut).await {
            crate::log_info!(
                "MPRIS",
                "Direct D-Bus call '{}' on '{}' succeeded in <150ms",
                method,
                target
            );
            return true;
        }

        if target != "org.mpris.MediaPlayer2.spotify" {
            let spot_fut = conn.call_method(
                Some("org.mpris.MediaPlayer2.spotify"),
                "/org/mpris/MediaPlayer2",
                Some("org.mpris.MediaPlayer2.Player"),
                method,
                &(),
            );
            if let Ok(Ok(_)) = tokio::time::timeout(Duration::from_millis(150), spot_fut).await {
                crate::log_info!(
                    "MPRIS",
                    "Fallback D-Bus call on 'org.mpris.MediaPlayer2.spotify' succeeded"
                );
                return true;
            }
        }
    }

    // 2. Only if D-Bus failed/timed out, execute playerctl fallback once
    run_playerctl_fallback(requested_bus, method).await
}

impl Default for MprisService {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_mpris_listener(players: Arc<ArcSwap<Vec<MediaTrack>>>) {
    loop {
        let current_players = poll_all_players_dbus().await;
        if !current_players.is_empty() || players.load().is_empty() {
            players.store(Arc::new(current_players));
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

async fn poll_single_player(connection: &Connection, mpris_name: String) -> Option<MediaTrack> {
    let builder = MediaPlayer2PlayerProxy::builder(connection)
        .destination(mpris_name.as_str())
        .ok()?;
    let proxy = tokio::time::timeout(Duration::from_millis(800), builder.build())
        .await
        .ok()?
        .ok()?;

    let status = tokio::time::timeout(Duration::from_millis(500), proxy.playback_status())
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();

    let is_playing = status == "Playing";

    let metadata = tokio::time::timeout(Duration::from_millis(500), proxy.metadata())
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or_default();

    let (title, artist, album, art_url, length_micros) = parse_metadata(&metadata);

    let position_micros = tokio::time::timeout(Duration::from_millis(300), proxy.position())
        .await
        .ok()
        .and_then(|r| r.ok());

    let can_go_next = tokio::time::timeout(Duration::from_millis(300), proxy.can_go_next())
        .await
        .ok()
        .and_then(|r| r.ok())
        .unwrap_or(true);

    let can_go_previous = tokio::time::timeout(Duration::from_millis(300), proxy.can_go_previous())
        .await
        .ok()
        .and_then(|r| r.ok())
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

    let final_bus_name = if mpris_name.to_lowercase().contains("spotify") {
        "org.mpris.MediaPlayer2.spotify".to_string()
    } else {
        mpris_name
    };

    Some(MediaTrack {
        bus_name: final_bus_name,
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

    let reply = match tokio::time::timeout(Duration::from_millis(2000), list_call)
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
            let n = name.to_lowercase();
            n.starts_with("org.mpris.mediaplayer2.")
                && !n.ends_with(".playerctld")
                && n.contains("spotify")
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
