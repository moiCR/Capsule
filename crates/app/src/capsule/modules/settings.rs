use gpui::{
    AnyElement, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, ParentElement,
    Render, ScrollHandle, Styled, Task, Window, canvas, div, prelude::*, px, svg,
};
use services::{AppState, CapsuleStyle};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::capsule::widgets::settings::{
    render_apps_section, render_lockscreen_formats_subsection, render_lockscreen_section,
    render_media_section, render_music_players_subsection, render_search_results, render_sidebar,
    render_system_section, render_ui_section,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    Capsule = 0,
    Apps = 1,
    Media = 2,
    LockScreen = 3,
    System = 4,
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
}

pub enum SettingsEvent {
    Close,
}

pub type SliderDragState = (SettingsField, f32, f32, f32, Rc<Cell<(f32, f32)>>);

pub struct SettingsModule {
    pub active_tab: SettingsTab,
    pub active_field: Option<SettingsField>,
    pub capsule_style: CapsuleStyle,

    pub terminal_input: String,
    pub browser_input: String,
    pub editor_input: String,
    pub file_manager_input: String,

    pub music_players: Vec<String>,
    pub music_player_input: String,
    pub show_lyrics: bool,

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
    pub section_anim_progress: f32,
    pub section_anim_task: Option<Task<()>>,
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

    focus_handle: FocusHandle,
    pub scroll_handle: ScrollHandle,
}

impl EventEmitter<SettingsEvent> for SettingsModule {}

impl SettingsModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();

        let mut module = Self {
            active_tab: SettingsTab::Capsule,
            active_field: None,
            capsule_style: CapsuleStyle::Normal,
            terminal_input: String::new(),
            browser_input: String::new(),
            editor_input: String::new(),
            file_manager_input: String::new(),
            music_players: Vec::new(),
            music_player_input: String::new(),
            show_lyrics: true,
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
            section_anim_progress: 1.0,
            section_anim_task: None,
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
        self.terminal_input = cfg.defaults.terminal.clone();
        self.browser_input = cfg.defaults.browser.clone();
        self.editor_input = cfg.defaults.editor.clone();
        self.file_manager_input = cfg.defaults.file_manager.clone();

        self.music_players = cfg.mpris.players.clone();
        self.music_player_input.clear();
        self.show_lyrics = cfg.mpris.show_lyrics;

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

    pub fn navigate_forward(&mut self, _cx: &mut Context<Self>) {}

    pub fn can_navigate_back(&self) -> bool {
        self.active_subsection.is_some()
    }

    pub fn can_navigate_forward(&self) -> bool {
        false
    }

    pub fn start_section_transition(&mut self, cx: &mut Context<Self>) {
        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };
        self.section_anim_progress = 0.0;
        let start = Instant::now();

        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 160.0;
            loop {
                let frame_dur = if let Some(ref comp) = compositor {
                    comp.get_frame_duration()
                } else {
                    Duration::from_millis(16)
                };

                cx.background_executor().timer(frame_dur).await;

                let finished = this
                    .update(cx, |module: &mut Self, cx| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let p = (elapsed / duration_ms).min(1.0);
                        let ease = 1.0 - (1.0 - p) * (1.0 - p);
                        module.section_anim_progress = ease;
                        cx.notify();
                        p >= 1.0
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });

        self.section_anim_task = Some(anim_task);
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

                cx.background_executor().timer(frame_dur).await;

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

    pub fn toggle_show_lyrics(&mut self, cx: &mut Context<Self>) {
        self.show_lyrics = !self.show_lyrics;
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

    pub fn set_capsule_style(&mut self, style: CapsuleStyle, cx: &mut Context<Self>) {
        self.capsule_style = style;
        self.save_to_app_config(cx);
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
        let terminal = self.terminal_input.clone();
        let browser = self.browser_input.clone();
        let editor = self.editor_input.clone();
        let file_manager = self.file_manager_input.clone();
        let show_lyrics = self.show_lyrics;

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

        let _ = cx.global::<AppState>().config.update(|cfg| {
            cfg.ui.capsule_style = capsule_style;
            cfg.defaults.terminal = terminal;
            cfg.defaults.browser = browser;
            cfg.defaults.editor = editor;
            cfg.defaults.file_manager = file_manager;
            cfg.mpris.players = music_players;
            cfg.mpris.show_lyrics = show_lyrics;

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
        });
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.section_anim_task = None;
        self.active_slider_drag = None;
        self.search_query.clear();
        self.active_subsection = None;
        self.set_tab(SettingsTab::Capsule, cx);
        cx.emit(SettingsEvent::Close);
    }

    #[allow(dead_code)]
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

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
                SettingsTab::Capsule => SettingsTab::Apps,
                SettingsTab::Apps => SettingsTab::Media,
                SettingsTab::Media => SettingsTab::LockScreen,
                SettingsTab::LockScreen => SettingsTab::System,
                SettingsTab::System => SettingsTab::Capsule,
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
            SettingsField::Language | SettingsField::VolumeOutput | SettingsField::VolumeInput => {
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
            SettingsField::Language | SettingsField::VolumeOutput | SettingsField::VolumeInput => {
                return;
            }
        };
        target.pop();
    }
}

impl Render for SettingsModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        window.focus(&self.focus_handle, cx);

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
                SettingsTab::Capsule => render_ui_section(self, &theme, cx).into_any_element(),
                SettingsTab::Apps => render_apps_section(self, &theme, cx).into_any_element(),
                SettingsTab::Media => render_media_section(self, &theme, cx).into_any_element(),
                SettingsTab::LockScreen => {
                    render_lockscreen_section(self, &theme, cx).into_any_element()
                }
                SettingsTab::System => render_system_section(self, &theme, cx).into_any_element(),
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
                cx.listener(|this, _event, _window, cx| {
                    if let Some((field, _, _, _, _)) = this.active_slider_drag {
                        this.active_slider_drag = None;
                        if (field == SettingsField::VolumeOutput
                            || field == SettingsField::VolumeInput)
                            && cx.has_global::<AppState>()
                        {
                            let sys = cx.global::<AppState>().system.clone();
                            cx.spawn(async move |_this, _cx| {
                                let _ = sys.refresh().await;
                            })
                            .detach();
                        }
                        cx.notify();
                    }
                }),
            )
            .flex()
            .flex_row()
            .w(px(840.0))
            .h(px(560.0))
            .rounded(px(capsule_radius))
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _, _| {
                        let top: f32 = bounds.origin.y.into();
                        let height: f32 = if bounds.size.height > px(0.0) {
                            bounds.size.height.into()
                        } else {
                            560.0
                        };
                        modal_bounds_cell.set((top, top + height));
                    },
                    |_, _, _, _| {},
                )
                .inset_0()
                .absolute(),
            )
            .child(render_sidebar(self, &theme, cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .child(render_top_navigation_bar(self, &theme, cx))
                    .child(
                        div()
                            .id("settings-content-scroll")
                            .track_scroll(&self.scroll_handle)
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .px_6()
                            .pb_6()
                            .overflow_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .w_full()
                                    .min_w_0()
                                    .opacity(self.section_anim_progress)
                                    .mt(px((1.0 - self.section_anim_progress) * 8.0))
                                    .child(content_view),
                            ),
                    ),
            )
    }
}

fn render_top_navigation_bar(
    module: &SettingsModule,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let can_back = module.can_navigate_back();
    let can_fwd = module.can_navigate_forward();

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .min_w_0()
        .h(px(46.0))
        .px_6()
        .pt_2()
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .child(
                    div()
                        .id("settings-nav-back")
                        .w(px(26.0))
                        .h(px(26.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(if can_back { 0.5 } else { 0.2 }))
                        .hover(|s| {
                            if can_back {
                                s.bg(theme.surface().opacity(0.8))
                            } else {
                                s
                            }
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor(if can_back {
                            gpui::CursorStyle::PointingHand
                        } else {
                            gpui::CursorStyle::Arrow
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.can_navigate_back() {
                                this.navigate_back(cx);
                            }
                        }))
                        .child(svg().path("chevron-left.svg").size(px(14.0)).text_color(
                            if can_back {
                                theme.foreground()
                            } else {
                                theme.foreground_muted().opacity(0.4)
                            },
                        )),
                )
                .child(
                    div()
                        .id("settings-nav-fwd")
                        .w(px(26.0))
                        .h(px(26.0))
                        .rounded_full()
                        .bg(theme.surface().opacity(if can_fwd { 0.5 } else { 0.2 }))
                        .hover(|s| {
                            if can_fwd {
                                s.bg(theme.surface().opacity(0.8))
                            } else {
                                s
                            }
                        })
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor(if can_fwd {
                            gpui::CursorStyle::PointingHand
                        } else {
                            gpui::CursorStyle::Arrow
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if this.can_navigate_forward() {
                                this.navigate_forward(cx);
                            }
                        }))
                        .child(svg().path("chevron-right.svg").size(px(14.0)).text_color(
                            if can_fwd {
                                theme.foreground()
                            } else {
                                theme.foreground_muted().opacity(0.4)
                            },
                        )),
                ),
        )
        .child(
            div()
                .id("settings-close-btn")
                .w(px(26.0))
                .h(px(26.0))
                .rounded_full()
                .bg(theme.surface().opacity(0.45))
                .hover(|s| s.bg(theme.surface().opacity(0.8)))
                .active(|s| s.bg(theme.surface()))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.close(cx);
                }))
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(12.0))
                        .text_color(theme.foreground_muted()),
                ),
        )
}
