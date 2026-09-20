use anyhow::{Context, Result};
use arc_swap::ArcSwap;
use serde_json::{Map, Value};
use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CaptureStatus {
    pub camera_in_use: bool,
    pub microphone_in_use: bool,
}

#[derive(Default)]
struct CaptureObject {
    is_node: bool,
    running: bool,
    props: Map<String, Value>,
}

#[derive(Default)]
struct CaptureGraph {
    objects: HashMap<u64, CaptureObject>,
}

impl CaptureGraph {
    fn update(&mut self, changes: Vec<Value>) -> CaptureStatus {
        for change in changes {
            let Some(id) = change["id"].as_u64() else {
                continue;
            };
            if change.get("info").is_some_and(Value::is_null) {
                self.objects.remove(&id);
                continue;
            }
            let kind = change["type"].as_str();
            if !matches!(
                kind,
                Some("PipeWire:Interface:Node" | "PipeWire:Interface:Device")
            ) && !self.objects.contains_key(&id)
            {
                continue;
            }
            let object = self.objects.entry(id).or_default();
            if let Some(kind) = kind {
                object.is_node = kind == "PipeWire:Interface:Node";
            }
            if let Some(state) = change["info"]["state"].as_str() {
                object.running = state == "running";
            }
            if let Some(props) = change["info"]["props"].as_object() {
                object.props = props.clone();
            }
        }

        let mut status = CaptureStatus::default();
        for object in self
            .objects
            .values()
            .filter(|object| object.is_node && object.running)
        {
            let props = &object.props;
            let media_class = props.get("media.class").and_then(Value::as_str);
            if media_class == Some("Audio/Source")
                && !props
                    .get("stream.monitor")
                    .is_some_and(|value| value == true || value == "true")
                && !props
                    .get("node.name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.ends_with(".monitor"))
            {
                status.microphone_in_use = true;
            }
            if media_class != Some("Video/Source") {
                continue;
            }
            let device = props
                .get("device.id")
                .and_then(|id| {
                    id.as_u64()
                        .or_else(|| id.as_str().and_then(|id| id.parse().ok()))
                })
                .and_then(|id| self.objects.get(&id));
            let device_api = props
                .get("device.api")
                .or_else(|| device.and_then(|device| device.props.get("device.api")))
                .and_then(Value::as_str);
            let factory = props
                .get("factory.name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            status.camera_in_use |= matches!(device_api, Some("v4l2" | "libcamera"))
                || factory.starts_with("api.v4l2.")
                || factory.starts_with("api.libcamera.")
                || props.get("media.role").and_then(Value::as_str) == Some("Camera");
        }
        status
    }
}

pub(super) async fn monitor(status: Arc<ArcSwap<CaptureStatus>>) {
    loop {
        let _ = monitor_connection(&status).await;
        status.store(Arc::new(CaptureStatus::default()));
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn monitor_connection(status: &ArcSwap<CaptureStatus>) -> Result<()> {
    let mut child = Command::new("pw-dump")
        .args(["--monitor", "--no-colors", "--raw"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Failed to monitor PipeWire capture devices")?;
    let stdout = child
        .stdout
        .take()
        .context("Missing PipeWire monitor output")?;
    let mut lines = BufReader::new(stdout).lines();
    let mut graph = CaptureGraph::default();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let changes = serde_json::from_str(&line)?;
        let next = graph.update(changes);
        if **status.load() != next {
            status.store(Arc::new(next));
        }
    }
    child.wait().await?;
    Ok(())
}
