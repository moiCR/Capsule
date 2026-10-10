use crate::new_capsule::widgets::style;
use gpui::{
    App, ColorExt, Context, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, Styled, Window, canvas, div, prelude::*, px, svg,
};
use std::cell::Cell;
use std::rc::Rc;
use ui::theme::Theme;

use crate::capsule::modules::settings::{SettingsField, SettingsModule};

pub fn render_hero_header(
    icon: &'static str,
    title: &str,
    subtitle: &str,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.0))
        .w_full()
        .min_w_0()
        .py(px(8.0))
        .child(style::circle_button(
            "settings-section-icon",
            icon,
            36.0,
            theme,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .gap(px(3.0))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(26.0))
                        .child(title.to_owned()),
                )
                .child(
                    div()
                        .text_size(px(14.0))
                        .text_color(theme.foreground_muted())
                        .child(subtitle.to_owned()),
                ),
        )
}

pub fn render_card_container(theme: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .rounded(px(style::CARD_RADIUS))
        .bg(style::surface(theme))
        .overflow_hidden()
}

pub fn render_row_divider(theme: &Theme) -> impl IntoElement {
    div()
        .w_full()
        .h(px(1.0))
        .flex_shrink_0()
        .bg(style::border(theme))
}

pub fn render_toggle_row(
    id: ElementId,
    title: &str,
    subtitle: Option<&str>,
    is_on: bool,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let bg = if is_on {
        theme.accent()
    } else {
        style::raised(theme)
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py_3()
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .pr_4()
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
        .child(
            div()
                .id(id)
                .w(px(44.0))
                .h(px(26.0))
                .rounded_full()
                .bg(bg)
                .p(px(3.0))
                .cursor_pointer()
                .on_click(on_click)
                .child(
                    div()
                        .w(px(20.0))
                        .h(px(20.0))
                        .rounded_full()
                        .bg(theme.foreground())
                        .shadow_sm()
                        .ml(if is_on { px(18.0) } else { px(0.0) }),
                ),
        )
}

#[allow(clippy::too_many_arguments)]
pub fn render_slider_row(
    field: SettingsField,
    title: &str,
    val_str: &str,
    unit: &str,
    min: f32,
    max: f32,
    step: f32,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement + use<> {
    let current_val: f32 = val_str.parse().unwrap_or(min);
    let pct = if max > min {
        ((current_val - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let display_val = if unit.is_empty() {
        format!("{current_val:.0}")
    } else {
        format!("{current_val:.0} {unit}")
    };

    let bounds_cell = Rc::new(Cell::new((0.0f32, 1.0f32)));
    let bounds_cell_clone = bounds_cell.clone();

    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py(px(3.0))
        .gap(px(6.0))
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
                        .child(display_val),
                ),
        )
        .child(
            div()
                .id(ElementId::NamedInteger("slider-track".into(), field as u64))
                .relative()
                .flex()
                .items_center()
                .w_full()
                .h(px(18.0))
                .cursor_pointer()
                .child(
                    canvas(
                        move |bounds, _, _| {
                            let left: f32 = bounds.origin.x.into();
                            let width: f32 = bounds.size.width.into();
                            bounds_cell_clone.set((left, width));
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                        let (left, width) = bounds_cell.get();
                        if width > 0.0 {
                            let x: f32 = event.position.x.into();
                            let rel = (x - left).clamp(0.0, width);
                            let p = rel / width;
                            let raw_val = min + p * (max - min);
                            let stepped = (raw_val / step).round() * step;
                            let final_val = stepped.clamp(min, max);
                            this.set_field_float(field, final_val, cx);
                            this.active_slider_drag =
                                Some((field, min, max, step, bounds_cell.clone()));
                        }
                    }),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(5.0))
                        .rounded_full()
                        .bg(style::raised(theme))
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(pct))
                                .rounded_full()
                                .bg(theme.accent()),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .left(gpui::relative(pct))
                        .ml(px(-14.0 * pct))
                        .w(px(14.0))
                        .h(px(14.0))
                        .rounded_full()
                        .bg(theme.foreground())
                        .shadow_sm(),
                ),
        )
}

pub fn render_int_slider_row(
    field: SettingsField,
    title: &str,
    val_secs: u64,
    theme: &Theme,
    cx: &mut Context<SettingsModule>,
) -> impl IntoElement {
    let val_mins = (val_secs / 60) as f32;
    let min = 0.0f32;
    let max = 60.0f32;
    let step = 1.0f32;

    let pct = ((val_mins - min) / (max - min)).clamp(0.0, 1.0);

    let display_val = if val_secs == 0 {
        if cx.has_global::<services::AppState>() {
            cx.global::<services::AppState>()
                .language
                .get("lockscreen.disabled")
        } else {
            "Desactivado".to_string()
        }
    } else {
        format!("{val_mins:.0} min")
    };

    let bounds_cell = Rc::new(Cell::new((0.0f32, 1.0f32)));
    let bounds_cell_clone = bounds_cell.clone();

    div()
        .flex()
        .flex_col()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py_3()
        .gap(px(6.0))
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
                        .child(display_val),
                ),
        )
        .child(
            div()
                .id(ElementId::NamedInteger(
                    "int-slider-track".into(),
                    field as u64,
                ))
                .relative()
                .flex()
                .items_center()
                .w_full()
                .h(px(18.0))
                .cursor_pointer()
                .child(
                    canvas(
                        move |bounds, _, _| {
                            let left: f32 = bounds.origin.x.into();
                            let width: f32 = bounds.size.width.into();
                            bounds_cell_clone.set((left, width));
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, event: &gpui::MouseDownEvent, _window, cx| {
                        let (left, width) = bounds_cell.get();
                        if width > 0.0 {
                            let x: f32 = event.position.x.into();
                            let rel = (x - left).clamp(0.0, width);
                            let p = rel / width;
                            let raw_mins = min + p * (max - min);
                            let stepped_mins = (raw_mins / step).round() * step;
                            let final_mins = stepped_mins.clamp(min, max);
                            let final_secs = (final_mins * 60.0) as u64;
                            this.set_field_int(field, final_secs, cx);
                            this.active_slider_drag =
                                Some((field, min, max, step, bounds_cell.clone()));
                        }
                    }),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(5.0))
                        .rounded_full()
                        .bg(style::raised(theme))
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(gpui::relative(pct))
                                .rounded_full()
                                .bg(theme.accent()),
                        ),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .left(gpui::relative(pct))
                        .ml(px(-14.0 * pct))
                        .w(px(14.0))
                        .h(px(14.0))
                        .rounded_full()
                        .bg(theme.foreground())
                        .shadow_sm(),
                ),
        )
}

pub fn render_control_row(
    title: &str,
    subtitle: Option<&str>,
    control: impl IntoElement,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .px(px(18.0))
        .py_3()
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .pr_4()
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
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .flex_shrink_0()
                .child(control),
        )
}

pub fn render_subsection_trigger_button(
    id: &'static str,
    label: &str,
    count: usize,
    theme: &Theme,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let text = if count > 0 {
        format!("{label} ({count})")
    } else {
        label.to_string()
    };

    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .px_3()
        .py_1p5()
        .rounded(px(8.0))
        .bg(theme.accent().opacity(0.12))
        .hover(|s| s.bg(theme.accent().opacity(0.22)))
        .active(|s| s.bg(theme.accent().opacity(0.32)))
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.accent())
                .child(text),
        )
        .child(
            svg()
                .path("chevron-right.svg")
                .size(px(12.0))
                .text_color(theme.accent()),
        )
}
