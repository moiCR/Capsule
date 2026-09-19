#![allow(dead_code)]

use gpui::{
    AnyElement, Context, IntoElement, ScrollWheelEvent, Transformation, div, img, prelude::*, px,
    radians, svg,
};
use services::{MediaTrack, MprisService};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use ui::theme::Theme;

use crate::capsule::modules::default::DefaultModule;
use crate::capsule::widgets::dashboard::media_player::resolve_art_path;

fn get_or_create_vinyl_frame(art_path: &str, disc_rotation: f32) -> Option<String> {
    let mut hasher = DefaultHasher::new();
    art_path.hash(&mut hasher);
    let hash = hasher.finish();

    let frame_idx = ((disc_rotation / (2.0 * std::f32::consts::PI) * 36.0) as usize) % 36;
    let dir = PathBuf::from(format!("/tmp/capsule_vinyl/{:x}", hash));
    let frame_path = dir.join(format!("{}.png", frame_idx));

    if frame_path.exists() {
        return Some(frame_path.to_string_lossy().to_string());
    }

    generate_vinyl_frames(art_path, &dir);

    if frame_path.exists() {
        Some(frame_path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn generate_vinyl_frames(src_path: &str, out_dir: &Path) {
    let Ok(img) = image::open(src_path) else {
        return;
    };
    let _ = std::fs::create_dir_all(out_dir);

    let size = 48u32;
    let resized = img
        .resize_exact(size, size, image::imageops::FilterType::Triangle)
        .to_rgba8();

    let cx = 23.5f32;
    let cy = 23.5f32;
    let r_out = 23.0f32;
    let r_hole = 2.5f32;

    for k in 0..36 {
        let theta = (k as f32) * (2.0 * std::f32::consts::PI / 36.0);
        let cos = (-theta).cos();
        let sin = (-theta).sin();

        let mut out = image::RgbaImage::new(size, size);

        for y in 0..size {
            for x in 0..size {
                let dx = (x as f32) - cx;
                let dy = (y as f32) - cy;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist > r_out {
                    continue;
                } else if dist <= r_hole {
                    out.put_pixel(x, y, image::Rgba([18, 18, 24, 255]));
                } else if dist >= r_out - 1.2 {
                    out.put_pixel(x, y, image::Rgba([22, 22, 28, 255]));
                } else {
                    let sx = (cx + dx * cos - dy * sin).round() as i32;
                    let sy = (cy + dx * sin + dy * cos).round() as i32;
                    let sx = sx.clamp(0, (size - 1) as i32) as u32;
                    let sy = sy.clamp(0, (size - 1) as i32) as u32;

                    let mut p = *resized.get_pixel(sx, sy);
                    let groove = 0.95 + 0.05 * (dist * 3.5).cos();
                    p[0] = ((p[0] as f32) * groove).min(255.0) as u8;
                    p[1] = ((p[1] as f32) * groove).min(255.0) as u8;
                    p[2] = ((p[2] as f32) * groove).min(255.0) as u8;
                    out.put_pixel(x, y, p);
                }
            }
        }

        let frame_file = out_dir.join(format!("{}.png", k));
        let _ = out.save(frame_file);
    }
}

pub fn render_media_dock(
    track: &MediaTrack,
    disc_rotation: f32,
    theme: &Theme,
    cx: &mut Context<DefaultModule>,
) -> Option<AnyElement> {
    if !track.has_media || (track.title.is_empty() && !track.is_playing) {
        return None;
    }

    let bus_disc = track.bus_name.clone();
    let bus_prev = track.bus_name.clone();
    let bus_play = track.bus_name.clone();
    let bus_next = track.bus_name.clone();
    let bus_scroll = track.bus_name.clone();

    let art_path = resolve_art_path(track);

    let disc_visual = if let Some(frame_path) = art_path
        .as_deref()
        .and_then(|p| get_or_create_vinyl_frame(p, disc_rotation))
    {
        div().size(px(18.0)).rounded_full().overflow_hidden().child(
            img(frame_path)
                .size(px(18.0))
                .rounded_full()
                .aspect_ratio(1.0)
                .object_fit(gpui::ObjectFit::Cover),
        )
    } else {
        div().size(px(18.0)).rounded_full().overflow_hidden().child(
            svg()
                .path("vinyl.svg")
                .size(px(18.0))
                .text_color(theme.accent())
                .with_transformation(Transformation::rotate(radians(disc_rotation))),
        )
    };

    let disc_btn = div()
        .id("media-disc-btn")
        .flex()
        .items_center()
        .justify_center()
        .size(px(18.0))
        .cursor_pointer()
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_disc.clone();
            tokio::spawn(async move {
                let _ = MprisService::play_pause_bus(&bus).await;
            });
        }))
        .child(disc_visual);

    let prev_btn = div()
        .id("media-btn-prev")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_prev.clone();
            tokio::spawn(async move {
                let _ = MprisService::previous_bus(&bus).await;
            });
        }))
        .child(
            svg()
                .path("skip-back.svg")
                .size(px(9.5))
                .text_color(theme.foreground_muted()),
        );

    let (play_icon, play_color) = if track.is_playing {
        ("pause.svg", theme.accent())
    } else {
        ("play.svg", theme.foreground())
    };

    let play_btn = div()
        .id("media-btn-play")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_play.clone();
            tokio::spawn(async move {
                let _ = MprisService::play_pause_bus(&bus).await;
            });
        }))
        .child(svg().path(play_icon).size(px(10.0)).text_color(play_color));

    let next_btn = div()
        .id("media-btn-next")
        .flex()
        .items_center()
        .justify_center()
        .size(px(16.0))
        .rounded(px(4.0))
        .cursor_pointer()
        .hover(|s| s.bg(theme.surface().opacity(0.5)))
        .on_click(cx.listener(move |_this, _, _, _| {
            let bus = bus_next.clone();
            tokio::spawn(async move {
                let _ = MprisService::next_bus(&bus).await;
            });
        }))
        .child(
            svg()
                .path("skip-forward.svg")
                .size(px(9.5))
                .text_color(theme.foreground_muted()),
        );

    Some(
        div()
            .id("default-media-dock")
            .flex()
            .flex_shrink_0()
            .flex_row()
            .items_center()
            .h(px(22.0))
            .px(px(4.0))
            .gap(px(2.0))
            .rounded(px(7.0))
            .bg(theme.surface().opacity(0.35))
            .border_1()
            .border_color(theme.surface().opacity(0.2))
            .on_scroll_wheel(cx.listener(move |_this, event: &ScrollWheelEvent, _, _| {
                let bus = bus_scroll.clone();
                let delta_y = match event.delta {
                    gpui::ScrollDelta::Lines(lines) => lines.y,
                    gpui::ScrollDelta::Pixels(pixels) => pixels.y.into(),
                };
                tokio::spawn(async move {
                    if delta_y > 0.0 {
                        let _ = MprisService::previous_bus(&bus).await;
                    } else if delta_y < 0.0 {
                        let _ = MprisService::next_bus(&bus).await;
                    }
                });
            }))
            .child(disc_btn)
            .child(prev_btn)
            .child(play_btn)
            .child(next_btn)
            .into_any_element(),
    )
}
