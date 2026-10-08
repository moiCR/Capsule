mod app_tile;
mod search;

pub use app_tile::{render_app_tile, render_calculator_tile};
pub use search::render_search;

pub const COLUMNS: usize = 4;
pub const TILE_HEIGHT: f32 = 100.0;
pub const GRID_GAP: f32 = 8.0;
pub const WIDTH: f32 = 520.0;
pub const HEIGHT: f32 = 440.0;

pub fn next_selection(current: usize, count: usize, horizontal: i32, vertical: i32) -> usize {
    if count == 0 {
        return 0;
    }
    let candidate = current as i64 + horizontal as i64 + vertical as i64 * COLUMNS as i64;
    candidate.clamp(0, count as i64 - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_navigation_handles_partial_rows_and_empty_results() {
        assert_eq!(next_selection(0, 0, 1, 0), 0);
        assert_eq!(next_selection(0, 7, -1, 0), 0);
        assert_eq!(next_selection(1, 7, 0, 1), 5);
        assert_eq!(next_selection(5, 7, 0, -1), 1);
        assert_eq!(next_selection(3, 7, 0, 1), 6);
        assert_eq!(next_selection(6, 7, 1, 0), 6);
    }
}
