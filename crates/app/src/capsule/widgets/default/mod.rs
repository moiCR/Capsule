pub mod clock;
pub mod flip_clock;
pub mod media;
pub mod privacy_dock;
pub mod status_dock;
pub mod workspaces;

pub use clock::render_clock_widget;
pub use flip_clock::FlipClock;
pub use media::render_media_dock;
pub use privacy_dock::render_privacy_dock;
pub use status_dock::render_status_dock;
pub use workspaces::render_workspaces_widget;
