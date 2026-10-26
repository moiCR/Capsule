pub mod items;

use gpui::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px, svg,
};
use ui::theme::Theme;

use crate::settings::Settings;
pub use items::{SettingsTab, SidebarItem, TabSidebarItem};

pub struct Sidebar {
    pub items: Vec<Box<dyn SidebarItem>>,
}

impl Sidebar {
    pub fn new() -> Self {
        let items: Vec<Box<dyn SidebarItem>> = vec![
            Box::new(TabSidebarItem::new(
                SettingsTab::Appearance,
                "settings.tab_appearance",
                "palette.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::Island,
                "settings.tab_island",
                "island.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::Bar,
                "settings.tab_bar",
                "bar.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::Apps,
                "settings.tab_apps",
                "grid-dots.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::Media,
                "settings.tab_media",
                "music.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::LockScreen,
                "settings.tab_lockscreen",
                "lock.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::System,
                "settings.tab_system",
                "settings.svg",
            )),
            Box::new(TabSidebarItem::new(
                SettingsTab::Record,
                "settings.tab_record",
                "play.svg",
            )),
        ];

        Self { items }
    }

    pub fn render(
        &self,
        active_tab: SettingsTab,
        has_search_query: bool,
        theme: &Theme,
        cx: &mut Context<Settings>,
    ) -> AnyElement {
        div()
            .id("settings-tabs-scroll")
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w(px(184.0))
            .max_w(gpui::relative(0.5))
            .h_full()
            .min_h_0()
            .min_w_0()
            .flex_shrink_0()
            .overflow_y_scroll()
            .children(self.items.iter().map(|item| {
                let tab = item.tab();
                let selected = active_tab == tab && !has_search_query;
                let bg_color = if selected {
                    theme.primary_container()
                } else {
                    gpui::transparent_black()
                };
                let foreground = if selected {
                    theme.on_primary_container()
                } else {
                    theme.on_surface_variant()
                };
                let hover_bg = if selected {
                    theme.primary_hover()
                } else {
                    theme.surface_container_high()
                };
                let title = item.title(cx);
                let icon = item.icon();

                div()
                    .id(("settings-tab", tab as usize))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .w_full()
                    .min_w_0()
                    .whitespace_nowrap()
                    .h(px(44.0))
                    .px(px(16.0))
                    .flex_shrink_0()
                    .rounded_full()
                    .bg(bg_color)
                    .text_color(foreground)
                    .text_size(px(13.5))
                    .font_weight(if selected {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.set_tab(tab, cx);
                    }))
                    .child(
                        svg()
                            .path(icon)
                            .size(px(18.0))
                            .flex_shrink_0()
                            .text_color(foreground),
                    )
                    .child(div().flex_1().min_w_0().text_ellipsis().child(title))
            }))
            .into_any_element()
    }
}
