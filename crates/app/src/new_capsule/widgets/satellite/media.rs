use crate::new_capsule::{
    module::dashboard::DashboardModule,
    widgets::dashboard::{DashboardAction, button, empty},
};
use gpui::{AnyElement, Context, IntoElement, div, prelude::*, px};
use ui::theme::Theme;

pub(crate) fn render(
    module: &DashboardModule,
    theme: &Theme,
    cx: &mut Context<DashboardModule>,
) -> AnyElement {
    let mut list = div()
        .id("satellite-players")
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .overflow_y_scroll();
    if module.snapshot.players.is_empty() {
        list = list.child(empty(module.text("dashboard.no_media", cx), theme));
    }
    for player in &module.snapshot.players {
        list = list.child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .gap(px(12.0))
                .child(super::super::dashboard::media::artwork(player, 48.0, theme))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(button(
                            format!("satellite-player-{}", player.bus_name),
                            player.player_name.clone(),
                            "music.svg",
                            DashboardAction::Player(player.bus_name.clone()),
                            module.player_bus.as_deref() == Some(&player.bus_name),
                            theme,
                            cx,
                        ))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.foreground_muted())
                                .text_ellipsis()
                                .child(player.title.clone()),
                        ),
                ),
        );
    }
    list.into_any_element()
}
