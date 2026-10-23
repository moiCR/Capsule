use gpui::{
    AnyElement, App, Bounds, ColorExt, Div, DurationWithEasing, ElementId, FontWeight, IntoElement,
    ParentElement, Pixels, Styled, Window, div, prelude::*, px, svg,
};
use ui::components::{Button, ButtonSize, ButtonVariant, Slider, Switch};
use ui::theme::Theme;
use ui::tracker::DimensionTracker;

use crate::island::widgets::style;

pub fn hero_header(icon: &'static str, title: &str, subtitle: &str, theme: &Theme) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap(px(14.0))
        .w_full()
        .min_w_0()
        .py(px(4.0))
        .child(
            div()
                .size(px(38.0))
                .rounded_full()
                .bg(style::surface(theme))
                .flex()
                .items_center()
                .justify_center()
                .child(svg().path(icon).size(px(18.0)).text_color(theme.accent())),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .gap(px(2.0))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(22.0))
                        .text_color(theme.foreground())
                        .child(title.to_owned()),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.foreground_muted())
                        .child(subtitle.to_owned()),
                ),
        )
        .into_any_element()
}

pub fn card(theme: &Theme) -> Div {
    style::card(theme, style::CARD_RADIUS)
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .overflow_hidden()
}

pub fn heading(title: &str, theme: &Theme) -> Div {
    div()
        .px(px(18.0))
        .pt(px(14.0))
        .pb(px(4.0))
        .text_size(px(15.0))
        .line_height(px(20.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(theme.foreground())
        .child(title.to_owned())
}

pub fn row_divider(theme: &Theme) -> AnyElement {
    div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme))
        .into_any_element()
}

pub fn control_row(
    title: &str,
    subtitle: Option<&str>,
    control: impl IntoElement,
    theme: &Theme,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .pr(px(16.0))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .min_w_0()
                        .text_ellipsis()
                        .text_color(theme.foreground())
                        .child(title.to_string()),
                )
                .when_some(subtitle, |this, sub| {
                    this.child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .child(sub.to_string()),
                    )
                }),
        )
        .child(div().flex().items_center().flex_shrink_0().child(control))
        .into_any_element()
}

pub fn toggle_row(
    id: ElementId,
    title: &str,
    subtitle: Option<&str>,
    is_on: bool,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py(px(12.0))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .pr(px(16.0))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .min_w_0()
                        .text_ellipsis()
                        .text_color(theme.foreground())
                        .child(title.to_string()),
                )
                .when_some(subtitle, |this, sub| {
                    this.child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.foreground_muted())
                            .child(sub.to_string()),
                    )
                }),
        )
        .child(Switch::new(id).checked(is_on).on_click(on_click))
        .into_any_element()
}

pub fn slider_row(
    id: ElementId,
    title: &str,
    display_val: &str,
    pct: f32,
    tracker: &DimensionTracker,
    theme: &Theme,
    on_mouse_down: impl Fn(&gpui::MouseDownEvent, &mut Window, &mut App) + 'static,
    on_mouse_up: impl Fn(&gpui::MouseUpEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py(px(10.0))
        .gap(px(8.0))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .w_full()
                .min_w_0()
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(14.0))
                        .line_height(px(20.0))
                        .min_w_0()
                        .text_ellipsis()
                        .text_color(theme.foreground())
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .px(px(8.0))
                        .py(px(3.0))
                        .rounded(px(8.0))
                        .bg(style::raised(theme))
                        .flex_shrink_0()
                        .text_color(theme.foreground())
                        .child(display_val.to_string()),
                ),
        )
        .child(
            Slider::new(id)
                .fraction(pct)
                .height(28.0)
                .track_bounds(tracker)
                .on_mouse_down(on_mouse_down)
                .on_mouse_up(on_mouse_up),
        )
        .into_any_element()
}

pub fn pill_button(
    id: ElementId,
    label: &str,
    is_selected: bool,
    _theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    Button::new(id)
        .label(label.to_string())
        .selected(is_selected)
        .variant(if is_selected {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        })
        .size(ButtonSize::Sm)
        .pill(true)
        .on_click(on_click)
        .into_any_element()
}

pub fn segmented_selector<T: PartialEq + Clone + 'static>(
    id_prefix: &'static str,
    options: impl IntoIterator<Item = (T, &'static str, String)>,
    selected: T,
    theme: &Theme,
    cx: &mut Context<crate::settings::Settings>,
    on_select: impl Fn(
        &mut crate::settings::Settings,
        T,
        &mut Window,
        &mut Context<crate::settings::Settings>,
    )
    + 'static
    + Copy,
) -> Div {
    let mut container = div()
        .flex()
        .flex_row()
        .items_center()
        .flex_shrink_0()
        .p(px(3.0))
        .gap(px(4.0))
        .rounded_full()
        .bg(theme.surface().opacity(0.35))
        .border_1()
        .border_color(theme.surface().opacity(0.25));

    for (val, id, label) in options {
        let is_active = selected == val;
        let val_clone = val.clone();
        container = container.child(
            div()
                .id(ElementId::Name(format!("{id_prefix}-{id}").into()))
                .flex()
                .flex_row()
                .items_center()
                .px(px(12.0))
                .py(px(5.0))
                .rounded_full()
                .bg(if is_active {
                    theme.accent()
                } else {
                    gpui::transparent_black()
                })
                .transitions(|t| t.bg(gpui::millis(160).with_easing(gpui::ease_in_out)))
                .hover(|s| {
                    if !is_active {
                        s.bg(theme.surface().opacity(0.4))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, window, cx| {
                    on_select(this, val_clone.clone(), window, cx);
                }))
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_active {
                            theme.background()
                        } else {
                            theme.foreground()
                        })
                        .transitions(|t| {
                            t.text_color(gpui::millis(160).with_easing(gpui::ease_in_out))
                        })
                        .child(label),
                ),
        );
    }

    container
}

pub fn compute_dropdown_placement(
    scroll_bounds: Bounds<Pixels>,
    trigger_bounds: (f32, f32),
    fallback_local: (f32, f32),
    item_count: usize,
) -> (bool, f32) {
    let scroll_top: f32 = scroll_bounds.top().into();
    let scroll_bottom: f32 = scroll_bounds.bottom().into();

    let (viewport_top, viewport_bottom) = if scroll_bottom > scroll_top {
        (scroll_top, scroll_bottom)
    } else {
        (0.0, 560.0)
    };

    let (trigger_top, trigger_bottom) = if trigger_bounds.1 > trigger_bounds.0 {
        (trigger_bounds.0, trigger_bounds.1)
    } else {
        (
            viewport_top + fallback_local.0,
            viewport_top + fallback_local.1,
        )
    };

    let margin = 12.0;
    let space_above = (trigger_top - viewport_top - margin).max(0.0);
    let space_below = (viewport_bottom - trigger_bottom - margin).max(0.0);

    let desired_height = (item_count as f32 * 32.0 + 14.0).clamp(40.0, 220.0);

    let (open_upwards, available_space) = if space_below >= desired_height {
        (false, space_below)
    } else if space_above >= desired_height || space_above > space_below {
        (true, space_above)
    } else {
        (false, space_below)
    };

    let max_height = available_space.clamp(48.0, 220.0);
    (open_upwards, max_height)
}
