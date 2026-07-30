use crate::CompositorService;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WallpaperEngine {
    #[default]
    Awww,
    Swww,
}

impl WallpaperEngine {
    pub fn detect() -> Self {
        if Command::new("awww").arg("--version").output().is_ok() {
            Self::Awww
        } else {
            Self::Swww
        }
    }

    pub fn cli_cmd(&self) -> &'static str {
        match self {
            Self::Awww => "awww",
            Self::Swww => "swww",
        }
    }

    pub fn daemon_cmd(&self) -> &'static str {
        match self {
            Self::Awww => "awww-daemon",
            Self::Swww => "swww-daemon",
        }
    }
}

#[derive(Clone)]
pub struct WallpaperService {
    current: Arc<Mutex<Option<PathBuf>>>,
    pub engine: WallpaperEngine,
    compositor: CompositorService,
    config_file: PathBuf,
}

impl WallpaperService {
    pub fn new(compositor: CompositorService) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("capsule");
        let _ = fs::create_dir_all(&config_dir);
        let config_file = config_dir.join("current_wallpaper");
        let engine = WallpaperEngine::detect();

        let service = Self {
            current: Arc::new(Mutex::new(None)),
            engine,
            compositor,
            config_file,
        };

        service.ensure_daemon_in_background();
        service.restore_saved_wallpaper();
        service
    }

    pub fn get_current_wallpaper(&self) -> Option<PathBuf> {
        if let Ok(guard) = self.current.lock() {
            if let Some(ref path) = *guard {
                return Some(path.clone());
            }
        }
        if self.config_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_file) {
                let path = PathBuf::from(content.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Checks if the daemon is active.
    fn is_daemon_running(&self) -> bool {
        Command::new(self.engine.cli_cmd())
            .arg("query")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub fn ensure_daemon_in_background(&self) {
        if self.is_daemon_running() {
            return;
        }

        let daemon = self.engine.daemon_cmd();
        println!("[WallpaperService] Spawning background daemon: {}", daemon);

        let _ = Command::new(daemon)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();

        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(150));
            if self.is_daemon_running() {
                break;
            }
        }
    }

    /// Restores previously saved wallpaper from disk.
    fn restore_saved_wallpaper(&self) {
        if self.config_file.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_file) {
                let path = PathBuf::from(content.trim());
                if path.exists() {
                    self.set_wallpaper_internal(&path, false);
                }
            }
        }
    }

    pub async fn pick_wallpaper_file() -> Option<PathBuf> {
        tokio::task::spawn_blocking(|| {
            let zenity_output = Command::new("zenity")
                .arg("--file-selection")
                .arg("--title=Select Wallpaper")
                .arg("--file-filter=*.png *.jpg *.jpeg *.webp")
                .output();

            if let Ok(out) = zenity_output {
                if out.status.success() {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        return Some(PathBuf::from(path_str));
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
                        return Some(PathBuf::from(path_str));
                    }
                }
            }

            None
        })
        .await
        .unwrap_or(None)
    }

    pub fn set_wallpaper(&self, path: impl AsRef<Path>) -> bool {
        self.set_wallpaper_internal(path.as_ref(), true)
    }

    fn set_wallpaper_internal(&self, path: &Path, save_to_disk: bool) -> bool {
        if !path.exists() {
            eprintln!("[WallpaperService] File does not exist: {:?}", path);
            return false;
        }

        self.ensure_daemon_in_background();

        let path_buf = path.to_path_buf();
        let path_str = path_buf.to_string_lossy().to_string();
        let engine = self.engine;
        let compositor = self.compositor.clone();

        if let Ok(mut current_guard) = self.current.lock() {
            *current_guard = Some(path_buf);
        }

        if save_to_disk {
            let _ = fs::write(&self.config_file, &path_str);
        }

        thread::spawn(move || {
            let refresh_rate = compositor.get_refresh_rate();
            let current_fps = if refresh_rate.is_finite() && refresh_rate > 0.0 {
                (refresh_rate.round() as u32).max(30)
            } else {
                60
            };

            let output = Command::new(engine.cli_cmd())
                .args([
                    "img",
                    &path_str,
                    "--transition-type",
                    "outer",
                    "--transition-step",
                    "90",
                    "--transition-fps",
                    &current_fps.to_string(),
                    "--transition-duration",
                    "0.8",
                ])
                .output();

            match output {
                Ok(out) => {
                    if !out.status.success() {
                        let err = String::from_utf8_lossy(&out.stderr);
                        eprintln!("[WallpaperService] {} img error: {}", engine.cli_cmd(), err);
                    } else {
                        println!(
                            "[WallpaperService] Wallpaper successfully changed to: {}",
                            path_str
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[WallpaperService] Failed to execute {}: {}",
                        engine.cli_cmd(),
                        e
                    );
                }
            }
        });

        true
    }

    pub fn get_current(&self) -> Option<PathBuf> {
        self.current.lock().ok().and_then(|guard| guard.clone())
    }
}
