use gpui::{IntoElement, SharedString};
use ui::components::volume_control::VolumeControlBar;
use ui::theme::Theme;

pub fn render_volume_bar(
    volume: f32,
    is_muted: bool,
    muted_label: impl Into<SharedString>,
    _theme: &Theme,
) -> impl IntoElement {
    VolumeControlBar::new(volume, is_muted).with_muted_label(muted_label)
}
