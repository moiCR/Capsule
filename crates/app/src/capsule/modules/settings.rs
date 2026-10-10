use gpui::{
    AnyElement, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, ParentElement,
    Render, ScrollHandle, Styled, Task, Window, canvas, div, prelude::*, px,
};
use services::{AppState, CapsuleStyle};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::capsule::widgets::settings::{
    render_apps_section, render_lockscreen_formats_subsection, render_lockscreen_section,
    render_media_section, render_music_players_subsection, render_record_section,
    render_search_results, render_system_section,
};

use crate::new_capsule::animator::Animator;
use crate::new_capsule::widgets::settings::{
    HEIGHT, render_header, render_tabs, render_ui_section,
};
use crate::new_capsule::widgets::style;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    Appearance = 0,
    Apps = 1,
    Media = 2,
    LockScreen = 3,
    System = 4,
    Record = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsSubSection {
    MusicPlayers = 0,
    LockScreenFormats = 1,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsField {
    Terminal = 0,
    Browser = 1,
    Editor = 2,
    CapsuleRound = 3,
    SatelliteRound = 4,
    CardsRound = 5,
    MarginTop = 6,
    IdleHeight = 7,
    Gap = 8,
    AnimDuration = 9,
    IdleTimeout = 10,
    TimeFormat = 11,
    DateFormat = 12,
    Language = 13,
    MusicPlayer = 14,
    Search = 15,
    FileManager = 16,
    VolumeOutput = 17,
    VolumeInput = 18,
    DynamicColors = 19,
    DarkMode = 20,
    ClockFormat = 21,
    CapsuleStyle = 22,
    LegacyCapsule = 23,
}

pub enum SettingsEvent {
    Close,
}

pub type SliderDragState = (SettingsField, f32, f32, f32, Rc<Cell<(f32, f32)>>);

pub struct SettingsModule {
    pub appearance_error: Option<String>,
    pub motion_preview: Animator,
    motion_preview_generation: u64,
    pub active_tab: SettingsTab,
    pub active_field: Option<SettingsField>,
    pub capsule_style: CapsuleStyle,
    pub use_new_capsule: bool,

    pub terminal_input: String,
    pub browser_input: String,
    pub editor_input: String,
    pub file_manager_input: String,

    pub music_players: Vec<String>,
    pub music_player_input: String,

    pub capsule_round_input: String,
    pub satellite_round_input: String,
    pub cards_round_input: String,
    pub margin_top_input: String,
    pub idle_height_input: String,
    pub gap_input: String,
    pub anim_duration_input: String,

    pub idle_timeout_input: String,
    pub time_format_input: String,
    pub date_format_input: String,
    pub show_clock: bool,
    pub show_media_player: bool,
    pub open_dropdown: Option<SettingsField>,
    pub dropdown_anim_field: Option<SettingsField>,
    pub dropdown_anim_progress: f32,
    pub dropdown_anim_task: Option<Task<()>>,

    pub search_query: String,
    pub active_subsection: Option<SettingsSubSection>,
    section_animator: Animator,
    section_anim_generation: u64,
    window_handle: Option<gpui::AnyWindowHandle>,
    pub active_slider_drag: Option<SliderDragState>,

    pub settings_bounds: Rc<Cell<(f32, f32)>>,
    pub language_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub terminal_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub browser_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub editor_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub file_manager_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub show_output_devices: bool,
    pub show_input_devices: bool,
    pub output_slider_bounds: Rc<Cell<(f32, f32)>>,
    pub input_slider_bounds: Rc<Cell<(f32, f32)>>,

    pub record_output: String,
    pub record_available_monitors: Vec<String>,
    pub record_fps: u32,
    pub record_resolution: String,
    pub record_quality: String,
    pub record_container: String,
    pub record_audio: String,
    pub record_include_cursor: bool,

    focus_handle: FocusHandle,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<SettingsEvent> for SettingsModule {}

impl SettingsModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();

        let mut module = Self {
            appearance_error: None,
            motion_preview: Animator::new(gpui::size(px(72.0), px(22.0)), Duration::ZERO),
            motion_preview_generation: 0,
            active_tab: SettingsTab::Appearance,
            active_field: None,
            capsule_style: CapsuleStyle::Normal,
            use_new_capsule: false,
            terminal_input: String::new(),
            browser_input: String::new(),
            editor_input: String::new(),
            file_manager_input: String::new(),
            music_players: Vec::new(),
            music_player_input: String::new(),
            capsule_round_input: String::new(),
            satellite_round_input: String::new(),
            cards_round_input: String::new(),
            margin_top_input: String::new(),
            idle_height_input: String::new(),
            gap_input: String::new(),
            anim_duration_input: String::new(),
            idle_timeout_input: String::new(),
            time_format_input: String::new(),
            date_format_input: String::new(),
            show_clock: true,
            show_media_player: true,
            open_dropdown: None,
            dropdown_anim_field: None,
            dropdown_anim_progress: 1.0,
            dropdown_anim_task: None,
            search_query: String::new(),
            active_subsection: None,
            section_animator: Animator::new(gpui::size(px(1.0), px(1.0)), Duration::ZERO),
            section_anim_generation: 0,
            window_handle: None,
            active_slider_drag: None,
            settings_bounds: Rc::new(Cell::new((0.0, 560.0))),
            language_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            terminal_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            browser_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            editor_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            file_manager_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            show_output_devices: false,
            show_input_devices: false,
            output_slider_bounds: Rc::new(Cell::new((0.0, 0.0))),
            input_slider_bounds: Rc::new(Cell::new((0.0, 0.0))),
            record_output: "screen".to_string(),
            record_available_monitors: Vec::new(),
            record_fps: 60,
            record_resolution: "native".to_string(),
            record_quality: "very_high".to_string(),
            record_container: "mp4".to_string(),
            record_audio: "desktop".to_string(),
            record_include_cursor: true,
            focus_handle,
            scroll_handle,
        };

        if cx.has_global::<AppState>() {
            let audio_changed = cx.global::<AppState>().system.audio_changed();
            cx.spawn(async move |this, cx| {
                loop {
                    audio_changed.notified().await;
                    let _ = this.update(cx, |_view, cx| cx.notify());
                }
            })
            .detach();
        }

        module.reload_from_config(cx);
        module
    }

    pub fn reload_from_config(&mut self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

        let cfg = cx.global::<AppState>().config.get();

        self.capsule_style = cfg.ui.capsule_style;
        self.use_new_capsule = cfg.ui.use_new_capsule;
        self.terminal_input = cfg.defaults.terminal.clone();
        self.browser_input = cfg.defaults.browser.clone();
        self.editor_input = cfg.defaults.editor.clone();
        self.file_manager_input = cfg.defaults.file_manager.clone();

        self.music_players = cfg.mpris.players.clone();
        self.music_player_input.clear();

        self.capsule_round_input = format!("{:.0}", cfg.ui.capsule_round);
        self.satellite_round_input = format!("{:.0}", cfg.ui.satellite_round);
        self.cards_round_input = format!("{:.0}", cfg.ui.cards_round);
        self.margin_top_input = format!("{:.0}", cfg.ui.margin_top);
        self.idle_height_input = format!("{:.0}", cfg.ui.idle_height);
        self.gap_input = format!("{:.0}", cfg.ui.gap);
        self.anim_duration_input = format!("{:.0}", cfg.ui.animation_duration_ms);

        self.idle_timeout_input = format!("{}", cfg.lockscreen.idle_timeout);
        self.time_format_input = cfg.lockscreen.time_format.clone();
        self.date_format_input = cfg.lockscreen.date_format.clone();
        self.show_clock = cfg.lockscreen.show_clock;
        self.show_media_player = cfg.lockscreen.show_media_player;

        self.record_output = cfg.record.output.clone();
        self.record_available_monitors = services::RecordService::list_monitors();
        self.record_fps = cfg.record.fps;
        self.record_resolution = cfg.record.resolution.clone();
        self.record_quality = cfg.record.quality.clone();
        self.record_container = cfg.record.container.clone();
        self.record_audio = cfg.record.audio.clone();
        self.record_include_cursor = cfg.record.include_cursor;
    }

    pub fn add_music_player(&mut self, player: String, cx: &mut Context<Self>) {
        let clean = player.trim().to_string();
        if clean.is_empty() {
            return;
        }
        if !self
            .music_players
            .iter()
            .any(|p| p.eq_ignore_ascii_case(&clean))
        {
            self.music_players.push(clean);
            self.music_player_input.clear();
            self.save_to_app_config(cx);
            cx.notify();
        }
    }

    pub fn remove_music_player(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.music_players.len() {
            self.music_players.remove(index);
            self.save_to_app_config(cx);
            cx.notify();
        }
    }

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        let changed = self.active_tab != tab
            || self.active_subsection.is_some()
            || !self.search_query.is_empty();
        self.motion_preview_generation += 1;
        self.active_slider_drag = None;
        self.active_tab = tab;
        self.active_subsection = None;
        self.search_query.clear();
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.scroll_handle.scroll_to_item(0);
        if changed {
            self.start_section_transition(cx);
        }
        cx.notify();
    }

    pub fn open_subsection(&mut self, sub: SettingsSubSection, cx: &mut Context<Self>) {
        self.active_subsection = Some(sub);
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.scroll_handle.scroll_to_item(0);
        self.start_section_transition(cx);
        cx.notify();
    }

    pub fn close_subsection(&mut self, cx: &mut Context<Self>) {
        if self.active_subsection.is_some() {
            self.active_subsection = None;
            self.active_field = None;
            self.open_dropdown = None;
            self.dropdown_anim_field = None;
            self.dropdown_anim_task = None;
            self.scroll_handle.scroll_to_item(0);
            self.start_section_transition(cx);
            cx.notify();
        }
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) {
        self.close_subsection(cx);
    }

    pub fn can_navigate_back(&self) -> bool {
        self.active_subsection.is_some()
    }

    pub fn play_motion_preview(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.motion_preview_generation += 1;
        let duration = self.anim_duration_input.parse::<u64>().unwrap_or(250);
        self.motion_preview
            .set_duration(Duration::from_millis(duration));
        if duration == 0 {
            self.motion_preview
                .transition_to(gpui::size(px(72.0), px(22.0)), Instant::now());
            self.motion_preview.advance(Instant::now());
            cx.notify();
            return;
        }
        self.motion_preview
            .transition_to(gpui::size(px(152.0), px(42.0)), Instant::now());
        Self::queue_preview_frame(
            window,
            cx.entity().downgrade(),
            self.motion_preview_generation,
            false,
        );
        cx.notify();
    }

    fn queue_preview_frame(
        window: &Window,
        entity: gpui::WeakEntity<Self>,
        generation: u64,
        returning: bool,
    ) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |this, cx| {
                if this.motion_preview_generation != generation
                    || this.active_tab != SettingsTab::Appearance
                    || !this.search_query.is_empty()
                    || this.active_subsection.is_some()
                {
                    return;
                }
                let running = this.motion_preview.advance(Instant::now());
                if running {
                    Self::queue_preview_frame(
                        window,
                        cx.entity().downgrade(),
                        generation,
                        returning,
                    );
                } else if !returning {
                    this.motion_preview
                        .transition_to(gpui::size(px(72.0), px(22.0)), Instant::now());
                    Self::queue_preview_frame(window, cx.entity().downgrade(), generation, true);
                }
                cx.notify();
            });
        });
    }

    pub fn start_section_transition(&mut self, cx: &mut Context<Self>) {
        self.section_anim_generation += 1;
        let Some(handle) = self.window_handle else {
            self.section_animator.set_duration(Duration::ZERO);
            self.section_animator.reveal_content(Instant::now());
            return;
        };
        let duration = if cx.has_global::<AppState>() {
            Duration::from_millis(
                cx.global::<AppState>()
                    .config
                    .get()
                    .ui
                    .animation_duration_ms as u64,
            )
        } else {
            Duration::from_millis(250)
        };
        self.section_animator.set_duration(duration);
        self.section_animator.reveal_content(Instant::now());
        if !duration.is_zero() {
            let entity = cx.entity().downgrade();
            let generation = self.section_anim_generation;
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_, window, _| {
                    Self::queue_section_frame(window, entity, generation);
                });
            });
        }
    }

    fn queue_section_frame(window: &Window, entity: gpui::WeakEntity<Self>, generation: u64) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |this, cx| {
                if this.section_anim_generation != generation {
                    return;
                }
                let running = this.section_animator.advance(Instant::now());
                cx.notify();
                if running {
                    Self::queue_section_frame(window, cx.entity().downgrade(), generation);
                }
            });
        });
    }

    pub fn toggle_dropdown(&mut self, field: SettingsField, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(field) {
            self.open_dropdown = None;
            self.dropdown_anim_field = None;
            self.dropdown_anim_task = None;
        } else {
            self.open_dropdown = Some(field);
            self.active_field = None;
            self.start_dropdown_anim(field, cx);
        }
        cx.notify();
    }

    fn start_dropdown_anim(&mut self, field: SettingsField, cx: &mut Context<Self>) {
        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };
        self.dropdown_anim_field = Some(field);
        self.dropdown_anim_progress = 0.0;
        let start = Instant::now();

        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 180.0;
            loop {
                let frame_dur = if let Some(ref comp) = compositor {
                    comp.get_frame_duration()
                } else {
                    Duration::from_millis(16)
                };

                cx.background_executor()
                    .timer(frame_dur.max(Duration::from_millis(2)))
                    .await;

                let finished = this
                    .update(cx, |module: &mut Self, cx| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let p = (elapsed / duration_ms).min(1.0);
                        module.dropdown_anim_progress = p;
                        cx.notify();
                        p >= 1.0
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });

        self.dropdown_anim_task = Some(anim_task);
    }

    pub fn select_app_command(
        &mut self,
        field: SettingsField,
        command: String,
        cx: &mut Context<Self>,
    ) {
        match field {
            SettingsField::Terminal => self.terminal_input = command,
            SettingsField::Browser => self.browser_input = command,
            SettingsField::Editor => self.editor_input = command,
            SettingsField::FileManager => self.file_manager_input = command,
            _ => {}
        }
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.active_field = None;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn select_language(&mut self, target_name: String, cx: &mut Context<Self>) {
        if cx.has_global::<AppState>() {
            let app_state = cx.global::<AppState>();
            let _ = app_state.language.set_language(&target_name);
            let _ = app_state.config.update(|cfg| cfg.ui.language = target_name);
        }
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.active_field = None;
        cx.notify();
    }

    pub fn set_active_field(&mut self, field: Option<SettingsField>, cx: &mut Context<Self>) {
        self.active_field = field;
        cx.notify();
    }

    pub fn apply_preset(&mut self, field: SettingsField, val: String, cx: &mut Context<Self>) {
        match field {
            SettingsField::Terminal => self.terminal_input = val,
            SettingsField::Browser => self.browser_input = val,
            SettingsField::Editor => self.editor_input = val,
            SettingsField::FileManager => self.file_manager_input = val,
            SettingsField::TimeFormat => self.time_format_input = val,
            SettingsField::DateFormat => self.date_format_input = val,
            _ => {}
        }
        self.active_field = None;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_field_float(&mut self, field: SettingsField, val: f32, cx: &mut Context<Self>) {
        let str_val = format!("{val:.0}");
        match field {
            SettingsField::CapsuleRound => self.capsule_round_input = str_val,
            SettingsField::SatelliteRound => self.satellite_round_input = str_val,
            SettingsField::CardsRound => self.cards_round_input = str_val,
            SettingsField::MarginTop => self.margin_top_input = str_val,
            SettingsField::IdleHeight => self.idle_height_input = str_val,
            SettingsField::Gap => self.gap_input = str_val,
            SettingsField::AnimDuration => self.anim_duration_input = str_val,
            _ => {}
        }
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_field_int(&mut self, field: SettingsField, val: u64, cx: &mut Context<Self>) {
        if let SettingsField::IdleTimeout = field {
            self.idle_timeout_input = format!("{val}");
        }
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn toggle_show_clock(&mut self, cx: &mut Context<Self>) {
        self.show_clock = !self.show_clock;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn toggle_output_devices(&mut self, cx: &mut Context<Self>) {
        self.show_output_devices = !self.show_output_devices;
        cx.notify();
    }

    pub fn toggle_input_devices(&mut self, cx: &mut Context<Self>) {
        self.show_input_devices = !self.show_input_devices;
        cx.notify();
    }

    pub fn toggle_setting(&mut self, field: SettingsField, cx: &mut Context<Self>) {
        let config = cx.global::<AppState>().config.clone();
        let result = config.update(|cfg| match field {
            SettingsField::DynamicColors => cfg.ui.dynamic_colors = !cfg.ui.dynamic_colors,
            SettingsField::DarkMode => cfg.ui.dark_mode = !cfg.ui.dark_mode,
            SettingsField::ClockFormat => cfg.ui.clock_24_hour = !cfg.ui.clock_24_hour,
            SettingsField::LegacyCapsule => cfg.ui.use_new_capsule = !cfg.ui.use_new_capsule,
            SettingsField::CapsuleStyle => {
                cfg.ui.capsule_style = if cfg.ui.capsule_style == CapsuleStyle::Normal {
                    CapsuleStyle::Concave
                } else {
                    CapsuleStyle::Normal
                }
            }
            _ => {}
        });
        self.appearance_error = result.err().map(|error| error.to_string());
        let cfg = config.get();
        self.use_new_capsule = cfg.ui.use_new_capsule;
        self.capsule_style = cfg.ui.capsule_style;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.active_field = Some(field);
        cx.notify();
    }

    pub fn toggle_show_media_player(&mut self, cx: &mut Context<Self>) {
        self.show_media_player = !self.show_media_player;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn save_to_app_config(&self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

        let capsule_style = self.capsule_style;
        let use_new_capsule = self.use_new_capsule;
        let terminal = self.terminal_input.clone();
        let browser = self.browser_input.clone();
        let editor = self.editor_input.clone();
        let file_manager = self.file_manager_input.clone();

        let capsule_round = self.capsule_round_input.parse::<f32>().unwrap_or(24.0);
        let satellite_round = self.satellite_round_input.parse::<f32>().unwrap_or(20.0);
        let cards_round = self.cards_round_input.parse::<f32>().unwrap_or(18.0);
        let margin_top = self.margin_top_input.parse::<f32>().unwrap_or(8.0);
        let idle_height = self.idle_height_input.parse::<f32>().unwrap_or(32.0);
        let gap = self.gap_input.parse::<f32>().unwrap_or(10.0);
        let animation_duration = self.anim_duration_input.parse::<f32>().unwrap_or(250.0);

        let idle_timeout = self.idle_timeout_input.parse::<u64>().unwrap_or(900);
        let time_format = self.time_format_input.clone();
        let date_format = self.date_format_input.clone();
        let show_clock = self.show_clock;
        let show_media_player = self.show_media_player;
        let music_players = self.music_players.clone();

        let record_output = self.record_output.clone();
        let record_fps = self.record_fps;
        let record_resolution = self.record_resolution.clone();
        let record_quality = self.record_quality.clone();
        let record_container = self.record_container.clone();
        let record_audio = self.record_audio.clone();
        let record_include_cursor = self.record_include_cursor;

        let _ = cx.global::<AppState>().config.update(|cfg| {
            cfg.ui.capsule_style = capsule_style;
            cfg.ui.use_new_capsule = use_new_capsule;
            cfg.defaults.terminal = terminal;
            cfg.defaults.browser = browser;
            cfg.defaults.editor = editor;
            cfg.defaults.file_manager = file_manager;
            cfg.mpris.players = music_players;

            cfg.ui.capsule_round = capsule_round;
            cfg.ui.satellite_round = satellite_round;
            cfg.ui.cards_round = cards_round;
            cfg.ui.margin_top = margin_top;
            cfg.ui.idle_height = idle_height;
            cfg.ui.gap = gap;
            cfg.ui.animation_duration_ms = animation_duration as u32;

            cfg.lockscreen.idle_timeout = idle_timeout;
            cfg.lockscreen.time_format = time_format;
            cfg.lockscreen.date_format = date_format;
            cfg.lockscreen.show_clock = show_clock;
            cfg.lockscreen.show_media_player = show_media_player;

            cfg.record.output = record_output;
            cfg.record.fps = record_fps;
            cfg.record.resolution = record_resolution;
            cfg.record.quality = record_quality;
            cfg.record.container = record_container;
            cfg.record.audio = record_audio;
            cfg.record.include_cursor = record_include_cursor;
        });
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.motion_preview_generation += 1;
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.section_anim_generation += 1;
        self.section_animator.set_duration(Duration::ZERO);
        self.section_animator.reveal_content(Instant::now());
        self.active_slider_drag = None;
        self.search_query.clear();
        self.active_subsection = None;
        self.set_tab(SettingsTab::Appearance, cx);
        cx.emit(SettingsEvent::Close);
    }

    pub fn set_record_output(&mut self, output: String, cx: &mut Context<Self>) {
        self.record_output = output;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_record_fps(&mut self, fps: u32, cx: &mut Context<Self>) {
        self.record_fps = fps;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_record_resolution(&mut self, resolution: String, cx: &mut Context<Self>) {
        self.record_resolution = resolution;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_record_quality(&mut self, quality: String, cx: &mut Context<Self>) {
        self.record_quality = quality;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_record_container(&mut self, container: String, cx: &mut Context<Self>) {
        self.record_container = container;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn set_record_audio(&mut self, audio: String, cx: &mut Context<Self>) {
        self.record_audio = audio;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn toggle_record_include_cursor(&mut self, cx: &mut Context<Self>) {
        self.record_include_cursor = !self.record_include_cursor;
        self.save_to_app_config(cx);
        cx.notify();
    }

    #[allow(dead_code)]
    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.window_handle = Some(window.window_handle());
        window.focus(&self.focus_handle, cx);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if let Some(
            field @ (SettingsField::DynamicColors
            | SettingsField::DarkMode
            | SettingsField::ClockFormat
            | SettingsField::CapsuleStyle
            | SettingsField::LegacyCapsule),
        ) = self.active_field
        {
            if key == "space" || key == "enter" {
                self.toggle_setting(field, cx);
                return;
            }
            if key == "tab" {
                self.active_field = None;
            } else if key != "escape" && !(ctrl && key == "f") {
                return;
            }
        }
        if ctrl && key == "f" {
            self.set_active_field(Some(SettingsField::Search), cx);
            return;
        }
        if key == "space"
            && self.active_field.is_none()
            && self.active_tab == SettingsTab::Appearance
        {
            self.play_motion_preview(window, cx);
            return;
        }
        if key == "escape" {
            if self.open_dropdown.is_some() {
                self.open_dropdown = None;
                self.dropdown_anim_field = None;
                cx.notify();
                return;
            }
            if self.active_field == Some(SettingsField::Search) && !self.search_query.is_empty() {
                self.search_query.clear();
                self.active_field = None;
                self.start_section_transition(cx);
                cx.notify();
                return;
            }
            if self.active_field.is_some() {
                self.active_field = None;
                cx.notify();
                return;
            }
            if self.active_subsection.is_some() {
                self.close_subsection(cx);
                return;
            }
            self.close(cx);
            return;
        }

        if key == "tab" && self.active_field.is_none() {
            let next_tab = match self.active_tab {
                SettingsTab::Appearance => SettingsTab::Apps,
                SettingsTab::Apps => SettingsTab::Media,
                SettingsTab::Media => SettingsTab::LockScreen,
                SettingsTab::LockScreen => SettingsTab::System,
                SettingsTab::System => SettingsTab::Record,
                SettingsTab::Record => SettingsTab::Appearance,
            };
            self.set_tab(next_tab, cx);
            return;
        }

        let Some(field) = self.active_field else {
            return;
        };

        if ctrl && key == "v" {
            if let Some(item) = cx.read_from_clipboard()
                && let Some(text) = item.text()
            {
                let clean: String = text.chars().filter(|c| !c.is_control()).collect();
                self.append_text_to_field(field, &clean);
                if field != SettingsField::Search {
                    self.save_to_app_config(cx);
                }
                cx.notify();
            }
            return;
        }

        match key {
            "enter" => {
                if self.active_field == Some(SettingsField::MusicPlayer) {
                    let text = self.music_player_input.clone();
                    self.add_music_player(text, cx);
                }
                self.active_field = None;
                if field != SettingsField::Search {
                    self.save_to_app_config(cx);
                }
                cx.notify();
            }
            "backspace" => {
                self.pop_char_from_field(field);
                if field != SettingsField::Search {
                    self.save_to_app_config(cx);
                }
                cx.notify();
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1 && !ctrl {
                    self.append_text_to_field(field, text);
                    if field != SettingsField::Search {
                        self.save_to_app_config(cx);
                    }
                    cx.notify();
                }
            }
        }
    }

    fn append_text_to_field(&mut self, field: SettingsField, text: &str) {
        let target = match field {
            SettingsField::Terminal => &mut self.terminal_input,
            SettingsField::Browser => &mut self.browser_input,
            SettingsField::Editor => &mut self.editor_input,
            SettingsField::FileManager => &mut self.file_manager_input,
            SettingsField::MusicPlayer => &mut self.music_player_input,
            SettingsField::CapsuleRound => &mut self.capsule_round_input,
            SettingsField::SatelliteRound => &mut self.satellite_round_input,
            SettingsField::CardsRound => &mut self.cards_round_input,
            SettingsField::MarginTop => &mut self.margin_top_input,
            SettingsField::IdleHeight => &mut self.idle_height_input,
            SettingsField::Gap => &mut self.gap_input,
            SettingsField::AnimDuration => &mut self.anim_duration_input,
            SettingsField::IdleTimeout => &mut self.idle_timeout_input,
            SettingsField::TimeFormat => &mut self.time_format_input,
            SettingsField::DateFormat => &mut self.date_format_input,
            SettingsField::Search => &mut self.search_query,
            SettingsField::Language
            | SettingsField::VolumeOutput
            | SettingsField::VolumeInput
            | SettingsField::DynamicColors
            | SettingsField::DarkMode
            | SettingsField::ClockFormat
            | SettingsField::CapsuleStyle
            | SettingsField::LegacyCapsule => {
                return;
            }
        };
        target.push_str(text);
    }

    fn pop_char_from_field(&mut self, field: SettingsField) {
        let target = match field {
            SettingsField::Terminal => &mut self.terminal_input,
            SettingsField::Browser => &mut self.browser_input,
            SettingsField::Editor => &mut self.editor_input,
            SettingsField::FileManager => &mut self.file_manager_input,
            SettingsField::MusicPlayer => &mut self.music_player_input,
            SettingsField::CapsuleRound => &mut self.capsule_round_input,
            SettingsField::SatelliteRound => &mut self.satellite_round_input,
            SettingsField::CardsRound => &mut self.cards_round_input,
            SettingsField::MarginTop => &mut self.margin_top_input,
            SettingsField::IdleHeight => &mut self.idle_height_input,
            SettingsField::Gap => &mut self.gap_input,
            SettingsField::AnimDuration => &mut self.anim_duration_input,
            SettingsField::IdleTimeout => &mut self.idle_timeout_input,
            SettingsField::TimeFormat => &mut self.time_format_input,
            SettingsField::DateFormat => &mut self.date_format_input,
            SettingsField::Search => &mut self.search_query,
            SettingsField::Language
            | SettingsField::VolumeOutput
            | SettingsField::VolumeInput
            | SettingsField::DynamicColors
            | SettingsField::DarkMode
            | SettingsField::ClockFormat
            | SettingsField::CapsuleStyle
            | SettingsField::LegacyCapsule => {
                return;
            }
        };
        target.pop();
    }
}

impl Render for SettingsModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let content_view: AnyElement = if !self.search_query.trim().is_empty() {
            render_search_results(self, &theme, cx).into_any_element()
        } else if let Some(sub) = self.active_subsection {
            match sub {
                SettingsSubSection::MusicPlayers => {
                    render_music_players_subsection(self, &theme, cx).into_any_element()
                }
                SettingsSubSection::LockScreenFormats => {
                    render_lockscreen_formats_subsection(self, &theme, cx).into_any_element()
                }
            }
        } else {
            match self.active_tab {
                SettingsTab::Appearance => render_ui_section(self, &theme, cx).into_any_element(),
                SettingsTab::Apps => render_apps_section(self, &theme, cx).into_any_element(),
                SettingsTab::Media => render_media_section(self, &theme, cx).into_any_element(),
                SettingsTab::LockScreen => {
                    render_lockscreen_section(self, &theme, cx).into_any_element()
                }
                SettingsTab::System => render_system_section(self, &theme, cx).into_any_element(),
                SettingsTab::Record => render_record_section(self, &theme, cx).into_any_element(),
            }
        };

        let capsule_radius = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.capsule_round
        } else {
            24.0
        };

        let modal_bounds_cell = self.settings_bounds.clone();

        div()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(
                cx.listener(|this, event: &gpui::MouseMoveEvent, _window, cx| {
                    if let Some((field, min, max, step, ref bounds_cell)) = this.active_slider_drag
                    {
                        let (left, width) = bounds_cell.get();
                        if width > 0.0 {
                            let x: f32 = event.position.x.into();
                            let rel = (x - left).clamp(0.0, width);
                            let p = rel / width;
                            let raw = min + p * (max - min);
                            let stepped = (raw / step).round() * step;
                            let final_val = stepped.clamp(min, max);
                            if field == SettingsField::IdleTimeout {
                                this.set_field_int(field, (final_val * 60.0) as u64, cx);
                            } else if field == SettingsField::VolumeOutput {
                                if cx.has_global::<AppState>() {
                                    cx.global::<AppState>()
                                        .system
                                        .set_volume_fast(final_val as u32);
                                }
                                cx.notify();
                            } else if field == SettingsField::VolumeInput {
                                if cx.has_global::<AppState>() {
                                    cx.global::<AppState>()
                                        .system
                                        .set_input_volume_fast(final_val as u32);
                                }
                                cx.notify();
                            } else {
                                this.set_field_float(field, final_val, cx);
                            }
                        }
                    }
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _event, window, cx| {
                    if let Some((field, _, _, _, _)) = this.active_slider_drag {
                        this.active_slider_drag = None;
                        if field == SettingsField::AnimDuration {
                            this.play_motion_preview(window, cx);
                        }
                        if (field == SettingsField::VolumeOutput
                            || field == SettingsField::VolumeInput)
                            && cx.has_global::<AppState>()
                        {
                            let sys = cx.global::<AppState>().system.clone();
                            services::spawn_tokio(async move {
                                let _ = sys.refresh().await;
                            });
                        }
                        cx.notify();
                    }
                }),
            )
            .flex()
            .flex_col()
            .gap(px(20.0))
            .p(px(24.0))
            .w_full()
            .h_full()
            .bg(style::background(&theme))
            .font_family(theme.font_family())
            .text_color(theme.foreground())
            .rounded(px(capsule_radius))
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _, _| {
                        let top: f32 = bounds.origin.y.into();
                        let height: f32 = if bounds.size.height > px(0.0) {
                            bounds.size.height.into()
                        } else {
                            HEIGHT
                        };
                        modal_bounds_cell.set((top, top + height));
                    },
                    |_, _, _, _| {},
                )
                .inset_0()
                .absolute(),
            )
            .child(render_header(self, &theme, cx))
            .child(
                div()
                    .flex()
                    .gap(px(28.0))
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(render_tabs(self, &theme, cx))
                    .child(
                        div()
                            .id("settings-content-scroll")
                            .track_scroll(&self.scroll_handle)
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .min_w_0()
                                    .flex_shrink_0()
                                    .when(self.section_animator.content_progress() < 1.0, |s| {
                                        s.blur(px(
                                            (1.0 - self.section_animator.content_progress()) * 5.0
                                        ))
                                    })
                                    .child(content_view),
                            ),
                    ),
            )
    }
}
