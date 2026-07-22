use serde::Deserialize;
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
    cache: Arc<Mutex<Option<(String, String, Option<TrackLyrics>)>>>,
}

impl LyricsService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn get_lyrics_for_track(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_secs: Option<u64>,
    ) -> Option<TrackLyrics> {
        if title.is_empty() || title == "Silence" {
            return None;
        }

        let clean_title = clean_track_title(title);

        if let Ok(guard) = self.cache.lock() {
            if let Some((cached_title, cached_artist, lyrics)) = guard.as_ref() {
                if cached_title == &clean_title && cached_artist == artist {
                    return lyrics.clone();
                }
            }
        }

        let result = fetch_lrclib(&self.client, &clean_title, artist, album, duration_secs).await;

        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some((clean_title, artist.to_string(), result.clone()));
        }

        result
    }
}

impl Default for LyricsService {
    fn default() -> Self {
        Self::new()
    }
}

fn clean_track_title(title: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc() {
        let sample = r#"
[00:12.34] Line one
[00:15.500] Line two
[01:02.00] Line three
"#;
        let lines = parse_lrc(sample);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].timestamp, Duration::from_millis(12340));
        assert_eq!(lines[0].text, "Line one");
        assert_eq!(lines[1].timestamp, Duration::from_millis(15500));
        assert_eq!(lines[1].text, "Line two");
        assert_eq!(lines[2].timestamp, Duration::from_millis(62000));
        assert_eq!(lines[2].text, "Line three");
    }

    #[test]
    fn test_get_current_line() {
        let sample = r#"
[00:10.00] Intro line
[00:20.00] Verse line
[00:30.00] Chorus line
"#;
        let synced = parse_lrc(sample);
        let lyrics = TrackLyrics {
            track_title: "Test".into(),
            artist_name: "Artist".into(),
            synced_lines: synced,
            plain_lyrics: None,
        };

        assert_eq!(lyrics.get_current_line(Duration::from_secs(5)), None);
        assert_eq!(lyrics.get_current_line(Duration::from_secs(12)), Some("Intro line"));
        assert_eq!(lyrics.get_current_line(Duration::from_secs(25)), Some("Verse line"));
        assert_eq!(lyrics.get_current_line(Duration::from_secs(35)), Some("Chorus line"));
    }
}
