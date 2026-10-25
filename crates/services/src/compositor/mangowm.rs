use crate::compositor::{Compositor, WorkspaceInfo};
use arc_swap::ArcSwap;
use futures::StreamExt;
use mango_ipc::{Socket, dispatch, query, request, response};
use std::collections::HashMap;
use std::future::poll_fn;
use std::io::ErrorKind;
use std::os::fd::AsFd;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use tokio::sync::broadcast;
use wayland_client::{
    Connection, Dispatch, QueueHandle, WEnum,
    protocol::{wl_output, wl_registry},
};

const IPC_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Default)]
pub struct Mango {
    monitors: Arc<ArcSwap<Vec<response::Monitor>>>,
    socket: Option<Socket>,
    output_rates: Arc<ArcSwap<HashMap<String, f64>>>,
}

impl Mango {
    pub async fn logout(&self) -> anyhow::Result<()> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mango IPC is unavailable"))?;
        let response = tokio::time::timeout(IPC_TIMEOUT, socket.dispatch(dispatch::Quit)).await??;
        anyhow::ensure!(response.success, "Mango rejected logout");
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            socket: Socket::try_default().ok(),
            ..Self::default()
        }
    }

    fn active_monitor(monitors: &[response::Monitor]) -> Option<&response::Monitor> {
        monitors.iter().find(|monitor| monitor.active)
    }

    fn workspace(index: u32) -> Option<WorkspaceInfo> {
        Some(WorkspaceInfo {
            id: i64::from(index),
            num: i32::try_from(index).ok()?,
            name: index.to_string(),
            is_special: false,
            special_name: None,
        })
    }

    pub async fn focused_application(&self) -> Option<String> {
        let socket = self.socket.as_ref()?;
        let client = tokio::time::timeout(IPC_TIMEOUT, socket.get(query::FocusingClient))
            .await
            .ok()?
            .ok()?;
        (!client.appid.is_empty()).then_some(client.appid)
    }

    pub async fn refresh(&self) {
        let Some(socket) = &self.socket else {
            return;
        };
        if let Ok(Ok(response)) =
            tokio::time::timeout(IPC_TIMEOUT, socket.get(query::AllMonitors)).await
        {
            self.monitors.store(Arc::new(response.monitors));
        }
    }
}

impl Compositor for Mango {
    fn logout(&self) -> futures::future::BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(Mango::logout(self))
    }

    fn get_refresh_rate(&self) -> f64 {
        let monitors = self.monitors.load();
        Self::active_monitor(&monitors)
            .and_then(|monitor| self.output_rates.load().get(&monitor.name).copied())
            .unwrap_or(60.0)
    }

    fn get_workspace(&self) -> Option<WorkspaceInfo> {
        let monitors = self.monitors.load();
        let monitor = Self::active_monitor(&monitors)?;
        let index = monitor
            .tags
            .iter()
            .find(|tag| tag.is_active)
            .map(|tag| tag.index.get())
            .or_else(|| monitor.active_tags.first().copied())?;
        Self::workspace(index)
    }

    fn get_workspaces(&self) -> Vec<WorkspaceInfo> {
        let monitors = self.monitors.load();
        let Some(monitor) = Self::active_monitor(&monitors) else {
            return Vec::new();
        };
        let mut workspaces: Vec<_> = monitor
            .tags
            .iter()
            .filter_map(|tag| Self::workspace(tag.index.get()))
            .collect();
        workspaces.sort_by_key(|workspace| workspace.num);
        workspaces
    }

    fn switch_workspace(&self, id: i64) {
        let Some(tag) = u32::try_from(id).ok().and_then(request::Tag::new) else {
            return;
        };
        let Some(socket) = self.socket.clone() else {
            return;
        };
        crate::spawn_tokio(async move {
            match tokio::time::timeout(
                IPC_TIMEOUT,
                socket.dispatch(dispatch::View::new(request::TagMaskOrAllOrPrev::TagMask(
                    tag.into(),
                ))),
            )
            .await
            {
                Ok(Ok(response)) if response.success => {}
                result => {
                    crate::log_error!("COMPOSITOR", "Mango workspace dispatch failed: {result:?}")
                }
            }
        });
    }
}

struct WaylandOutput {
    proxy: wl_output::WlOutput,
    name: Option<String>,
    rate: Option<f64>,
}

struct OutputState {
    outputs: HashMap<u32, WaylandOutput>,
    rates: Arc<ArcSwap<HashMap<String, f64>>>,
}

impl OutputState {
    fn publish(&self) {
        let rates = self
            .outputs
            .values()
            .filter_map(|output| Some((output.name.as_ref()?.clone(), output.rate?)))
            .collect();
        self.rates.store(Arc::new(rates));
    }
}

fn current_mode_rate(flags: WEnum<wl_output::Mode>, refresh: i32) -> Option<Option<f64>> {
    match flags {
        WEnum::Value(flags) if flags.contains(wl_output::Mode::Current) => {
            Some((refresh > 0).then(|| (f64::from(refresh) / 1000.0).clamp(30.0, 360.0)))
        }
        _ => None,
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for OutputState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } if interface == "wl_output" && version >= 4 => {
                let proxy = registry.bind::<wl_output::WlOutput, _, _>(name, 4, queue, name);
                state.outputs.insert(
                    name,
                    WaylandOutput {
                        proxy,
                        name: None,
                        rate: None,
                    },
                );
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some(output) = state.outputs.remove(&name) {
                    output.proxy.release();
                    state.publish();
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for OutputState {
    fn event(
        state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        id: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(output) = state.outputs.get_mut(id) else {
            return;
        };
        match event {
            wl_output::Event::Name { name } => output.name = Some(name),
            wl_output::Event::Mode { flags, refresh, .. } => {
                if let Some(rate) = current_mode_rate(flags, refresh) {
                    output.rate = rate;
                }
            }
            wl_output::Event::Done => state.publish(),
            _ => {}
        }
    }
}

async fn monitor_outputs(rates: Arc<ArcSwap<HashMap<String, f64>>>) -> anyhow::Result<()> {
    let connection = Connection::connect_to_env()?;
    let socket =
        unsafe { tokio::io::unix::AsyncFd::register(connection.as_fd().try_clone_to_owned()?) }?;
    let mut events = connection.new_event_queue();
    let _registry = connection.display().get_registry(&events.handle(), ());
    let mut state = OutputState {
        outputs: HashMap::new(),
        rates,
    };
    loop {
        events.dispatch_pending(&mut state)?;
        loop {
            match connection.flush() {
                Ok(()) => break,
                Err(wayland_client::backend::WaylandError::Io(error))
                    if error.kind() == ErrorKind::WouldBlock =>
                {
                    socket.writable().await?.clear_ready();
                }
                Err(error) => return Err(error.into()),
            }
        }
        poll_fn(|cx| {
            let Some(read) = events.prepare_read() else {
                return Poll::Ready(Ok::<(), anyhow::Error>(()));
            };
            match socket.poll_read_ready(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(error)) => Poll::Ready(Err(error.into())),
                Poll::Ready(Ok(mut readiness)) => match read.read() {
                    Ok(0) => {
                        readiness.clear_ready();
                        Poll::Ready(Ok(()))
                    }
                    Ok(_) => Poll::Ready(Ok(())),
                    Err(wayland_client::backend::WaylandError::Io(error))
                        if error.kind() == ErrorKind::WouldBlock =>
                    {
                        readiness.clear_ready();
                        Poll::Ready(Ok(()))
                    }
                    Err(error) => Poll::Ready(Err(error.into())),
                },
            }
        })
        .await?;
    }
}

pub async fn run_output_listener(backend: Mango) {
    loop {
        if let Err(error) = monitor_outputs(backend.output_rates.clone()).await {
            crate::log_warn!(
                "COMPOSITOR",
                "Mango Wayland output listener failed: {error}"
            );
        }
        backend.output_rates.store(Arc::new(HashMap::new()));
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

pub async fn run_events_listener(
    backend: Mango,
    current_workspace: Arc<ArcSwap<WorkspaceInfo>>,
    tx: broadcast::Sender<WorkspaceInfo>,
) {
    loop {
        if let Some(socket) = &backend.socket
            && let Ok(Ok(mut events)) =
                tokio::time::timeout(IPC_TIMEOUT, socket.watch(query::AllMonitors)).await
        {
            while let Some(Ok(response)) = events.next().await {
                backend.monitors.store(Arc::new(response.monitors));
                if let Some(workspace) = backend.get_workspace()
                    && **current_workspace.load() != workspace
                {
                    current_workspace.store(Arc::new(workspace.clone()));
                    let _ = tx.send(workspace);
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    fn monitor(active: bool, selected: u32) -> response::Monitor {
        serde_json::from_value(serde_json::json!({
            "name": "DP-1", "active": active, "x": 0, "y": 0,
            "width": 1920, "height": 1080, "scale": 1.0,
            "layout_index": 0, "layout_symbol": "[]=", "last_open_surface": "",
            "tags": [
                {"index": 3, "is_active": selected == 3, "is_urgent": false,
                 "layout": "tile", "client_count": 0},
                {"index": 1, "is_active": selected == 1, "is_urgent": false,
                 "layout": "tile", "client_count": 2}
            ],
            "active_tags": [selected], "active_client": null,
            "keymode": "normal", "keyboardlayout": "us"
        }))
        .unwrap()
    }

    #[test]
    fn refresh_rate_tracks_active_monitor_and_falls_back_when_missing() {
        let backend = Mango::default();
        let mut second = monitor(true, 3);
        second.name = "DP-2".into();
        backend
            .monitors
            .store(Arc::new(vec![monitor(false, 1), second.clone()]));
        backend.output_rates.store(Arc::new(HashMap::from([
            ("DP-1".into(), 60.0),
            ("DP-2".into(), 144.0),
        ])));
        assert_eq!(backend.get_refresh_rate(), 144.0);
        second.active = false;
        backend
            .monitors
            .store(Arc::new(vec![monitor(true, 1), second]));
        assert_eq!(backend.get_refresh_rate(), 60.0);
        backend.output_rates.store(Arc::new(HashMap::new()));
        assert_eq!(backend.get_refresh_rate(), 60.0);
    }

    #[test]
    fn only_current_modes_supply_valid_normalized_rates() {
        let current = WEnum::Value(wl_output::Mode::Current);
        assert_eq!(current_mode_rate(current, 144000), Some(Some(144.0)));
        assert_eq!(current_mode_rate(current, 59950), Some(Some(59.95)));
        assert_eq!(current_mode_rate(current, 0), Some(None));
        assert_eq!(current_mode_rate(current, -1), Some(None));
        assert_eq!(current_mode_rate(current, 10000), Some(Some(30.0)));
        assert_eq!(current_mode_rate(current, 1000000), Some(Some(360.0)));
        assert_eq!(
            current_mode_rate(WEnum::Value(wl_output::Mode::Preferred), 240000),
            None
        );
        assert_eq!(current_mode_rate(WEnum::Unknown(128), 240000), None);
    }

    #[tokio::test]
    #[ignore = "requires a running Wayland compositor"]
    async fn reads_named_output_rates_from_wayland() {
        let backend = Mango::default();
        let listener = tokio::spawn(run_output_listener(backend.clone()));
        let received = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let rates = backend.output_rates.load();
                if !rates.is_empty() {
                    return (**rates).clone();
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await;
        listener.abort();
        let rates = received.unwrap();
        assert!(
            rates
                .iter()
                .all(|(name, rate)| !name.is_empty() && (30.0..=360.0).contains(rate))
        );
    }

    #[tokio::test]
    #[ignore = "requires a running Wayland compositor"]
    async fn output_listener_waits_without_consuming_idle_cpu() {
        fn thread_cpu_time() -> Duration {
            let mut time = libc::timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            assert_eq!(
                unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut time) },
                0
            );
            Duration::new(time.tv_sec as u64, time.tv_nsec as u32)
        }

        let backend = Mango::default();
        let listener = tokio::spawn(monitor_outputs(backend.output_rates.clone()));
        tokio::time::timeout(Duration::from_secs(5), async {
            while backend.output_rates.load().is_empty() {
                assert!(!listener.is_finished());
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .expect("initial output events");
        let started = thread_cpu_time();
        tokio::time::sleep(Duration::from_millis(300)).await;
        let elapsed = thread_cpu_time() - started;
        listener.abort();
        assert!(
            listener
                .await
                .expect_err("listener cancelled")
                .is_cancelled()
        );
        assert!(
            elapsed < Duration::from_millis(50),
            "idle CPU time: {elapsed:?}"
        );
    }

    #[test]
    fn focused_monitor_supplies_workspace_and_empty_tags() {
        let backend = Mango::new();
        backend
            .monitors
            .store(Arc::new(vec![monitor(false, 1), monitor(true, 3)]));
        assert_eq!(backend.get_workspace(), Mango::workspace(3));
        assert_eq!(
            backend.get_workspaces(),
            vec![Mango::workspace(1).unwrap(), Mango::workspace(3).unwrap()]
        );
    }

    #[test]
    fn missing_active_monitor_has_no_workspace() {
        let backend = Mango::new();
        assert!(backend.get_workspace().is_none());
        backend.monitors.store(Arc::new(vec![monitor(false, 1)]));
        assert!(backend.get_workspace().is_none());
        assert!(backend.get_workspaces().is_empty());
    }

    #[test]
    fn multiple_active_tags_choose_first_and_indices_stay_numeric() {
        let backend = Mango::new();
        let mut output = monitor(true, 1);
        output.tags[0].is_active = true;
        output.active_tags = vec![3, 1];
        backend.monitors.store(Arc::new(vec![output]));
        assert_eq!(backend.get_workspace(), Mango::workspace(3));
        assert_eq!(
            dispatch::View::new(request::TagMaskOrAllOrPrev::TagMask(
                request::Tag::new(3).unwrap().into()
            ))
            .mask
            .to_string(),
            "3"
        );
    }

    #[tokio::test]
    async fn logout_uses_native_quit_and_reports_rejected_requests() {
        assert!(Mango::default().logout().await.is_err());
        let path = std::env::temp_dir().join(format!(
            "capsule-mango-logout-test-{}.sock",
            std::process::id()
        ));
        let listener = tokio::net::UnixListener::bind(&path).expect("mock socket");
        let backend = Mango {
            socket: Some(Socket::new(path.clone())),
            ..Mango::default()
        };
        let server = tokio::spawn(async move {
            for success in [false, true] {
                let (stream, _) = listener.accept().await.expect("mock connection");
                let mut stream = BufReader::new(stream);
                let mut request = String::new();
                stream.read_line(&mut request).await.expect("quit request");
                assert_eq!(request.trim(), "dispatch quit");
                let response = serde_json::json!({"success": success});
                stream
                    .get_mut()
                    .write_all(format!("{response}\n").as_bytes())
                    .await
                    .expect("mock response");
            }
        });
        assert!(backend.logout().await.is_err());
        assert!(backend.logout().await.is_ok());
        server.await.expect("mock server");
        std::fs::remove_file(path).expect("mock socket cleanup");
    }

    #[tokio::test]
    async fn socket_queries_dispatches_and_workspace_events() {
        let path =
            std::env::temp_dir().join(format!("capsule-mango-test-{}.sock", std::process::id()));
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let backend = Mango {
            socket: Some(Socket::new(path.clone())),
            ..Mango::default()
        };
        let (dispatch_tx, dispatch_rx) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let mut dispatch_tx = Some(dispatch_tx);
            for expected in ["get all-monitors", "dispatch view,3", "watch all-monitors"] {
                let (stream, _) = listener.accept().await.unwrap();
                let mut stream = BufReader::new(stream);
                let mut request = String::new();
                stream.read_line(&mut request).await.unwrap();
                assert_eq!(request.trim(), expected);
                let response = if expected == "dispatch view,3" {
                    serde_json::json!({"success": true})
                } else {
                    serde_json::json!({"monitors": [monitor(true, 3)]})
                };
                stream
                    .get_mut()
                    .write_all(format!("{response}\n").as_bytes())
                    .await
                    .unwrap();
                if expected == "dispatch view,3" {
                    dispatch_tx.take().unwrap().send(()).unwrap();
                }
            }
        });
        backend.refresh().await;
        assert_eq!(backend.get_workspace(), Mango::workspace(3));
        backend.switch_workspace(3);
        tokio::time::timeout(Duration::from_secs(5), dispatch_rx)
            .await
            .unwrap()
            .unwrap();
        let current = Arc::new(ArcSwap::from_pointee(WorkspaceInfo::default()));
        let (tx, mut rx) = broadcast::channel(4);
        let events = tokio::spawn(run_events_listener(backend, current.clone(), tx));
        let workspace = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(Some(workspace), Mango::workspace(3));
        assert_eq!(**current.load(), Mango::workspace(3).unwrap());
        server.await.unwrap();
        events.abort();
        std::fs::remove_file(path).unwrap();
    }
}
