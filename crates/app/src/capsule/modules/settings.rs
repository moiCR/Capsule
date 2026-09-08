use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, ParentElement, Render,
    ScrollHandle, Styled, Task, Window, canvas, div, prelude::*, px,
};
use services::{AppState, CapsuleStyle};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use crate::capsule::widgets::settings::{
    render_dropdown_overlay, render_general_section, render_lockscreen_section, render_sidebar,
    render_ui_section,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    General = 0,
    UI = 1,
    LockScreen = 2,
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
}

pub enum SettingsEvent {
    Close,
}

pub struct SettingsModule {
    pub active_tab: SettingsTab,
    pub active_field: Option<SettingsField>,
    pub capsule_style: CapsuleStyle,

    pub terminal_input: String,
    pub browser_input: String,
    pub editor_input: String,

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
    pub splat_anim_field: Option<SettingsField>,
    pub splat_anim_progress: f32,
    pub splat_anim_task: Option<Task<()>>,

    pub terminal_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub browser_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub editor_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub language_trigger_bounds: Rc<Cell<(f32, f32)>>,
    pub settings_top_y: Rc<Cell<f32>>,

    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

impl EventEmitter<SettingsEvent> for SettingsModule {}

impl SettingsModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();

        let mut module = Self {
            active_tab: SettingsTab::General,
            active_field: None,
            capsule_style: CapsuleStyle::Normal,
            terminal_input: String::new(),
            browser_input: String::new(),
            editor_input: String::new(),
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
            splat_anim_field: None,
            splat_anim_progress: 1.0,
            splat_anim_task: None,
            terminal_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            browser_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            editor_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            language_trigger_bounds: Rc::new(Cell::new((0.0, 0.0))),
            settings_top_y: Rc::new(Cell::new(0.0)),
            focus_handle,
            scroll_handle,
        };

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
        self.active_tab = tab;
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.splat_anim_field = None;
        self.splat_anim_task = None;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
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
            let duration_ms = 220.0;
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
            _ => {}
        }
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.active_field = None;
        self.start_splat_anim(field, cx);
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
        self.start_splat_anim(SettingsField::Language, cx);
        cx.notify();
    }

    fn start_splat_anim(&mut self, field: SettingsField, cx: &mut Context<Self>) {
        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };
        self.splat_anim_field = Some(field);
        self.splat_anim_progress = 0.0;
        let start = Instant::now();

        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 220.0;
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
                        module.splat_anim_progress = p;
                        if p >= 1.0 {
                            module.splat_anim_field = None;
                        }
                        cx.notify();
                        p >= 1.0
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });

        self.splat_anim_task = Some(anim_task);
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
        });
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.active_field = None;
        self.open_dropdown = None;
        self.dropdown_anim_field = None;
        self.dropdown_anim_task = None;
        self.splat_anim_field = None;
        self.splat_anim_task = None;
        self.set_tab(SettingsTab::General, cx);
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
            if self.active_field.is_some() {
                self.active_field = None;
                cx.notify();
            } else {
                self.close(cx);
            }
            return;
        }

        if key == "tab" && self.active_field.is_none() {
            let next_tab = match self.active_tab {
                SettingsTab::General => SettingsTab::UI,
                SettingsTab::UI => SettingsTab::LockScreen,
                SettingsTab::LockScreen => SettingsTab::General,
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
                self.save_to_app_config(cx);
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
                self.save_to_app_config(cx);
                cx.notify();
            }
            "backspace" => {
                self.pop_char_from_field(field);
                self.save_to_app_config(cx);
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
                    self.save_to_app_config(cx);
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
            SettingsField::Language => return,
        };
        target.push_str(text);
    }

    fn pop_char_from_field(&mut self, field: SettingsField) {
        let target = match field {
            SettingsField::Terminal => &mut self.terminal_input,
            SettingsField::Browser => &mut self.browser_input,
            SettingsField::Editor => &mut self.editor_input,
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
            SettingsField::Language => return,
        };
        target.pop();
    }
}

impl Render for SettingsModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        window.focus(&self.focus_handle, cx);

        let content_view = match self.active_tab {
            SettingsTab::General => render_general_section(self, &theme, cx).into_any_element(),
            SettingsTab::UI => render_ui_section(self, &theme, cx).into_any_element(),
            SettingsTab::LockScreen => {
                render_lockscreen_section(self, &theme, cx).into_any_element()
            }
        };

        let dropdown_overlay = if self.active_tab == SettingsTab::General {
            render_dropdown_overlay(self, &theme, cx)
        } else {
            None
        };

        let capsule_radius = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.capsule_round
        } else {
            36.0
        };

        let settings_top_cell = self.settings_top_y.clone();

        div()
            .relative()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_row()
            .w(px(840.0))
            .h(px(560.0))
            .rounded(px(capsule_radius))
            .overflow_hidden()
            .child(
                canvas(
                    move |bounds, _, _| {
                        settings_top_cell.set(bounds.origin.y.into());
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_0(),
            )
            .child(render_sidebar(self.active_tab, &theme, cx))
            .child(
                div()
                    .id("settings-content-scroll")
                    .track_scroll(&self.scroll_handle)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .h_full()
                    .p_6()
                    .overflow_scroll()
                    .child(content_view),
            )
            .children(dropdown_overlay)
    }
}
