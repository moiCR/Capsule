pub mod general_section;
pub mod lockscreen_section;
pub mod record_section;
pub mod search_section;
pub mod setting_item;
pub mod sidebar;
pub mod ui_section;

pub use general_section::{
    render_apps_section, render_media_section, render_music_players_subsection,
    render_system_section,
};
pub use lockscreen_section::{render_lockscreen_formats_subsection, render_lockscreen_section};
pub use record_section::render_record_section;
pub use search_section::render_search_results;
pub use sidebar::render_sidebar;
pub use ui_section::render_ui_section;
