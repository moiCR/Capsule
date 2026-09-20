pub mod app_state;
pub mod calendar;
pub mod clipboard;
pub mod compositor;
pub mod config;
pub mod dbus_util;
pub mod emoji;
pub mod idle;
pub mod ipc;
pub mod language;
pub mod launcher;
pub mod logger;
pub mod lyrics;
pub mod mpris;
pub mod network;
pub mod notifications;
pub mod pam;
pub mod polkit;
pub mod power;
pub mod record;
pub mod shelf;
pub mod system;
pub mod tray;
pub mod wallpaper;

pub use calendar::CalendarService;

pub use network::{BluetoothDeviceItem, NetworkService, NetworkStatus, WifiAccessPoint};
pub use shelf::{ShelfItem, ShelfService};

pub use app_state::AppState;
pub use clipboard::{ClipboardItem, ClipboardService, Snippet};
pub use compositor::{CompositorService, WorkspaceInfo};
pub use config::{
    AppConfig, CapsuleStyle, ConfigService, Defaults, LockScreenConfig, LockscreenConfig,
    MprisConfig, UIConfig, UiConfig,
};
pub use emoji::{EmojiItem, EmojiService};
pub use idle::{IdleEvent, IdleService};
pub use ipc::{
    IpcCommand, IpcMessage, IpcSubscriber, decode_command, pop_ipc_command, push_ipc_command,
};
pub use language::{LangService, LanguageInfo};
pub use launcher::{Application, LauncherService};
pub use logger::init_logger;
pub use lyrics::{LyricLine, LyricsService, TrackLyrics};
pub use mpris::{MediaTrack, MprisService};
pub use notifications::{NotificationItem, NotificationStore, start_notification_server};
pub use pam::PamService;
pub use polkit::{
    PolkitAuthRequest, PolkitService, authenticate_user, pop_polkit_request, push_polkit_request,
    start_polkit_agent,
};
pub use power::{BatteryStatus, PowerProfile, PowerService};
pub use record::{
    RecordAudio, RecordBackend, RecordOptions, RecordService, RecordStatus, RecordTarget,
};
pub use system::{AudioSink, AudioSource, SystemService, SystemStatus};
pub use tray::{SniHostService, SniItem, TrayAction, TrayService};

use std::sync::OnceLock;

static TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

/// Initializes the global Tokio handle.
pub fn init_tokio_handle(handle: tokio::runtime::Handle) {
    let _ = TOKIO_HANDLE.set(handle);
}

/// Returns a handle to the Tokio runtime.
///
/// If `init_tokio_handle` was called, returns that handle.
/// If called from within an active Tokio runtime context, captures and returns that handle.
/// Otherwise, lazily initializes a multi-threaded Tokio runtime and returns its handle.
pub fn tokio_handle() -> tokio::runtime::Handle {
    if let Some(handle) = TOKIO_HANDLE.get() {
        return handle.clone();
    }

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let _ = TOKIO_HANDLE.set(handle.clone());
        return handle;
    }

    static FALLBACK_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    let rt = FALLBACK_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("capsule-tokio")
            .build()
            .expect("Failed to create fallback Tokio runtime")
    });
    let handle = rt.handle().clone();
    let _ = TOKIO_HANDLE.set(handle.clone());
    handle
}

/// Spawns a future on the global Tokio runtime.
pub fn spawn_tokio<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio_handle().spawn(future)
}

/// Spawns a blocking task on the global Tokio runtime.
pub fn spawn_blocking<F, R>(task: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio_handle().spawn_blocking(task)
}
