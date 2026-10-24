use gpui::{
    AnyElement, App, Context, ElementId, IntoElement, MouseMoveEvent, MouseUpEvent, ParentElement,
    Styled, Window, div, img, prelude::*, px, svg,
};
use services::{AppState, CapsuleAnimation, ShellMode, WallpaperTransition};
use ui::components::{Button, ButtonSize, ButtonVariant};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use super::SettingsView;
use super::common::{
    card, control_row, heading, hero_header, row_divider, segmented_selector, slider_row,
    toggle_row,
};
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
pub enum AppearanceSlider {
    AnimDuration,
}

pub struct AppearanceView {
    pub appearance_error: Option<String>,
    active_slider: Option<(AppearanceSlider, DimensionTracker, f32, f32, f32)>,
    anim_duration_tracker: DimensionTracker,
}

impl AppearanceView {
    pub fn new() -> Self {
        Self {
            appearance_error: None,
            active_slider: None,
            anim_duration_tracker: DimensionTracker::new(),
        }
    }

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
                        AppearanceSlider::AnimDuration => cfg.ui.animation_duration_ms = val as u32,
                    });
                }
                cx.notify();
            }
        }
    }
}

impl Default for AppearanceView {
    fn default() -> Self {
        Self::new()
    }
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

        let error = self.appearance_error.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .w_full()
            .min_w_0()
            .child(hero_header(
                "palette.svg",
                &text("settings.tab_appearance", cx),
                &text("settings.appearance_subtitle", cx),
                &theme,
            ))
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
                                        .text_color(theme.foreground_muted()),
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
            .child(
                card(&theme)
                    .child(heading(&text("settings.shell_mode", cx), &theme))
                    .child(control_row(
                        &text("settings.shell_mode", cx),
                        Some(&text("settings.shell_mode_hint", cx)),
                        segmented_selector(
                            "shell-mode-selector",
                            [
                                (
                                    ShellMode::Island,
                                    "mode-island",
                                    text("settings.mode_island", cx),
                                ),
                                (
                                    ShellMode::Bar,
                                    "mode-bar",
                                    text("settings.mode_bar", cx),
                                ),
                            ],
                            config.ui.shell_mode,
                            &theme,
                            cx,
                            |_settings, mode, _window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.shell_mode = mode;
                                    });
                                }
                                crate::panel::CapsulePanel::sync_shell_mode(cx);
                                cx.notify();
                            },
                        ),
                        &theme,
                    )),
            )
            .child(
                card(&theme)
                    .child(heading(&text("settings.motion_heading", cx), &theme))
                    .child(toggle_row(
                        ElementId::Name("setting-reduced-motion".into()),
                        &text("settings.reduced_motion_title", cx),
                        Some(&text("settings.reduced_motion_hint", cx)),
                        config.ui.reduced_motion,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.reduced_motion = !cfg.ui.reduced_motion;
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
                            "animation-curve-selector",
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
                            config.ui.capsule_animation,
                            &theme,
                            cx,
                            |_settings, anim, _window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.capsule_animation = anim;
                                    });
                                }
                                cx.notify();
                            },
                        ),
                        &theme,
                    ))
                    .child(row_divider(&theme))
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
                            |_settings, ms, _window, cx| {
                                if cx.has_global::<AppState>() {
                                    let _ = cx.global::<AppState>().config.update(|cfg| {
                                        cfg.ui.animation_duration_ms = ms;
                                    });
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
                    ))
                    .child(row_divider(&theme))
                    .child(toggle_row(
                        ElementId::Name("setting-stagger-animation".into()),
                        &text("settings.stagger_animation_title", cx),
                        Some(&text("settings.stagger_animation_hint", cx)),
                        config.ui.stagger_animation,
                        &theme,
                        cx.listener(|_this, _, _, cx| {
                            if cx.has_global::<AppState>() {
                                let _ = cx.global::<AppState>().config.update(|cfg| {
                                    cfg.ui.stagger_animation = !cfg.ui.stagger_animation;
                                });
                            }
                            cx.notify();
                        }),
                    )),
            )
            .into_any_element()
    }
}
