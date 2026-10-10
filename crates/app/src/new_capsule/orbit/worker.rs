use super::Snapshot;
use services::{RecordService, ShelfService};
use std::time::Duration;
use tokio::sync::{broadcast, watch};

pub(super) fn start(
    shelf: ShelfService,
    record: RecordService,
) -> (watch::Receiver<Snapshot>, tokio::task::JoinHandle<()>) {
    let (sender, receiver) = watch::channel(Snapshot::default());
    let worker = services::spawn_tokio(async move {
        let mut shelf_changes = shelf.subscribe_changes();
        let mut record_changes = record.subscribe_status();
        let mut fallback = tokio::time::interval(Duration::from_secs(2));
        fallback.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            let snapshot = Snapshot {
                shelf_count: shelf.count(),
                record_status: record.get_status(),
            };
            sender.send_if_modified(|previous| {
                if *previous == snapshot {
                    return false;
                }
                *previous = snapshot;
                true
            });
            tokio::select! {
                _ = sender.closed() => break,
                change = shelf_changes.recv() => {
                    if matches!(change, Err(broadcast::error::RecvError::Closed)) {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        shelf_changes = shelf.subscribe_changes();
                    }
                },
                change = record_changes.recv() => {
                    if matches!(change, Err(broadcast::error::RecvError::Closed)) {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        record_changes = record.subscribe_status();
                    }
                },
                _ = fallback.tick() => {},
            }
        }
    });
    (receiver, worker)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_shelf_items_and_removals_update_without_idle_notifications() {
        services::tokio_handle().block_on(async {
            let shelf = ShelfService::new();
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
            assert_eq!(shelf.add_paths(&[path]), 1);
            let (mut updates, worker) = start(shelf.clone(), RecordService::new());
            tokio::time::timeout(Duration::from_secs(2), updates.changed())
                .await
                .expect("initial snapshot")
                .expect("worker open");
            assert_eq!(updates.borrow_and_update().shelf_count, 1);
            shelf.clear();
            tokio::time::timeout(Duration::from_secs(2), updates.changed())
                .await
                .expect("shelf removal")
                .expect("worker open");
            assert_eq!(updates.borrow_and_update().shelf_count, 0);
            assert!(
                tokio::time::timeout(Duration::from_millis(2200), updates.changed())
                    .await
                    .is_err()
            );
            drop(updates);
            tokio::time::timeout(Duration::from_secs(2), worker)
                .await
                .expect("worker stops when receiver drops")
                .expect("worker completes");
        });
    }
}
