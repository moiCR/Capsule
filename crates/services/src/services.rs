pub mod app_state;
pub mod calendar;
pub mod clipboard;
pub mod compositor;
pub mod dbus_util;
pub mod emoji;
pub mod ipc;
pub mod launcher;
pub mod logger;
pub mod lyrics;
pub mod mpris;
pub mod network;
pub mod notifications;
pub mod polkit;
pub mod power;
pub mod system;
pub mod tray;
pub mod wallpaper;

pub use calendar::CalendarService;

pub use network::{BluetoothDeviceItem, NetworkService, NetworkStatus, WifiAccessPoint};

pub use app_state::AppState;
pub use clipboard::{ClipboardItem, ClipboardService};
pub use compositor::CompositorService;
pub use emoji::{EmojiItem, EmojiService};
pub use ipc::{
    IpcCommand, IpcMessage, IpcSubscriber, decode_command, pop_ipc_command, push_ipc_command,
};
pub use launcher::{Application, LauncherService};
pub use logger::init_logger;
pub use lyrics::{LyricLine, LyricsService, TrackLyrics};
pub use mpris::{MediaTrack, MprisService};
pub use notifications::{NotificationItem, NotificationStore, start_notification_server};
pub use polkit::{
    PolkitAuthRequest, PolkitService, authenticate_user, pop_polkit_request, push_polkit_request,
    start_polkit_agent,
};
pub use power::{PowerProfile, PowerService};
pub use system::{SystemService, SystemStatus};
pub use tray::{SniHostService, SniItem, TrayAction, TrayService};
