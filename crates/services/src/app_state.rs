use crate::{
    CalendarService, ClipboardService, CompositorService, IdleService, LauncherService,
    LyricsService, MprisService, NetworkService, PolkitService, PowerService, SniHostService,
    SystemService, wallpaper::WallpaperService,
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
    pub network: NetworkService,
    pub calendar: CalendarService,
    pub power: PowerService,
    pub clipboard: ClipboardService,
    pub idle: IdleService,
}

impl gpui::Global for AppState {}

impl AppState {
    pub fn new() -> Self {
        let sni_host = SniHostService::new();
        sni_host.start();

        let compositor = CompositorService::new();
        let wallpaper = WallpaperService::new(compositor.clone());
        let network = NetworkService::new();
        let calendar = CalendarService::new();
        let power = PowerService::new();
        let clipboard = ClipboardService::new();
        let idle = IdleService::new();

        Self {
            launcher: LauncherService::new(),
            mpris: MprisService::new(),
            system: SystemService::new(),
            lyrics: LyricsService::new(),
            compositor,
            polkit: PolkitService::new(),
            sni_host,
            wallpaper,
            network,
            calendar,
            power,
            clipboard,
            idle,
        }
    }
}
