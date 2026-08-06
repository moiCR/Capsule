use gpui::{
    Context, ElementId, EventEmitter, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    Task, Window, div, img, prelude::*, px, svg,
};
use services::AppState;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use ui::theme::Theme;

use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WallpaperItem {
    pub name: String,
    pub path: PathBuf,
    pub thumb_path: PathBuf,
}

fn get_or_create_thumbnail(original_path: &PathBuf) -> PathBuf {
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

    if let Ok(st) = res {
        if st.success() && thumb_path.exists() {
            return thumb_path;
        }
    }

    let res_ff = Command::new("ffmpeg")
        .args(["-y", "-i", &orig_str, "-vf", "scale=300:-1", &thumb_str])
        .status();

    if let Ok(st) = res_ff {
        if st.success() && thumb_path.exists() {
            return thumb_path;
        }
    }

    original_path.clone()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperEvent {
    CloseRequested,
    WallpaperSelected(PathBuf),
}

pub struct WallpaperModule {
    pub focus_handle: FocusHandle,
    pub items: Vec<WallpaperItem>,
    pub selected_idx: usize,
    pub anim_progress: f32,
    pub anim_direction: f32,
    pub is_animating: bool,
    pub anim_task: Option<Task<()>>,
    pub is_initialized: bool,
}

impl EventEmitter<WallpaperEvent> for WallpaperModule {}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn get_card_props(abs_pos: f32) -> (f32, f32, f32, f32) {
    if abs_pos <= 1.0 {
        let t = abs_pos;
        (
            lerp(195.0, 120.0, t),
            lerp(120.0, 75.0, t),
            lerp(1.0, 0.5, t),
            lerp(3.0, 0.0, t),
        )
    } else if abs_pos <= 2.0 {
        let t = abs_pos - 1.0;
        (
            lerp(120.0, 90.0, t),
            lerp(75.0, 55.0, t),
            lerp(0.5, 0.25, t),
            0.0,
        )
    } else {
        let t = (abs_pos - 2.0).min(1.0);
        (
            lerp(90.0, 60.0, t),
            lerp(55.0, 35.0, t),
            lerp(0.25, 0.0, t),
            0.0,
        )
    }
}

impl WallpaperModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let mut module = Self {
            focus_handle,
            items: Vec::new(),
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

    fn navigate(&mut self, dir: f32, new_idx: usize, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        let total = self.items.len();
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

    pub fn reload_items(&mut self, cx: &mut Context<Self>) {
        let mut new_items = Vec::new();

        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/moi".to_string());
        let dir = PathBuf::from(home).join("Wallpapers");

        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
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
        }

        new_items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

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
            if let Some(curr) = current_path {
                if let Some(pos) = self.items.iter().position(|i| i.path == curr) {
                    self.selected_idx = pos;
                }
            }
            self.is_initialized = true;
            cx.notify();
        }
    }

    pub fn clear_cache(&mut self, cx: &mut Context<Self>) {
        self.items.clear();
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
        let n = self.items.len();
        if n == 0 {
            if event.keystroke.key == "escape" {
                cx.emit(WallpaperEvent::CloseRequested);
            }
            return;
        }

        match event.keystroke.key.as_str() {
            "left" | "h" => {
                let next_idx = if self.selected_idx == 0 {
                    n - 1
                } else {
                    self.selected_idx - 1
                };
                self.navigate(-1.0, next_idx, cx);
            }
            "right" | "l" => {
                let next_idx = (self.selected_idx + 1) % n;
                self.navigate(1.0, next_idx, cx);
            }
            "enter" | "space" => {
                if let Some(item) = self.items.get(self.selected_idx) {
                    cx.emit(WallpaperEvent::WallpaperSelected(item.path.clone()));
                }
            }
            "escape" => {
                cx.emit(WallpaperEvent::CloseRequested);
            }
            _ => {}
        }
    }
}

impl Render for WallpaperModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let lang = if cx.has_global::<ui::language::Language>() {
            cx.global::<ui::language::Language>().clone()
        } else {
            ui::language::Language::default()
        };
        let total = self.items.len();

        let active_name = self
            .items
            .get(self.selected_idx)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| lang.wallpaper.no_wallpapers.clone());

        let mut carousel_row = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .w_full()
            .gap_3()
            .h(px(145.0));

        if total == 0 {
            carousel_row = carousel_row.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.foreground_muted())
                    .text_size(px(13.0))
                    .child(lang.wallpaper.no_wallpapers),
            );
        } else {
            let eased = if self.is_animating {
                ease_out_cubic(self.anim_progress)
            } else {
                1.0
            };

            let shift = (1.0 - eased) * self.anim_direction;

            for offset in -2i32..=2i32 {
                let idx = (self.selected_idx as i32 + offset).rem_euclid(total as i32) as usize;
                let item = &self.items[idx];
                let item_path = item.path.clone();

                let vis_pos = offset as f32 + shift;
                let abs_pos = vis_pos.abs();

                let (card_w, card_h, opacity, border_w) = get_card_props(abs_pos);
                let is_center = offset == 0;

                let slot_key = match offset {
                    -2 => "slot-prev2",
                    -1 => "slot-prev1",
                    0 => "slot-center",
                    1 => "slot-next1",
                    2 => "slot-next2",
                    _ => "slot-other",
                };

                let card = div()
                    .id(ElementId::from(slot_key))
                    .flex()
                    .flex_col()
                    .items_center()
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if is_center {
                            cx.emit(WallpaperEvent::WallpaperSelected(item_path.clone()));
                        } else {
                            let dir = if offset > 0 { 1.0 } else { -1.0 };
                            this.navigate(dir, idx, cx);
                        }
                    }))
                    .child(
                        div()
                            .w(px(card_w))
                            .h(px(card_h))
                            .rounded_xl()
                            .overflow_hidden()
                            .bg(theme.surface())
                            .border(px(border_w))
                            .border_color(theme.accent().opacity((border_w / 3.0).min(1.0)))
                            .opacity(opacity)
                            .shadow_lg()
                            .child(img(item.thumb_path.clone()).w_full().h_full()),
                    );

                carousel_row = carousel_row.child(card);
            }
        }

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .w(px(700.0))
            .h(px(240.0))
            .p_4()
            .gap_2()
            .rounded(px(32.0))
            .bg(theme.background())
            .border_1()
            .border_color(theme.surface().opacity(0.6))
            .shadow_lg()
            .child(carousel_row)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(13.0))
                            .text_color(theme.foreground())
                            .child(active_name),
                    )
                    .child(
                        svg()
                            .path("chevron-right.svg")
                            .size(px(12.0))
                            .text_color(theme.foreground_muted()),
                    ),
            )
    }
}
