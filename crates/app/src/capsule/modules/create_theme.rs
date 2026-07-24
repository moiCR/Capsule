use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, FontWeight, KeyDownEvent, Render, ScrollHandle,
    Window, div, prelude::*, px, svg,
};
use ui::theme::theme_manager::ThemeManager;
use ui::theme::{Color, Theme, ThemeMode, parse_hex_to_hsla};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreateThemeEvent {
    ThemeCreated,
    Cancelled,
}

pub struct CreateThemeModule {
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
    name: String,
    mode: ThemeMode,
    bg_color: String,
    bg_alt_color: String,
    surface_color: String,
    fg_color: String,
    fg_muted_color: String,
    accent_color: String,
    red_color: String,
    green_color: String,
    active_field: usize,
    is_error: bool,
    error_msg: String,
}

impl CreateThemeModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = ScrollHandle::new();
        let default_theme = Theme::default();

        Self {
            focus_handle,
            scroll_handle,
            name: "Nuevo Tema".to_string(),
            mode: ThemeMode::Dark,
            bg_color: default_theme.background_color.hex,
            bg_alt_color: default_theme.background_color_alt.hex,
            surface_color: default_theme.surface_color.hex,
            fg_color: default_theme.foreground_color.hex,
            fg_muted_color: default_theme.foreground_color_muted.hex,
            accent_color: "#007BFF".to_string(),
            red_color: "#FF3B30".to_string(),
            green_color: "#34C759".to_string(),
            active_field: 0,
            is_error: false,
            error_msg: String::new(),
        }
    }

    pub fn reset_form(&mut self, cx: &mut Context<Self>) {
        let default_theme = Theme::default();
        self.name = "Nuevo Tema".to_string();
        self.mode = ThemeMode::Dark;
        self.bg_color = default_theme.background_color.hex;
        self.bg_alt_color = default_theme.background_color_alt.hex;
        self.surface_color = default_theme.surface_color.hex;
        self.fg_color = default_theme.foreground_color.hex;
        self.fg_muted_color = default_theme.foreground_color_muted.hex;
        self.accent_color = "#007BFF".to_string();
        self.red_color = "#FF3B30".to_string();
        self.green_color = "#34C759".to_string();
        self.active_field = 0;
        self.is_error = false;
        self.error_msg.clear();
        cx.notify();
    }

    fn current_preview_theme(&self) -> Theme {
        Theme {
            mode: self.mode.clone(),
            background_color: Color::from(self.bg_color.clone()),
            background_color_alt: Color::from(self.bg_alt_color.clone()),
            surface_color: Color::from(self.surface_color.clone()),
            foreground_color: Color::from(self.fg_color.clone()),
            foreground_color_muted: Color::from(self.fg_muted_color.clone()),
            accent_color: Color::from(self.accent_color.clone()),
            red_color: Color::from(self.red_color.clone()),
            green_color: Color::from(self.green_color.clone()),
        }
    }

    fn save_theme(&mut self, cx: &mut Context<Self>) {
        let name = self.name.trim();
        if name.is_empty() {
            self.is_error = true;
            self.error_msg = "El nombre no puede estar vacío".to_string();
            cx.notify();
            return;
        }

        let new_theme = self.current_preview_theme();

        if let Ok(_) = ThemeManager::create_theme(name, &new_theme) {
            if cx.has_global::<ThemeManager>() {
                cx.global_mut::<ThemeManager>().set_theme(new_theme.clone());
                cx.set_global(new_theme);
            }
            cx.emit(CreateThemeEvent::ThemeCreated);
        } else {
            self.is_error = true;
            self.error_msg = "Error al guardar el tema".to_string();
            cx.notify();
        }
    }

    fn active_field_mut(&mut self) -> Option<&mut String> {
        match self.active_field {
            0 => Some(&mut self.name),
            1 => Some(&mut self.bg_color),
            2 => Some(&mut self.bg_alt_color),
            3 => Some(&mut self.surface_color),
            4 => Some(&mut self.fg_color),
            5 => Some(&mut self.fg_muted_color),
            6 => Some(&mut self.accent_color),
            7 => Some(&mut self.red_color),
            8 => Some(&mut self.green_color),
            _ => None,
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        match key {
            "tab" => {
                if event.keystroke.modifiers.shift {
                    if self.active_field > 0 {
                        self.active_field -= 1;
                    } else {
                        self.active_field = 8;
                    }
                } else {
                    if self.active_field < 8 {
                        self.active_field += 1;
                    } else {
                        self.active_field = 0;
                    }
                }
                cx.notify();
            }
            "enter" => self.save_theme(cx),
            "escape" => cx.emit(CreateThemeEvent::Cancelled),
            "backspace" => {
                if let Some(target) = self.active_field_mut() {
                    if !target.is_empty() {
                        target.pop();
                        self.is_error = false;
                        cx.notify();
                    }
                }
            }
            _ => {
                if let Some(keystroke_text) = &event.keystroke.key_char {
                    if !keystroke_text.chars().any(|c| c.is_control()) {
                        if let Some(target) = self.active_field_mut() {
                            target.push_str(keystroke_text);
                            self.is_error = false;
                            cx.notify();
                        }
                    }
                }
            }
        }
    }
}

impl EventEmitter<CreateThemeEvent> for CreateThemeModule {}

impl Focusable for CreateThemeModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CreateThemeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let preview = self.current_preview_theme();

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
                                    .child("Crear Tema"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.foreground_muted())
                                    .child("Personaliza los colores"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(
                        div()
                            .id("cancel-create-theme-btn")
                            .px_2p5()
                            .py_1p5()
                            .rounded_xl()
                            .bg(theme.background_alt())
                            .cursor_pointer()
                            .on_click(cx.listener(|_this, _, _, cx| {
                                cx.emit(CreateThemeEvent::Cancelled);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.foreground_muted())
                                    .child("Cancelar"),
                            ),
                    )
                    .child(
                        div()
                            .id("save-theme-btn")
                            .px_3()
                            .py_1p5()
                            .rounded_xl()
                            .bg(theme.accent())
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.save_theme(cx);
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.background())
                                    .child("Guardar"),
                            ),
                    ),
            );

        let divider = div().w_full().h(px(1.0)).bg(theme.background_alt());

        let visualizer_card = div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .rounded_xl()
            .bg(preview.background())
            .border_1()
            .border_color(preview.surface())
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(preview.foreground())
                            .child("Vista Previa (Capsule)"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_full()
                            .bg(preview.accent())
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(preview.background())
                            .child(match preview.mode {
                                ThemeMode::Dark => "Oscuro",
                                ThemeMode::Light => "Claro",
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .rounded_lg()
                    .bg(preview.background_alt())
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(preview.foreground())
                            .child("Reproductor de audio"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(preview.foreground_muted())
                            .child("03:12"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_lg()
                            .bg(preview.surface())
                            .text_xs()
                            .text_color(preview.foreground())
                            .child("Superficie"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(preview.green())
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(preview.background())
                                    .child("OK"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_full()
                                    .bg(preview.red())
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(preview.background())
                                    .child("ERR"),
                            ),
                    ),
            );

        let fields_config = [
            ("Nombre", &self.name, 0),
            ("Color Fondo (BG)", &self.bg_color, 1),
            ("Fondo Alt", &self.bg_alt_color, 2),
            ("Superficie", &self.surface_color, 3),
            ("Texto (FG)", &self.fg_color, 4),
            ("Texto Secundario", &self.fg_muted_color, 5),
            ("Color Acento", &self.accent_color, 6),
            ("Rojo", &self.red_color, 7),
            ("Verde", &self.green_color, 8),
        ];

        let mut fields_grid = div().grid().grid_cols(2).gap_2().w_full();

        for (label, val, idx) in fields_config {
            let is_focused = self.active_field == idx;

            let field_bg = if is_focused {
                theme.surface()
            } else {
                theme.background_alt()
            };

            let border_c = if is_focused {
                theme.accent()
            } else {
                theme.surface().opacity(0.5)
            };

            let swatch_c = parse_hex_to_hsla(val);

            let field_el = div()
                .id(format!("field-idx-{idx}"))
                .flex()
                .items_center()
                .justify_between()
                .px_2p5()
                .py_1p5()
                .rounded_xl()
                .bg(field_bg)
                .border_1()
                .border_color(border_c)
                .cursor_text()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.active_field = idx;
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .child(
                            div()
                                .text_xs()
                                .text_color(if is_focused {
                                    theme.accent()
                                } else {
                                    theme.foreground_muted()
                                })
                                .child(label),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(if val.is_empty() {
                                    "|".to_string()
                                } else if is_focused {
                                    format!("{val}|")
                                } else {
                                    val.to_string()
                                }),
                        ),
                )
                .child(if idx > 0 {
                    div()
                        .size_3p5()
                        .rounded_full()
                        .bg(swatch_c)
                        .border_1()
                        .border_color(theme.surface())
                } else {
                    div()
                });

            fields_grid = fields_grid.child(field_el);
        }

        let mode_selector = div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .py_1p5()
            .rounded_xl()
            .bg(theme.background_alt())
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted())
                    .child("Modo del Tema"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(
                        div()
                            .id("mode-dark-btn")
                            .px_2p5()
                            .py_1()
                            .rounded_lg()
                            .bg(match self.mode {
                                ThemeMode::Dark => theme.accent(),
                                ThemeMode::Light => theme.surface(),
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.mode = ThemeMode::Dark;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(match self.mode {
                                        ThemeMode::Dark => theme.background(),
                                        ThemeMode::Light => theme.foreground(),
                                    })
                                    .child("Oscuro"),
                            ),
                    )
                    .child(
                        div()
                            .id("mode-light-btn")
                            .px_2p5()
                            .py_1()
                            .rounded_lg()
                            .bg(match self.mode {
                                ThemeMode::Light => theme.accent(),
                                ThemeMode::Dark => theme.surface(),
                            })
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.mode = ThemeMode::Light;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(match self.mode {
                                        ThemeMode::Light => theme.background(),
                                        ThemeMode::Dark => theme.foreground(),
                                    })
                                    .child("Claro"),
                            ),
                    ),
            );

        let error_banner = if self.is_error {
            div()
                .px_3()
                .py_1p5()
                .rounded_xl()
                .bg(theme.red())
                .text_xs()
                .text_color(theme.background())
                .child(self.error_msg.clone())
        } else {
            div()
        };

        let scrollable_content = div()
            .id("create-theme-scroll-body")
            .track_scroll(&self.scroll_handle)
            .flex()
            .flex_col()
            .gap_2p5()
            .w_full()
            .flex_1()
            .overflow_scroll()
            .child(error_banner)
            .child(mode_selector)
            .child(visualizer_card)
            .child(fields_grid);

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .w(px(348.0))
            .h(px(500.0))
            .p_4()
            .gap_3p5()
            .rounded(px(28.0))
            .overflow_hidden()
            .bg(theme.background())
            .child(header)
            .child(divider)
            .child(scrollable_content)
    }
}
