use gpui::prelude::*;
use gpui::{
    AnyElement, App, ColorExt, Context, ElementId, FontWeight, IntoElement, MouseMoveEvent,
    MouseUpEvent, ParentElement, Point, Render, Styled, Window, div, px, svg,
};
use services::{AppState, BarConfig, BarWidgetId, BarZone, CapsuleAnimation};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::SettingsView;
use super::common::{
    card, control_row, heading, hero_header, row_divider, segmented_selector, slider_row,
    toggle_row,
};
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarWidgetScale {
    pub scale: f32,
    pub icon_size: f32,
    pub font_size: f32,
    pub small_font_size: f32,
    pub pad_x: f32,
    pub item_height: f32,
    pub gap: f32,
    pub drop_gap: f32,
    pub drop_width: f32,
    pub compact_labels: bool,
}

impl Default for BarWidgetScale {
    fn default() -> Self {
        Self {
            scale: 1.0,
            icon_size: 14.0,
            font_size: 12.0,
            small_font_size: 11.0,
            pad_x: 8.0,
            item_height: 26.0,
            gap: 4.0,
            drop_gap: 2.0,
            drop_width: 6.0,
            compact_labels: false,
        }
    }
}

impl BarWidgetScale {
    pub fn compute(total: usize, max_zone: usize) -> Self {
        if max_zone >= 6 || total >= 12 {
            Self {
                scale: 0.72,
                icon_size: 11.0,
                font_size: 9.5,
                small_font_size: 8.5,
                pad_x: 4.0,
                item_height: 22.0,
                gap: 1.5,
                drop_gap: 1.0,
                drop_width: 4.0,
                compact_labels: true,
            }
        } else if max_zone >= 5 || total >= 9 {
            Self {
                scale: 0.82,
                icon_size: 12.0,
                font_size: 10.5,
                small_font_size: 9.5,
                pad_x: 5.0,
                item_height: 24.0,
                gap: 2.0,
                drop_gap: 1.5,
                drop_width: 5.0,
                compact_labels: true,
            }
        } else if max_zone >= 4 || total >= 7 {
            Self {
                scale: 0.90,
                icon_size: 13.0,
                font_size: 11.5,
                small_font_size: 10.0,
                pad_x: 6.5,
                item_height: 25.0,
                gap: 3.0,
                drop_gap: 2.0,
                drop_width: 6.0,
                compact_labels: false,
            }
        } else {
            Self::default()
        }
    }
}

pub struct DraggedWidgetPreview {
    pub id: BarWidgetId,
    pub scale: BarWidgetScale,
}

impl Render for DraggedWidgetPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        render_mini_widget_visual(self.id, &self.scale, &theme)
            .px(px(self.scale.pad_x + 3.0))
            .py(px(4.0))
            .rounded_full()
            .bg(theme.surface_container_high())
            .border_1()
            .border_color(theme.primary())
            .shadow_lg()
    }
}

fn render_mini_widget_visual(id: BarWidgetId, scale: &BarWidgetScale, theme: &Theme) -> gpui::Div {
    match id {
        BarWidgetId::Workspaces => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 5.0))
            .child(
                svg()
                    .path("pacman.svg")
                    .size(px(scale.icon_size + 1.0))
                    .text_color(theme.accent()),
            )
            .child(
                svg()
                    .path("ghost.svg")
                    .size(px(scale.icon_size))
                    .text_color(theme.foreground().opacity(0.85)),
            )
            .child(
                div()
                    .size(px((scale.scale * 5.0).max(3.0)))
                    .rounded_full()
                    .bg(theme.foreground_muted().opacity(0.45)),
            ),
        BarWidgetId::Clock => div().flex().items_center().child(
            div()
                .text_size(px(scale.font_size + 0.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child("12:34"),
        ),
        BarWidgetId::Media => {
            let mut media_row = div().flex().items_center().gap(px(scale.scale * 4.0));
            media_row = media_row.child(
                div()
                    .text_size(px(scale.font_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child(if scale.compact_labels {
                        "Song"
                    } else {
                        "Song title"
                    }),
            );
            if !scale.compact_labels {
                media_row = media_row.child(
                    div()
                        .text_size(px(scale.small_font_size))
                        .text_color(theme.foreground_muted())
                        .child("• Artist"),
                );
            }
            media_row
        }
        BarWidgetId::Status => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 6.0))
            .child(
                svg()
                    .path("wifi-high.svg")
                    .size(px(scale.icon_size))
                    .text_color(theme.foreground()),
            )
            .child(
                svg()
                    .path("volume-2.svg")
                    .size(px(scale.icon_size))
                    .text_color(theme.foreground()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(scale.scale * 3.5))
                    .child(
                        svg()
                            .path("battery-full.svg")
                            .size(px(scale.icon_size))
                            .text_color(theme.green()),
                    )
                    .child(
                        div()
                            .text_size(px(scale.small_font_size))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child("76%"),
                    ),
            ),
        BarWidgetId::Tray => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 5.0))
            .child(
                svg()
                    .path("bluetooth.svg")
                    .size(px(scale.icon_size - 0.5))
                    .text_color(theme.accent()),
            )
            .child(
                svg()
                    .path("bell.svg")
                    .size(px(scale.icon_size - 0.5))
                    .text_color(theme.foreground()),
            )
            .child(
                svg()
                    .path("cpu.svg")
                    .size(px(scale.icon_size - 0.5))
                    .text_color(theme.foreground_muted()),
            ),
        BarWidgetId::Power => div().flex().items_center().child(
            svg()
                .path("power.svg")
                .size(px(scale.icon_size + 0.5))
                .text_color(theme.red()),
        ),
        BarWidgetId::Resources => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 4.0))
            .child(
                svg()
                    .path("cpu.svg")
                    .size(px(scale.icon_size - 1.0))
                    .text_color(theme.accent()),
            )
            .child(
                div()
                    .text_size(px(scale.small_font_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child("24%"),
            ),
        BarWidgetId::Record => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 3.5))
            .child(
                svg()
                    .path("camera.svg")
                    .size(px(scale.icon_size - 1.0))
                    .text_color(theme.red()),
            )
            .child(
                div()
                    .text_size(px(scale.small_font_size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.red())
                    .child("REC"),
            ),
        BarWidgetId::Clipboard => div().flex().items_center().child(
            svg()
                .path("clipboard-list.svg")
                .size(px(scale.icon_size))
                .text_color(theme.foreground()),
        ),
        BarWidgetId::Caffeine => div().flex().items_center().child(
            svg()
                .path("coffee.svg")
                .size(px(scale.icon_size))
                .text_color(theme.accent()),
        ),
        BarWidgetId::ActiveWindow => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 4.0))
            .child(
                svg()
                    .path("monitor.svg")
                    .size(px(scale.icon_size - 1.0))
                    .text_color(theme.foreground_muted()),
            )
            .child(
                div()
                    .text_size(px(scale.small_font_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child(if scale.compact_labels {
                        "App"
                    } else {
                        "Active App"
                    }),
            ),
        BarWidgetId::Keyboard => div().flex().items_center().child(
            div()
                .text_size(px(scale.small_font_size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.foreground())
                .child("ES"),
        ),
        BarWidgetId::Notifications => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 4.0))
            .child(
                svg()
                    .path("bell.svg")
                    .size(px(scale.icon_size - 0.5))
                    .text_color(theme.foreground()),
            )
            .child(div().size(px((scale.scale * 5.0).max(3.0))).rounded_full().bg(theme.accent())),
        BarWidgetId::Weather => div()
            .flex()
            .items_center()
            .gap(px(scale.scale * 4.0))
            .child(
                svg()
                    .path("cloud-sun.svg")
                    .size(px(scale.icon_size - 0.5))
                    .text_color(theme.yellow()),
            )
            .child(
                div()
                    .text_size(px(scale.small_font_size))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground())
                    .child("22°C"),
            ),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BarSlider {
    AnimDuration,
    SpringBounciness,
}

pub struct BarView {
    active_slider: Option<(BarSlider, DimensionTracker, f32, f32, f32)>,
    anim_duration_tracker: DimensionTracker,
    spring_bounciness_tracker: DimensionTracker,
}

impl BarView {
    pub fn new() -> Self {
        Self {
            active_slider: None,
            anim_duration_tracker: DimensionTracker::new(),
            spring_bounciness_tracker: DimensionTracker::new(),
        }
    }

    fn slider_control_row(
        &self,
        id: &'static str,
        label: &str,
        value_text: &str,
        slider_kind: BarSlider,
        value: f32,
        range: (f32, f32, f32),
        tracker: &DimensionTracker,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> AnyElement {
        let (min, max, step) = range;
        let fraction = ((value - min) / (max - min)).clamp(0.0, 1.0);
        let tracker_clone = tracker.clone();
        slider_row(
            ElementId::Name(id.into()),
            label,
            value_text,
            fraction,
            tracker,
            theme,
            cx.listener(move |this, event: &gpui::MouseDownEvent, _, cx| {
                if let Some(view) = this.view_as_mut::<BarView>() {
                    view.active_slider = Some((slider_kind, tracker_clone.clone(), min, max, step));
                    view.update_slider_value(event.position.x.into(), cx);
                }
            }),
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if let Some(view) = this.view_as_mut::<BarView>() {
                    if view.handle_mouse_up(event, cx) {
                        cx.notify();
                    }
                }
            }),
        )
    }

    fn update_slider_value(&mut self, x: f32, cx: &mut Context<Settings>) {
        if let Some((slider, tracker, min, max, step)) = &self.active_slider {
            let left = tracker.left();
            let width = tracker.width(0.0);
            if width > 0.0 {
                let frac = ((x - left) / width).clamp(0.0, 1.0);
                let raw_val = min + frac * (max - min);
                let stepped = (raw_val / step).round() * step;
                let val = stepped.clamp(*min, *max);

                if cx.has_global::<AppState>() {
                    let s = *slider;
                    let _ = cx.global::<AppState>().config.update(|cfg| match s {
                        BarSlider::AnimDuration => cfg.ui.bar.animation_duration_ms = val as u32,
                        BarSlider::SpringBounciness => cfg.ui.bar.spring_bounciness = val,
                    });
                }
                cx.notify();
            }
        }
    }

    fn move_widget(
        widget: BarWidgetId,
        zone: BarZone,
        index: Option<usize>,
        cx: &mut Context<Settings>,
    ) {
        if let Some(state) = cx.try_global::<AppState>() {
            let mut cfg = (*state.config.get()).clone();
            cfg.ui.bar.move_widget(widget, zone, index);
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

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Settings>) -> bool {
        if self.active_slider.is_some() {
            self.update_slider_value(event.position.x.into(), cx);
            return true;
        }
        false
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _cx: &mut Context<Settings>) -> bool {
        if self.active_slider.is_some() {
            self.active_slider = None;
            return true;
        }
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
            .child(self.render_bar_motion_card(&bar_config, &theme, cx))
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
            .bg(theme.background().opacity(0.35))
            .border_1()
            .border_color(theme.surface().opacity(0.35))
            .on_drop(
                cx.listener(move |_settings, dragged: &DraggedBarWidget, _, cx| {
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

        let total_widgets =
            bar_config.left.len() + bar_config.center.len() + bar_config.right.len();
        let max_zone = bar_config
            .left
            .len()
            .max(bar_config.center.len())
            .max(bar_config.right.len());
        let scale = BarWidgetScale::compute(total_widgets, max_zone);

        let live_bar_strip = div()
            .w_full()
            .max_w(px(880.0))
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
            .gap(px(scale.gap * 2.0))
            .child(
                div()
                    .w(px(0.0))
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_start()
                    .child(self.render_bar_zone_slot(
                        BarZone::Left,
                        &bar_config.left,
                        scale,
                        text("settings.bar_zone_left", cx),
                        theme,
                        cx,
                    )),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .px(px(scale.gap))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.render_bar_zone_slot(
                        BarZone::Center,
                        &bar_config.center,
                        scale,
                        text("settings.bar_zone_center", cx),
                        theme,
                        cx,
                    )),
            )
            .child(
                div()
                    .w(px(0.0))
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_end()
                    .child(self.render_bar_zone_slot(
                        BarZone::Right,
                        &bar_config.right,
                        scale,
                        text("settings.bar_zone_right", cx),
                        theme,
                        cx,
                    )),
            );

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
                    ),
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
        let payload = DraggedBarWidget {
            id,
            from_zone: placed_zone,
        };

        let def_scale = BarWidgetScale::default();
        div()
            .id(format!("palette-widget-pill-{:?}", id))
            .h(px(34.0))
            .px(px(14.0))
            .rounded_full()
            .bg(if is_placed {
                theme.surface_container_low()
            } else {
                theme.surface_container_high()
            })
            .border_1()
            .border_color(if is_placed {
                theme.outline_variant().opacity(0.20)
            } else {
                theme.outline_variant().opacity(0.35)
            })
            .opacity(if is_placed { 0.42 } else { 1.0 })
            .flex()
            .items_center()
            .cursor_pointer()
            .hover(|s| s.opacity(0.95).border_color(theme.primary()))
            .on_drag(
                payload,
                move |dragged: &DraggedBarWidget, _: Point<gpui::Pixels>, _, cx| {
                    let id = dragged.id;
                    cx.new(|_| DraggedWidgetPreview {
                        id,
                        scale: def_scale,
                    })
                },
            )
            .when(is_placed, |d| {
                d.on_click(cx.listener(move |_, _, _, cx| {
                    Self::remove_widget(id, cx);
                }))
            })
            .child(render_mini_widget_visual(id, &def_scale, theme))
    }

    fn render_bar_zone_slot(
        &self,
        zone: BarZone,
        widgets: &[BarWidgetId],
        scale: BarWidgetScale,
        label: String,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> impl IntoElement {
        let mut slot = div()
            .id(format!("bar-live-zone-{:?}", zone))
            .h(px(32.0))
            .px(px(2.0))
            .flex()
            .items_center()
            .gap(px(scale.gap))
            .when(zone == BarZone::Left, |d| d.justify_start())
            .when(zone == BarZone::Center, |d| d.justify_center())
            .when(zone == BarZone::Right, |d| d.justify_end())
            .on_drop(
                cx.listener(move |_settings, dragged: &DraggedBarWidget, _, cx| {
                    let fallback_index = match zone {
                        BarZone::Right => Some(0),
                        _ => None,
                    };
                    Self::move_widget(dragged.id, zone, fallback_index, cx);
                }),
            );

        if widgets.is_empty() {
            slot = slot.child(
                div()
                    .h(px(scale.item_height))
                    .px(px(scale.pad_x * 1.5))
                    .rounded_full()
                    .border_1()
                    .border_dashed()
                    .border_color(theme.outline_variant().opacity(0.35))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(scale.font_size - 1.0))
                            .text_color(theme.foreground_muted().opacity(0.45))
                            .child(label),
                    ),
            );
        } else {
            for (idx, &w_id) in widgets.iter().enumerate() {
                let pre_drop = div()
                    .id(format!("bar-drop-before-{:?}-{}", zone, idx))
                    .w(px(scale.drop_width))
                    .h(px(scale.item_height))
                    .rounded_full()
                    .cursor_pointer()
                    .on_drop(
                        cx.listener(move |_settings, dragged: &DraggedBarWidget, _, cx| {
                            cx.stop_propagation();
                            Self::move_widget(dragged.id, zone, Some(idx), cx);
                        }),
                    );
                slot = slot.child(pre_drop);

                let payload = DraggedBarWidget {
                    id: w_id,
                    from_zone: Some(zone),
                };
                let hover_bg = theme.red().opacity(0.16);
                let item = div()
                    .id(format!("live-bar-item-{:?}-{:?}", zone, w_id))
                    .h(px(scale.item_height))
                    .px(px(scale.pad_x))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_drag(
                        payload,
                        move |dragged: &DraggedBarWidget, _: Point<gpui::Pixels>, _, cx| {
                            let id = dragged.id;
                            cx.new(|_| DraggedWidgetPreview { id, scale })
                        },
                    )
                    .on_drop(
                        cx.listener(move |_settings, dragged: &DraggedBarWidget, _, cx| {
                            cx.stop_propagation();
                            Self::move_widget(dragged.id, zone, Some(idx), cx);
                        }),
                    )
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.stop_propagation();
                        Self::remove_widget(w_id, cx);
                    }))
                    .child(render_mini_widget_visual(w_id, &scale, theme));

                slot = slot.child(item);
            }

            let last_idx = widgets.len();
            let post_drop = div()
                .id(format!("bar-drop-after-{:?}", zone))
                .w(px(scale.drop_width))
                .h(px(scale.item_height))
                .rounded_full()
                .cursor_pointer()
                .on_drop(
                    cx.listener(move |_settings, dragged: &DraggedBarWidget, _, cx| {
                        cx.stop_propagation();
                        Self::move_widget(dragged.id, zone, Some(last_idx), cx);
                    }),
                );
            slot = slot.child(post_drop);
        }

        slot
    }

    fn render_bar_motion_card(
        &self,
        bar_config: &BarConfig,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> impl IntoElement {
        let is_spring = bar_config.animation == CapsuleAnimation::Spring;
        let mut motion_card = card(theme)
            .child(heading(&text("settings.motion_heading", cx), theme))
            .child(toggle_row(
                ElementId::Name("setting-bar-reduced-motion".into()),
                &text("settings.reduced_motion_title", cx),
                Some(&text("settings.reduced_motion_hint", cx)),
                bar_config.reduced_motion,
                theme,
                cx.listener(|_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        let _ = cx.global::<AppState>().config.update(|cfg| {
                            cfg.ui.bar.reduced_motion = !cfg.ui.bar.reduced_motion;
                        });
                    }
                    cx.notify();
                }),
            ))
            .child(row_divider(theme))
            .child(control_row(
                &text("settings.animation_style", cx),
                Some(&text("settings.animation_style_hint", cx)),
                segmented_selector(
                    "bar-animation-curve-selector",
                    [
                        (
                            CapsuleAnimation::Fluid,
                            "curve-fluid",
                            text("settings.animation_fluid", cx),
                        ),
                        (
                            CapsuleAnimation::Cubic,
                            "curve-cubic",
                            text("settings.animation_cubic", cx),
                        ),
                        (
                            CapsuleAnimation::Expo,
                            "curve-expo",
                            text("settings.animation_expo", cx),
                        ),
                        (
                            CapsuleAnimation::Back,
                            "curve-back",
                            text("settings.animation_back", cx),
                        ),
                        (
                            CapsuleAnimation::Spring,
                            "curve-spring",
                            text("settings.animation_spring", cx),
                        ),
                    ],
                    bar_config.animation,
                    theme,
                    cx,
                    |_settings, anim, _window, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.bar.animation = anim;
                            });
                        }
                        cx.notify();
                    },
                ),
                theme,
            ))
            .child(row_divider(theme))
            .child(control_row(
                &text("settings.speed_presets_title", cx),
                None,
                segmented_selector(
                    "bar-speed-preset",
                    [
                        (120u32, "snappy", text("settings.speed_snappy", cx)),
                        (220u32, "balanced", text("settings.speed_balanced", cx)),
                        (380u32, "smooth", text("settings.speed_smooth", cx)),
                    ],
                    bar_config.animation_duration_ms,
                    theme,
                    cx,
                    |_settings, ms, _window, cx| {
                        if cx.has_global::<AppState>() {
                            let _ = cx.global::<AppState>().config.update(|cfg| {
                                cfg.ui.bar.animation_duration_ms = ms;
                            });
                        }
                        cx.notify();
                    },
                ),
                theme,
            ))
            .child(row_divider(theme))
            .child(self.slider_control_row(
                "bar-slider-anim-duration",
                &text("settings.animation_short", cx),
                &format!("{} ms", bar_config.animation_duration_ms),
                BarSlider::AnimDuration,
                bar_config.animation_duration_ms as f32,
                (0.0, 1000.0, 20.0),
                &self.anim_duration_tracker,
                theme,
                cx,
            ));

        if is_spring {
            motion_card = motion_card
                .child(row_divider(theme))
                .child(self.slider_control_row(
                    "bar-slider-spring-bounciness",
                    &text("settings.spring_bounciness_title", cx),
                    &format!("{:.0}%", bar_config.spring_bounciness * 100.0),
                    BarSlider::SpringBounciness,
                    bar_config.spring_bounciness,
                    (0.0, 1.0, 0.05),
                    &self.spring_bounciness_tracker,
                    theme,
                    cx,
                ));
        }

        motion_card
            .child(row_divider(theme))
            .child(toggle_row(
                ElementId::Name("setting-bar-squash-stretch".into()),
                &text("settings.squash_stretch_title", cx),
                Some(&text("settings.squash_stretch_hint", cx)),
                bar_config.squash_stretch,
                theme,
                cx.listener(|_this, _, _, cx| {
                    if cx.has_global::<AppState>() {
                        let _ = cx.global::<AppState>().config.update(|cfg| {
                            cfg.ui.bar.squash_stretch = !cfg.ui.bar.squash_stretch;
                        });
                    }
                    cx.notify();
                }),
            ))
    }
}
