use crate::compositor::{Compositor, WorkspaceInfo};
use arc_swap::ArcSwap;
use futures::StreamExt;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

#[zbus::proxy(
    interface = "org.kde.KWin.VirtualDesktopManager",
    default_service = "org.kde.KWin",
    default_path = "/VirtualDesktopManager"
)]
trait VirtualDesktopManager {
    #[zbus(property, name = "current")]
    fn current_desktop(&self) -> zbus::Result<String>;

    #[zbus(property, name = "desktops")]
    fn desktop_data(&self) -> zbus::Result<zbus::zvariant::OwnedValue>;

    #[zbus(property, name = "current")]
    fn set_current_desktop(&self, desktop: &str) -> zbus::fdo::Result<()>;

    #[zbus(name = "setCurrent")]
    fn set_current(&self, desktop: &str) -> zbus::Result<()>;

    #[zbus(name = "createDesktop")]
    fn create_desktop(&self, position: u32, name: &str) -> zbus::Result<String>;

    #[zbus(signal, name = "currentChanged")]
    fn current_changed(&self, desktop: &str) -> zbus::Result<()>;

    #[zbus(signal, name = "desktopsChanged")]
    fn desktops_changed(&self) -> zbus::Result<()>;
}

async fn desktop_entries(
    proxy: &VirtualDesktopManagerProxy<'_>,
) -> zbus::Result<Vec<(String, String)>> {
    let value = proxy.desktop_data().await?;
    if value.value_signature() == <Vec<String> as zbus::zvariant::Type>::SIGNATURE {
        let ids = Vec::<String>::try_from(value)?;
        return Ok(ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| (id, (i + 1).to_string()))
            .collect());
    }
    let mut desktops = Vec::<(u32, String, String)>::try_from(value)?;
    desktops.sort_by_key(|(position, _, _)| *position);
    Ok(desktops
        .into_iter()
        .map(|(_, id, name)| (id, name))
        .collect())
}

#[derive(Default)]
struct WorkspaceState {
    current: Option<WorkspaceInfo>,
    workspaces: Vec<WorkspaceInfo>,
}

#[derive(Clone, Default)]
pub struct KWin {
    state: Arc<ArcSwap<WorkspaceState>>,
}

impl KWin {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn focused_application(&self) -> Option<String> {
        let mut command = tokio::process::Command::new("kdotool");
        command.args(["getactivewindow", "getwindowclassname"]);
        command.kill_on_drop(true);

        if let Ok(Ok(output)) =
            tokio::time::timeout(Duration::from_millis(200), command.output()).await
        {
            if output.status.success() {
                let class_name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !class_name.is_empty() && !class_name.eq_ignore_ascii_case("failed") {
                    return Some(class_name);
                }
            }
        }

        None
    }

    async fn listen(
        &self,
        current_workspace: &ArcSwap<WorkspaceInfo>,
        tx: &broadcast::Sender<WorkspaceInfo>,
    ) -> zbus::Result<()> {
        let connection = crate::dbus_util::get_shared_session_conn()
            .await
            .ok_or_else(|| zbus::Error::Failure("No session D-Bus connection available".into()))?;
        let proxy = VirtualDesktopManagerProxy::builder(&connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?;
        let mut changes = proxy.receive_current_changed().await?;
        let mut desktop_changes = proxy.receive_desktops_changed().await?;

        let current = proxy.current_desktop().await.unwrap_or_default();
        let _ = self.update(&proxy, &current, current_workspace, tx).await;

        loop {
            tokio::select! {
                change = changes.next() => {
                    let Some(change) = change else { break };
                    let args = change.args()?;
                    self.update(&proxy, args.desktop(), current_workspace, tx).await?;
                }
                change = desktop_changes.next() => {
                    if change.is_none() { break; }
                    let current = proxy.current_desktop().await.unwrap_or_default();
                    self.update(&proxy, &current, current_workspace, tx).await?;
                }
            }
        }
        Err(zbus::Error::Failure(
            "KWin desktop signal stream ended".into(),
        ))
    }

    async fn update(
        &self,
        proxy: &VirtualDesktopManagerProxy<'_>,
        current: &str,
        current_workspace: &ArcSwap<WorkspaceInfo>,
        tx: &broadcast::Sender<WorkspaceInfo>,
    ) -> zbus::Result<()> {
        let desktops = desktop_entries(proxy).await?;
        let workspaces: Vec<_> = desktops
            .iter()
            .enumerate()
            .map(|(index, (_, name))| {
                let num = i32::try_from(index + 1)
                    .map_err(|error| zbus::Error::Failure(error.to_string()))?;
                let display_name = if name.trim().is_empty() {
                    num.to_string()
                } else {
                    name.clone()
                };
                Ok(WorkspaceInfo {
                    id: i64::from(num),
                    num,
                    name: display_name,
                    is_special: false,
                    special_name: None,
                })
            })
            .collect::<zbus::Result<_>>()?;
        let workspace = desktops
            .iter()
            .position(|(id, _)| id == current)
            .and_then(|index| workspaces.get(index))
            .cloned()
            .or_else(|| workspaces.first().cloned());
        self.state.store(Arc::new(WorkspaceState {
            current: workspace.clone(),
            workspaces,
        }));
        if let Some(workspace) = workspace {
            current_workspace.store(Arc::new(workspace.clone()));
            let _ = tx.send(workspace);
        }
        Ok(())
    }
}

fn parse_kscreen_refresh_rate(json_bytes: &[u8]) -> Option<f64> {
    let v: serde_json::Value = serde_json::from_slice(json_bytes).ok()?;
    let outputs = v.get("outputs")?.as_array()?;

    let active_output = outputs
        .iter()
        .find(|o| {
            o.get("enabled").and_then(|e| e.as_bool()) == Some(true)
                && o.get("primary").and_then(|p| p.as_bool()) == Some(true)
        })
        .or_else(|| {
            outputs
                .iter()
                .find(|o| o.get("enabled").and_then(|e| e.as_bool()) == Some(true))
        })?;

    let current_mode_id = active_output.get("currentModeId")?;
    let modes = active_output.get("modes")?.as_array()?;

    for mode in modes {
        let mode_id = mode.get("id")?;
        let is_current = if let (Some(curr_str), Some(m_str)) =
            (current_mode_id.as_str(), mode_id.as_str())
        {
            curr_str == m_str
        } else if let (Some(curr_num), Some(m_num)) = (current_mode_id.as_i64(), mode_id.as_i64()) {
            curr_num == m_num
        } else {
            current_mode_id.to_string() == mode_id.to_string()
        };

        if is_current {
            if let Some(rate) = mode.get("refreshRate").and_then(|r| r.as_f64()) {
                return Some(rate);
            }
        }
    }

    None
}

impl Compositor for KWin {
    fn get_refresh_rate(&self) -> f64 {
        let rate = Command::new("kscreen-doctor")
            .arg("-j")
            .output()
            .ok()
            .filter(|out| out.status.success())
            .and_then(|out| parse_kscreen_refresh_rate(&out.stdout));

        rate.unwrap_or(60.0)
    }

    fn get_workspace(&self) -> Option<WorkspaceInfo> {
        self.state.load().current.clone()
    }

    fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        self.state.load().workspaces.clone()
    }

    fn switch_workspace(&self, id: i64) {
        let Some(index) = id.checked_sub(1).and_then(|id| usize::try_from(id).ok()) else {
            crate::log_warn!("COMPOSITOR", "Invalid KWin workspace ID: {id}");
            return;
        };
        crate::spawn_tokio(async move {
            let result: zbus::Result<()> = async {
                let connection = crate::dbus_util::get_shared_session_conn()
                    .await
                    .ok_or_else(|| {
                        zbus::Error::Failure("No session D-Bus connection available".into())
                    })?;
                let proxy = VirtualDesktopManagerProxy::builder(&connection)
                    .cache_properties(zbus::proxy::CacheProperties::No)
                    .build()
                    .await?;
                let desktops = desktop_entries(&proxy).await?;
                if let Some((desktop, _)) = desktops.get(index) {
                    match proxy.set_current_desktop(desktop).await {
                        Err(
                            zbus::fdo::Error::UnknownProperty(_)
                            | zbus::fdo::Error::PropertyReadOnly(_)
                            | zbus::fdo::Error::UnknownMethod(_),
                        ) => proxy.set_current(desktop).await,
                        result => result.map_err(Into::into),
                    }
                } else {
                    let target_pos = u32::try_from(index).unwrap_or(0);
                    let name = format!("Desktop {}", index + 1);
                    if let Ok(new_id) = proxy.create_desktop(target_pos, &name).await {
                        let _ = proxy.set_current_desktop(&new_id).await;
                    }
                    Ok(())
                }
            }
            .await;
            if let Err(error) = result {
                crate::log_warn!("COMPOSITOR", "Failed to switch KWin workspace: {error}");
            }
        });
    }

    fn logout(&self) -> futures::future::BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async {
            if let Some(conn) = crate::dbus_util::get_shared_session_conn().await {
                let call = conn
                    .call_method(
                        Some("org.kde.Shutdown"),
                        "/Shutdown",
                        Some("org.kde.Shutdown"),
                        "logout",
                        &(),
                    )
                    .await;
                if call.is_ok() {
                    return Ok(());
                }
            }
            crate::SystemService::logout().await
        })
    }
}

pub async fn run_events_listener(
    backend: KWin,
    current_workspace: Arc<ArcSwap<WorkspaceInfo>>,
    tx: broadcast::Sender<WorkspaceInfo>,
) {
    loop {
        if let Err(error) = backend.listen(&current_workspace, &tx).await {
            crate::log_warn!("COMPOSITOR", "KWin workspace listener failed: {error}");
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
