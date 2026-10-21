use super::{HEIGHT, Settings, WIDTH, header, views};
use crate::island::widgets::style;
use gpui::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, px};
use ui::theme::Theme;

pub(crate) fn render_settings(
    settings: &mut Settings,
    window: &mut Window,
    cx: &mut Context<Settings>,
) -> AnyElement {
    let theme = cx.global::<Theme>().clone();
    let capsule_radius = if cx.has_global::<services::AppState>() {
        cx.global::<services::AppState>()
            .config
            .get()
            .ui
            .capsule_round
    } else {
        24.0
    };

    let viewport = window.viewport_size();
    let width = (f32::from(viewport.width) - 48.0).clamp(0.0, WIDTH);
    let height = (f32::from(viewport.height) - 48.0).clamp(0.0, HEIGHT);

    let active_tab = settings.active_tab;
    let progress = settings.animator.section_progress();

    let query = settings.search_input.text();
    let active_view_content = if !query.trim().is_empty() {
        views::search::render_search_results(&query, &theme, cx)
    } else if let Some((_, view)) = settings.views.iter_mut().find(|(t, _)| *t == active_tab) {
        view.render(window, cx)
    } else {
        div().into_any_element()
    };

    let header = header::render_header(&settings.search_input, &theme, cx);
    let sidebar_element = settings
        .sidebar
        .render(settings.active_tab, !query.is_empty(), &theme, cx);

    let card = div()
        .id("settings-modal-card")
        .occlude()
        .on_click(cx.listener(|this, _, window, cx| {
            window.focus(&this.focus_handle, cx);
        }))
        .relative()
        .top(px(settings
            .animator
            .offset_y
            .min((f32::from(viewport.height) - height).max(0.0) / 2.0)))
        .w(px(width))
        .h(px(height))
        .max_w_full()
        .max_h_full()
        .flex()
        .flex_col()
        .gap(px(20.0))
        .p(px(24.0))
        .flex_shrink_0()
        .rounded(px(capsule_radius))
        .bg(style::background(&theme))
        .border_1()
        .border_color(style::border(&theme))
        .shadow_xl()
        .font_family(theme.font_family())
        .text_color(theme.foreground())
        .overflow_hidden()
        .opacity(settings.animator.card_opacity)
        .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
            if let Some((_, view)) = this.views.iter_mut().find(|(t, _)| *t == this.active_tab) {
                if view.handle_mouse_move(event, cx) {
                    cx.notify();
                }
            }
        }))
        .capture_any_mouse_up(cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
            if let Some((_, view)) = this.views.iter_mut().find(|(t, _)| *t == this.active_tab) {
                if view.handle_mouse_up(event, cx) {
                    cx.notify();
                }
            }
        }))
        .on_mouse_up(
            gpui::MouseButton::Left,
            cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                if let Some((_, view)) = this.views.iter_mut().find(|(t, _)| *t == this.active_tab) {
                    if view.handle_mouse_up(event, cx) {
                        cx.notify();
                    }
                }
            }),
        )
        .child(header)
        .child(
            div()
                .flex()
                .gap(px(28.0))
                .flex_1()
                .min_h_0()
                .min_w_0()
                .w_full()
                .child(sidebar_element)
                .child(
                    div()
                        .id("settings-content-scroll")
                        .track_scroll(&settings.scroll_handle)
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .overflow_y_scroll()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .min_w_0()
                                .flex_shrink_0()
                                .when(progress < 1.0, |s| s.blur(px((1.0 - progress) * 5.0)))
                                .child(active_view_content),
                        ),
                ),
        );

    let entity = cx.entity().downgrade();
    let entity_move = entity.clone();
    let entity_up = entity.clone();

    div()
        .id("settings-window-root")
        .track_focus(&settings.focus_handle)
        .on_key_down(cx.listener(Settings::handle_key_down))
        .on_mouse_move_all(move |event, phase, _hitbox, _window, cx| {
            if phase == gpui::DispatchPhase::Bubble {
                let _ = entity_move.update(cx, |this, cx| {
                    if let Some((_, view)) = this.views.iter_mut().find(|(t, _)| *t == this.active_tab) {
                        if view.handle_mouse_move(event, cx) {
                            cx.notify();
                        }
                    }
                });
            }
        })
        .on_mouse_up_all(move |event, _phase, _hitbox, _window, cx| {
            let _ = entity_up.update(cx, |this, cx| {
                if let Some((_, view)) = this.views.iter_mut().find(|(t, _)| *t == this.active_tab) {
                    if view.handle_mouse_up(event, cx) {
                        cx.notify();
                    }
                }
            });
        })
        .relative()
        .size_full()
        .overflow_hidden()
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .id("settings-backdrop")
                .absolute()
                .inset_0()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.request_close(cx);
                })),
        )
        .child(card)
        .into_any_element()
}
