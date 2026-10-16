use ashpd::desktop::file_chooser::{FileFilter, OpenFileRequest};
use std::path::PathBuf;
use std::process::Command;

pub async fn pick_wallpaper() -> Option<PathBuf> {
    // 1. Try XDG Desktop Portal via ashpd
    let request = OpenFileRequest::default()
        .title("Select Wallpaper")
        .modal(false)
        .multiple(false)
        .filter(
            FileFilter::new("Images")
                .glob("*.png")
                .glob("*.jpg")
                .glob("*.jpeg")
                .glob("*.webp"),
        );

    match request.send().await {
        Ok(response) => {
            if let Ok(files) = response.response() {
                if let Some(uri) = files.uris().first() {
                    if let Ok(parsed_url) = url::Url::parse(uri.as_str()) {
                        if let Ok(path) = parsed_url.to_file_path() {
                            if path.exists() {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("[WallpaperPortal] XDG portal returned error or was cancelled: {err}");
        }
    }

    // 2. Fallback to zenity / kdialog if portal wasn't available
    pick_fallback().await
}

async fn pick_fallback() -> Option<PathBuf> {
    services::spawn_blocking(|| {
        let zenity_output = Command::new("zenity")
            .arg("--file-selection")
            .arg("--title=Select Wallpaper")
            .arg("--file-filter=*.png *.jpg *.jpeg *.webp")
            .output();

        if let Ok(out) = zenity_output {
            if out.status.success() {
                let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        let kdialog_output = Command::new("kdialog")
            .args(["--getopenfilename", ".", "*.png *.jpg *.jpeg *.webp"])
            .output();

        if let Ok(out) = kdialog_output {
            if out.status.success() {
                let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let path = PathBuf::from(path_str);
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    })
    .await
    .unwrap_or(None)
}
