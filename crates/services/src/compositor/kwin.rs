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

    #[zbus(signal, name = "currentChanged")]
    fn current_changed(&self, desktop: &str) -> zbus::Result<()>;

    #[zbus(signal, name = "desktopsChanged")]
    fn desktops_changed(&self) -> zbus::Result<()>;
}

async fn desktop_ids(proxy: &VirtualDesktopManagerProxy<'_>) -> zbus::Result<Vec<String>> {
    let value = proxy.desktop_data().await?;
    if value.value_signature() == <Vec<String> as zbus::zvariant::Type>::SIGNATURE {
        return Ok(Vec::<String>::try_from(value)?);
    }
    let mut desktops = Vec::<(u32, String, String)>::try_from(value)?;
    desktops.sort_by_key(|(position, _, _)| *position);
    Ok(desktops.into_iter().map(|(_, id, _)| id).collect())
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

    async fn listen(
        &self,
        current_workspace: &ArcSwap<WorkspaceInfo>,
        tx: &broadcast::Sender<WorkspaceInfo>,
    ) -> zbus::Result<()> {
        let connection = zbus::Connection::session().await?;
        let proxy = VirtualDesktopManagerProxy::builder(&connection)
            .cache_properties(zbus::proxy::CacheProperties::No)
            .build()
            .await?;
        let mut changes = proxy.receive_current_changed().await?;
        let mut desktop_changes = proxy.receive_desktops_changed().await?;
        let current = proxy.current_desktop().await?;
        self.update(&proxy, &current, current_workspace, tx).await?;

        loop {
            tokio::select! {
                change = changes.next() => {
                    let Some(change) = change else { break };
                    let args = change.args()?;
                    self.update(&proxy, args.desktop(), current_workspace, tx).await?;
                }
                change = desktop_changes.next() => {
                    if change.is_none() { break; }
                    let current = proxy.current_desktop().await?;
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
        let desktops = desktop_ids(proxy).await?;
        let workspaces: Vec<_> = desktops
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let num = i32::try_from(index + 1)
                    .map_err(|error| zbus::Error::Failure(error.to_string()))?;
                Ok(WorkspaceInfo {
                    id: i64::from(num),
                    num,
                    name: num.to_string(),
                    is_special: false,
                    special_name: None,
                })
            })
            .collect::<zbus::Result<_>>()?;
        let workspace = desktops
            .iter()
            .position(|desktop| desktop == current)
            .and_then(|index| workspaces.get(index))
            .cloned();
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

    let current_mode_id = active_output.get("currentModeId")?.as_str()?;
    let modes = active_output.get("modes")?.as_array()?;

    for mode in modes {
        let is_current = mode.get("id").and_then(|id| id.as_str()) == Some(current_mode_id);
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
                let connection = zbus::Connection::session().await?;
                let proxy = VirtualDesktopManagerProxy::builder(&connection)
                    .cache_properties(zbus::proxy::CacheProperties::No)
                    .build()
                    .await?;
                let desktops = desktop_ids(&proxy).await?;
                let desktop = desktops.get(index).ok_or_else(|| {
                    zbus::Error::Failure(format!("Unknown KWin workspace ID: {id}"))
                })?;
                match proxy.set_current_desktop(desktop).await {
                    Err(
                        zbus::fdo::Error::UnknownProperty(_)
                        | zbus::fdo::Error::PropertyReadOnly(_)
                        | zbus::fdo::Error::UnknownMethod(_),
                    ) => proxy.set_current(desktop).await,
                    result => result.map_err(Into::into),
                }
            }
            .await;
            if let Err(error) = result {
                crate::log_warn!("COMPOSITOR", "Failed to switch KWin workspace: {error}");
            }
        });
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
