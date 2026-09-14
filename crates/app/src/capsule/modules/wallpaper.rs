use gpui::{
    Context, ElementId, EventEmitter, FocusHandle, Focusable, FontWeight, IntoElement,
    KeyDownEvent, Render, Task, Window, div, img, prelude::*, px, svg,
};
use services::AppState;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use ui::theme::Theme;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallpaperItem {
    pub name: String,
    pub path: PathBuf,
    pub thumb_path: PathBuf,
}

fn get_or_create_thumbnail(original_path: &Path) -> PathBuf {
    let thumbs_dir = PathBuf::from("/tmp/capsule_thumbs");
    let _ = std::fs::create_dir_all(&thumbs_dir);

    let filename = original_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("thumb");
    let thumb_filename = format!("thumb_{}", filename);
    let thumb_path = thumbs_dir.join(&thumb_filename);

    if thumb_path.exists() {
        return thumb_path;
    }

    let orig_str = original_path.to_string_lossy().to_string();
    let thumb_str = thumb_path.to_string_lossy().to_string();

    let res = Command::new("magick")
        .args([&orig_str, "-resize", "300x", &thumb_str])
        .status();

    if let Ok(st) = res
        && st.success()
        && thumb_path.exists()
    {
        return thumb_path;
    }

    let res_ff = Command::new("ffmpeg")
        .args(["-y", "-i", &orig_str, "-vf", "scale=300:-1", &thumb_str])
        .status();

    if let Ok(st) = res_ff
        && st.success()
        && thumb_path.exists()
    {
        return thumb_path;
    }

    // Both tools failed: create a minimal 1x1 placeholder PNG to avoid loading
    // full-resolution (4K/8K) images into GPUI's texture memory
    let placeholder_path = thumbs_dir.join(format!("placeholder_{}", filename));
    if !placeholder_path.exists() {
        // Minimal valid 1x1 grey PNG (67 bytes)
        let png_data: [u8; 69] = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
            0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
            0xD7, 0x63, 0x60, 0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0xF6, 0x17, 0x8A,
            0x44, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let _ = std::fs::write(&placeholder_path, png_data);
    }
    placeholder_path
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperEvent {
    CloseRequested,
    WallpaperSelected(PathBuf),
}

pub struct WallpaperModule {
    pub focus_handle: FocusHandle,
    pub items: Vec<WallpaperItem>,
    pub query: String,
    pub selected_idx: usize,
    pub anim_progress: f32,
    pub anim_direction: f32,
    pub is_animating: bool,
    pub anim_task: Option<Task<()>>,
    pub is_initialized: bool,
}

impl EventEmitter<WallpaperEvent> for WallpaperModule {}

impl Focusable for WallpaperModule {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub struct WallpaperCardProps {
    pub card_w: f32,
    pub card_h: f32,
    pub y_offset: f32,
    pub opacity: f32,
    pub border_w: f32,
    pub border_alpha: f32,
    pub card_bg_alpha: f32,
    pub corner_radius: f32,
}

pub fn get_wallpaper_card_props(abs_pos: f32) -> WallpaperCardProps {
    if abs_pos <= 1.0 {
        let t = abs_pos;
        WallpaperCardProps {
            card_w: lerp(204.0, 152.0, t),
            card_h: lerp(126.0, 94.0, t),
            y_offset: lerp(-10.0, 0.0, t),
            opacity: lerp(1.0, 0.55, t),
            border_w: lerp(2.0, 1.0, t),
            border_alpha: lerp(1.0, 0.25, t),
            card_bg_alpha: lerp(0.6, 0.25, t),
            corner_radius: lerp(16.0, 14.0, t),
        }
    } else if abs_pos <= 2.0 {
        let t = abs_pos - 1.0;
        WallpaperCardProps {
            card_w: lerp(152.0, 110.0, t),
            card_h: lerp(94.0, 68.0, t),
            y_offset: lerp(0.0, 3.0, t),
            opacity: lerp(0.55, 0.25, t),
            border_w: lerp(1.0, 0.5, t),
            border_alpha: lerp(0.25, 0.1, t),
            card_bg_alpha: lerp(0.25, 0.15, t),
            corner_radius: lerp(14.0, 12.0, t),
        }
    } else {
        let t = (abs_pos - 2.0).min(1.0);
        WallpaperCardProps {
            card_w: lerp(110.0, 80.0, t),
            card_h: lerp(68.0, 50.0, t),
            y_offset: lerp(3.0, 5.0, t),
            opacity: lerp(0.25, 0.0, t),
            border_w: lerp(0.5, 0.0, t),
            border_alpha: lerp(0.1, 0.0, t),
            card_bg_alpha: lerp(0.15, 0.0, t),
            corner_radius: lerp(12.0, 10.0, t),
        }
    }
}

pub fn render_wallpaper_card(
    slot_id: ElementId,
    item: &WallpaperItem,
    target_idx: usize,
    offset: i32,
    props: &WallpaperCardProps,
    theme: &Theme,
    cx: &mut Context<WallpaperModule>,
) -> impl IntoElement {
    let is_center = offset == 0;
    let item_path = item.path.clone();
    let card_radius = px(props.corner_radius);

    let border_color = if is_center {
        theme.accent().opacity(props.border_alpha)
    } else {
        theme.surface().opacity(props.border_alpha.max(0.15))
    };

    div()
        .id(slot_id)
        .relative()
        .top(px(props.y_offset))
        .flex_shrink_0()
        .w(px(props.card_w))
        .h(px(props.card_h))
        .rounded(card_radius)
        .overflow_hidden()
        .bg(theme.surface().opacity(props.card_bg_alpha))
        .border(px(props.border_w))
        .border_color(border_color)
        .opacity(props.opacity)
        .when(is_center, |s| s.shadow_lg())
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.45)))
        .on_click(cx.listener(move |this, _, _, cx| {
            if is_center {
                cx.emit(WallpaperEvent::WallpaperSelected(item_path.clone()));
            } else {
                let dir = if offset > 0 { 1.0 } else { -1.0 };
                this.navigate(dir, target_idx, cx);
            }
        }))
        .child(
            img(item.thumb_path.clone())
                .size_full()
                .object_fit(gpui::ObjectFit::Cover)
                .rounded(card_radius),
        )
}

impl WallpaperModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let mut module = Self {
            focus_handle,
            items: Vec::new(),
            query: String::new(),
            selected_idx: 0,
            anim_progress: 1.0,
            anim_direction: 0.0,
            is_animating: false,
            anim_task: None,
            is_initialized: false,
        };
        module.reload_items(cx);
        module
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn filtered_items(&self) -> Vec<WallpaperItem> {
        if self.query.is_empty() {
            self.items.clone()
        } else {
            let q = self.query.to_lowercase();
            self.items
                .iter()
                .filter(|i| i.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        }
    }

    pub fn navigate(&mut self, dir: f32, new_idx: usize, cx: &mut Context<Self>) {
        let filtered = self.filtered_items();
        if filtered.is_empty() {
            return;
        }
        let total = filtered.len();
        self.selected_idx = new_idx % total;
        self.anim_direction = dir;
        self.anim_progress = 0.0;
        self.is_animating = true;

        // Abort previous animation task if still running to prevent task collision
        self.anim_task = None;

        let compositor = if cx.has_global::<AppState>() {
            Some(cx.global::<AppState>().compositor.clone())
        } else {
            None
        };

        let anim_task = cx.spawn(async move |this, cx| {
            let duration_ms = 220.0;
            let start = Instant::now();
            loop {
                let frame_dur = if let Some(ref comp) = compositor {
                    comp.get_frame_duration()
                } else {
                    Duration::from_millis(16)
                };

                cx.background_executor().timer(frame_dur).await;

                let finished = this
                    .update(cx, |module: &mut Self, cx| {
                        let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                        let p = (elapsed / duration_ms).min(1.0);
                        module.anim_progress = p;
                        if p >= 1.0 {
                            module.is_animating = false;
                        }
                        cx.notify();
                        p >= 1.0
                    })
                    .unwrap_or(true);

                if finished {
                    break;
                }
            }
        });

        self.anim_task = Some(anim_task);
    }

    pub fn select_prev(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_items();
        if filtered.len() > 1 {
            let total = filtered.len();
            let next_idx = if self.selected_idx == 0 {
                total - 1
            } else {
                self.selected_idx - 1
            };
            self.navigate(-1.0, next_idx, cx);
        }
    }

    pub fn select_next(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_items();
        if filtered.len() > 1 {
            let total = filtered.len();
            let next_idx = (self.selected_idx + 1) % total;
            self.navigate(1.0, next_idx, cx);
        }
    }

    pub fn apply_selected(&mut self, cx: &mut Context<Self>) {
        let filtered = self.filtered_items();
        if !filtered.is_empty() {
            let idx = self.selected_idx.min(filtered.len() - 1);
            let path = filtered[idx].path.clone();
            cx.emit(WallpaperEvent::WallpaperSelected(path));
        }
    }

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        let mut new_items = Vec::new();

        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/moi".to_string());
        let dir = PathBuf::from(home).join("Wallpapers");

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file()
                    && let Some(ext) = p.extension().and_then(|e| e.to_str())
                {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "webp") {
                        let name = p
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Wallpaper")
                            .to_string();
                        let thumb = get_or_create_thumbnail(&p);
                        new_items.push(WallpaperItem {
                            name,
                            path: p,
                            thumb_path: thumb,
                        });
                    }
                }
            }
        }

        new_items.sort_by_key(|a| a.name.to_lowercase());

        let current_path = if cx.has_global::<AppState>() {
            cx.global::<AppState>().wallpaper.get_current_wallpaper()
        } else {
            None
        };

        if self.items != new_items {
            self.items = new_items;
            if let Some(curr) = current_path {
                if let Some(pos) = self.items.iter().position(|i| i.path == curr) {
                    self.selected_idx = pos;
                } else if self.selected_idx >= self.items.len() {
                    self.selected_idx = 0;
                }
            } else if self.selected_idx >= self.items.len() {
                self.selected_idx = 0;
            }
            self.is_initialized = true;
            cx.notify();
        } else if !self.is_initialized {
            if let Some(curr) = current_path
                && let Some(pos) = self.items.iter().position(|i| i.path == curr)
            {
                self.selected_idx = pos;
            }
            self.is_initialized = true;
            cx.notify();
        }
    }

    pub fn clear_cache(&mut self, cx: &mut Context<Self>) {
        self.items.clear();
        self.query.clear();
        self.anim_task = None;
        self.is_initialized = false;
        self.is_animating = false;
        self.selected_idx = 0;
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;

        if ctrl {
            match key {
                "u" => {
                    self.query.clear();
                    self.selected_idx = 0;
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
                    cx.notify();
                    return;
                }
                "w" => {
                    let trimmed = self.query.trim_end();
                    let new_q = if let Some(idx) = trimmed.rfind(' ') {
                        trimmed[..idx].to_string()
                    } else {
                        String::new()
                    };
                    self.query = new_q;
                    self.selected_idx = 0;
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
                    cx.notify();
                    return;
                }
                "v" => {
                    if let Some(item) = cx.read_from_clipboard()
                        && let Some(text) = item.text()
                    {
                        let clean_text: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !clean_text.is_empty() {
                            self.query.push_str(&clean_text);
                            self.selected_idx = 0;
                            self.anim_progress = 1.0;
                            self.anim_direction = 0.0;
                            self.is_animating = false;
                            self.anim_task = None;
                            cx.notify();
                        }
                    }
                    return;
                }
                _ => {}
            }
        }

        match key {
            "escape" => {
                cx.emit(WallpaperEvent::CloseRequested);
            }
            "left" | "h" => {
                self.select_prev(cx);
            }
            "right" | "l" => {
                self.select_next(cx);
            }
            "enter" | "space" => {
                self.apply_selected(cx);
            }
            "backspace" => {
                if !self.query.is_empty() {
                    self.query.pop();
                    self.selected_idx = 0;
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
                    cx.notify();
                }
            }
            _ => {
                let text = event
                    .keystroke
                    .key_char
                    .as_deref()
                    .unwrap_or(event.keystroke.key.as_str());
                if text.chars().count() == 1 && !ctrl {
                    self.query.push_str(text);
                    self.selected_idx = 0;
                    self.anim_progress = 1.0;
                    self.anim_direction = 0.0;
                    self.is_animating = false;
                    self.anim_task = None;
                    cx.notify();
                }
            }
        }
    }
}

impl Render for WallpaperModule {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let (search_placeholder, no_wallpapers, apply_hint) = if cx.has_global::<services::AppState>() {
            let lang = &cx.global::<services::AppState>().language;
            (
                lang.get("wallpaper.search_placeholder"),
                lang.get("wallpaper.no_wallpapers"),
                lang.get("wallpaper.apply_hint"),
            )
        } else {
            (
                "Buscar fondos...".to_string(),
                "No hay imágenes en ~/Wallpapers".to_string(),
                "↵ Aplicar".to_string(),
            )
        };

        window.focus(&self.focus_handle, cx);

        let filtered = self.filtered_items();
        let total = filtered.len();
        if total > 0 && self.selected_idx >= total {
            self.selected_idx = 0;
        }
        let current_pos = if total == 0 {
            0
        } else {
            (self.selected_idx % total) + 1
        };

        let has_query = !self.query.is_empty();

        let active_name = filtered
            .get(self.selected_idx)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| no_wallpapers.clone());

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2p5()
                    .child(
                        svg()
                            .path("search.svg")
                            .size(px(14.0))
                            .text_color(theme.foreground_muted().opacity(0.8)),
                    )
                    .child(if has_query {
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground())
                            .child(self.query.clone())
                    } else {
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.foreground_muted().opacity(0.5))
                            .child(search_placeholder)
                    }),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground_muted().opacity(0.8))
                    .child(format!("{}/{}", current_pos, total)),
            );

        let mut carousel_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .gap(px(10.0))
            .h(px(146.0));

        if total == 0 {
            carousel_row = carousel_row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(120.0))
                    .text_size(px(13.0))
                    .text_color(theme.foreground_muted())
                    .child(no_wallpapers),
            );
        } else if total == 1 {
            let item = &filtered[0];
            let props = get_wallpaper_card_props(0.0);
            let card = render_wallpaper_card(
                ElementId::from("slot-center"),
                item,
                0,
                0,
                &props,
                &theme,
                cx,
            );
            carousel_row = carousel_row.child(card);
        } else {
            let eased = if self.is_animating {
                ease_out_cubic(self.anim_progress)
            } else {
                1.0
            };

            let shift = (1.0 - eased) * self.anim_direction;

            for offset in -2i32..=2i32 {
                let idx = (self.selected_idx as i32 + offset).rem_euclid(total as i32) as usize;
                let item = &filtered[idx];

                let vis_pos = offset as f32 + shift;
                let abs_pos = vis_pos.abs();
                let props = get_wallpaper_card_props(abs_pos);

                let slot_key = match offset {
                    -2 => "slot-prev2",
                    -1 => "slot-prev1",
                    0 => "slot-center",
                    1 => "slot-next1",
                    2 => "slot-next2",
                    _ => "slot-other",
                };

                let card = render_wallpaper_card(
                    ElementId::from(slot_key),
                    item,
                    idx,
                    offset,
                    &props,
                    &theme,
                    cx,
                );
                carousel_row = carousel_row.child(card);
            }
        }

        let footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .size(px(6.0))
                            .rounded_full()
                            .bg(if total > 0 {
                                theme.accent()
                            } else {
                                theme.foreground_muted().opacity(0.4)
                            }),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground())
                            .max_w(px(380.0))
                            .truncate()
                            .child(active_name),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme.foreground_muted().opacity(0.6))
                            .child(apply_hint),
                    ),
            );

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .justify_between()
            .w(px(700.0))
            .h(px(240.0))
            .px(px(24.0))
            .py(px(16.0))
            .overflow_hidden()
            .child(header)
            .child(carousel_row)
            .child(footer)
    }
}
