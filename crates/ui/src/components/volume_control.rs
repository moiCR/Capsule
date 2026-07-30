use crate::theme::Theme;
use gpui::{
    App, DefiniteLength, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, svg,
};

#[derive(IntoElement)]
pub struct VolumeControlBar {
    volume: u32,
    is_muted: bool,
}

impl VolumeControlBar {
    pub fn new(volume: u32, is_muted: bool) -> Self {
        Self { volume, is_muted }
    }
}

impl RenderOnce for VolumeControlBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let current_vol = self.volume.min(100);
        let icon_path = if self.is_muted {
            "bell-off.svg"
        } else {
            "music.svg"
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w(px(280.0))
            .px_4()
            .py_2()
            .gap_3()
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
                            .w(DefiniteLength::Fraction(current_vol as f32 / 100.0))
                            .bg(if self.is_muted {
                                theme.foreground_muted()
                            } else {
                                theme.accent()
                            }),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(gpui::FontWeight::BOLD)
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
