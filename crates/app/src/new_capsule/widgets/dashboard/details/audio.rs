use super::super::DashboardAction as Action;
use super::list;
use super::{button, empty};
use crate::new_capsule::module::dashboard::DashboardModule;
use gpui::{AnyElement, Context, IntoElement, prelude::*};
use ui::theme::Theme;
pub(super) fn audio(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut content = list("dashboard-audio-list");
    if module.snapshot.audio.audio_sinks.is_empty() {
        content = content.child(empty(module.text("dashboard_new.no_outputs", cx), theme));
    }
    for sink in &module.snapshot.audio.audio_sinks {
        content = content.child(button(
            format!("dashboard-sink-{}", sink.name),
            sink.description.clone(),
            "volume-2.svg",
            Action::Sink(sink.name.clone()),
            sink.is_default,
            theme,
            cx,
        ));
    }
    content.into_any_element()
}
