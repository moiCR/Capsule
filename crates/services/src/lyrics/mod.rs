use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub timestamp: Duration,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackLyrics {
    pub track_title: String,
    pub artist_name: String,
    pub synced_lines: Vec<LyricLine>,
    pub plain_lyrics: Option<String>,
}

impl TrackLyrics {
    pub fn get_current_line(&self, position: Duration) -> Option<&str> {
        if self.synced_lines.is_empty() {
            return None;
        }

        let idx = self
            .synced_lines
            .partition_point(|l| l.timestamp <= position);
        if idx > 0 {
            let line = &self.synced_lines[idx - 1];
            if line.text.is_empty() {
                None
            } else {
                Some(&line.text)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct LrclibResponse {
    pub id: Option<u64>,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    pub instrumental: Option<bool>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

#[derive(Clone)]
pub struct LyricsService {
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, Option<TrackLyrics>>>>,
    fetching: Arc<Mutex<HashSet<String>>>,
}

impl LyricsService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap_or_default(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            fetching: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn get_cached_lyrics(&self, title: &str, artist: &str) -> Option<Option<TrackLyrics>> {
        let clean_title = clean_track_title(title);
        let key = format!("{} - {}", clean_title.to_lowercase(), artist.to_lowercase());
        let raw_key = format!("{} - {}", title.to_lowercase(), artist.to_lowercase());

        if let Ok(guard) = self.cache.lock() {
            if let Some(res) = guard.get(&key) {
                return Some(res.clone());
            }
            if let Some(res) = guard.get(&raw_key) {
                return Some(res.clone());
            }
        }
        None
    }

    pub fn fetch_lyrics_in_background(
        &self,
        title: String,
        artist: String,
        album: Option<String>,
        duration_secs: Option<u64>,
    ) {
        if title.is_empty() || title == "Silence" {
            return;
        }

        let clean_title = clean_track_title(&title);
        let key = format!("{} - {}", clean_title.to_lowercase(), artist.to_lowercase());
        let raw_key = format!("{} - {}", title.to_lowercase(), artist.to_lowercase());

        if let Ok(guard) = self.cache.lock() {
            if guard.contains_key(&key) || guard.contains_key(&raw_key) {
                return;
            }
        }

        if let Ok(mut guard) = self.fetching.lock() {
            if guard.contains(&key) {
                return;
            }
            guard.insert(key.clone());
        }

        let client = self.client.clone();
        let cache = self.cache.clone();
        let fetching = self.fetching.clone();
        let _raw_title = title.clone();
        let raw_artist = artist.clone();

        tokio::spawn(async move {
            let result = fetch_lrclib(
                &client,
                &clean_title,
                &raw_artist,
                album.as_deref(),
                duration_secs,
            )
            .await;

            if let Ok(mut guard) = cache.lock() {
                guard.insert(key.clone(), result.clone());
                guard.insert(raw_key, result);
            }

            if let Ok(mut guard) = fetching.lock() {
                guard.remove(&key);
            }
        });
    }

    pub async fn get_lyrics_for_track(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: Option<u64>,
    ) -> Option<TrackLyrics> {
        if let Some(cached) = self.get_cached_lyrics(title, artist) {
            return cached;
        }

        self.fetch_lyrics_in_background(
            title.to_string(),
            artist.to_string(),
            album.map(|s| s.to_string()),
            duration_secs,
        );

        None
    }
}

impl Default for LyricsService {
    fn default() -> Self {
        Self::new()
    }
}

pub fn clean_track_title(title: &str) -> String {
    title.trim().to_string()
}

async fn fetch_lrclib(
    client: &reqwest::Client,
    title: &str,
    artist: &str,
    album: Option<&str>,
    duration_secs: Option<u64>,
) -> Option<TrackLyrics> {
    // Attempt 1: GET /api/get
    let mut query = vec![("track_name", title), ("artist_name", artist)];
    if let Some(alb) = album {
        if !alb.is_empty() {
            query.push(("album_name", alb));
        }
    }
    let dur_str;
    if let Some(dur) = duration_secs {
        if dur > 0 {
            dur_str = dur.to_string();
            query.push(("duration", &dur_str));
        }
    }

    if let Ok(resp) = client
        .get("https://lrclib.net/api/get")
        .query(&query)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<LrclibResponse>().await {
                if let Some(synced) = &data.synced_lyrics {
                    if !synced.trim().is_empty() {
                        let lines = parse_lrc(synced);
                        if !lines.is_empty() {
                            return Some(TrackLyrics {
                                track_title: title.to_string(),
                                artist_name: artist.to_string(),
                                synced_lines: lines,
                                plain_lyrics: data.plain_lyrics,
                            });
                        }
                    }
                }
            }
        }
    }

    // Attempt 2: Fallback GET /api/search
    let search_query = vec![("track_name", title), ("artist_name", artist)];
    if let Ok(resp) = client
        .get("https://lrclib.net/api/search")
        .query(&search_query)
        .send()
        .await
    {
        if resp.status().is_success() {
            if let Ok(items) = resp.json::<Vec<LrclibResponse>>().await {
                for item in items {
                    if let Some(synced) = &item.synced_lyrics {
                        if !synced.trim().is_empty() {
                            let lines = parse_lrc(synced);
                            if !lines.is_empty() {
                                return Some(TrackLyrics {
                                    track_title: title.to_string(),
                                    artist_name: artist.to_string(),
                                    synced_lines: lines,
                                    plain_lyrics: item.plain_lyrics,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

pub fn parse_lrc(lrc: &str) -> Vec<LyricLine> {
    let mut lines = Vec::new();
    for line in lrc.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut rest = line;
        let mut timestamps = Vec::new();

        while rest.starts_with('[') {
            if let Some(close_idx) = rest.find(']') {
                let tag = &rest[1..close_idx];
                if let Some(ts) = parse_timestamp(tag) {
                    timestamps.push(ts);
                }
                rest = &rest[close_idx + 1..];
            } else {
                break;
            }
        }

        let text = rest.trim().to_string();
        for ts in timestamps {
            lines.push(LyricLine {
                timestamp: ts,
                text: text.clone(),
            });
        }
    }

    lines.sort_by_key(|l| l.timestamp);
    lines
}

fn parse_timestamp(tag: &str) -> Option<Duration> {
    let parts: Vec<&str> = tag.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let minutes: u64 = parts[0].trim().parse().ok()?;
    let seconds_millis = parts[1].trim();

    let (seconds, millis): (u64, u64) = if let Some(dot_idx) = seconds_millis.find('.') {
        let secs: u64 = seconds_millis[..dot_idx].parse().ok()?;
        let frac = &seconds_millis[dot_idx + 1..];
        let ms: u64 = match frac.len() {
            1 => frac.parse::<u64>().ok()? * 100,
            2 => frac.parse::<u64>().ok()? * 10,
            3 => frac.parse::<u64>().ok()?,
            _ => frac[..3].parse::<u64>().ok()?,
        };
        (secs, ms)
    } else {
        let secs: u64 = seconds_millis.parse().ok()?;
        (secs, 0)
    };

    Some(Duration::from_millis(
        minutes * 60 * 1000 + seconds * 1000 + millis,
    ))
}
