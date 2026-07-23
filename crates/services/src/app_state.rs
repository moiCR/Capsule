use crate::{CompositorService, LauncherService, LyricsService, MprisService, SystemService};

/// Global application state holding all singleton services.
/// Initialize once in `main.rs` via `cx.set_global(AppState::new())`.
/// Read anywhere via `cx.global::<AppState>()`.
#[derive(Clone)]
pub struct AppState {
    pub launcher: LauncherService,
    pub mpris: MprisService,
    pub system: SystemService,
    pub lyrics: LyricsService,
    pub compositor: CompositorService,
}

impl gpui::Global for AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {
            launcher: LauncherService::new(),
            mpris: MprisService::new(),
            system: SystemService::new(),
            lyrics: LyricsService::new(),
            compositor: CompositorService::new(),
        }
    }
}
