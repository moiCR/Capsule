use gpui::{Context, FontWeight, IntoElement, ParentElement, Styled, div, prelude::*, px};
use services::AppState;
use ui::theme::Theme;

use crate::lockscreen::LockScreen;

pub fn render_lyrics(
    theme: &Theme,
    active_idx: usize,
    anim_progress: f32,
    cx: &mut Context<LockScreen>,
) -> impl IntoElement {
    let (active_track, lyrics) = if cx.has_global::<AppState>() {
        let app_state = cx.global::<AppState>();
        let track = app_state.mpris.get_current_track();
        if track.has_media {
            let lyrics_opt = app_state
                .lyrics
                .get_cached_lyrics(&track.title, &track.artist)
                .flatten();
            (track, lyrics_opt)
        } else {
            (track, None)
        }
    } else {
        (services::MediaTrack::default(), None)
    };

    let lyrics = match lyrics {
        Some(l) if !l.synced_lines.is_empty() => l,
        _ => return div().into_any_element(),
    };

    if !active_track.has_media {
        return div().into_any_element();
    }

    let p = 1.0 - anim_progress;
    let eased_t = 1.0 - p * p * p;

    let start_idx = active_idx.saturating_sub(1);
    let end_idx = (start_idx + 3).min(lyrics.synced_lines.len());

    let mut lines_col = div().flex().flex_col().items_center().gap(px(4.0)).w_full();

    for (i, line) in lyrics.synced_lines[start_idx..end_idx].iter().enumerate() {
        let actual_idx = start_idx + i;
        let is_active = actual_idx == active_idx;
        let is_past = actual_idx < active_idx;

        if line.text.is_empty() {
            continue;
        }

        let opacity = if is_active {
            0.45 + 0.55 * eased_t
        } else if is_past {
            (0.55 - 0.25 * eased_t).max(0.3)
        } else {
            0.4
        };

        let font_size = if is_active {
            12.0 + 1.5 * eased_t
        } else {
            11.5
        };

        let y_offset = if is_active {
            3.0 * (1.0 - eased_t)
        } else {
            0.0
        };

        lines_col = lines_col.child(
            div()
                .w_full()
                .text_center()
                .font_family(theme.font_family())
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::MEDIUM
                })
                .text_size(px(font_size))
                .text_color(if is_active {
                    theme.accent()
                } else if is_past {
                    theme.foreground_muted()
                } else {
                    theme.foreground()
                })
                .opacity(opacity)
                .relative()
                .top(px(y_offset))
                .truncate()
                .child(line.text.clone()),
        );
    }

    div()
        .id("lockscreen-lyrics")
        .flex()
        .flex_col()
        .items_center()
        .w_full()
        .px(px(14.0))
        .py(px(8.0))
        .rounded(px(20.0))
        .bg(theme.surface().opacity(0.3))
        .border_1()
        .border_color(theme.surface().opacity(0.2))
        .overflow_hidden()
        .child(lines_col)
        .into_any_element()
}
