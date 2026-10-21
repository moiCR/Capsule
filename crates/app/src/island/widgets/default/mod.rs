mod clock;
mod status;
mod workspaces;

pub use clock::render_clock;
pub use status::{StatusSnapshot, render_status};
pub use workspaces::{WorkspaceKey, render_workspaces, workspace_keys};

pub const WORKSPACES_WIDTH: f32 = 76.0;
pub const CLOCK_WIDTH: f32 = 42.0;
pub const STATUS_WIDTH: f32 = WORKSPACES_WIDTH;
pub const PADDING: f32 = 16.0;
pub const GAP: f32 = 12.0;
pub const WIDTH: f32 = WORKSPACES_WIDTH + CLOCK_WIDTH + STATUS_WIDTH + PADDING * 2.0 + GAP * 2.0;

pub fn module_size(height: f32) -> gpui::Size<gpui::Pixels> {
    gpui::size(gpui::px(WIDTH), gpui::px(height.max(40.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_match_the_three_zones_and_configured_height() {
        assert_eq!(WIDTH, 250.0);
        assert_eq!(WORKSPACES_WIDTH, STATUS_WIDTH);
        assert_eq!(
            PADDING + WORKSPACES_WIDTH + GAP + CLOCK_WIDTH / 2.0,
            WIDTH / 2.0
        );
        for height in [16.0, 25.0, 60.0] {
            assert_eq!(
                module_size(height),
                gpui::size(gpui::px(250.0), gpui::px(height.max(40.0)))
            );
        }
    }
}
