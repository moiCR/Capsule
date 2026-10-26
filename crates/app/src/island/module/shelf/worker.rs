use services::{ShelfItem, ShelfService};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

pub(super) enum Request {
    Refresh,
    Add(Vec<PathBuf>),
    Remove(String),
    Clear,
    Copy(Vec<PathBuf>, u64),
}

pub(super) struct Reply {
    pub items: Vec<ShelfItem>,
    pub previews: HashMap<String, Arc<gpui::Image>>,
    pub copied: Option<u64>,
    pub error: bool,
    pub finished: bool,
}

pub(super) fn start(
    service: ShelfService,
) -> (
    tokio::sync::mpsc::UnboundedSender<Request>,
    tokio::sync::mpsc::UnboundedReceiver<Reply>,
    tokio::task::JoinHandle<()>,
) {
    let (sender, mut requests) = tokio::sync::mpsc::unbounded_channel();
    let (results, receiver) = tokio::sync::mpsc::unbounded_channel();
    let worker = services::spawn_tokio(async move {
        let mut previews = HashMap::<String, Arc<gpui::Image>>::new();
        let mut changes = service.subscribe_changes();
        loop {
            let request = tokio::select! {
                request = requests.recv() => { let Some(request) = request else { break; }; request },
                change = changes.recv() => {
                    if matches!(change, Err(tokio::sync::broadcast::error::RecvError::Closed)) {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        changes = service.subscribe_changes();
                    }
                    Request::Refresh
                },
            };
            let operation_service = service.clone();
            let mut cached_previews = previews.clone();
            let result = tokio::task::spawn_blocking(move || {
                let finished = !matches!(request, Request::Refresh);
                let mut copied = None;
                let mut error = false;
                match request {
                    Request::Add(paths) => {
                        operation_service.add_paths(&paths);
                    }
                    Request::Remove(id) => {
                        operation_service.remove_item(&id);
                    }
                    Request::Clear => operation_service.clear(),
                    Request::Copy(paths, generation) => {
                        if operation_service.copy_paths(&paths) {
                            copied = Some(generation);
                        } else {
                            error = true;
                        }
                    }
                    Request::Refresh => {}
                }
                let items = operation_service.get_items();
                cached_previews.retain(|id, _| items.iter().any(|item| &item.id == id));
                for item in &items {
                    if item.is_image
                        && !cached_previews.contains_key(&item.id)
                        && let Some(image) = preview(item)
                    {
                        cached_previews.insert(item.id.clone(), image);
                    }
                }
                Reply {
                    items,
                    previews: cached_previews,
                    copied,
                    error,
                    finished,
                }
            })
            .await;
            let reply = result.unwrap_or_else(|_| Reply {
                items: service.get_items(),
                previews: previews.clone(),
                copied: None,
                error: true,
                finished: true,
            });
            previews = reply.previews.clone();
            if results.send(reply).is_err() {
                break;
            }
        }
    });
    (sender, receiver, worker)
}

fn preview(item: &ShelfItem) -> Option<Arc<gpui::Image>> {
    let image = image::ImageReader::open(&item.path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .thumbnail(168, 120);
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Png,
        png.into_inner(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shelf_worker_is_silent_between_events() {
        services::tokio_handle().block_on(async {
            let (sender, mut replies, worker) = start(ShelfService::new());
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(100), replies.recv())
                    .await
                    .is_err()
            );
            sender.send(Request::Refresh).expect("refresh");
            let reply = tokio::time::timeout(std::time::Duration::from_secs(2), replies.recv())
                .await
                .expect("reply timeout")
                .expect("reply");
            assert!(!reply.finished);
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(100), replies.recv())
                    .await
                    .is_err()
            );
            worker.abort();
        });
    }

    #[test]
    fn dropped_paths_are_deduplicated_and_removal_is_reported() {
        services::tokio_handle().block_on(async {
            let root = std::env::temp_dir()
                .join(format!("capsule-new-shelf-worker-{}", std::process::id()));
            let first = root.join("first/shared.txt");
            let second = root.join("second/shared.txt");
            std::fs::create_dir_all(first.parent().expect("first parent")).expect("first folder");
            std::fs::create_dir_all(second.parent().expect("second parent"))
                .expect("second folder");
            std::fs::write(&first, "first").expect("first file");
            std::fs::write(&second, "second").expect("second file");
            let service = ShelfService::new();
            let (sender, mut replies, worker) = start(service);
            sender
                .send(Request::Add(vec![
                    first.clone(),
                    first.clone(),
                    second.clone(),
                ]))
                .expect("add request");
            let added = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    let reply = replies.recv().await.expect("worker reply");
                    if reply.finished {
                        break reply;
                    }
                }
            })
            .await
            .expect("add reply");
            assert_eq!(added.items.len(), 2);
            assert_ne!(added.items[0].id, added.items[1].id);
            assert!(!added.error);
            sender
                .send(Request::Remove(added.items[0].id.clone()))
                .expect("remove request");
            let removed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                loop {
                    let reply = replies.recv().await.expect("worker reply");
                    if reply.finished {
                        break reply;
                    }
                }
            })
            .await
            .expect("remove reply");
            assert_eq!(removed.items.len(), 1);
            assert_eq!(removed.items[0].path, second);
            assert!(!removed.error);
            worker.abort();
            std::fs::remove_dir_all(root).expect("remove fixtures");
        });
    }
}
