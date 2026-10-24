use gpui::prelude::*;
use gpui::{
    AnyElement, App, ColorExt, Context, DragMoveEvent, ElementId, FontWeight, IntoElement,
    MouseMoveEvent, MouseUpEvent, ParentElement, Point, Render, Styled, Window, div, px, svg,
};
use services::{AppState, BarConfig, BarWidgetId, BarZone};
use ui::theme::Theme;

use super::SettingsView;
use super::common::{card, hero_header, toggle_row};
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraggedBarWidget {
    pub id: BarWidgetId,
    pub from_zone: Option<BarZone>,
}

pub struct DraggedWidgetPreview {
    pub id: BarWidgetId,
}

impl Render for DraggedWidgetPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        render_mini_widget_visual(self.id, &theme)
            .px(px(10.0))
            .py(px(6.0))
            .rounded(px(10.0))
            .bg(theme.surface().opacity(0.95))
            .border_1()
            .border_color(theme.accent())
            .shadow_lg()
    }
}

fn render_mini_widget_visual(id: BarWidgetId, theme: &Theme) -> gpui::Div {
    match id {
        BarWidgetId::Workspaces => div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .child(
                svg()
                    .path("pacman.svg")
                    .size(px(15.0))
                    .text_color(theme.accent()),
            )
            .child(
                svg()
                    .path("ghost.svg")
                    .size(px(14.0))
                    .text_color(theme.foreground().opacity(0.85)),
            )
            .child(
                div()
                    .size(px(5.0))
                    .rounded_full()
                    .bg(theme.foreground_muted().opacity(0.45)),
            ),
        BarWidgetId::Clock => div().flex().items_center().child(
            div()
                .text_size(px(12.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child("12:34"),
        ),
        BarWidgetId::Media => div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child("Song title"),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.foreground_muted())
                    .child("• Artist"),
            ),
        BarWidgetId::Status => div()
            .flex()
            .items_center()
            .gap(px(7.0))
            .child(
                svg()
                    .path("wifi-high.svg")
                    .size(px(14.0))
                    .text_color(theme.foreground()),
            )
            .child(
                svg()
                    .path("volume-2.svg")
                    .size(px(14.0))
                    .text_color(theme.foreground()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        svg()
                            .path("battery-full.svg")
                            .size(px(14.0))
                            .text_color(theme.green()),
                    )
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child("76%"),
                    ),
            ),
        BarWidgetId::Tray => div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                svg()
                    .path("bluetooth.svg")
                    .size(px(13.5))
                    .text_color(theme.accent()),
            )
            .child(
                svg()
                    .path("bell.svg")
                    .size(px(13.5))
                    .text_color(theme.foreground()),
            )
            .child(
                svg()
                    .path("cpu.svg")
                    .size(px(13.5))
                    .text_color(theme.foreground_muted()),
            ),
        BarWidgetId::Power => div().flex().items_center().child(
            svg()
                .path("power.svg")
                .size(px(14.5))
                .text_color(theme.red()),
        ),
    }
}

pub struct BarView {
    hovered_zone: Option<BarZone>,
    hovered_palette: bool,
    selected_zone: BarZone,
}

impl BarView {
    pub fn new() -> Self {
        Self {
            hovered_zone: None,
            hovered_palette: false,
            selected_zone: BarZone::Right,
        }
    }

    fn place_widget(widget: BarWidgetId, zone: BarZone, cx: &mut Context<Settings>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let mut cfg = (*state.config.get()).clone();
            cfg.ui.bar.add_widget_to_zone(widget, zone);
            let _ = state.config.set(cfg);
            cx.notify();
        }
    }

    fn remove_widget(widget: BarWidgetId, cx: &mut Context<Settings>) {
        if let Some(state) = cx.try_global::<AppState>() {
            let mut cfg = (*state.config.get()).clone();
            cfg.ui.bar.remove_widget(widget);
            let _ = state.config.set(cfg);
            cx.notify();
        }
    }
}

impl Default for BarView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView for BarView {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn handle_mouse_move(&mut self, _event: &MouseMoveEvent, _cx: &mut Context<Settings>) -> bool {
        false
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _cx: &mut Context<Settings>) -> bool {
        false
    }

    fn render(&mut self, _window: &mut Window, cx: &mut Context<Settings>) -> AnyElement {
        let theme = cx.global::<Theme>().clone();
        let bar_config = if let Some(state) = cx.try_global::<AppState>() {
            state.config.get().ui.bar.clone()
        } else {
            BarConfig::default()
        };

        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .w_full()
            .min_w_0()
            .child(hero_header(
                "bar.svg",
                &text("settings.tab_bar", cx),
                &text("settings.bar_subtitle", cx),
                &theme,
            ))
            .child(card(&theme).child(toggle_row(
                ElementId::Name("setting-convex-bar".into()),
                &text("settings.convex_bar", cx),
                Some(&text("settings.convex_bar_desc", cx)),
                bar_config.convex,
                &theme,
                cx.listener(|_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        let _ = cx.global::<AppState>().config.update(|cfg| {
                            cfg.ui.bar.convex = !cfg.ui.bar.convex;
                        });
                    }
                    cx.notify();
                }),
            )))
            .child(self.render_unified_bar_canvas(&bar_config, &theme, cx))
            .into_any_element()
    }
}

impl BarView {
    fn render_unified_bar_canvas(
        &self,
        bar_config: &BarConfig,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> impl IntoElement {
        let mut palette_row = div()
            .id("bar-widget-palette")
            .w_full()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(10.0))
            .p(px(10.0))
            .rounded(px(14.0))
            .bg(if self.hovered_palette {
                theme.red().opacity(0.08)
            } else {
                theme.background().opacity(0.35)
            })
            .border_1()
            .border_color(if self.hovered_palette {
                theme.red().opacity(0.5)
            } else {
                theme.surface().opacity(0.35)
            })
            .on_drag_move::<DraggedBarWidget>(cx.listener(
                move |settings, _event: &DragMoveEvent<DraggedBarWidget>, _, cx| {
                    if let Some(bar_view) = settings.view_as_mut::<BarView>()
                        && !bar_view.hovered_palette
                    {
                        bar_view.hovered_palette = true;
                        bar_view.hovered_zone = None;
                        cx.notify();
                    }
                },
            ))
            .on_drop(
                cx.listener(move |settings, dragged: &DraggedBarWidget, _, cx| {
                    if let Some(bar_view) = settings.view_as_mut::<BarView>() {
                        bar_view.hovered_palette = false;
                    }
                    Self::remove_widget(dragged.id, cx);
                }),
            );

        for id in BarWidgetId::ALL {
            let zone = bar_config.zone_of(id);
            palette_row = palette_row.child(self.render_palette_pill(id, zone, theme, cx));
        }

        let bar_bg = theme.background();
        let bar_border = theme.surface().opacity(0.65);
        let convex = bar_config.convex;

        let live_bar_strip = div()
            .w_full()
            .h(px(44.0))
            .px(px(14.0))
            .bg(bar_bg)
            .border_t_1()
            .border_l_1()
            .border_r_1()
            .when(!convex, |d| d.border_b_1().rounded(px(12.0)))
            .when(convex, |d| d.rounded_t(px(12.0)))
            .border_color(bar_border)
            .shadow_lg()
            .flex()
            .items_center()
            .justify_between()
            .gap(px(12.0))
            .child(self.render_bar_zone_slot(
                BarZone::Left,
                &bar_config.left,
                text("settings.bar_zone_left", cx),
                theme,
                cx,
            ))
            .child(self.render_bar_zone_slot(
                BarZone::Center,
                &bar_config.center,
                text("settings.bar_zone_center", cx),
                theme,
                cx,
            ))
            .child(self.render_bar_zone_slot(
                BarZone::Right,
                &bar_config.right,
                text("settings.bar_zone_right", cx),
                theme,
                cx,
            ));

        let bar_mockup = if convex {
            let r = 14.0;
            div()
                .w_full()
                .flex()
                .flex_col()
                .child(live_bar_strip)
                .child(
                    div()
                        .w_full()
                        .h(px(r))
                        .relative()
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left(px(r))
                                .right(px(r))
                                .h(px(1.0))
                                .bg(bar_border),
                        )
                        .child(div().absolute().top_0().left_0().size(px(r)).child(
                            crate::bar::shared_panel::view::convex_bar_ear_canvas(
                                r, false, false, bar_bg, bar_border,
                            ),
                        ))
                        .child(div().absolute().top_0().right_0().size(px(r)).child(
                            crate::bar::shared_panel::view::convex_bar_ear_canvas(
                                r, true, false, bar_bg, bar_border,
                            ),
                        )),
                )
                .into_any_element()
        } else {
            live_bar_strip.into_any_element()
        };

        let zone_selector = div().flex().items_center().gap(px(6.0)).children(
            [
                (BarZone::Left, text("settings.bar_zone_left", cx)),
                (BarZone::Center, text("settings.bar_zone_center", cx)),
                (BarZone::Right, text("settings.bar_zone_right", cx)),
            ]
            .into_iter()
            .map(|(z, lbl)| {
                let active = self.selected_zone == z;
                div()
                    .id(format!("bar-target-zone-btn-{:?}", z))
                    .h(px(26.0))
                    .px(px(10.0))
                    .rounded(px(8.0))
                    .bg(if active {
                        theme.accent().opacity(0.2)
                    } else {
                        theme.surface().opacity(0.35)
                    })
                    .border_1()
                    .border_color(if active {
                        theme.accent().opacity(0.6)
                    } else {
                        theme.surface().opacity(0.3)
                    })
                    .flex()
                    .items_center()
                    .text_size(px(11.5))
                    .font_weight(if active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(if active {
                        theme.accent()
                    } else {
                        theme.foreground_muted()
                    })
                    .cursor_pointer()
                    .on_click(cx.listener(move |settings, _, _, cx| {
                        if let Some(bar_view) = settings.view_as_mut::<BarView>() {
                            bar_view.selected_zone = z;
                            cx.notify();
                        }
                    }))
                    .child(lbl)
            }),
        );

        card(theme)
            .p(px(22.0))
            .flex()
            .flex_col()
            .gap(px(24.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(14.5))
                                    .text_color(theme.foreground())
                                    .child(text("settings.bar_preview_title", cx)),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(theme.foreground_muted())
                                    .child(text("settings.bar_preview_desc", cx)),
                            ),
                    )
                    .child(zone_selector),
            )
            .child(
                div()
                    .w_full()
                    .rounded(px(18.0))
                    .bg(theme.background().opacity(0.55))
                    .border_1()
                    .border_color(theme.surface().opacity(0.45))
                    .p(px(20.0))
                    .flex()
                    .flex_col()
                    .gap(px(36.0))
                    .child(palette_row)
                    .child(
                        div()
                            .w_full()
                            .py(px(10.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(bar_mockup),
                    ),
            )
    }

    fn render_palette_pill(
        &self,
        id: BarWidgetId,
        placed_zone: Option<BarZone>,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> impl IntoElement {
        let is_placed = placed_zone.is_some();
        let target_zone = self.selected_zone;
        let payload = DraggedBarWidget {
            id,
            from_zone: placed_zone,
        };

        div()
            .id(format!("palette-widget-pill-{:?}", id))
            .h(px(34.0))
            .px(px(12.0))
            .rounded(px(10.0))
            .bg(if is_placed {
                theme.surface().opacity(0.18)
            } else {
                theme.surface().opacity(0.65)
            })
            .border_1()
            .border_color(if is_placed {
                theme.surface().opacity(0.25)
            } else {
                theme.surface().opacity(0.6)
            })
            .opacity(if is_placed { 0.42 } else { 1.0 })
            .flex()
            .items_center()
            .cursor_pointer()
            .hover(|s| s.opacity(0.95).border_color(theme.accent().opacity(0.65)))
            .on_drag(
                payload,
                move |dragged: &DraggedBarWidget, _: Point<gpui::Pixels>, _, cx| {
                    let id = dragged.id;
                    cx.new(|_| DraggedWidgetPreview { id })
                },
            )
            .on_click(cx.listener(move |_, _, _, cx| {
                if is_placed {
                    Self::remove_widget(id, cx);
                } else {
                    Self::place_widget(id, target_zone, cx);
                }
            }))
            .child(render_mini_widget_visual(id, theme))
    }

    fn render_bar_zone_slot(
        &self,
        zone: BarZone,
        widgets: &[BarWidgetId],
        label: String,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> impl IntoElement {
        let is_hovered = self.hovered_zone == Some(zone);
        let is_selected = self.selected_zone == zone;

        let mut slot = div()
            .id(format!("bar-live-zone-{:?}", zone))
            .h(px(32.0))
            .px(px(8.0))
            .rounded(px(8.0))
            .bg(if is_hovered {
                theme.accent().opacity(0.18)
            } else if is_selected {
                theme.surface().opacity(0.28)
            } else {
                gpui::transparent_black()
            })
            .border_1()
            .border_color(if is_hovered {
                theme.accent()
            } else if is_selected {
                theme.accent().opacity(0.35)
            } else {
                gpui::transparent_black()
            })
            .flex()
            .items_center()
            .gap(px(10.0))
            .when(zone == BarZone::Left, |d| d.flex_1().justify_start())
            .when(zone == BarZone::Center, |d| {
                d.flex_shrink_0().justify_center().min_w(px(120.0))
            })
            .when(zone == BarZone::Right, |d| d.flex_1().justify_end())
            .on_click(cx.listener(move |settings, _, _, cx| {
                if let Some(bar_view) = settings.view_as_mut::<BarView>() {
                    bar_view.selected_zone = zone;
                    cx.notify();
                }
            }))
            .on_drag_move::<DraggedBarWidget>(cx.listener(
                move |settings, _event: &DragMoveEvent<DraggedBarWidget>, _, cx| {
                    if let Some(bar_view) = settings.view_as_mut::<BarView>()
                        && (bar_view.hovered_zone != Some(zone) || bar_view.hovered_palette)
                    {
                        bar_view.hovered_zone = Some(zone);
                        bar_view.hovered_palette = false;
                        cx.notify();
                    }
                },
            ))
            .on_drop(
                cx.listener(move |settings, dragged: &DraggedBarWidget, _, cx| {
                    if let Some(bar_view) = settings.view_as_mut::<BarView>() {
                        bar_view.hovered_zone = None;
                        bar_view.hovered_palette = false;
                        bar_view.selected_zone = zone;
                    }
                    Self::place_widget(dragged.id, zone, cx);
                }),
            );

        if widgets.is_empty() {
            slot = slot.child(
                div()
                    .text_size(px(11.0))
                    .text_color(theme.foreground_muted().opacity(0.45))
                    .child(label),
            );
        } else {
            for &w_id in widgets {
                let payload = DraggedBarWidget {
                    id: w_id,
                    from_zone: Some(zone),
                };
                let hover_bg = theme.red().opacity(0.16);
                let item = div()
                    .id(format!("live-bar-item-{:?}-{:?}", zone, w_id))
                    .h(px(26.0))
                    .px(px(8.0))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_drag(
                        payload,
                        move |dragged: &DraggedBarWidget, _: Point<gpui::Pixels>, _, cx| {
                            let id = dragged.id;
                            cx.new(|_| DraggedWidgetPreview { id })
                        },
                    )
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.stop_propagation();
                        Self::remove_widget(w_id, cx);
                    }))
                    .child(render_mini_widget_visual(w_id, theme));
                slot = slot.child(item);
            }
        }

        slot
    }
}
