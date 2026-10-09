use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tar::Archive;

pub const REPO: &str = "moiCR/Capsule";
const USER_AGENT: &str = "capsule-installer/0.1.0";

#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    #[allow(dead_code)]
    pub tag_name: String,
    pub version: String,
    pub download_url: String,
    #[allow(dead_code)]
    pub is_prerelease: bool,
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    prerelease: bool,
}

pub fn fetch_release_info(target: &str, beta_mode: bool) -> Result<ReleaseInfo> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    if beta_mode {
        let url = format!("https://api.github.com/repos/{}/releases?per_page=50", REPO);
        let resp = client.get(&url).send().context("Failed to query GitHub API for releases")?;
        let releases: Vec<GhRelease> = resp.json().context("Failed to parse releases JSON")?;

        for r in releases {
            if r.prerelease && r.tag_name.contains("-beta") {
                let tag = r.tag_name;
                let version = tag.trim_start_matches('v').to_string();
                let download_url = format!(
                    "https://github.com/{}/releases/download/{}/capsule-{}.tar.gz",
                    REPO, tag, target
                );
                return Ok(ReleaseInfo {
                    tag_name: tag,
                    version,
                    download_url,
                    is_prerelease: true,
                });
            }
        }
        Err(anyhow!("No published beta release found on GitHub"))
    } else {
        let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
        let resp = client.get(&url).send();

        match resp {
            Ok(res) if res.status().is_success() => {
                let rel: GhRelease = res.json().context("Failed to parse latest release JSON")?;
                let tag = rel.tag_name;
                let version = tag.trim_start_matches('v').to_string();
                let download_url = format!(
                    "https://github.com/{}/releases/download/{}/capsule-{}.tar.gz",
                    REPO, tag, target
                );
                Ok(ReleaseInfo {
                    tag_name: tag,
                    version,
                    download_url,
                    is_prerelease: false,
                })
            }
            _ => {
                let download_url = format!(
                    "https://github.com/{}/releases/latest/download/capsule-{}.tar.gz",
                    REPO, target
                );
                Ok(ReleaseInfo {
                    tag_name: "latest".to_string(),
                    version: "latest".to_string(),
                    download_url,
                    is_prerelease: false,
                })
            }
        }
    }
}

pub fn download_and_extract_binary<F>(
    download_url: &str,
    progress_callback: F,
) -> Result<PathBuf>
where
    F: Fn(f64, &str),
{
    progress_callback(0.05, "Connecting to GitHub Releases...");
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let mut response = client
        .get(download_url)
        .send()
        .context("Failed to send request for release asset")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Download failed with HTTP status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!("capsule-install-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    let tar_path = temp_dir.join("capsule.tar.gz");
    let mut file = File::create(&tar_path)?;

    progress_callback(0.15, "Downloading capsule archive...");
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 16384];

    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])?;
        downloaded += read as u64;

        if total_size > 0 {
            let frac = 0.15 + 0.60 * (downloaded as f64 / total_size as f64);
            let mb = downloaded as f64 / (1024.0 * 1024.0);
            let total_mb = total_size as f64 / (1024.0 * 1024.0);
            progress_callback(frac, &format!("Downloading archive ({:.1} MB / {:.1} MB)...", mb, total_mb));
        } else {
            let mb = downloaded as f64 / (1024.0 * 1024.0);
            progress_callback(0.40, &format!("Downloading archive ({:.1} MB)...", mb));
        }
    }

    progress_callback(0.80, "Extracting binary from archive...");
    let tar_file = File::open(&tar_path)?;
    let decompressor = GzDecoder::new(tar_file);
    let mut archive = Archive::new(decompressor);

    archive.unpack(&temp_dir).context("Failed to unpack archive")?;

    let extracted_capsule = temp_dir.join("capsule");
    if extracted_capsule.exists() {
        progress_callback(0.90, "Archive extracted successfully.");
        Ok(extracted_capsule)
    } else {
        let extracted_capital = temp_dir.join("Capsule");
        if extracted_capital.exists() {
            progress_callback(0.90, "Archive extracted successfully.");
            Ok(extracted_capital)
        } else {
            Err(anyhow!("No 'capsule' binary found inside downloaded archive"))
        }
    }
}
