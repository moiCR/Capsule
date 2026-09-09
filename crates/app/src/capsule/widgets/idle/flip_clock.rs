use gpui::{FontWeight, IntoElement, ParentElement, Styled, div, px};
use std::time::Instant;
use ui::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlipDigitState {
    pub current: char,
    pub target: char,
    pub prev: char,
    pub progress: f32,
    pub is_flipping: bool,
    pub start_time: Option<Instant>,
}

impl FlipDigitState {
    pub fn new(c: char) -> Self {
        Self {
            current: c,
            target: c,
            prev: c,
            progress: 1.0,
            is_flipping: false,
            start_time: None,
        }
    }

    pub fn set_target(&mut self, c: char) -> bool {
        if self.current == c && self.target == c {
            return false;
        }
        if self.target != c {
            self.prev = self.current;
            self.target = c;
            self.progress = 0.0;
            self.is_flipping = true;
            self.start_time = Some(Instant::now());
            return true;
        }
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlipClock {
    pub digits: [FlipDigitState; 4],
}

fn ease_out_cubic(t: f32) -> f32 {
    let p = 1.0 - t.clamp(0.0, 1.0);
    1.0 - p * p * p
}

impl FlipClock {
    pub const DIGIT_WIDTH: f32 = 7.8;
    pub const COLON_WIDTH: f32 = 4.5;
    pub const HEIGHT: f32 = 18.0;

    pub fn new(time_str: &str) -> Self {
        let chars: Vec<char> = time_str.chars().collect();
        let d0 = chars.first().copied().unwrap_or('0');
        let d1 = chars.get(1).copied().unwrap_or('0');
        let d2 = chars.get(3).copied().unwrap_or('0');
        let d3 = chars.get(4).copied().unwrap_or('0');
        Self {
            digits: [
                FlipDigitState::new(d0),
                FlipDigitState::new(d1),
                FlipDigitState::new(d2),
                FlipDigitState::new(d3),
            ],
        }
    }

    pub fn update_time(&mut self, time_str: &str) -> bool {
        let chars: Vec<char> = time_str.chars().collect();
        let targets = [
            chars.first().copied().unwrap_or('0'),
            chars.get(1).copied().unwrap_or('0'),
            chars.get(3).copied().unwrap_or('0'),
            chars.get(4).copied().unwrap_or('0'),
        ];
        let mut any_started = false;
        for (i, &t) in targets.iter().enumerate() {
            if self.digits[i].set_target(t) {
                any_started = true;
            }
        }
        any_started
    }

    #[allow(dead_code)]
    pub fn is_any_flipping(&self) -> bool {
        self.digits.iter().any(|d| d.is_flipping)
    }

    pub fn tick(&mut self, duration_ms: f32) -> bool {
        let mut all_done = true;
        for digit in &mut self.digits {
            if digit.is_flipping {
                if let Some(start) = digit.start_time {
                    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
                    let p = (elapsed / duration_ms).min(1.0);
                    digit.progress = p;
                    if p >= 1.0 {
                        digit.current = digit.target;
                        digit.prev = digit.target;
                        digit.is_flipping = false;
                        digit.start_time = None;
                    } else {
                        all_done = false;
                    }
                } else {
                    digit.is_flipping = false;
                }
            }
        }
        all_done
    }

    pub fn render(&self, theme: &Theme) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(Self::HEIGHT))
            .child(render_digit(&self.digits[0], theme))
            .child(render_digit(&self.digits[1], theme))
            .child(
                div()
                    .w(px(Self::COLON_WIDTH))
                    .h(px(Self::HEIGHT))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(theme.font_family())
                    .font_weight(FontWeight::BOLD)
                    .text_size(px(13.0))
                    .text_color(theme.foreground())
                    .child(":"),
            )
            .child(render_digit(&self.digits[2], theme))
            .child(render_digit(&self.digits[3], theme))
    }
}

fn render_digit(digit: &FlipDigitState, theme: &Theme) -> impl IntoElement {
    let font_family = theme.font_family();
    let foreground = theme.foreground();

    let render_text = move |c: char| {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .font_family(font_family.clone())
            .font_weight(FontWeight::BOLD)
            .text_size(px(13.0))
            .text_color(foreground)
            .child(c.to_string())
    };

    if !digit.is_flipping || digit.progress >= 1.0 {
        div()
            .w(px(FlipClock::DIGIT_WIDTH))
            .h(px(FlipClock::HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .child(render_text(digit.current))
    } else {
        let t = ease_out_cubic(digit.progress);
        div()
            .relative()
            .w(px(FlipClock::DIGIT_WIDTH))
            .h(px(FlipClock::HEIGHT))
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .top(px(t * FlipClock::HEIGHT))
                    .left(px(0.0))
                    .w_full()
                    .h(px(FlipClock::HEIGHT))
                    .opacity(1.0 - t)
                    .child(render_text(digit.prev)),
            )
            .child(
                div()
                    .absolute()
                    .top(px((1.0 - t) * -FlipClock::HEIGHT))
                    .left(px(0.0))
                    .w_full()
                    .h(px(FlipClock::HEIGHT))
                    .opacity(t)
                    .child(render_text(digit.target)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let clock = FlipClock::new("18:00");
        assert_eq!(clock.digits[0].current, '1');
        assert_eq!(clock.digits[1].current, '8');
        assert_eq!(clock.digits[2].current, '0');
        assert_eq!(clock.digits[3].current, '0');
        assert!(!clock.is_any_flipping());
    }

    #[test]
    fn test_only_changing_digit_flips() {
        let mut clock = FlipClock::new("18:00");
        let changed = clock.update_time("18:01");
        assert!(changed);
        assert!(!clock.digits[0].is_flipping);
        assert!(!clock.digits[1].is_flipping);
        assert!(!clock.digits[2].is_flipping);
        assert!(clock.digits[3].is_flipping);
        assert_eq!(clock.digits[3].prev, '0');
        assert_eq!(clock.digits[3].target, '1');
    }

    #[test]
    fn test_multiple_digits_flip() {
        let mut clock = FlipClock::new("18:09");
        let changed = clock.update_time("18:10");
        assert!(changed);
        assert!(!clock.digits[0].is_flipping);
        assert!(!clock.digits[1].is_flipping);
        assert!(clock.digits[2].is_flipping);
        assert!(clock.digits[3].is_flipping);
        assert_eq!(clock.digits[2].prev, '0');
        assert_eq!(clock.digits[2].target, '1');
        assert_eq!(clock.digits[3].prev, '9');
        assert_eq!(clock.digits[3].target, '0');
    }

    #[test]
    fn test_same_time_no_flip() {
        let mut clock = FlipClock::new("18:00");
        let changed = clock.update_time("18:00");
        assert!(!changed);
        assert!(!clock.is_any_flipping());
    }
}
