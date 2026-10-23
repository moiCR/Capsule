use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, ColorExt, Context, Div, ElementId, FontWeight, IntoElement, MouseMoveEvent,
    MouseUpEvent, ParentElement, Styled, Window, div, img, prelude::*, px, svg,
};
use services::{
    AppState, CapsuleAnimation, CapsuleLocation, CapsuleStyle, ContentTransition,
    WallpaperTransition,
};
use ui::components::{Button, ButtonSize, ButtonVariant};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::SettingsView;
use super::common::{
    card, control_row, heading, hero_header, pill_button, row_divider, segmented_selector,
    slider_row, toggle_row,
};
use crate::capsule::animator::Animator;
use crate::capsule::orbit::Preview;
use crate::capsule::widgets::style;
use crate::settings::Settings;

fn text(key: &str, cx: &App) -> String {
    if cx.has_global::<AppState>() {
        cx.global::<AppState>().language.get(key)
    } else {
        key.to_string()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppearanceSlider {
    IdleHeight,
    MarginTop,
    Gap,
    CapsuleRound,
    SatelliteRound,
    CardsRound,
    AnimDuration,
    SpringBounciness,
}

pub struct AppearanceView {
    pub appearance_error: Option<String>,
    pub motion_preview: Animator,
    pub(crate) orbit_preview: Preview,
    motion_preview_generation: u64,

    active_slider: Option<(AppearanceSlider, DimensionTracker, f32, f32, f32)>,

    idle_height_tracker: DimensionTracker,
    margin_top_tracker: DimensionTracker,
    gap_tracker: DimensionTracker,
    capsule_round_tracker: DimensionTracker,
    satellite_round_tracker: DimensionTracker,
    cards_round_tracker: DimensionTracker,
    anim_duration_tracker: DimensionTracker,
    spring_bounciness_tracker: DimensionTracker,
}

impl AppearanceView {
    pub fn new() -> Self {
        Self {
            appearance_error: None,
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
            anim_duration_tracker: DimensionTracker::new(),
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
                if let Some(view) = settings.view_as_mut::<AppearanceView>() {
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
        slider_kind: AppearanceSlider,
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
                if let Some(view) = this.view_as_mut::<AppearanceView>() {
                    view.active_slider = Some((slider_kind, tracker_clone.clone(), min, max, step));
                    view.update_slider_value(event.position.x.into(), cx);
                }
            }),
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if let Some(view) = this.view_as_mut::<AppearanceView>() {
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
                        AppearanceSlider::IdleHeight => cfg.ui.idle_height = val,
                        AppearanceSlider::MarginTop => cfg.ui.margin_top = val,
                        AppearanceSlider::Gap => cfg.ui.gap = val,
                        AppearanceSlider::CapsuleRound => cfg.ui.capsule_round = val,
                        AppearanceSlider::SatelliteRound => cfg.ui.satellite_round = val,
                        AppearanceSlider::CardsRound => cfg.ui.cards_round = val,
                        AppearanceSlider::AnimDuration => cfg.ui.animation_duration_ms = val as u32,
                        AppearanceSlider::SpringBounciness => cfg.ui.spring_bounciness = val,
                    });
                }
                cx.notify();
            }
        }
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

impl SettingsView for AppearanceView {
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

        let error = self.appearance_error.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            // Hero Header
            .child(hero_header(
                "palette.svg",
                &text("settings.tab_appearance", cx),
                &text("settings.appearance_subtitle", cx),
                &theme,
            ))
            // General Group (Wallpaper Preview, Selector, Transitions, Dynamic colors & Dark Mode)
            .child({
                let current_wallpaper = if cx.has_global::<AppState>() {
                    cx.global::<AppState>().wallpaper.get_current_wallpaper()
                } else {
                    config.ui.wallpaper_path.clone()
                };

                let default_aspect = 16.0 / 9.0;
                let aspect = current_wallpaper
                    .as_deref()
                    .and_then(|p| image::image_dimensions(p).ok())
                    .map(|(w, h)| if h > 0 { w as f32 / h as f32 } else { default_aspect })
                    .unwrap_or(default_aspect);

                let wallpaper_preview = if let Some(ref path) = current_wallpaper {
                    div()
                        .w_full()
                        .flex()
                        .justify_center()
                        .px(px(18.0))
                        .pt(px(10.0))
                        .pb(px(12.0))
                        .child(
                            div()
                                .w(px(540.0))
                                .aspect_ratio(aspect)
                                .rounded(px(16.0))
                                .overflow_hidden()
                                .border_1()
                                .border_color(style::border(&theme))
                                .shadow_md()
                                .child({
                                    let preview_path = services::wallpaper::WallpaperService::create_or_get_screen_wallpaper(path, 1080, 720);
                                    img(preview_path)
                                        .size_full()
                                        .aspect_ratio(aspect)
                                        .rounded(px(16.0))
                                        .object_fit(gpui::ObjectFit::Cover)
                                }),
                        )
                } else {
                    div()
                        .w_full()
                        .flex()
                        .justify_center()
                        .px(px(18.0))
                        .pt(px(10.0))
                        .pb(px(12.0))
                        .child(
                            div()
                                .w(px(540.0))
                                .aspect_ratio(16.0 / 9.0)
                                .rounded(px(16.0))
                                .overflow_hidden()
                                .bg(style::surface(&theme))
                                .border_1()
                                .border_color(style::border(&theme))
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap(px(8.0))
                                .child(
                                    svg()
                                        .path("image.svg")
                                        .size(px(32.0))
                                        .text_color(theme.foreground_muted().opacity(0.6)),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(theme.foreground_muted())
                                        .child(text("settings.no_wallpaper", cx)),
                                ),
                        )
                };

                card(&theme)
                    .child(heading(&text("settings.general_heading", cx), &theme))
                    .child(wallpaper_preview)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .justify_end()
                            .px(px(18.0))
                            .pb(px(12.0))
                            .child(
                                Button::new("btn-select-wallpaper")
                                    .label(text("settings.wallpaper_select", cx))
                                    .variant(ButtonVariant::Primary)
                                    .size(ButtonSize::Sm)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if cx.has_global::<AppState>() {
                                            let state = cx.global::<AppState>().clone();
                                            this.request_close(cx);

                                            services::spawn_tokio(async move {
                                                if let Some(path) =
                                                    crate::wallpaper::portal::pick_wallpaper().await
                                                {
                                                    let _ = state.config.update(|cfg| {
                                                        cfg.ui.wallpaper_path = Some(path.clone());
                                                    });
                                                    state.wallpaper.set_wallpaper(path);
                                                }
                                            });
                                        }
                                    })),
                            ),
                    )
                    .child(row_divider(&theme))
                    .child(control_row(
                        &text("settings.wallpaper_transition", cx),
                        None,
                        segmented_selector(
                            "wallpaper-transition",
                            [
                                (
                                    WallpaperTransition::Fade,
                                    "fade",
                                    text("settings.transition_fade", cx),
                                ),
                                (
                                    WallpaperTransition::Slide,
                                    "slide",
                                    text("settings.transition_slide", cx),
                                ),
                                (
                                    WallpaperTransition::Zoom,
                                    "zoom",
                                    text("settings.transition_zoom", cx),
                                ),
                                (
                                    WallpaperTransition::Wipe,
                                    "wipe",
                                    text("settings.transition_wipe", cx),
                                ),
                                (
                                    WallpaperTransition::None,
                                    "none",
                                    text("settings.transition_none", cx),
                                ),
                            ],
                            config.ui.wallpaper_transition,
                            &theme,
                            cx,
                            |_settings, transition, _window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.wallpaper_transition = transition;
                                    });
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-dynamic-colors".into()),
                        &text("settings.dynamic_colors", cx),
                        Some(&text("settings.dynamic_colors_hint", cx)),
                        config.ui.dynamic_colors,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.dynamic_colors = !cfg.ui.dynamic_colors;
                                });
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-dark-mode".into()),
                        &text("settings.dark_mode", cx),
                        Some(&text("settings.color_mode_hint", cx)),
                        config.ui.dark_mode,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.dark_mode = !cfg.ui.dark_mode;
                                });
                            }
                            cx.notify();
                        }),
                    ))
                    .when_some(error, |s, err| {
                        s.child(
                            div()
                                .px(px(18.0))
                                .pb(px(12.0))
                                .text_size(px(12.0))
                                .text_color(theme.accent())
                                .child(err),
                        )
                    })
            })
            // Orbits Group
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    )),
            )
            // Capsule Group (Location, Style, Clock format)
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
                                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                                    cfg.ui.capsule_location = CapsuleLocation::Top;
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
                                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                                    cfg.ui.capsule_location = CapsuleLocation::Bottom;
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
                                    cfg.ui.capsule_style = if cfg.ui.capsule_style == CapsuleStyle::Normal {
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
                                if let Some(view) = settings.view_as_mut::<AppearanceView>() {
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
                                if let Some(view) = settings.view_as_mut::<AppearanceView>() {
                                    view.play_motion_preview(window, cx);
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .when(config.ui.capsule_animation == CapsuleAnimation::Spring, |card| {
                        card.child(row_divider(&theme)).child(self.slider_control_row(
                            "slider-spring-bounciness",
                            &text("settings.spring_bounciness_title", cx),
                            &format!("{:.0}%", config.ui.spring_bounciness * 100.0),
                            AppearanceSlider::SpringBounciness,
                            config.ui.spring_bounciness,
                            (0.0, 1.0, 0.05),
                            &self.spring_bounciness_tracker,
                            &theme,
                            cx,
                        ))
                    })
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
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
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-reduced-motion".into()),
                        &text("settings.reduced_motion_title", cx),
                        Some(&text("settings.reduced_motion_hint", cx)),
                        config.ui.reduced_motion,
                        &theme,
                        cx.listener(|this, _, window, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.reduced_motion = !cfg.ui.reduced_motion;
                                });
                            }
                            if let Some(view) = this.view_as_mut::<AppearanceView>() {
                                view.play_motion_preview(window, cx);
                            }
                            cx.notify();
                        }),
                    )),
            )
            // Dimensions & Radii Sliders Group
            .child(
                card(&theme)
                    .child(heading(&text("settings.dimensions_heading", cx), &theme))
                    .child(self.slider_control_row(
                        "slider-idle-height",
                        &text("settings.height_short", cx),
                        &format!("{:.0} px", config.ui.idle_height),
                        AppearanceSlider::IdleHeight,
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
                        AppearanceSlider::MarginTop,
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
                        AppearanceSlider::Gap,
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
                        AppearanceSlider::CapsuleRound,
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
                        AppearanceSlider::SatelliteRound,
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
                        AppearanceSlider::CardsRound,
                        config.ui.cards_round,
                        (0.0, 24.0, 1.0),
                        &self.cards_round_tracker,
                        &theme,
                        cx,
                    ))
                    .child(row_divider(&theme))
                    .child(heading(&text("settings.motion_heading", cx), &theme))
                    .child(control_row(
                        &text("settings.speed_presets_title", cx),
                        None,
                        segmented_selector(
                            "speed-preset",
                            [
                                (120u32, "snappy", text("settings.speed_snappy", cx)),
                                (250u32, "balanced", text("settings.speed_balanced", cx)),
                                (400u32, "smooth", text("settings.speed_smooth", cx)),
                            ],
                            config.ui.animation_duration_ms,
                            &theme,
                            cx,
                            |settings, ms, window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.animation_duration_ms = ms;
                                    });
                                }
                                if let Some(view) = settings.view_as_mut::<AppearanceView>() {
                                    view.play_motion_preview(window, cx);
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .child(row_divider(&theme))
                    .child(self.slider_control_row(
                        "slider-anim-duration",
                        &text("settings.animation_short", cx),
                        &format!("{:.0} ms", config.ui.animation_duration_ms),
                        AppearanceSlider::AnimDuration,
                        config.ui.animation_duration_ms as f32,
                        (0.0, 1000.0, 25.0),
                        &self.anim_duration_tracker,
                        &theme,
                        cx,
                    )),
            )
            // Live Motion & Orbit Preview
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
                            .id("settings-motion-preview-btn")
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
                                if let Some(view) = this.view_as_mut::<AppearanceView>() {
                                    view.play_motion_preview(window, cx);
                                }
                            }))
                            .child(svg().path("play.svg").size(px(14.0)).text_color(theme.accent()))
                            .child(text("settings.preview_motion", cx)),
                    ),
            )
            .into_any_element()
    }
}
