use super::PendingAction;
use services::{RecordOptions, RecordService, RecordStatus, config::RecordConfig};

pub(super) enum Request {
    Active(bool),
    Refresh,
    Start(RecordConfig),
    Pause,
    Resume,
    Stop(u64),
}
pub(super) struct Reply {
    pub status: RecordStatus,
    pub seconds: u64,
    pub error: Option<String>,
    pub finished: Option<PendingAction>,
    pub stopped: Option<u64>,
}

pub(super) fn start(
    service: RecordService,
) -> (
    tokio::sync::mpsc::UnboundedSender<Request>,
    tokio::sync::mpsc::UnboundedReceiver<Reply>,
    tokio::task::JoinHandle<()>,
) {
    let (sender, mut requests) = tokio::sync::mpsc::unbounded_channel();
    let (results, receiver) = tokio::sync::mpsc::unbounded_channel();
    let worker = services::spawn_tokio(async move {
        let mut changes = service.subscribe_status();
        let mut active = false;
        let mut last = None;
        let mut clock = tokio::time::interval(std::time::Duration::from_millis(500));
        let mut fallback = tokio::time::interval(std::time::Duration::from_secs(2));
        clock.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        fallback.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let request = tokio::select! {
                request = requests.recv() => { let Some(request) = request else { break; }; request },
                change = changes.recv() => {
                    if matches!(change, Err(tokio::sync::broadcast::error::RecvError::Closed)) {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        changes = service.subscribe_status();
                    }
                    Request::Refresh
                },
                _ = clock.tick(), if active && service.is_recording() => Request::Refresh,
                _ = fallback.tick(), if active => Request::Refresh,
            };
            if let Request::Active(value) = request {
                active = value;
                continue;
            }
            let finished = match request {
                Request::Start(_) => Some(PendingAction::Start),
                Request::Pause => Some(PendingAction::Pause),
                Request::Resume => Some(PendingAction::Resume),
                Request::Stop(_) => Some(PendingAction::Stop),
                Request::Refresh | Request::Active(_) => None,
            };
            let stopping = if let Request::Stop(generation) = &request {
                Some(*generation)
            } else {
                None
            };
            let operation = match request {
                Request::Start(config) => {
                    match tokio::task::spawn_blocking(move || RecordOptions::from_config(&config))
                        .await
                    {
                        Ok(options) => service.start(options).await.map(|_| ()),
                        Err(error) => Err(error.to_string()),
                    }
                }
                Request::Pause => service.pause().await,
                Request::Resume => service.resume().await,
                Request::Stop(_) if service.get_status() == RecordStatus::Stopped => Ok(()),
                Request::Stop(_) => service.stop().await.map(|_| ()),
                Request::Active(_) | Request::Refresh => Ok(()),
            };
            let status = service.get_status();
            let seconds = service.get_duration().await.as_secs();
            let snapshot = (status, seconds);
            if last.as_ref() != Some(&snapshot) || finished.is_some() {
                last = Some(snapshot);
                let stopped = stopping.filter(|_| operation.is_ok());
                if results
                    .send(Reply {
                        status,
                        seconds,
                        error: operation.err(),
                        finished,
                        stopped,
                    })
                    .is_err()
                {
                    break;
                }
            }
        }
    });
    (sender, receiver, worker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_pause_reports_error_without_requesting_close() {
        services::tokio_handle().block_on(async {
            let (sender, mut replies, worker) = start(RecordService::new());
            sender.send(Request::Pause).expect("pause request");
            let reply = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    let reply = replies.recv().await.expect("worker reply");
                    if reply.finished.is_some() {
                        break reply;
                    }
                }
            })
            .await
            .expect("stop reply");
            assert_eq!(reply.status, RecordStatus::Stopped);
            assert!(reply.error.is_some());
            assert!(reply.stopped.is_none());
            assert_eq!(reply.seconds, 0);
            worker.abort();
        });
    }
}
