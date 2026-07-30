use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent, Render, ScrollHandle,
    Window, div, prelude::*, px, svg,
};
use ui::theme::Theme;
use ui::theme::theme_manager::{ThemeItem, ThemeManager};

use crate::capsule::widgets::select_theme::theme_card::render_theme_card;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectThemeEvent {
    CreateThemeRequested,
    ThemeSelected,
}

pub struct SelectThemeModule {
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    themes: Vec<ThemeItem>,
}

impl SelectThemeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let themes = Self::load_themes(cx);
        Self {
            focus_handle,
            scroll_handle,
            themes,
        }
    }

    pub fn refresh_themes(&mut self, cx: &mut Context<Self>) {
        self.themes = Self::load_themes(cx);
        cx.notify();
    }

    fn load_themes(cx: &mut Context<Self>) -> Vec<ThemeItem> {
        if cx.has_global::<ThemeManager>() {
            cx.global::<ThemeManager>().list_themes()
        } else {
            Vec::new()
        }
    }

    pub fn select_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if cx.has_global::<ThemeManager>() {
            cx.global_mut::<ThemeManager>().set_theme(theme);
            cx.set_global(cx.global::<ThemeManager>().current_theme.clone());
        }
        cx.emit(SelectThemeEvent::ThemeSelected);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if key == "escape" {
            cx.emit(SelectThemeEvent::ThemeSelected);
        }
    }
}

impl EventEmitter<SelectThemeEvent> for SelectThemeModule {}

impl Focusable for SelectThemeModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SelectThemeModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let current_theme = theme.clone();

        window.focus(&self.focus_handle, cx);

        let lang = if cx.has_global::<ui::language::Language>() {
            cx.global::<ui::language::Language>().clone()
        } else {
            ui::language::Language::default()
        };

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        svg()
                            .path("palette_2.svg")
                            .size(px(16.0))
                            .text_color(theme.accent()),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(14.0))
                            .text_color(theme.foreground())
                            .child(lang.themes.select_title),
                    ),
            )
            .child(
                div()
                    .id("create-theme-btn")
                    .cursor_pointer()
                    .px_3()
                    .py_1p5()
                    .rounded_full()
                    .bg(theme.accent())
                    .on_click(cx.listener(|_this, _, _, cx| {
                        cx.emit(SelectThemeEvent::CreateThemeRequested);
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.background())
                            .child(lang.themes.create_button),
                    ),
            );

        let divider = div().w_full().h(px(1.0)).bg(theme.background_alt());

        let mut theme_cards = div()
            .id("select-theme-list")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .gap_2p5()
            .w_full()
            .max_h(px(380.0))
            .overflow_scroll();

        for item in &self.themes {
            let card = render_theme_card(item, &current_theme, &theme, cx);
            theme_cards = theme_cards.child(card);
        }

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(348.0))
            .p_4()
            .gap_3p5()
            .overflow_hidden()
            .child(header)
            .child(divider)
            .child(theme_cards)
    }
}
