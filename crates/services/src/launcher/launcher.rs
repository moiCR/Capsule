use super::Application;
use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

static ICON_MAP_CACHE: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();

#[derive(Clone, Default)]
pub struct LauncherService {
    apps: Arc<ArcSwap<Vec<Application>>>,
}

impl LauncherService {
    pub fn new() -> Self {
        let service = Self {
            apps: Arc::new(ArcSwap::from_pointee(Vec::new())),
        };

        let service_clone = service.clone();
        tokio::spawn(async move {
            loop {
                if let Err(err) = service_clone.refresh().await {
                    crate::log_warn!("LAUNCHER", "LauncherService refresh warning: {err}");
                }
                tokio::time::sleep(std::time::Duration::from_secs(120)).await;
            }
        });

        service
    }

    pub fn get_apps(&self) -> Arc<Vec<Application>> {
        self.apps.load_full()
    }

    pub fn search(&self, query: &str) -> Vec<Application> {
        let apps_arc = self.apps.load();

        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return (**apps_arc).clone();
        }

        apps_arc
            .iter()
            .filter(|app| {
                app.name.to_lowercase().contains(&q)
                    || app
                        .generic_name
                        .as_ref()
                        .map_or(false, |g| g.to_lowercase().contains(&q))
                    || app
                        .comment
                        .as_ref()
                        .map_or(false, |c| c.to_lowercase().contains(&q))
                    || app.keywords.iter().any(|k| k.to_lowercase().contains(&q))
                    || app.exec.to_lowercase().contains(&q)
            })
            .cloned()
            .collect()
    }

    pub fn launch_app(app: &Application) -> Result<()> {
        let clean_exec = Self::clean_exec_command(&app.exec);
        if clean_exec.is_empty() {
            anyhow::bail!("Empty exec command");
        }

        let cmd_str = if app.terminal {
            format!("x-terminal-emulator -e {clean_exec} >/dev/null 2>&1 &")
        } else {
            format!("{clean_exec} >/dev/null 2>&1 &")
        };

        crate::log_info!("LAUNCHER", "Launching app '{}': {cmd_str}", app.name);

        std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .spawn()
            .context("Failed to launch application")?;

        Ok(())
    }

    pub async fn refresh(&self) -> Result<()> {
        let mut app_dirs = vec![
            PathBuf::from("/usr/share/applications"),
            PathBuf::from("/usr/local/share/applications"),
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        ];

        if let Some(home) = dirs::home_dir() {
            app_dirs.push(home.join(".local/share/applications"));
            app_dirs.push(home.join(".local/share/flatpak/exports/share/applications"));
            app_dirs.push(home.join(".local/share/snap/desktop/applications"));
        }

        if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_dirs.split(':') {
                let path = Path::new(dir).join("applications");
                if path.exists() && !app_dirs.contains(&path) {
                    app_dirs.push(path);
                }
            }
        }

        let mut discovered: HashMap<String, Application> = HashMap::new();

        let icon_map = Self::get_cached_icon_map().await;

        for dir in app_dirs {
            if !dir.exists() {
                continue;
            }

            let mut entries = match tokio::fs::read_dir(&dir).await {
                Ok(e) => e,
                Err(_) => continue,
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("desktop") {
                    if let Ok(app) = Self::parse_desktop_file(&path, &icon_map).await {
                        discovered.entry(app.id.clone()).or_insert(app);
                    }
                }
            }
        }

        let mut app_list: Vec<Application> = discovered.into_values().collect();
        app_list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        crate::log_info!(
            "LAUNCHER",
            "Indexed {} desktop applications",
            app_list.len()
        );

        self.apps.store(Arc::new(app_list));

        Ok(())
    }

    fn clean_exec_command(exec: &str) -> String {
        exec.split_whitespace()
            .filter(|arg| !arg.starts_with('%'))
            .collect::<Vec<_>>()
            .join(" ")
    }
    async fn get_cached_icon_map() -> &'static HashMap<String, PathBuf> {
        if let Some(map) = ICON_MAP_CACHE.get() {
            return map;
        }
        let map = Self::build_icon_map().await;
        let _ = ICON_MAP_CACHE.set(map);
        ICON_MAP_CACHE.get().unwrap()
    }

    async fn build_icon_map() -> HashMap<String, PathBuf> {
        let mut map = HashMap::new();
        let subdirs = [
            "hicolor/scalable/apps",
            "hicolor/512x512/apps",
            "hicolor/256x256/apps",
            "hicolor/128x128/apps",
            "hicolor/48x48/apps",
            "pixmaps",
        ];
        let mut base_dirs = vec![
            PathBuf::from("/usr/share/icons"),
            PathBuf::from("/usr/share"),
        ];
        if let Some(home) = dirs::home_dir() {
            base_dirs.push(home.join(".local/share/icons"));
        }

        for base in base_dirs {
            for subdir in &subdirs {
                let dir = base.join(subdir);
                if let Ok(mut entries) = tokio::fs::read_dir(&dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            map.entry(stem.to_string()).or_insert(path);
                        }
                    }
                }
            }
        }
        map
    }

    async fn parse_desktop_file(
        path: &Path,
        icon_map: &HashMap<String, PathBuf>,
    ) -> Result<Application> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut in_desktop_entry = false;

        let mut name = None;
        let mut generic_name = None;
        let mut comment = None;
        let mut exec = None;
        let mut icon_name = None;
        let mut keywords = Vec::new();
        let mut no_display = false;
        let mut hidden = false;
        let mut terminal = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry || line.starts_with('#') || !line.contains('=') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim();

                match key {
                    "Name" if name.is_none() => name = Some(val.to_string()),
                    "GenericName" if generic_name.is_none() => generic_name = Some(val.to_string()),
                    "Comment" if comment.is_none() => comment = Some(val.to_string()),
                    "Exec" if exec.is_none() => exec = Some(val.to_string()),
                    "Icon" if icon_name.is_none() => icon_name = Some(val.to_string()),
                    "NoDisplay" => no_display = val.eq_ignore_ascii_case("true"),
                    "Hidden" => hidden = val.eq_ignore_ascii_case("true"),
                    "Terminal" => terminal = val.eq_ignore_ascii_case("true"),
                    "Keywords" => {
                        keywords = val
                            .split(';')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                    _ => {}
                }
            }
        }

        if no_display || hidden {
            anyhow::bail!("Application is hidden or NoDisplay");
        }

        let name = name.context("Missing Name field in desktop entry")?;
        let exec = exec.context("Missing Exec field in desktop entry")?;
        let file_stem = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("app")
            .to_string();

        let icon_path = icon_name.as_deref().and_then(|ic| {
            let p = Path::new(ic);
            if p.is_absolute() && p.exists() {
                Some(p.to_path_buf())
            } else {
                icon_map.get(ic).cloned()
            }
        });

        Ok(Application {
            id: file_stem,
            name,
            generic_name,
            comment,
            exec,
            icon_name,
            icon_path,
            keywords,
            terminal,
        })
    }
}
