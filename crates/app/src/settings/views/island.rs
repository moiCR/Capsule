use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Context, Div, ElementId, FontWeight, IntoElement, MouseMoveEvent,
    MouseUpEvent, ParentElement, Styled, Window, div, prelude::*, px, svg,
};
use services::{AppState, CapsuleAnimation, CapsuleLocation, CapsuleStyle, ContentTransition};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::SettingsView;
use super::common::{
    card, control_row, heading, hero_header, pill_button, row_divider, segmented_selector,
    slider_row, toggle_row,
};
use crate::island::animator::Animator;
use crate::island::orbit::Preview;
use crate::island::widgets::style;
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IslandSlider {
    IdleHeight,
    MarginTop,
    Gap,
    CapsuleRound,
    SatelliteRound,
    CardsRound,
    SpringBounciness,
}

pub struct IslandView {
    pub motion_preview: Animator,
    pub(crate) orbit_preview: Preview,
    motion_preview_generation: u64,

    active_slider: Option<(IslandSlider, DimensionTracker, f32, f32, f32)>,

    idle_height_tracker: DimensionTracker,
    margin_top_tracker: DimensionTracker,
    gap_tracker: DimensionTracker,
    capsule_round_tracker: DimensionTracker,
    satellite_round_tracker: DimensionTracker,
    cards_round_tracker: DimensionTracker,
    spring_bounciness_tracker: DimensionTracker,
}

impl IslandView {
    pub fn new() -> Self {
        Self {
            orbit_preview: Preview::new(Default::default()),
            motion_preview: Animator::new(gpui::size(px(72.0), px(22.0)), Duration::ZERO),
            motion_preview_generation: 0,
            active_slider: None,
            idle_height_tracker: DimensionTracker::new(),
            margin_top_tracker: DimensionTracker::new(),
            gap_tracker: DimensionTracker::new(),
            capsule_round_tracker: DimensionTracker::new(),
            satellite_round_tracker: DimensionTracker::new(),
            cards_round_tracker: DimensionTracker::new(),
            spring_bounciness_tracker: DimensionTracker::new(),
        }
    }

    fn sync_preview_config(&mut self, open: bool, cx: &App) -> u64 {
        let ui = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get().ui.clone()
        } else {
            Default::default()
        };
        let duration = if ui.reduced_motion {
            0
        } else {
            ui.animation_duration_ms as u64
        };
        self.motion_preview
            .set_duration(Duration::from_millis(duration));
        self.motion_preview.set_animation(ui.capsule_animation);
        self.motion_preview
            .set_content_transition(ui.content_transition);
        self.motion_preview
            .set_spring_bounciness(ui.spring_bounciness);
        self.motion_preview.set_squash_stretch(ui.squash_stretch);
        self.motion_preview.set_reduced_motion(ui.reduced_motion);
        self.orbit_preview.set_motion_options(
            ui.stagger_animation,
            ui.squash_stretch,
            ui.reduced_motion,
        );
        self.orbit_preview.transition(
            ui.orbits,
            ui.capsule_animation,
            Duration::from_millis(duration),
            open && duration != 0,
        );
        duration
    }

    pub fn play_motion_preview(&mut self, window: &mut Window, cx: &mut Context<Settings>) {
        self.motion_preview_generation += 1;
        let duration = self.sync_preview_config(true, cx);

        if duration == 0 {
            self.motion_preview
                .transition_to(gpui::size(px(72.0), px(22.0)), Instant::now());
            self.motion_preview.advance(Instant::now());
            cx.notify();
            return;
        }

        self.motion_preview
            .transition_to(gpui::size(px(152.0), px(42.0)), Instant::now());
        Self::queue_preview_frame(
            window,
            cx.entity().downgrade(),
            self.motion_preview_generation,
            false,
        );
        cx.notify();
    }

    fn queue_preview_frame(
        window: &Window,
        entity: gpui::WeakEntity<Settings>,
        generation: u64,
        returning: bool,
    ) {
        window.on_next_frame(move |window, cx| {
            let _ = entity.update(cx, |settings, cx| {
                if let Some(view) = settings.view_as_mut::<IslandView>() {
                    if view.motion_preview_generation != generation {
                        return;
                    }

                    let now = Instant::now();
                    let capsule_running = view.motion_preview.advance(now);
                    let orbits_running = view.orbit_preview.advance(now);
                    let running = capsule_running || orbits_running;

                    if running {
                        Self::queue_preview_frame(
                            window,
                            cx.entity().downgrade(),
                            generation,
                            returning,
                        );
                    } else if !returning {
                        view.sync_preview_config(false, cx);
                        view.motion_preview
                            .transition_to(gpui::size(px(72.0), px(22.0)), Instant::now());
                        Self::queue_preview_frame(
                            window,
                            cx.entity().downgrade(),
                            generation,
                            true,
                        );
                    }
                    cx.notify();
                }
            });
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn slider_control_row(
        &self,
        id: &'static str,
        label: &str,
        value_text: &str,
        slider_kind: IslandSlider,
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
                if let Some(view) = this.view_as_mut::<IslandView>() {
                    view.active_slider = Some((slider_kind, tracker_clone.clone(), min, max, step));
                    view.update_slider_value(event.position.x.into(), cx);
                }
            }),
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if let Some(view) = this.view_as_mut::<IslandView>() {
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
                        IslandSlider::IdleHeight => cfg.ui.idle_height = val,
                        IslandSlider::MarginTop => cfg.ui.margin_top = val,
                        IslandSlider::Gap => cfg.ui.gap = val,
                        IslandSlider::CapsuleRound => cfg.ui.capsule_round = val,
                        IslandSlider::SatelliteRound => cfg.ui.satellite_round = val,
                        IslandSlider::CardsRound => cfg.ui.cards_round = val,
                        IslandSlider::SpringBounciness => cfg.ui.spring_bounciness = val,
                    });
                }
                cx.notify();
            }
        }
    }
}

impl Default for IslandView {
    fn default() -> Self {
        Self::new()
    }
}

fn silhouette(
    concave: bool,
    bottom: bool,
    width: f32,
    height: f32,
    radius: f32,
    theme: &Theme,
) -> Div {
    div()
        .relative()
        .w(px(width))
        .h(px(height))
        .flex_shrink_0()
        .when(!concave, |s| {
            s.child(div().size_full().bg(theme.accent()).rounded(px(radius)))
        })
        .when(concave, |s| {
            s.child(style::concave_background(
                gpui::size(px(width), px(height)),
                radius,
                bottom,
                theme.accent(),
            ))
        })
}

impl SettingsView for IslandView {
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
        let config = if cx.has_global::<AppState>() {
            cx.global::<AppState>().config.get()
        } else {
            Default::default()
        };

        let concave = config.ui.capsule_style == CapsuleStyle::Concave;
        let bottom = config.ui.capsule_location == CapsuleLocation::Bottom;
        let radius = config.ui.capsule_round;
        let margin = if concave { 0.0 } else { config.ui.margin_top };
        let gap = config.ui.gap;

        let size = self.motion_preview.size();
        let width = 250.0 + (f32::from(size.width) - 72.0) * 1.875;
        let base_height = config.ui.idle_height.max(40.0);
        let height = base_height + (f32::from(size.height) - 22.0);

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            .child(hero_header(
                "island.svg",
                &text("settings.tab_island", cx),
                &text("settings.island_subtitle", cx),
                &theme,
            ))
            .child(
                card(&theme)
                    .child(heading(&text("settings.orbits_heading", cx), &theme))
                    .child(toggle_row(
                        ElementId::Name("setting-orbit-power".into()),
                        &text("settings.orbit_power", cx),
                        Some(&text("settings.orbit_always_hint", cx)),
                        config.ui.orbits.power,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.orbits.power = !cfg.ui.orbits.power;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-orbit-usage".into()),
                        &text("settings.orbit_usage", cx),
                        Some(&text("settings.orbit_always_hint", cx)),
                        config.ui.orbits.usage,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.orbits.usage = !cfg.ui.orbits.usage;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-orbit-record".into()),
                        &text("settings.orbit_record", cx),
                        Some(&text("settings.orbit_record_hint", cx)),
                        config.ui.orbits.record,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.orbits.record = !cfg.ui.orbits.record;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-orbit-shelf".into()),
                        &text("settings.orbit_shelf", cx),
                        Some(&text("settings.orbit_shelf_hint", cx)),
                        config.ui.orbits.shelf,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.orbits.shelf = !cfg.ui.orbits.shelf;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    )),
            )
            .child(
                card(&theme)
                    .child(heading(&text("settings.tab_capsule", cx), &theme))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(18.0))
                            .py(px(12.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_size(px(14.0))
                                            .text_color(theme.foreground())
                                            .child(text("settings.location", cx)),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .text_color(theme.foreground_muted())
                                            .child(text("settings.location_hint", cx)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.0))
                                    .child(pill_button(
                                        ElementId::Name("loc-top".into()),
                                        &text("settings.location_top", cx),
                                        !bottom,
                                        &theme,
                                        cx.listener(|_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.ui.capsule_location =
                                                            CapsuleLocation::Top;
                                                    });
                                            }
                                            cx.notify();
                                        }),
                                    ))
                                    .child(pill_button(
                                        ElementId::Name("loc-bottom".into()),
                                        &text("settings.location_bottom", cx),
                                        bottom,
                                        &theme,
                                        cx.listener(|_this, _, _, cx| {
                                            if cx.has_global::<AppState>() {
                                                let _ =
                                                    cx.global::<AppState>().config.update(|cfg| {
                                                        cfg.ui.capsule_location =
                                                            CapsuleLocation::Bottom;
                                                    });
                                            }
                                            cx.notify();
                                        }),
                                    )),
                            ),
                    )
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-concave-style".into()),
                        &text("settings.concave_style", cx),
                        Some(&text("settings.style_description", cx)),
                        concave,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.capsule_style =
                                        if cfg.ui.capsule_style == CapsuleStyle::Normal {
                                            CapsuleStyle::Concave
                                        } else {
                                            CapsuleStyle::Normal
                                        };
                                });
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-clock-format".into()),
                        &text("settings.clock_24", cx),
                        Some(&text("settings.clock_format_hint", cx)),
                        config.ui.clock_24_hour,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.clock_24_hour = !cfg.ui.clock_24_hour;
                                });
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.animation_style", cx),
                        Some(&text("settings.animation_style_hint", cx)),
                        segmented_selector(
                            "capsule-anim",
                            [
                                (
                                    CapsuleAnimation::Fluid,
                                    "fluid",
                                    text("settings.animation_fluid", cx),
                                ),
                                (
                                    CapsuleAnimation::Cubic,
                                    "cubic",
                                    text("settings.animation_cubic", cx),
                                ),
                                (
                                    CapsuleAnimation::Expo,
                                    "expo",
                                    text("settings.animation_expo", cx),
                                ),
                                (
                                    CapsuleAnimation::Back,
                                    "back",
                                    text("settings.animation_back", cx),
                                ),
                                (
                                    CapsuleAnimation::Spring,
                                    "spring",
                                    text("settings.animation_spring", cx),
                                ),
                            ],
                            config.ui.capsule_animation,
                            &theme,
                            cx,
                            |settings, anim, window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.capsule_animation = anim;
                                    });
                                }
                                if let Some(view) = settings.view_as_mut::<IslandView>() {
                                    view.play_motion_preview(window, cx);
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.content_transition_title", cx),
                        Some(&text("settings.content_transition_hint", cx)),
                        segmented_selector(
                            "content-trans",
                            [
                                (
                                    ContentTransition::FadeScale,
                                    "fadescale",
                                    text("settings.content_transition_fade_scale", cx),
                                ),
                                (
                                    ContentTransition::Fade,
                                    "fade",
                                    text("settings.content_transition_fade", cx),
                                ),
                            ],
                            config.ui.content_transition,
                            &theme,
                            cx,
                            |settings, trans, window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.content_transition = trans;
                                    });
                                }
                                if let Some(view) = settings.view_as_mut::<IslandView>() {
                                    view.play_motion_preview(window, cx);
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .when(
                        config.ui.capsule_animation == CapsuleAnimation::Spring,
                        |card| {
                            card.child(row_divider(&theme))
                                .child(self.slider_control_row(
                                    "slider-spring-bounciness",
                                    &text("settings.spring_bounciness_title", cx),
                                    &format!("{:.0}%", config.ui.spring_bounciness * 100.0),
                                    IslandSlider::SpringBounciness,
                                    config.ui.spring_bounciness,
                                    (0.0, 1.0, 0.05),
                                    &self.spring_bounciness_tracker,
                                    &theme,
                                    cx,
                                ))
                        },
                    )
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-squash-stretch".into()),
                        &text("settings.squash_stretch_title", cx),
                        Some(&text("settings.squash_stretch_hint", cx)),
                        config.ui.squash_stretch,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.squash_stretch = !cfg.ui.squash_stretch;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-stagger-animation".into()),
                        &text("settings.stagger_animation_title", cx),
                        Some(&text("settings.stagger_animation_hint", cx)),
                        config.ui.stagger_animation,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.stagger_animation = !cfg.ui.stagger_animation;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<IslandView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    )),
            )
            .child(
                card(&theme)
                    .child(heading(&text("settings.dimensions_heading", cx), &theme))
                    .child(self.slider_control_row(
                        "slider-idle-height",
                        &text("settings.height_short", cx),
                        &format!("{:.0} px", config.ui.idle_height),
                        IslandSlider::IdleHeight,
                        config.ui.idle_height,
                        (16.0, 60.0, 1.0),
                        &self.idle_height_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(self.slider_control_row(
                        "slider-margin-top",
                        &text("settings.margin_short", cx),
                        &format!("{:.0} px", config.ui.margin_top),
                        IslandSlider::MarginTop,
                        config.ui.margin_top,
                        (0.0, 40.0, 1.0),
                        &self.margin_top_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(self.slider_control_row(
                        "slider-gap",
                        &text("settings.gap_short", cx),
                        &format!("{:.0} px", config.ui.gap),
                        IslandSlider::Gap,
                        config.ui.gap,
                        (0.0, 32.0, 1.0),
                        &self.gap_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(heading(&text("settings.radii_heading", cx), &theme))
                    .child(self.slider_control_row(
                        "slider-capsule-round",
                        &text("settings.capsule_radius_short", cx),
                        &format!("{:.0} px", config.ui.capsule_round),
                        IslandSlider::CapsuleRound,
                        config.ui.capsule_round,
                        (0.0, 24.0, 1.0),
                        &self.capsule_round_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(self.slider_control_row(
                        "slider-satellite-round",
                        &text("settings.satellite_radius_short", cx),
                        &format!("{:.0} px", config.ui.satellite_round),
                        IslandSlider::SatelliteRound,
                        config.ui.satellite_round,
                        (0.0, 24.0, 1.0),
                        &self.satellite_round_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(self.slider_control_row(
                        "slider-cards-round",
                        &text("settings.cards_radius_short", cx),
                        &format!("{:.0} px", config.ui.cards_round),
                        IslandSlider::CardsRound,
                        config.ui.cards_round,
                        (0.0, 24.0, 1.0),
                        &self.cards_round_tracker,
                        &theme,
                        cx,
                    )),
            )
            .child(
                card(&theme)
                    .p(px(16.0))
                    .gap(px(12.0))
                    .items_center()
                    .child(
                        div()
                            .relative()
                            .flex()
                            .items_start()
                            .when(bottom, |s| s.items_end())
                            .justify_center()
                            .w_full()
                            .h(px((base_height + 20.0 + margin + 16.0).max(120.0)))
                            .rounded(px(10.0))
                            .bg(style::background(&theme))
                            .overflow_hidden()
                            .child(
                                div()
                                    .absolute()
                                    .top(px(8.0))
                                    .when(bottom, |s| s.top_auto().bottom(px(8.0)))
                                    .w_full()
                                    .h(px(1.0))
                                    .bg(style::border(&theme)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .w(px(width))
                                    .h(px(height))
                                    .mt(px(8.0 + margin))
                                    .when(bottom, |s| s.mt_0().mb(px(8.0 + margin)))
                                    .child(silhouette(
                                        concave, bottom, width, height, radius, &theme,
                                    ))
                                    .child(self.orbit_preview.render(
                                        gpui::size(px(width), px(height)),
                                        px(gap),
                                        bottom,
                                        &theme,
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .id("settings-island-preview-btn")
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .px(px(14.0))
                            .py(px(6.0))
                            .rounded_full()
                            .bg(style::surface(&theme))
                            .hover(|s| s.bg(style::hover(&theme)))
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                if let Some(view) = this.view_as_mut::<IslandView>() {
                                    view.play_motion_preview(window, cx);
                                }
                            }))
                            .child(
                                svg()
                                    .path("play.svg")
                                    .size(px(14.0))
                                    .text_color(theme.accent()),
                            )
                            .child(text("settings.preview_motion", cx)),
                    ),
            )
            .into_any_element()
    }
}
