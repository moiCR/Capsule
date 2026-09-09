use crate::theme::Theme;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, canvas, deferred, div,
    px, svg,
};
use std::cell::Cell;
use std::rc::Rc;

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub type ClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct SelectItem {
    id: ElementId,
    label: SharedString,
    icon: Option<AnyElement>,
    selected: bool,
    on_click: Option<ClickListener>,
}

impl SelectItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            selected: false,
            on_click: None,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for SelectItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let mut row = div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .px_2p5()
            .py_1p5()
            .rounded(px(8.0))
            .bg(if self.selected {
                theme.accent().opacity(0.12)
            } else {
                gpui::transparent_black()
            })
            .text_color(if self.selected {
                theme.accent()
            } else {
                theme.foreground()
            })
            .hover(|s| s.bg(theme.surface().opacity(0.6)))
            .cursor_pointer();

        if let Some(on_click) = self.on_click {
            row = row.on_click(move |evt, window, app| {
                on_click(evt, window, app);
            });
        }

        row.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap_2()
                .children(self.icon)
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(if self.selected {
                            gpui::FontWeight::SEMIBOLD
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(self.label),
                ),
        )
        .children(if self.selected {
            Some(
                div()
                    .size(px(6.0))
                    .rounded_full()
                    .bg(theme.accent())
                    .flex_shrink_0(),
            )
        } else {
            None
        })
    }
}

#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    selected_label: Option<SharedString>,
    placeholder: SharedString,
    icon: Option<AnyElement>,
    is_open: bool,
    anim_progress: f32,
    open_upwards: bool,
    width: f32,
    max_menu_height: f32,
    items: Vec<AnyElement>,
    on_toggle: Option<ClickListener>,
    bounds_cell: Option<Rc<Cell<(f32, f32)>>>,
}

impl Select {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected_label: None,
            placeholder: "Select...".into(),
            icon: None,
            is_open: false,
            anim_progress: 1.0,
            open_upwards: false,
            width: 240.0,
            max_menu_height: 220.0,
            items: Vec::new(),
            on_toggle: None,
            bounds_cell: None,
        }
    }

    pub fn selected_label(mut self, label: impl Into<SharedString>) -> Self {
        self.selected_label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn anim_progress(mut self, progress: f32) -> Self {
        self.anim_progress = progress;
        self
    }

    pub fn open_upwards(mut self, open_upwards: bool) -> Self {
        self.open_upwards = open_upwards;
        self
    }

    pub fn track_bounds(mut self, cell: Rc<Cell<(f32, f32)>>) -> Self {
        self.bounds_cell = Some(cell);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn max_menu_height(mut self, max_height: f32) -> Self {
        self.max_menu_height = max_height;
        self
    }

    pub fn on_toggle<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_toggle = Some(Box::new(listener));
        self
    }

    pub fn item(mut self, item: impl IntoElement) -> Self {
        self.items.push(item.into_any_element());
        self
    }

    pub fn items<I, E>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = E>,
        E: IntoElement,
    {
        for item in items {
            self.items.push(item.into_any_element());
        }
        self
    }
}

impl ParentElement for Select {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.items.extend(elements);
    }
}

impl RenderOnce for Select {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _bounds = window.bounds();
        let theme = cx.global::<Theme>();
        let display_label = self
            .selected_label
            .clone()
            .unwrap_or_else(|| self.placeholder.clone());
        let is_placeholder = self.selected_label.is_none();

        let mut trigger = div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w(px(self.width))
            .h(px(36.0))
            .px_3()
            .rounded(px(10.0))
            .bg(if self.is_open {
                theme.accent().opacity(0.1)
            } else {
                theme.surface().opacity(0.3)
            })
            .border_1()
            .border_color(if self.is_open {
                theme.accent()
            } else {
                theme.surface().opacity(0.4)
            })
            .hover(|s| {
                if !self.is_open {
                    s.border_color(theme.accent().opacity(0.7))
                        .bg(theme.surface().opacity(0.5))
                } else {
                    s
                }
            })
            .cursor_pointer();

        if let Some(on_toggle) = self.on_toggle {
            trigger = trigger.on_click(move |evt, window, app| {
                on_toggle(evt, window, app);
            });
        }

        if let Some(cell) = self.bounds_cell {
            trigger = trigger.child(
                canvas(
                    move |bounds, _, _| {
                        let top: f32 = bounds.origin.y.into();
                        let height: f32 = if bounds.size.height > px(0.0) {
                            bounds.size.height.into()
                        } else {
                            36.0
                        };
                        cell.set((top, top + height));
                    },
                    |_, _, _, _| {},
                )
                .inset_0()
                .absolute(),
            );
        }

        let trigger_content = div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .min_w_0()
            .flex_1()
            .children(self.icon)
            .child(
                div()
                    .text_size(px(12.5))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(if is_placeholder {
                        theme.foreground_muted()
                    } else {
                        theme.foreground()
                    })
                    .truncate()
                    .child(display_label),
            );

        let chevron = svg()
            .path("chevron-down.svg")
            .size(px(14.0))
            .flex_shrink_0()
            .text_color(if self.is_open {
                theme.accent()
            } else {
                theme.foreground_muted()
            });

        let trigger_element = trigger.child(trigger_content).child(chevron);

        let dropdown_menu = if self.is_open && self.anim_progress > 0.0 {
            let p = ease_out_cubic(self.anim_progress);
            let target_max = self.max_menu_height.max(32.0);
            let min_h = 32.0f32.min(target_max);
            let menu_h = (target_max * p).clamp(min_h, target_max);
            let offset = 40.0 - 4.0 * (1.0 - p);

            let mut menu = div()
                .id("select-menu-dropdown")
                .absolute()
                .right_0()
                .w(px(self.width))
                .max_h(px(menu_h))
                .opacity(p)
                .bg(theme.background())
                .border_1()
                .border_color(theme.surface().opacity(0.5))
                .rounded(px(12.0))
                .shadow_lg()
                .p_1p5()
                .gap(px(2.0))
                .flex()
                .flex_col()
                .overflow_scroll()
                .occlude()
                .children(self.items);

            if self.open_upwards {
                menu = menu.bottom(px(offset));
            } else {
                menu = menu.top(px(offset));
            }

            Some(deferred(menu).priority(10))
        } else {
            None
        };

        div()
            .relative()
            .child(trigger_element)
            .children(dropdown_menu)
    }
}
