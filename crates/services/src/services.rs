pub mod app_state;
pub mod compositor;
pub mod ipc;
pub mod launcher;
pub mod logger;
pub mod lyrics;
pub mod mpris;
pub mod notifications;
pub mod polkit;
pub mod system;

pub use app_state::AppState;
pub use compositor::CompositorService;
pub use ipc::{
    IpcCommand, IpcMessage, IpcSubscriber, decode_command, pop_ipc_command, push_ipc_command,
};
pub use launcher::{Application, LauncherService};
pub use logger::init_logger;
pub use lyrics::{LyricLine, LyricsService, TrackLyrics};
pub use mpris::{MediaTrack, MprisService};
pub use notifications::{NotificationItem, NotificationStore, start_notification_server};
pub use polkit::{PolkitAuthRequest, authenticate_user, start_polkit_agent};
pub use system::{SystemService, SystemStatus};
