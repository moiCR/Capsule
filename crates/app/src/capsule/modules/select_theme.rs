use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent, Render, ScrollHandle,
    Window, canvas, div, prelude::*, px, svg,
};
use std::cell::Cell;
use std::rc::Rc;
use ui::theme::Theme;
use ui::theme::theme_manager::{ThemeItem, ThemeManager};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectThemeEvent {
    CreateThemeRequested,
    ThemeSelected,
}

pub struct SelectThemeModule {
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    themes: Vec<ThemeItem>,
    pub measured_top: Rc<Cell<f32>>,
    pub measured_bottom: Rc<Cell<f32>>,
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
            measured_top: Rc::new(Cell::new(0.0)),
            measured_bottom: Rc::new(Cell::new(0.0)),
        }
    }

    pub fn measured_size(&self) -> (f32, f32) {
        let height = (self.measured_bottom.get() - self.measured_top.get()).max(0.0);
        (348.0, height)
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

    fn select_theme(&mut self, theme: Theme, cx: &mut Context<Self>) {
        if cx.has_global::<ThemeManager>() {
            cx.global_mut::<ThemeManager>().set_theme(theme);
            cx.set_global(cx.global::<ThemeManager>().current_theme.clone());
        }
        cx.emit(SelectThemeEvent::ThemeSelected);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.keystroke.key == "escape" {
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let current_theme = cx.global::<ThemeManager>().current_theme.clone();

        let measured_top = self.measured_top.clone();
        let measured_bottom = self.measured_bottom.clone();

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2p5()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_8()
                            .h_8()
                            .rounded_xl()
                            .bg(theme.accent().opacity(0.15))
                            .child(
                                svg()
                                    .path("sparkles.svg")
                                    .w_4()
                                    .h_4()
                                    .text_color(theme.accent()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground())
                                    .child("Temas"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.foreground_muted())
                                    .child(format!("{} disponibles", self.themes.len())),
                            ),
                    ),
            )
            .child(
                div()
                    .id("create-theme-btn")
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_3()
                    .py_1p5()
                    .rounded_xl()
                    .bg(theme.accent())
                    .cursor_pointer()
                    .on_click(cx.listener(|_this, _, _, cx| {
                        cx.emit(SelectThemeEvent::CreateThemeRequested);
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme.background())
                            .child("+ Crear"),
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
            let is_active = item.theme.accent_color.hex == current_theme.accent_color.hex
                && item.theme.background_color.hex == current_theme.background_color.hex;

            let item_theme = item.theme.clone();

            let card_bg = if is_active {
                theme.surface()
            } else {
                theme.background_alt()
            };

            let active_border = if is_active {
                theme.accent()
            } else {
                theme.surface().opacity(0.4)
            };

            let active_badge = if is_active {
                div()
                    .px_2p5()
                    .py_1()
                    .rounded_full()
                    .bg(theme.accent())
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.background())
                    .child("Activo")
            } else {
                div()
            };

            let mode_label = match item.theme.mode {
                ui::theme::ThemeMode::Dark => "Oscuro",
                ui::theme::ThemeMode::Light => "Claro",
            };

            let card = div()
                .id(format!("theme-item-{}", item.name))
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .rounded_xl()
                .bg(card_bg)
                .border_1()
                .border_color(active_border)
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.select_theme(item_theme.clone(), cx);
                }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1p5()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.foreground())
                                        .child(item.name.clone()),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(theme.background())
                                        .text_xs()
                                        .text_color(theme.foreground_muted())
                                        .child(mode_label),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .size_3p5()
                                        .rounded_full()
                                        .bg(item.theme.background())
                                        .border_1()
                                        .border_color(theme.surface()),
                                )
                                .child(
                                    div()
                                        .size_3p5()
                                        .rounded_full()
                                        .bg(item.theme.surface())
                                        .border_1()
                                        .border_color(theme.background()),
                                )
                                .child(div().size_3p5().rounded_full().bg(item.theme.accent()))
                                .child(div().size_3p5().rounded_full().bg(item.theme.green()))
                                .child(div().size_3p5().rounded_full().bg(item.theme.red())),
                        ),
                )
                .child(active_badge);

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
            .rounded(px(28.0))
            .overflow_hidden()
            .bg(theme.background())
            .child(
                canvas(
                    move |bounds, _window, _cx| {
                        measured_top.set(bounds.origin.y.into());
                    },
                    |_, _, _, _| {},
                )
                .w_full(),
            )
            .child(header)
            .child(divider)
            .child(theme_cards)
            .child(
                canvas(
                    move |bounds, _window, _cx| {
                        measured_bottom.set(bounds.origin.y.into());
                    },
                    |_, _, _, _| {},
                )
                .w_full(),
            )
    }
}
