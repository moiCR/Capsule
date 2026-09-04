use gpui::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, ParentElement, Render,
    ScrollHandle, Styled, Window, div, prelude::*, px,
};
use services::AppState;
use ui::theme::Theme;

use crate::capsule::widgets::settings::{
    render_defaults_section, render_lockscreen_section, render_sidebar, render_system_section,
    render_ui_section,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    Defaults = 0,
    UI = 1,
    LockScreen = 2,
    System = 3,
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
}

pub enum SettingsEvent {
    Close,
}

pub struct SettingsModule {
    pub active_tab: SettingsTab,
    pub active_field: Option<SettingsField>,

    pub terminal_input: String,
    pub browser_input: String,
    pub editor_input: String,

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

    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

impl EventEmitter<SettingsEvent> for SettingsModule {}

impl SettingsModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();

        let mut module = Self {
            active_tab: SettingsTab::Defaults,
            active_field: None,
            open_dropdown: None,
            terminal_input: String::new(),
            browser_input: String::new(),
            editor_input: String::new(),
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

        self.terminal_input = cfg.defaults.terminal.clone();
        self.browser_input = cfg.defaults.browser.clone();
        self.editor_input = cfg.defaults.editor.clone();

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

    pub fn set_tab(&mut self, tab: SettingsTab, cx: &mut Context<Self>) {
        self.active_tab = tab;
        self.active_field = None;
        self.open_dropdown = None;
        self.scroll_handle.scroll_to_item(0);
        cx.notify();
    }

    pub fn toggle_dropdown(&mut self, field: SettingsField, cx: &mut Context<Self>) {
        if self.open_dropdown == Some(field) {
            self.open_dropdown = None;
        } else {
            self.open_dropdown = Some(field);
        }
        cx.notify();
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
        self.active_field = None;
        self.save_to_app_config(cx);
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

    pub fn toggle_show_media_player(&mut self, cx: &mut Context<Self>) {
        self.show_media_player = !self.show_media_player;
        self.save_to_app_config(cx);
        cx.notify();
    }

    pub fn save_to_app_config(&self, cx: &mut Context<Self>) {
        if !cx.has_global::<AppState>() {
            return;
        }

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

        let _ = cx.global::<AppState>().config.update(|cfg| {
            cfg.defaults.terminal = terminal;
            cfg.defaults.browser = browser;
            cfg.defaults.editor = editor;

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
                SettingsTab::Defaults => SettingsTab::UI,
                SettingsTab::UI => SettingsTab::LockScreen,
                SettingsTab::LockScreen => SettingsTab::System,
                SettingsTab::System => SettingsTab::Defaults,
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
        };
        target.push_str(text);
    }

    fn pop_char_from_field(&mut self, field: SettingsField) {
        let target = match field {
            SettingsField::Terminal => &mut self.terminal_input,
            SettingsField::Browser => &mut self.browser_input,
            SettingsField::Editor => &mut self.editor_input,
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
        };
        target.pop();
    }
}

impl Render for SettingsModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        window.focus(&self.focus_handle, cx);

        let content_view = match self.active_tab {
            SettingsTab::Defaults => render_defaults_section(self, &theme, cx).into_any_element(),
            SettingsTab::UI => render_ui_section(self, &theme, cx).into_any_element(),
            SettingsTab::LockScreen => {
                render_lockscreen_section(self, &theme, cx).into_any_element()
            }
            SettingsTab::System => render_system_section(self, &theme, cx).into_any_element(),
        };

        let capsule_radius = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.capsule_round
        } else {
            36.0
        };

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_row()
            .w(px(840.0))
            .h(px(560.0))
            .rounded(px(capsule_radius))
            .overflow_hidden()
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
    }
}
