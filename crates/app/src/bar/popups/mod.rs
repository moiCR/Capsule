pub mod animator;
pub mod launcher;
pub mod polkit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupEvent {
    Open,
    Close,
    SyncWindow,
    RequestAnimationFrame,
}
