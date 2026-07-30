use gpui::IntoElement;
use ui::components::volume_control::VolumeControlBar;
use ui::theme::Theme;

pub fn render_volume_bar(volume: u32, is_muted: bool, _theme: &Theme) -> impl IntoElement {
    VolumeControlBar::new(volume, is_muted)
}
