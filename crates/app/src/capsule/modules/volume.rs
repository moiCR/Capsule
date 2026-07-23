use gpui::{Context, FontWeight, IntoElement, Render, Window, div, prelude::*, px, svg};
use ui::theme::Theme;

pub struct VolumeModule {
    pub volume: u32,
    pub is_muted: bool,
}

impl VolumeModule {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            volume: 50,
            is_muted: false,
        }
    }

    pub fn update_status(&mut self, volume: u32, is_muted: bool, cx: &mut Context<Self>) {
        if self.volume != volume || self.is_muted != is_muted {
            self.volume = volume;
            self.is_muted = is_muted;
            cx.notify();
        }
    }
}

impl Render for VolumeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let icon_path = if self.is_muted {
            "bell-off.svg"
        } else {
            "music.svg"
        };

        let current_vol = self.volume.min(100);

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .size_full()
            .px_4()
            .py_2()
            .gap_3()
            // Audio Icon
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .w_6()
                    .h_6()
                    .rounded_md()
                    .bg(if self.is_muted {
                        theme.red_color.to_hsla()
                    } else {
                        theme.background_alt()
                    })
                    .child(
                        svg()
                            .path(icon_path)
                            .w_3p5()
                            .h_3p5()
                            .text_color(theme.foreground()),
                    ),
            )
            // Volume Fill Bar
            .child(
                div()
                    .flex_1()
                    .h(px(8.0))
                    .rounded_full()
                    .bg(theme.background_alt())
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .w(gpui::DefiniteLength::Fraction(current_vol as f32 / 100.0))
                            .bg(if self.is_muted {
                                theme.foreground_muted()
                            } else {
                                theme.accent()
                            }),
                    ),
            )
            // Percentage Label
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(if self.is_muted {
                        theme.red_color.to_hsla()
                    } else {
                        theme.foreground()
                    })
                    .child(if self.is_muted {
                        "Mute".to_string()
                    } else {
                        format!("{current_vol}%")
                    }),
            )
    }
}
