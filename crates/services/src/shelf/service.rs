use super::item::ShelfItem;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct ShelfService {
    items: Arc<Mutex<Vec<ShelfItem>>>,
    change_tx: broadcast::Sender<usize>,
}

impl Default for ShelfService {
    fn default() -> Self {
        Self::new()
    }
}

impl ShelfService {
    pub fn new() -> Self {
        let (change_tx, _) = broadcast::channel(16);
        Self {
            items: Arc::new(Mutex::new(Vec::new())),
            change_tx,
        }
    }

    pub fn subscribe_changes(&self) -> broadcast::Receiver<usize> {
        self.change_tx.subscribe()
    }

    pub fn add_paths(&self, paths: &[PathBuf]) -> usize {
        let mut guard = match self.items.lock() {
            Ok(guard) => guard,
            Err(_) => return 0,
        };

        let mut added = 0;
        for path in paths {
            if guard.iter().any(|item| item.path == *path) {
                continue;
            }

            if let Some(item) = ShelfItem::from_path(path.clone()) {
                guard.push(item);
                added += 1;
            }
        }

        if added > 0 {
            let count = guard.len();
            drop(guard);
            let _ = self.change_tx.send(count);
        }

        added
    }

    pub fn get_items(&self) -> Vec<ShelfItem> {
        let mut guard = match self.items.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        guard.retain(|item| item.path.exists());
        guard.clone()
    }

    pub fn count(&self) -> usize {
        let mut guard = match self.items.lock() {
            Ok(guard) => guard,
            Err(_) => return 0,
        };
        guard.retain(|item| item.path.exists());
        guard.len()
    }

    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    pub fn remove_item(&self, id: &str) -> bool {
        let mut guard = match self.items.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        let previous_len = guard.len();
        guard.retain(|item| item.id != id);
        let changed = guard.len() != previous_len;
        if changed {
            let count = guard.len();
            drop(guard);
            let _ = self.change_tx.send(count);
        }
        changed
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.items.lock() {
            guard.clear();
            drop(guard);
            let _ = self.change_tx.send(0);
        }
    }

    pub fn copy_item(&self, item: &ShelfItem) -> bool {
        copy_paths_to_clipboard(std::slice::from_ref(&item.path))
    }

    pub fn copy_all(&self) -> bool {
        let items = self.get_items();
        let paths: Vec<PathBuf> = items.into_iter().map(|item| item.path).collect();
        copy_paths_to_clipboard(&paths)
    }
}

fn copy_paths_to_clipboard(paths: &[PathBuf]) -> bool {
    if paths.is_empty() {
        return false;
    }

    let _ = Command::new("pkill")
        .args(["-f", "capsule_shelf_clipboard_helper"])
        .status();

    let paths_args: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let py_script = r#"
import sys
try:
    from PyQt6.QtGui import QGuiApplication, QImage
    from PyQt6.QtCore import QMimeData, QUrl, QByteArray, QTimer
    app = QGuiApplication(sys.argv[:1])
    clipboard = app.clipboard()
    mime = QMimeData()
    paths = sys.argv[2:]
    urls = [QUrl.fromLocalFile(p) for p in paths]
    mime.setUrls(urls)
    nl = chr(10)
    gnome_lines = ["copy"]
    for u in urls:
        gnome_lines.append(u.toEncoded().data().decode("utf-8"))
    gnome_data = nl.join(gnome_lines) + nl
    mime.setData("x-special/gnome-copied-files", QByteArray(gnome_data.encode("utf-8")))
    mime.setText(nl.join(paths))
    if len(paths) == 1:
        img = QImage(paths[0])
        if not img.isNull():
            mime.setImageData(img)
    clipboard.setMimeData(mime)
    def setup_watch():
        def on_dc():
            if not clipboard.ownsClipboard():
                sys.exit(0)
        clipboard.dataChanged.connect(on_dc)
    QTimer.singleShot(500, setup_watch)
    app.exec()
except Exception:
    sys.exit(1)
"#;

    let py_status = Command::new("python3")
        .arg("-c")
        .arg(py_script)
        .arg("capsule_shelf_clipboard_helper")
        .args(&paths_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if py_status.is_err() {
        let uris: Vec<String> = paths
            .iter()
            .map(|item| format!("file://{}", item.to_string_lossy()))
            .collect();
        let joined_uris = uris.join("\r\n");
        let _ = Command::new("wl-copy")
            .args(["-t", "text/uri-list", &joined_uris])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shelf_service_add_and_remove() {
        let service = ShelfService::new();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("capsule_shelf_test.txt");
        let _ = std::fs::write(&file_path, "hello");

        let added = service.add_paths(&[file_path.clone()]);
        assert_eq!(added, 1);
        assert_eq!(service.count(), 1);

        let items = service.get_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, file_path);

        let added_again = service.add_paths(&[file_path.clone()]);
        assert_eq!(added_again, 0);

        let removed = service.remove_item(&items[0].id);
        assert!(removed);
        assert_eq!(service.count(), 0);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_shelf_service_copy() {
        let service = ShelfService::new();
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("capsule_shelf_copy_test.txt");
        let _ = std::fs::write(&file_path, "copy test");

        service.add_paths(&[file_path.clone()]);
        let items = service.get_items();
        assert_eq!(items.len(), 1);

        let copied = service.copy_item(&items[0]);
        assert!(copied);

        let copied_all = service.copy_all();
        assert!(copied_all);

        let _ = std::fs::remove_file(file_path);
    }
}
