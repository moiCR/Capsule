pub mod clock;
pub mod flip_clock;
pub mod media;
pub mod status_dock;
pub mod workspaces;

pub use clock::render_clock_widget;
pub use flip_clock::FlipClock;
pub use status_dock::render_status_dock;
pub use workspaces::render_workspaces_widget;
