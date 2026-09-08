use crate::theme::Theme;
use gpui::{
    App, DefiniteLength, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, svg,
};

#[derive(IntoElement)]
pub struct VolumeControlBar {
    volume: f32,
    is_muted: bool,
    muted_label: Option<gpui::SharedString>,
}

impl VolumeControlBar {
    pub fn new(volume: f32, is_muted: bool) -> Self {
        Self {
            volume,
            is_muted,
            muted_label: None,
        }
    }

    pub fn with_muted_label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.muted_label = Some(label.into());
        self
    }
}

impl RenderOnce for VolumeControlBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let current_vol = self.volume.clamp(0.0, 100.0);
        let rounded_vol = current_vol.round() as u32;
        let icon_path = if self.is_muted || rounded_vol == 0 {
            "volume-x.svg"
        } else {
            "volume-2.svg"
        };

        let icon_color = if self.is_muted {
            theme.red()
        } else {
            theme.foreground().opacity(0.85)
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w(px(280.0))
            .h(px(42.0))
            .px(px(18.0))
            .gap(px(12.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(svg().path(icon_path).size(px(16.0)).text_color(icon_color)),
            )
            .child(
                div()
                    .flex_1()
                    .h(px(8.0))
                    .rounded_full()
                    .bg(theme.foreground().opacity(0.15))
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .w(DefiniteLength::Fraction(current_vol / 100.0))
                            .rounded_full()
                            .bg(if self.is_muted {
                                theme.foreground_muted().opacity(0.3)
                            } else {
                                theme.accent()
                            }),
                    ),
            )
            .child(
                div()
                    .min_w(px(34.0))
                    .text_right()
                    .font_family(theme.font_family())
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(if self.is_muted {
                        theme.red()
                    } else {
                        theme.foreground().opacity(0.9)
                    })
                    .flex_shrink_0()
                    .child(if self.is_muted {
                        self.muted_label.unwrap_or_else(|| "Mute".into())
                    } else {
                        format!("{rounded_vol}%").into()
                    }),
            )
    }
}
