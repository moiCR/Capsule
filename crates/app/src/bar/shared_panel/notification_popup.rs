use std::path::PathBuf;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    App, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent, Pixels, Render,
    Size, Task, Window, div, img, px, svg,
};
use services::NotificationStore;
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub struct NotificationPopupModule {
    pub notification_id: u32,
    focus: FocusHandle,
    _store_task: Task<()>,
    expiry_task: Option<Task<()>>,
}

impl NotificationPopupModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut rx = NotificationStore::global().subscribe();
        let store_task = cx.spawn(async move |this, cx| {
            while rx.changed().await.is_ok() {
                if this
                    .update(cx, |_, cx| {
                        cx.emit(SharedPanelEvent::NotificationChanged);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            notification_id: 0,
            focus: cx.focus_handle(),
            _store_task: store_task,
            expiry_task: None,
        }
    }

    pub fn open(&mut self, id: u32, timeout: Duration, cx: &mut Context<Self>) {
        self.notification_id = id;
        let delay = if timeout.is_zero() {
            Duration::from_secs(5)
        } else {
            timeout
        };
        self.expiry_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            if NotificationStore::global().is_hovered() {
                return;
            }
            let _ = this.update(cx, |module, cx| {
                if module.notification_id == id {
                    cx.emit(SharedPanelEvent::Close);
                }
            });
        }));
    }
}

impl EventEmitter<SharedPanelEvent> for NotificationPopupModule {}

impl SharedPanelItem for NotificationPopupModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        let id = self.notification_id;
        let notif_opt = NotificationStore::global()
            .get_latest_active_notification()
            .filter(|n| n.id == id)
            .or_else(|| {
                NotificationStore::global()
                    .get_all_notifications()
                    .into_iter()
                    .find(|n| n.id == id)
            });

        let (has_body, has_actions) = if let Some(ref item) = notif_opt {
            (!item.body.trim().is_empty(), !item.actions.is_empty())
        } else {
            (false, false)
        };

        let mut h = 108.0;
        if has_body {
            h += 46.0;
        }
        if has_actions {
            h += 38.0;
        }

        Size {
            width: px(440.0),
            height: px(h),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }
}

impl Render for NotificationPopupModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let id = self.notification_id;
        let theme = cx.global::<Theme>().clone();
        let notif_opt = NotificationStore::global()
            .get_latest_active_notification()
            .filter(|n| n.id == id)
            .or_else(|| {
                NotificationStore::global()
                    .get_all_notifications()
                    .into_iter()
                    .find(|n| n.id == id)
            });

        let Some(item) = notif_opt else {
            return div().into_any_element();
        };

        let icon_el = if !item.app_icon.is_empty()
            && (item.app_icon.starts_with('/') || item.app_icon.contains('.'))
        {
            img(PathBuf::from(&item.app_icon))
                .size(px(22.0))
                .rounded_md()
                .into_any_element()
        } else {
            svg()
                .path("bell.svg")
                .size(px(18.0))
                .text_color(theme.accent())
                .into_any_element()
        };

        let app_name = if item.app_name.is_empty() {
            "Sistema".to_string()
        } else {
            item.app_name.clone()
        };

        let hover_icon = style::hover(&theme);
        let hover_chip = style::hover(&theme);

        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(26.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(icon_el)
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted())
                            .line_clamp(1)
                            .child(app_name),
                    ),
            )
            .child(
                div()
                    .id("popup-notif-close")
                    .size(px(22.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_icon))
                    .child(
                        svg()
                            .path("close.svg")
                            .size(px(12.0))
                            .text_color(theme.foreground_muted()),
                    )
                    .on_click(cx.listener(move |_, _, _, cx| {
                        NotificationStore::global().remove_notification(id);
                        cx.emit(SharedPanelEvent::Close);
                    })),
            );

        let mut content = div()
            .id("shared-panel-notification-popup")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    cx.emit(SharedPanelEvent::Close);
                }
            }))
            .size_full()
            .px(px(18.0))
            .py(px(12.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .font_family(theme.font_family())
            .on_hover(cx.listener(|_, hovered, _, _| {
                NotificationStore::global().set_hovered(*hovered);
            }))
            .child(header);

        if !item.summary.is_empty() {
            content = content.child(
                div()
                    .text_size(px(14.5))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.foreground())
                    .line_clamp(1)
                    .child(item.summary),
            );
        }

        if !item.body.is_empty() {
            content = content.child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.foreground_muted())
                    .line_clamp(2)
                    .child(item.body),
            );
        }

        if !item.actions.is_empty() {
            let mut actions_row = div().flex().items_center().gap(px(8.0)).pt(px(4.0));
            for (idx, (key, label)) in item.actions.into_iter().enumerate() {
                let key_clone = key.clone();
                let label_str = if !label.is_empty() {
                    label
                } else {
                    "Abrir".to_string()
                };
                let chip_hover = hover_chip;
                actions_row = actions_row.child(
                    div()
                        .id(format!("notif-popup-act-{id}-{idx}"))
                        .h(px(24.0))
                        .px(px(10.0))
                        .rounded(px(6.0))
                        .bg(style::surface(&theme).opacity(0.85))
                        .border_1()
                        .border_color(style::border(&theme).opacity(0.16))
                        .hover(move |s| s.bg(chip_hover))
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .text_size(px(11.5))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground())
                        .child(label_str)
                        .on_click(cx.listener(move |_, _, _, cx| {
                            NotificationStore::global().invoke_action(id, key_clone.clone());
                            NotificationStore::global().remove_notification(id);
                            cx.emit(SharedPanelEvent::Close);
                        })),
                );
            }
            content = content.child(actions_row);
        }

        content.into_any_element()
    }
}
