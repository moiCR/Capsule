use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use chrono::{Datelike, Duration as ChronoDuration, Local, NaiveDate};
use gpui::prelude::*;
use gpui::{
    App, BoxShadow, ColorExt, Context, EventEmitter, FocusHandle, IntoElement, KeyDownEvent,
    Pixels, Render, Size, Task, Window, div, img, point, px, svg,
};
use services::{AppState, CalendarNotesService, NotificationStore};
use ui::components::{Switch, TextArea, TextAreaState};
use ui::theme::Theme;

use super::{SharedPanelEvent, SharedPanelItem};
use crate::bar::style;

pub const NOTIF_ENTER_DURATION: f32 = 0.18;
pub const NOTIF_EXIT_DURATION: f32 = 0.18;
pub const OVERLAY_DURATION: f32 = 0.18;

pub struct CalendarModule {
    focus: FocusHandle,
    seen_ids: HashSet<u32>,
    entering: HashMap<u32, Instant>,
    dismissing: HashMap<u32, Instant>,
    anim_task: Option<Task<()>>,
    selected_date: Option<NaiveDate>,
    note_state: TextAreaState,
    overlay_open: bool,
    overlay_progress: f32,
    overlay_anim_start: Option<Instant>,
    overlay_closing: bool,
}

impl CalendarModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_ids = NotificationStore::global()
            .get_all_notifications()
            .into_iter()
            .map(|n| n.id)
            .collect();

        let mut sub = NotificationStore::global().subscribe();
        cx.spawn(async move |this, cx| {
            while sub.changed().await.is_ok() {
                let _ = this.update(cx, |module, cx| {
                    module.check_new_notifications(cx);
                    cx.notify();
                });
            }
        })
        .detach();

        Self {
            focus: cx.focus_handle(),
            seen_ids: initial_ids,
            entering: HashMap::new(),
            dismissing: HashMap::new(),
            anim_task: None,
            selected_date: None,
            note_state: TextAreaState::new(""),
            overlay_open: false,
            overlay_progress: 0.0,
            overlay_anim_start: None,
            overlay_closing: false,
        }
    }

    pub fn check_new_notifications(&mut self, cx: &mut Context<Self>) {
        let notifs = NotificationStore::global().get_all_notifications();
        let now = Instant::now();
        let mut has_new = false;
        for notif in &notifs {
            if !self.seen_ids.contains(&notif.id) {
                self.seen_ids.insert(notif.id);
                self.entering.insert(notif.id, now);
                has_new = true;
            }
        }
        let current_ids: HashSet<u32> = notifs.iter().map(|n| n.id).collect();
        self.seen_ids
            .retain(|id| current_ids.contains(id) || self.dismissing.contains_key(id));

        if has_new {
            self.spawn_anim_task(cx);
        }
    }

    pub fn dismiss_notification(&mut self, id: u32, cx: &mut Context<Self>) {
        self.dismissing.insert(id, Instant::now());
        self.spawn_anim_task(cx);
        cx.notify();
    }

    pub fn dismiss_all_notifications(&mut self, cx: &mut Context<Self>) {
        let notifs = NotificationStore::global().get_all_notifications();
        let now = Instant::now();
        for notif in notifs {
            self.dismissing.insert(notif.id, now);
        }
        self.spawn_anim_task(cx);
        cx.notify();
    }

    pub fn open_note_subview(&mut self, date: NaiveDate, cx: &mut Context<Self>) {
        let key = format!("{}-{:02}-{:02}", date.year(), date.month(), date.day());
        let content = if let Some(state) = cx.try_global::<AppState>() {
            state.calendar_notes.get_note(&key).unwrap_or_default()
        } else {
            CalendarNotesService::global()
                .get_note(&key)
                .unwrap_or_default()
        };

        let placeholder = if let Some(state) = cx.try_global::<AppState>() {
            state.language.get("calendar_notes.placeholder")
        } else {
            "Escribe una nota para este día...".to_string()
        };

        self.selected_date = Some(date);
        self.note_state = TextAreaState::new(content)
            .with_placeholder(placeholder)
            .with_focused(true);
        self.overlay_open = true;
        self.overlay_closing = false;
        self.overlay_anim_start = Some(Instant::now());
        if self.overlay_progress <= 0.01 {
            self.overlay_progress = 0.01;
        }
        self.spawn_anim_task(cx);
        cx.emit(SharedPanelEvent::SyncWindow);
        cx.notify();
    }

    pub fn close_note_subview(&mut self, cx: &mut Context<Self>) {
        self.save_current_note(cx);
        self.overlay_closing = true;
        self.overlay_anim_start = Some(Instant::now());
        self.spawn_anim_task(cx);
        cx.emit(SharedPanelEvent::SyncWindow);
        cx.notify();
    }

    pub fn save_current_note(&mut self, cx: &mut Context<Self>) {
        if let Some(date) = self.selected_date {
            let key = format!("{}-{:02}-{:02}", date.year(), date.month(), date.day());
            let text = self.note_state.text();
            if let Some(state) = cx.try_global::<AppState>() {
                state.calendar_notes.set_note(key, text);
            } else {
                CalendarNotesService::global().set_note(key, text);
            }
        }
    }

    pub fn delete_current_note(&mut self, cx: &mut Context<Self>) {
        if let Some(date) = self.selected_date {
            let key = format!("{}-{:02}-{:02}", date.year(), date.month(), date.day());
            if let Some(state) = cx.try_global::<AppState>() {
                state.calendar_notes.delete_note(&key);
            } else {
                CalendarNotesService::global().delete_note(&key);
            }
            self.note_state.clear();
            cx.notify();
        }
    }

    pub fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        if self.overlay_open {
            if key == "escape" {
                self.close_note_subview(cx);
                cx.stop_propagation();
                return;
            }

            if self.note_state.handle_key_down(event, cx) {
                self.save_current_note(cx);
                cx.notify();
                cx.stop_propagation();
            }
            return;
        }

        if key == "escape" {
            cx.emit(SharedPanelEvent::Close);
            cx.stop_propagation();
        }
    }

    fn spawn_anim_task(&mut self, cx: &mut Context<Self>) {
        if self.anim_task.is_some() {
            return;
        }
        self.anim_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;
                let mut keep_running = false;
                let updated = this.update(cx, |module, cx| {
                    keep_running = module.advance_animations();
                    cx.notify();
                });
                if updated.is_err() || !keep_running {
                    if let Some(handle) = this.upgrade() {
                        let _ = handle.update(cx, |module, _| {
                            module.anim_task = None;
                        });
                    }
                    break;
                }
            }
        }));
    }

    fn advance_animations(&mut self) -> bool {
        let now = Instant::now();
        let mut keep_running = false;

        if let Some(start) = self.overlay_anim_start {
            let elapsed = now.duration_since(start).as_secs_f32();
            let raw = (elapsed / OVERLAY_DURATION).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - raw).powi(3);

            if self.overlay_closing {
                self.overlay_progress = (1.0 - eased).clamp(0.0, 1.0);
                if raw >= 1.0 {
                    self.overlay_progress = 0.0;
                    self.overlay_open = false;
                    self.overlay_anim_start = None;
                    self.overlay_closing = false;
                    self.selected_date = None;
                } else {
                    keep_running = true;
                }
            } else {
                self.overlay_progress = eased.clamp(0.0, 1.0);
                if raw >= 1.0 {
                    self.overlay_progress = 1.0;
                    self.overlay_anim_start = None;
                } else {
                    keep_running = true;
                }
            }
        }

        self.entering.retain(|_, start| {
            let elapsed = now.duration_since(*start).as_secs_f32();
            if elapsed < NOTIF_ENTER_DURATION {
                keep_running = true;
                true
            } else {
                false
            }
        });

        let mut finished_dismiss = Vec::new();
        for (&id, &start) in &self.dismissing {
            let elapsed = now.duration_since(start).as_secs_f32();
            if elapsed >= NOTIF_EXIT_DURATION {
                finished_dismiss.push(id);
            } else {
                keep_running = true;
            }
        }

        for id in finished_dismiss {
            self.dismissing.remove(&id);
            self.seen_ids.remove(&id);
            NotificationStore::global().remove_notification(id);
        }

        keep_running
    }

    fn render_notes_overlay(&mut self, theme: &Theme, cx: &mut Context<Self>) -> impl IntoElement {
        let progress = self.overlay_progress.clamp(0.0, 1.0);
        let offset_y = (1.0 - progress) * 18.0;
        let hover_bg = style::hover(theme);

        let (header_date_str, back_label) = if let Some(date) = self.selected_date {
            let app_state = cx.try_global::<AppState>();
            let (m_name, w_name, b_label) = if let Some(s) = app_state {
                let l = &s.language;
                let mn = l
                    .get_list("datetime.months")
                    .get(date.month().saturating_sub(1) as usize)
                    .cloned()
                    .unwrap_or_else(|| date.month().to_string());
                let wn = l
                    .get_list("datetime.days")
                    .get(date.weekday().num_days_from_monday() as usize)
                    .cloned()
                    .unwrap_or_else(|| date.weekday().to_string());
                (mn, wn, l.get("calendar_notes.back"))
            } else {
                (
                    date.month().to_string(),
                    date.weekday().to_string(),
                    "Volver".to_string(),
                )
            };
            (
                format!("{}, {} de {} {}", w_name, date.day(), m_name, date.year()),
                b_label,
            )
        } else {
            (String::new(), "Volver".to_string())
        };

        let header = div()
            .h(px(36.0))
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .child(
                        div()
                            .id("calendar-notes-back-btn")
                            .h(px(30.0))
                            .px(px(10.0))
                            .rounded_full()
                            .bg(theme.surface_container_high())
                            .border_1()
                            .border_color(theme.outline_variant().opacity(0.40))
                            .flex()
                            .items_center()
                            .gap(px(5.0))
                            .cursor_pointer()
                            .hover(move |s| s.bg(hover_bg))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.close_note_subview(cx);
                            }))
                            .child(
                                svg()
                                    .path("chevron-left.svg")
                                    .size(px(14.0))
                                    .text_color(theme.foreground()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.foreground())
                                    .child(back_label),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(15.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .child(header_date_str),
                    ),
            )
            .when(!self.note_state.is_empty(), |h| {
                h.child(
                    div()
                        .id("calendar-notes-delete-btn")
                        .size(px(30.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(theme.red().opacity(0.18)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.delete_current_note(cx);
                        }))
                        .child(
                            svg()
                                .path("trash.svg")
                                .size(px(14.0))
                                .text_color(theme.red()),
                        ),
                )
            });

        let body = div().flex_1().min_h_0().w_full().child(
            TextArea::new("calendar-note-textarea", &self.note_state)
                .bordered(false)
                .bg(gpui::transparent_black())
                .padding(px(4.0))
                .font_size(px(15.0))
                .line_height(px(26.0))
                .on_mouse_down(cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                    let theme = cx.global::<Theme>().clone();
                    this.note_state.handle_mouse_down(event, window, &theme);
                    cx.notify();
                }))
                .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, window, cx| {
                    if this.note_state.is_dragging {
                        let theme = cx.global::<Theme>().clone();
                        this.note_state.handle_mouse_move(event, window, &theme);
                        cx.notify();
                    }
                }))
                .on_mouse_up(cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.note_state.handle_mouse_up(event);
                    cx.notify();
                })),
        );

        div()
            .id("calendar-notes-overlay-backdrop")
            .absolute()
            .inset_0()
            .backdrop_blur(px(16.0 * progress))
            .bg(gpui::black().opacity(0.30 * progress))
            .flex()
            .flex_col()
            .p(px(8.0))
            .child(
                div()
                    .size_full()
                    .relative()
                    .top(px(offset_y))
                    .opacity(progress)
                    .rounded(px(18.0))
                    .bg(style::background(theme).opacity(0.96))
                    .border_1()
                    .border_color(style::border(theme).opacity(0.24))
                    .shadow(vec![BoxShadow {
                        color: gpui::black().opacity(0.40).into(),
                        offset: point(px(0.0), px(8.0)),
                        blur_radius: px(24.0),
                        spread_radius: px(0.0),
                        inset: false,
                    }])
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .child(header)
                    .child(body),
            )
    }
}

impl EventEmitter<SharedPanelEvent> for CalendarModule {}

impl SharedPanelItem for CalendarModule {
    fn size(&self, _cx: &App) -> Size<Pixels> {
        Size {
            width: px(720.0),
            height: px(450.0),
        }
    }

    fn focus_handle(&self) -> Option<FocusHandle> {
        Some(self.focus.clone())
    }

    fn needs_exclusive_keyboard(&self) -> bool {
        self.overlay_open
    }
}

impl Render for CalendarModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let now = Local::now();
        let app_state = cx.try_global::<AppState>();

        let (year, month) = if let Some(state) = app_state {
            state.calendar.get_view_date()
        } else {
            (now.year(), now.month())
        };

        let (
            time_str,
            weekday_full,
            month_name,
            weekdays_short,
            no_notifs_label,
            dnd_label,
            clear_label,
        ) = if let Some(state) = app_state {
            let fmt = state.config.get().ui.clock_format();
            let t_str = now.format(fmt).to_string();
            let w_full = state
                .language
                .get_list("datetime.days")
                .get(now.weekday().num_days_from_monday() as usize)
                .cloned()
                .unwrap_or_else(|| now.weekday().to_string());
            let m_name = state
                .language
                .get_list("datetime.months")
                .get(month.saturating_sub(1) as usize)
                .cloned()
                .unwrap_or_else(|| month.to_string());
            let w_short = state.language.get_list("datetime.weekdays_short");
            let no_notifs = state.language.get("dashboard.no_notifications");
            let dnd_txt = state.language.get("dashboard.dnd");
            let clear_txt = if state.language.current_target() == "es" {
                "Limpiar".to_string()
            } else {
                "Clear".to_string()
            };
            (
                t_str, w_full, m_name, w_short, no_notifs, dnd_txt, clear_txt,
            )
        } else {
            (
                now.format("%H:%M").to_string(),
                "Domingo".to_string(),
                month.to_string(),
                vec![
                    "Lun".into(),
                    "Mar".into(),
                    "Mié".into(),
                    "Jue".into(),
                    "Vie".into(),
                    "Sáb".into(),
                    "Dom".into(),
                ],
                "Sin notificaciones".to_string(),
                "No molestar".to_string(),
                "Limpiar".to_string(),
            )
        };

        let notifications = NotificationStore::global().get_all_notifications();
        let is_dnd = NotificationStore::global().is_dnd_enabled();
        let hover_bg = style::hover(&theme);

        let notif_card_content = if notifications.is_empty() {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(14.0))
                .child(
                    svg()
                        .path("bell.svg")
                        .size(px(62.0))
                        .text_color(theme.accent()),
                )
                .child(
                    div()
                        .text_size(px(15.5))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground_muted())
                        .child(no_notifs_label),
                )
                .into_any_element()
        } else {
            let mut list = div()
                .id("shared-notifications-list")
                .size_full()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(10.0))
                .p(px(14.0));

            for notif in notifications {
                let id = notif.id;
                let is_dismissing = self.dismissing.get(&id).copied();
                let is_entering = self.entering.get(&id).copied();

                let (opacity, offset_x, offset_y) = if let Some(start) = is_dismissing {
                    let elapsed = start.elapsed().as_secs_f32();
                    let raw = (elapsed / NOTIF_EXIT_DURATION).clamp(0.0, 1.0);
                    let eased = raw.powi(2);
                    (1.0 - eased, eased * 36.0, 0.0)
                } else if let Some(start) = is_entering {
                    let elapsed = start.elapsed().as_secs_f32();
                    let raw = (elapsed / NOTIF_ENTER_DURATION).clamp(0.0, 1.0);
                    let eased = 1.0 - (1.0 - raw).powi(3);
                    (eased, 0.0, (1.0 - eased) * -12.0)
                } else {
                    (1.0, 0.0, 0.0)
                };

                let icon_el = if !notif.app_icon.is_empty()
                    && (notif.app_icon.starts_with('/') || notif.app_icon.contains('.'))
                {
                    img(PathBuf::from(&notif.app_icon))
                        .size(px(22.0))
                        .rounded_md()
                        .into_any_element()
                } else {
                    svg()
                        .path("bell.svg")
                        .size(px(17.0))
                        .text_color(theme.accent())
                        .into_any_element()
                };

                let mut item_card = div()
                    .relative()
                    .opacity(opacity)
                    .left(px(offset_x))
                    .top(px(offset_y))
                    .p(px(12.0))
                    .rounded(px(12.0))
                    .bg(theme.surface_container_high())
                    .border_1()
                    .border_color(theme.outline_variant())
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(icon_el)
                                    .child(
                                        div()
                                            .text_size(px(12.0))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(theme.foreground_muted())
                                            .line_clamp(1)
                                            .child(if notif.app_name.is_empty() {
                                                "Sistema".to_string()
                                            } else {
                                                notif.app_name.clone()
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .id(format!("shared-notif-remove-{id}"))
                                    .size(px(22.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(hover_bg))
                                    .child(
                                        svg()
                                            .path("close.svg")
                                            .size(px(12.0))
                                            .text_color(theme.foreground_muted()),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.dismiss_notification(id, cx);
                                    })),
                            ),
                    )
                    .when(!notif.summary.is_empty(), |s| {
                        s.child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.foreground())
                                .line_clamp(1)
                                .child(notif.summary),
                        )
                    })
                    .when(!notif.body.is_empty(), |s| {
                        s.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(theme.foreground_muted())
                                .line_clamp(2)
                                .child(notif.body),
                        )
                    });

                if !notif.actions.is_empty() {
                    let mut actions_row = div().flex().items_center().gap(px(6.0)).pt(px(2.0));
                    for (act_idx, (key, label)) in notif.actions.into_iter().enumerate() {
                        let key_clone = key.clone();
                        let label_str = if label.is_empty() {
                            "Abrir".to_string()
                        } else {
                            label
                        };
                        actions_row = actions_row.child(
                            div()
                                .id(format!("shared-notif-act-{id}-{act_idx}"))
                                .h(px(24.0))
                                .px(px(10.0))
                                .rounded(px(6.0))
                                .bg(style::surface(&theme).opacity(0.85))
                                .border_1()
                                .border_color(style::border(&theme).opacity(0.16))
                                .hover(move |s| s.bg(hover_bg))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .text_size(px(11.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(theme.foreground())
                                .child(label_str)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    NotificationStore::global()
                                        .invoke_action(id, key_clone.clone());
                                    this.dismiss_notification(id, cx);
                                })),
                        );
                    }
                    item_card = item_card.child(actions_row);
                }

                list = list.child(item_card);
            }

            list.into_any_element()
        };

        let left_column = div()
            .flex_1()
            .h_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .rounded(px(14.0))
                    .bg(style::surface(&theme).opacity(0.55))
                    .border_1()
                    .border_color(style::border(&theme).opacity(0.18))
                    .overflow_hidden()
                    .child(notif_card_content),
            )
            .child(
                div()
                    .h(px(32.0))
                    .w_full()
                    .flex_shrink_0()
                    .px(px(4.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .id("shared-notif-dnd-row")
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, cx| {
                                NotificationStore::global().toggle_dnd();
                                cx.notify();
                            }))
                            .child(
                                Switch::new("shared-notif-dnd-switch")
                                    .checked(is_dnd)
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        cx.stop_propagation();
                                        NotificationStore::global().toggle_dnd();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.foreground())
                                    .child(dnd_label),
                            ),
                    )
                    .child({
                        let clear_btn_hover = theme.surface_container_highest();
                        div()
                            .id("shared-notif-clear-btn")
                            .h(px(28.0))
                            .px(px(14.0))
                            .rounded_full()
                            .bg(theme.surface_container_high())
                            .border_1()
                            .border_color(theme.outline_variant())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(move |s| s.bg(clear_btn_hover))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.dismiss_all_notifications(cx);
                            }))
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(theme.foreground())
                                    .child(clear_label),
                            )
                    }),
            );

        let today = now.date_naive();
        let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(today);
        let offset = first.weekday().num_days_from_monday() as i64;

        let mut weekdays_row = div().flex().items_center().h(px(26.0)).gap(px(4.0));
        for day_label in weekdays_short.into_iter().take(7) {
            weekdays_row = weekdays_row.child(
                div()
                    .flex_1()
                    .text_center()
                    .text_size(px(12.0))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.foreground_muted())
                    .child(day_label),
            );
        }

        let mut grid = div().flex_1().flex().flex_col().gap(px(4.0));
        for week in 0..6 {
            let mut row = div().flex_1().flex().gap(px(4.0));
            for weekday in 0..7 {
                let delta = (week * 7 + weekday) as i64 - offset;
                let cell_date = first + ChronoDuration::days(delta);
                let is_current_month = cell_date.month() == month && cell_date.year() == year;
                let is_today = cell_date == today;

                let cell_key =
                    format!("{}-{:02}-{:02}", cell_date.year(), cell_date.month(), cell_date.day());
                let has_note = if let Some(state) = app_state {
                    state.calendar_notes.has_note(&cell_key)
                } else {
                    CalendarNotesService::global().has_note(&cell_key)
                };

                let mut cell = div()
                    .id(format!("calendar-cell-{week}-{weekday}"))
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(1.0))
                    .rounded_full()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.open_note_subview(cell_date, cx);
                    }))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .line_height(px(14.0))
                            .child(cell_date.day().to_string()),
                    );

                if has_note {
                    cell = cell.child(
                        div()
                            .size(px(4.0))
                            .rounded_full()
                            .bg(if is_today {
                                theme.on_accent()
                            } else {
                                theme.accent()
                            }),
                    );
                } else {
                    cell = cell.child(div().size(px(4.0)));
                }

                if is_today {
                    cell = cell
                        .bg(theme.accent())
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.on_accent());
                } else if is_current_month {
                    cell = cell
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme.foreground())
                        .hover(move |s| s.bg(hover_bg));
                } else {
                    cell = cell
                        .text_color(theme.foreground_muted().opacity(0.38))
                        .hover(move |s| s.bg(hover_bg));
                }

                row = row.child(cell);
            }
            grid = grid.child(row);
        }

        let calendar_card = div()
            .flex_1()
            .w_full()
            .rounded(px(16.0))
            .bg(theme.surface_container_low())
            .border_1()
            .border_color(theme.outline_variant())
            .p(px(14.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(30.0))
                    .child(
                        div()
                            .id("shared-calendar-header-title")
                            .flex()
                            .items_center()
                            .cursor_pointer()
                            .on_click(cx.listener(|_, _, _, cx| {
                                if let Some(state) = cx.try_global::<AppState>() {
                                    state.calendar.reset_to_today();
                                    cx.notify();
                                }
                            }))
                            .child(
                                div()
                                    .text_size(px(15.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.accent())
                                    .child(format!("{month_name} {year}")),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .id("shared-calendar-prev-btn")
                                    .size(px(26.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(hover_bg))
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        if let Some(state) = cx.try_global::<AppState>() {
                                            state.calendar.prev_month();
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        svg()
                                            .path("chevron-left.svg")
                                            .size(px(14.0))
                                            .text_color(theme.foreground_muted()),
                                    ),
                            )
                            .child(
                                div()
                                    .id("shared-calendar-next-btn")
                                    .size(px(26.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .hover(move |s| s.bg(hover_bg))
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        if let Some(state) = cx.try_global::<AppState>() {
                                            state.calendar.next_month();
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        svg()
                                            .path("chevron-right.svg")
                                            .size(px(14.0))
                                            .text_color(theme.foreground_muted()),
                                    ),
                            ),
                    ),
            )
            .child(weekdays_row)
            .child(grid);

        let right_column = div()
            .w(px(308.0))
            .h_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .py(px(4.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_size(px(40.0))
                            .line_height(px(44.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.foreground())
                            .child(time_str),
                    )
                    .child(
                        div()
                            .text_size(px(18.0))
                            .line_height(px(22.0))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme.accent())
                            .child(weekday_full),
                    ),
            )
            .child(calendar_card);

        let mut root = div()
            .id("shared-panel-calendar")
            .relative()
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::handle_key_down))
            .size_full()
            .p(px(14.0))
            .flex()
            .items_center()
            .gap(px(14.0))
            .font_family(theme.font_family())
            .child(left_column)
            .child(right_column);

        if self.overlay_open || self.overlay_progress > 0.001 {
            root = root.child(self.render_notes_overlay(&theme, cx));
        }

        root
    }
}
