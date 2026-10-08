use services::{ClipboardItem, ClipboardService, Snippet, clipboard::ClipboardContent};
use std::collections::HashMap;

pub(super) enum Request {
    Active(bool),
    Refresh,
    Copy(ClipboardItem, u64),
    CopyText(String, u64),
    Pin(ClipboardItem),
    Remove(String),
    Clear,
}
pub(super) struct Reply {
    pub items: Vec<ClipboardItem>,
    pub snippets: Vec<Snippet>,
    pub images: HashMap<String, std::sync::Arc<gpui::Image>>,
    pub pinned: Vec<String>,
    pub copied: Option<(u64, ClipboardContent)>,
    pub error: Option<String>,
    pub finished: bool,
}

pub(super) fn start(
    service: ClipboardService,
) -> (
    tokio::sync::mpsc::UnboundedSender<Request>,
    tokio::sync::mpsc::UnboundedReceiver<Reply>,
    tokio::task::JoinHandle<()>,
) {
    let (sender, mut requests) = tokio::sync::mpsc::unbounded_channel();
    let (results, receiver) = tokio::sync::mpsc::unbounded_channel();
    let worker = services::spawn_tokio(async move {
        let mut active = false;
        let mut decoded = HashMap::<String, String>::new();
        let mut images = HashMap::<String, std::sync::Arc<gpui::Image>>::new();
        let mut refresh = tokio::time::interval(std::time::Duration::from_secs(2));
        refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let request = tokio::select! {
                request = requests.recv() => { let Some(request) = request else { break; }; request },
                _ = refresh.tick(), if active => Request::Refresh,
            };
            if let Request::Active(value) = request {
                active = value;
                continue;
            }
            let service = service.clone();
            let state = decoded.clone();
            let cached_images = images.clone();
            let result = tokio::task::spawn_blocking(move || {
                let (mut reply, state) = process(&service, request, state);
                let mut images = cached_images;
                images.retain(|id, _| reply.items.iter().any(|item| &item.id == id));
                for item in &reply.items {
                    if item.is_image
                        && !images.contains_key(&item.id)
                        && let Some(path) = &item.image_path
                        && let Ok(bytes) = std::fs::read(path)
                    {
                        images.insert(
                            item.id.clone(),
                            std::sync::Arc::new(gpui::Image::from_bytes(
                                gpui::ImageFormat::Png,
                                bytes,
                            )),
                        );
                    }
                }
                reply.images = images.clone();
                (reply, state, images)
            })
            .await;
            let Ok((reply, state, cached_images)) = result else {
                let _ = results.send(Reply {
                    items: Vec::new(),
                    snippets: Vec::new(),
                    images: HashMap::new(),
                    pinned: Vec::new(),
                    copied: None,
                    error: Some("clipboard.action_error".into()),
                    finished: true,
                });
                continue;
            };
            decoded = state;
            images = cached_images;
            if results.send(reply).is_err() {
                break;
            }
        }
    });
    (sender, receiver, worker)
}

fn process(
    service: &ClipboardService,
    request: Request,
    mut decoded: HashMap<String, String>,
) -> (Reply, HashMap<String, String>) {
    let finished = !matches!(request, Request::Active(_) | Request::Refresh);
    let mut copied = None;
    let operation = match request {
        Request::Copy(item, generation) => service.read_item(&item).and_then(|content| {
            let success = match &content {
                ClipboardContent::Text(text) => {
                    decoded.insert(item.id.clone(), text.clone());
                    service.copy_text(text)
                }
                ClipboardContent::Image(bytes) => service.copy_binary(bytes, "image/png"),
            };
            if success {
                copied = Some((generation, content));
                Ok(())
            } else {
                Err("clipboard.copy_error".into())
            }
        }),
        Request::CopyText(text, generation) => {
            if service.copy_text(&text) {
                copied = Some((generation, ClipboardContent::Text(text)));
                Ok(())
            } else {
                Err("clipboard.copy_error".into())
            }
        }
        Request::Pin(item) => service.read_item(&item).and_then(|content| {
            let ClipboardContent::Text(text) = content else {
                return Err("clipboard.action_error".into());
            };
            decoded.insert(item.id, text.clone());
            if let Some(snippet) = service
                .get_snippets()
                .iter()
                .find(|snippet| snippet.content == text)
            {
                if service.remove_snippet(&snippet.id) {
                    Ok(())
                } else {
                    Err("clipboard.action_error".into())
                }
            } else {
                let title: String = text.lines().next().unwrap_or("").chars().take(32).collect();
                if service.add_snippet(&title, &text).is_some() {
                    Ok(())
                } else {
                    Err("clipboard.action_error".into())
                }
            }
        }),
        Request::Remove(id) => {
            if service.remove_snippet(&id) {
                Ok(())
            } else {
                Err("clipboard.action_error".into())
            }
        }
        Request::Clear => {
            if service.clear_history() {
                decoded.clear();
                Ok(())
            } else {
                Err("clipboard.action_error".into())
            }
        }
        Request::Active(_) | Request::Refresh => Ok(()),
    };
    let items = service.fetch_history();
    let snippets = service.get_snippets();
    decoded.retain(|id, _| items.iter().any(|item| &item.id == id));
    let pinned = items
        .iter()
        .filter(|item| {
            !item.is_image
                && snippets.iter().any(|snippet| {
                    snippet.content == *decoded.get(&item.id).unwrap_or(&item.preview)
                })
        })
        .map(|item| item.id.clone())
        .collect();
    (
        Reply {
            items,
            snippets,
            images: HashMap::new(),
            pinned,
            copied,
            error: operation.err(),
            finished,
        },
        decoded,
    )
}
