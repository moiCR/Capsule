use crate::{
    CompositorService, LauncherService, LyricsService, MprisService, PolkitService, SniHostService, SystemService, wallpaper::WallpaperService,
};

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
    pub polkit: PolkitService,
    pub sni_host: SniHostService,
    pub wallpaper: WallpaperService,
}

impl gpui::Global for AppState {}

impl AppState {
    pub fn new() -> Self {
        let sni_host = SniHostService::new();
        sni_host.start();

        let compositor = CompositorService::new();
        let wallpaper = WallpaperService::new(compositor.clone());

        Self {
            launcher: LauncherService::new(),
            mpris: MprisService::new(),
            system: SystemService::new(),
            lyrics: LyricsService::new(),
            compositor,
            polkit: PolkitService::new(),
            sni_host,
            wallpaper,
        }
    }
}
