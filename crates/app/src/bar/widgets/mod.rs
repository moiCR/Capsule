pub mod active_window;
pub mod caffeine;
pub mod clipboard;
pub mod clock;
pub mod keyboard;
pub mod media;
pub mod notifications;
pub mod power;
pub mod record;
pub mod resources;
pub mod status;
pub mod tray;
pub mod weather;
pub mod workspaces;

use std::cell::Cell;
use std::rc::Rc;

use gpui::{AnyElement, Context, IntoElement, Pixels, Styled, canvas, px};
use services::{BarWidgetId, WorkspaceInfo};
use ui::theme::Theme;

use crate::bar::Bar;

pub fn track_anchor() -> (Rc<Cell<Pixels>>, impl IntoElement) {
    let center = Rc::new(Cell::new(px(0.0)));
    let center_track = center.clone();
    let tracker = canvas(
        move |bounds, _, _| {
            center_track.set(bounds.origin.x + bounds.size.width / 2.0);
        },
        |_, _, _, _| {},
    )
    .absolute()
    .size_full();
    (center, tracker)
}

pub fn render_widget(
    id: BarWidgetId,
    current: &WorkspaceInfo,
    workspaces: &[WorkspaceInfo],
    clock_center: &Rc<Cell<Pixels>>,
    status_center: &Rc<Cell<Pixels>>,
    theme: &Theme,
    cx: &mut Context<Bar>,
) -> AnyElement {
    match id {
        BarWidgetId::Workspaces => workspaces::render(current, workspaces, theme, cx),
        BarWidgetId::Clock => clock::render(clock_center, theme, cx),
        BarWidgetId::Media => media::render(theme, cx),
        BarWidgetId::Status => status::render(status_center, theme, cx),
        BarWidgetId::Tray => tray::render(theme, cx),
        BarWidgetId::Power => power::render(theme, cx),
        BarWidgetId::Resources => resources::render(theme, cx),
        BarWidgetId::Record => record::render(theme, cx),
        BarWidgetId::Clipboard => clipboard::render(theme, cx),
        BarWidgetId::Caffeine => caffeine::render(theme, cx),
        BarWidgetId::ActiveWindow => active_window::render(theme, cx),
        BarWidgetId::Keyboard => keyboard::render(theme, cx),
        BarWidgetId::Notifications => notifications::render(theme, cx),
        BarWidgetId::Weather => weather::render(theme, cx),
    }
}
