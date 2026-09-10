use crate::{
    CalendarService, ClipboardService, CompositorService, ConfigService, IdleService, LangService,
    LauncherService, LyricsService, MprisService, NetworkService, PolkitService, PowerService,
    RecordService, SniHostService, SystemService, wallpaper::WallpaperService,
};

#[derive(Clone)]
pub struct AppState {
    pub config: ConfigService,
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
    pub language: LangService,
    pub record: RecordService,
}

impl gpui::Global for AppState {}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let config = ConfigService::new();
        let sni_host = SniHostService::new();
        sni_host.start();

        let compositor = CompositorService::new();
        let wallpaper = WallpaperService::new(compositor.clone());
        let network = NetworkService::new();
        let calendar = CalendarService::new();
        let power = PowerService::new();
        let clipboard = ClipboardService::new();
        let idle = IdleService::new(config.clone());
        let language = LangService::new(&config.get().ui.language);
        let record = RecordService::new();

        Self {
            config: config.clone(),
            launcher: LauncherService::new(),
            mpris: MprisService::new(config),
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
            language,
            record,
        }
    }
}
