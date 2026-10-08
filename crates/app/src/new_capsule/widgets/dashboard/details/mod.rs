mod appearance;
mod audio;
mod calendar;
mod connectivity;
mod tray;
use super::{BODY, DashboardView as View, error};
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use ui::theme::Theme;
pub fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = div()
        .id("dashboard-detail")
        .h(px(BODY))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .overflow_hidden();
    if let Some(message) = &module.navigation.error {
        content = content.child(error(message.clone(), theme));
    }
    if module.pending {
        content = content.child(empty(module.text("dashboard_new.pending", cx), theme));
    }
    let body = match &module.navigation.view {
        View::Wifi => connectivity::wifi(module, theme, cx),
        View::Bluetooth => connectivity::bluetooth(module, theme, cx),
        View::Audio => audio::audio(module, theme, cx),
        View::Calendar => calendar::calendar(module, theme, cx),
        View::Tray { bus, path } => tray::tray(bus, path, module, theme, cx),
        View::Themes => appearance::themes(module, theme, cx),
        View::Wallpapers => appearance::wallpapers(module, theme, cx),
        View::Home => div().into_any_element(),
    };
    content.child(body).into_any_element()
}
pub(super) fn list(id: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .flex_1()
        .min_h_0()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .overflow_y_scroll()
}
