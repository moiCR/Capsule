use crate::theme::Theme;
use crate::tracker::dimension_tracker::DimensionTracker;
use gpui::{
    AnyElement, App, ClickEvent, ColorExt, DefiniteLength, DurationWithEasing, ElementId, Font,
    Hsla, InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, TextRun, Window, canvas, div, ease_in_out, millis, point,
    prelude::*, px, svg,
};
use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

pub type InputClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type InputMouseDownListener = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type InputMouseMoveListener = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type InputMouseUpListener = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug)]
pub struct InputState {
    pub chars: Vec<char>,
    pub cursor: usize,
    pub selection: Option<Range<usize>>,
    pub selection_anchor: Option<usize>,
    pub placeholder: String,
    pub masked: bool,
    pub focused: bool,
    pub max_len: usize,
    pub is_dragging: bool,
    pub centered: bool,
    pub tracker: DimensionTracker,
    pub scroll_offset: Rc<Cell<f32>>,
    pub font_size: Pixels,
    pub padding_left: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            chars: Vec::new(),
            cursor: 0,
            selection: None,
            selection_anchor: None,
            placeholder: String::new(),
            masked: false,
            focused: false,
            max_len: 120,
            is_dragging: false,
            centered: false,
            tracker: DimensionTracker::new(),
            scroll_offset: Rc::new(Cell::new(0.0)),
            font_size: px(14.0),
            padding_left: 14.0,
        }
    }
}

impl InputState {
    pub fn new(text: impl Into<String>) -> Self {
        let text_str = text.into();
        let chars: Vec<char> = text_str.chars().collect();
        let cursor = chars.len();
        Self {
            chars,
            cursor,
            selection: None,
            selection_anchor: None,
            placeholder: String::new(),
            masked: false,
            focused: false,
            max_len: 120,
            is_dragging: false,
            centered: false,
            tracker: DimensionTracker::new(),
            scroll_offset: Rc::new(Cell::new(0.0)),
            font_size: px(14.0),
            padding_left: 14.0,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn with_masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_max_len(mut self, max_len: usize) -> Self {
        self.max_len = max_len;
        self
    }

    pub fn with_font_size(mut self, font_size: Pixels) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_padding(mut self, padding_left: f32) -> Self {
        self.padding_left = padding_left;
        self
    }

    pub fn with_centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.chars = text.into().chars().collect();
        self.cursor = self.chars.len();
        self.selection = None;
        self.selection_anchor = None;
        self.scroll_offset.set(0.0);
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
        self.selection = None;
        self.selection_anchor = None;
        self.scroll_offset.set(0.0);
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.chars.len());
        self.selection = None;
        self.selection_anchor = None;
    }

    pub fn select_all(&mut self) {
        if !self.chars.is_empty() {
            self.selection = Some(0..self.chars.len());
            self.selection_anchor = Some(0);
            self.cursor = self.chars.len();
        }
    }

    pub fn selected_text(&self) -> Option<String> {
        let range = self.selection.as_ref()?;
        let start = range.start.min(self.chars.len());
        let end = range.end.min(self.chars.len());
        if start < end {
            Some(self.chars[start..end].iter().collect())
        } else {
            None
        }
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some(range) = self.selection.take() {
            let start = range.start.min(self.chars.len());
            let end = range.end.min(self.chars.len());
            if start < end {
                self.chars.drain(start..end);
                self.cursor = start;
                self.selection_anchor = None;
                return true;
            }
        }
        self.selection_anchor = None;
        false
    }

    pub fn insert(&mut self, text: &str) {
        self.delete_selection();
        let clean_chars: Vec<char> = text.chars().filter(|c| !c.is_control()).collect();
        let available = self.max_len.saturating_sub(self.chars.len());
        let to_insert = clean_chars.into_iter().take(available);

        for c in to_insert {
            if self.cursor <= self.chars.len() {
                self.chars.insert(self.cursor, c);
                self.cursor += 1;
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        if self.chars.len() < self.max_len && self.cursor <= self.chars.len() {
            self.chars.insert(self.cursor, c);
            self.cursor += 1;
        }
    }

    pub fn backspace(&mut self) {
        if !self.delete_selection() && self.cursor > 0 && self.cursor <= self.chars.len() {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if !self.delete_selection() && self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            if let Some(range) = self.selection.take() {
                self.selection_anchor = None;
                self.cursor = range.start;
                return;
            }
        }
        let new_cursor = self.cursor.saturating_sub(1);
        self.update_cursor(new_cursor, select);
    }

    pub fn move_right(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            if let Some(range) = self.selection.take() {
                self.selection_anchor = None;
                self.cursor = range.end.min(self.chars.len());
                return;
            }
        }
        let new_cursor = (self.cursor + 1).min(self.chars.len());
        self.update_cursor(new_cursor, select);
    }

    pub fn move_to_start(&mut self, select: bool) {
        self.update_cursor(0, select);
    }

    pub fn move_to_end(&mut self, select: bool) {
        let len = self.chars.len();
        self.update_cursor(len, select);
    }

    pub fn update_cursor(&mut self, new_cursor: usize, select: bool) {
        let new_cursor = new_cursor.min(self.chars.len());
        if select {
            let anchor = self.selection_anchor.get_or_insert(self.cursor);
            self.cursor = new_cursor;
            let start = (*anchor).min(self.cursor);
            let end = (*anchor).max(self.cursor);
            if start < end {
                self.selection = Some(start..end);
            } else {
                self.selection = None;
            }
        } else {
            self.selection = None;
            self.selection_anchor = None;
            self.cursor = new_cursor;
        }
    }

    pub fn prev_word_boundary(&self) -> usize {
        self.prev_word_boundary_from(self.cursor)
    }

    pub fn prev_word_boundary_from(&self, from: usize) -> usize {
        let mut idx = from.min(self.chars.len());
        if idx == 0 {
            return 0;
        }
        while idx > 0 && self.chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !self.chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        idx
    }

    pub fn next_word_boundary(&self) -> usize {
        self.next_word_boundary_from(self.cursor)
    }

    pub fn next_word_boundary_from(&self, from: usize) -> usize {
        let len = self.chars.len();
        let mut idx = from.min(len);
        if idx >= len {
            return len;
        }
        while idx < len && !self.chars[idx].is_whitespace() {
            idx += 1;
        }
        while idx < len && self.chars[idx].is_whitespace() {
            idx += 1;
        }
        idx
    }

    pub fn move_word_backward(&mut self, select: bool) {
        let prev = self.prev_word_boundary();
        self.update_cursor(prev, select);
    }

    pub fn move_word_forward(&mut self, select: bool) {
        let next = self.next_word_boundary();
        self.update_cursor(next, select);
    }

    pub fn delete_word_backward(&mut self) {
        if !self.delete_selection() {
            let prev = self.prev_word_boundary();
            if prev < self.cursor {
                self.chars.drain(prev..self.cursor);
                self.cursor = prev;
            }
        }
    }

    pub fn delete_word_forward(&mut self) {
        if !self.delete_selection() {
            let next = self.next_word_boundary();
            if self.cursor < next {
                self.chars.drain(self.cursor..next);
            }
        }
    }

    pub fn select_word_at(&mut self, index: usize) {
        let idx = index.min(self.chars.len());
        if self.chars.is_empty() {
            return;
        }
        let mut start = idx;
        while start > 0
            && !self.chars[start - 1].is_whitespace()
            && !self.chars[start - 1].is_ascii_punctuation()
        {
            start -= 1;
        }
        let mut end = idx;
        while end < self.chars.len()
            && !self.chars[end].is_whitespace()
            && !self.chars[end].is_ascii_punctuation()
        {
            end += 1;
        }
        if start == end && end < self.chars.len() {
            end += 1;
        }
        self.selection = Some(start..end);
        self.selection_anchor = Some(start);
        self.cursor = end;
    }

    pub fn char_index_from_local_point(
        &self,
        local_x: Pixels,
        window: &Window,
        theme: &Theme,
    ) -> usize {
        let display_str: String = if self.masked {
            "•".repeat(self.chars.len())
        } else {
            self.chars.iter().collect()
        };

        if display_str.is_empty() {
            return 0;
        }

        let run = TextRun {
            len: display_str.len(),
            font: Font {
                family: theme.font_family().into(),
                ..Default::default()
            },
            color: theme.foreground(),
            background_color: None,
            underline: None,
            strikethrough: None,
            letter_spacing: None,
        };

        let shaped_res = window.text_system().shape_text(
            display_str.clone(),
            self.font_size,
            &[run],
            None,
            None,
        );

        let Ok(shaped) = shaped_res else {
            return 0;
        };

        let mut hit_x = local_x;
        if self.centered {
            let available_width = px(self.tracker.right() - self.tracker.left());
            if available_width > shaped.width {
                let offset_x = (available_width - shaped.width) / 2.0;
                if hit_x < offset_x {
                    return 0;
                }
                if hit_x > offset_x + shaped.width {
                    return self.chars.len();
                }
                hit_x -= offset_x;
            }
        } else {
            hit_x += px(self.scroll_offset.get());
        }

        let byte_idx = match shaped.closest_byte_index_for_pixel_point(
            point(hit_x.max(px(0.0)), px(0.0)),
            self.font_size * 1.5,
        ) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };

        let safe_byte_idx = if display_str.is_char_boundary(byte_idx.min(display_str.len())) {
            byte_idx.min(display_str.len())
        } else {
            let mut b = byte_idx.min(display_str.len());
            while b > 0 && !display_str.is_char_boundary(b) {
                b -= 1;
            }
            b
        };

        let char_col = display_str[..safe_byte_idx].chars().count();
        char_col.min(self.chars.len())
    }

    pub fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &Window,
        theme: &Theme,
    ) {
        let left = px(self.tracker.left());
        let local_x = (event.position.x - left).max(px(0.0));

        let char_idx = self.char_index_from_local_point(local_x, window, theme);

        if event.click_count >= 3 {
            self.select_all();
            self.is_dragging = false;
        } else if event.click_count == 2 {
            self.select_word_at(char_idx);
            self.is_dragging = false;
        } else if event.modifiers.shift {
            let anchor = self.selection_anchor.unwrap_or(self.cursor);
            self.cursor = char_idx;
            self.selection_anchor = Some(anchor);
            if anchor != char_idx {
                self.selection = Some(anchor.min(char_idx)..anchor.max(char_idx));
            } else {
                self.selection = None;
            }
            self.is_dragging = true;
        } else {
            self.cursor = char_idx;
            self.selection_anchor = Some(char_idx);
            self.selection = None;
            self.is_dragging = true;
        }
        self.focused = true;
    }

    pub fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &Window,
        theme: &Theme,
    ) {
        if !self.is_dragging {
            return;
        }
        let left = px(self.tracker.left());
        let local_x = (event.position.x - left).max(px(0.0));

        let char_idx = self.char_index_from_local_point(local_x, window, theme);
        self.cursor = char_idx;
        let anchor = self.selection_anchor.unwrap_or(char_idx);
        if anchor != char_idx {
            self.selection = Some(anchor.min(char_idx)..anchor.max(char_idx));
        } else {
            self.selection = None;
        }
    }

    pub fn handle_mouse_up(&mut self, _event: &MouseUpEvent) {
        self.is_dragging = false;
        if let Some(ref range) = self.selection {
            if range.start == range.end {
                self.selection = None;
                self.selection_anchor = None;
            }
        }
    }

    pub fn copy(&self, cx: &mut App) {
        if self.masked {
            return;
        }
        let text = self.selected_text().unwrap_or_else(|| self.text());
        if !text.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    pub fn cut(&mut self, cx: &mut App) {
        if self.masked {
            return;
        }
        let text = self.selected_text().unwrap_or_else(|| self.text());
        if !text.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            if !self.delete_selection() {
                self.clear();
            }
        }
    }

    pub fn paste(&mut self, cx: &mut App) {
        if let Some(item) = cx.read_from_clipboard() {
            if let Some(text) = item.text() {
                self.insert(&text);
            }
        }
    }

    pub fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut App) -> bool {
        let key_lower = event.keystroke.key.to_lowercase();
        let key = key_lower.as_str();
        let ctrl = event.keystroke.modifiers.control || event.keystroke.modifiers.platform;
        let alt = event.keystroke.modifiers.alt;
        let shift = event.keystroke.modifiers.shift;

        if alt {
            return false;
        }

        if ctrl {
            match key {
                "a" => {
                    self.select_all();
                    return true;
                }
                "c" => {
                    self.copy(cx);
                    return true;
                }
                "x" => {
                    self.cut(cx);
                    return true;
                }
                "v" => {
                    self.paste(cx);
                    return true;
                }
                "u" => {
                    self.move_to_start(true);
                    self.delete_selection();
                    return true;
                }
                "w" | "backspace" => {
                    self.delete_word_backward();
                    return true;
                }
                "delete" => {
                    self.delete_word_forward();
                    return true;
                }
                "left" => {
                    self.move_word_backward(shift);
                    return true;
                }
                "right" => {
                    self.move_word_forward(shift);
                    return true;
                }
                "home" => {
                    self.move_to_start(shift);
                    return true;
                }
                "end" => {
                    self.move_to_end(shift);
                    return true;
                }
                _ => return false,
            }
        }

        match key {
            "backspace" => {
                self.backspace();
                true
            }
            "delete" => {
                self.delete();
                true
            }
            "left" => {
                self.move_left(shift);
                true
            }
            "right" => {
                self.move_right(shift);
                true
            }
            "home" => {
                self.move_to_start(shift);
                true
            }
            "end" => {
                self.move_to_end(shift);
                true
            }
            "space" => {
                self.insert(" ");
                true
            }
            "escape" => {
                if self.selection.is_some() {
                    self.selection = None;
                    self.selection_anchor = None;
                    true
                } else if !self.chars.is_empty() {
                    self.clear();
                    true
                } else {
                    false
                }
            }
            "enter" | "return" | "tab" => false,
            _ => {
                if let Some(ref ch) = event.keystroke.key_char {
                    let clean: String = ch.chars().filter(|c| !c.is_control()).collect();
                    if !clean.is_empty() {
                        self.insert(&clean);
                        return true;
                    }
                } else if event.keystroke.key.chars().count() == 1 {
                    let c = event.keystroke.key.chars().next().unwrap();
                    if !c.is_control() {
                        self.insert_char(c);
                        return true;
                    }
                }
                false
            }
        }
    }
}

#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    value: SharedString,
    placeholder: SharedString,
    focused: bool,
    masked: bool,
    error: bool,
    disabled: bool,
    cleanable: bool,
    centered: bool,
    cursor: Option<usize>,
    selection: Option<Range<usize>>,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    height: Option<Pixels>,
    width: Option<DefiniteLength>,
    max_width: Option<DefiniteLength>,
    rounded: Option<Pixels>,
    bg: Option<Hsla>,
    border_color: Option<Hsla>,
    focused_border_color: Option<Hsla>,
    text_size: Option<Pixels>,
    max_visible_chars: usize,
    cursor_visible: Option<bool>,
    flex_1: bool,
    padding_x: Option<Pixels>,
    gap: Option<Pixels>,
    tracker: Option<DimensionTracker>,
    scroll_offset: Option<Rc<Cell<f32>>>,
    on_click: Option<InputClickListener>,
    on_mouse_down: Option<InputMouseDownListener>,
    on_mouse_move: Option<InputMouseMoveListener>,
    on_mouse_up: Option<InputMouseUpListener>,
    on_clear: Option<InputClickListener>,
}

impl Input {
    pub fn new(id: impl Into<ElementId>, value: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            placeholder: SharedString::default(),
            focused: false,
            masked: false,
            error: false,
            disabled: false,
            cleanable: false,
            centered: false,
            cursor: None,
            selection: None,
            prefix: None,
            suffix: None,
            height: None,
            width: None,
            max_width: None,
            rounded: None,
            bg: None,
            border_color: None,
            focused_border_color: None,
            text_size: None,
            max_visible_chars: 120,
            cursor_visible: None,
            flex_1: false,
            padding_x: None,
            gap: None,
            tracker: None,
            scroll_offset: None,
            on_click: None,
            on_mouse_down: None,
            on_mouse_move: None,
            on_mouse_up: None,
            on_clear: None,
        }
    }

    pub fn from_state(id: impl Into<ElementId>, state: &InputState) -> Self {
        Self::new(id, state.text())
            .placeholder(state.placeholder.clone())
            .focused(state.focused)
            .masked(state.masked)
            .centered(state.centered)
            .cursor(state.cursor)
            .selection(state.selection.clone())
            .tracker(state.tracker.clone())
            .scroll_offset(state.scroll_offset.clone())
            .text_size(state.font_size)
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn centered(mut self, centered: bool) -> Self {
        self.centered = centered;
        self
    }

    pub fn cursor(mut self, cursor: usize) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn selection(mut self, selection: Option<Range<usize>>) -> Self {
        self.selection = selection;
        self
    }

    pub fn prefix(mut self, prefix: impl IntoElement) -> Self {
        self.prefix = Some(prefix.into_any_element());
        self
    }

    pub fn suffix(mut self, suffix: impl IntoElement) -> Self {
        self.suffix = Some(suffix.into_any_element());
        self
    }

    pub fn tracker(mut self, tracker: DimensionTracker) -> Self {
        self.tracker = Some(tracker);
        self
    }

    pub fn scroll_offset(mut self, scroll_offset: Rc<Cell<f32>>) -> Self {
        self.scroll_offset = Some(scroll_offset);
        self
    }

    pub fn h(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn h_px(mut self, px_val: f32) -> Self {
        self.height = Some(px(px_val));
        self
    }

    pub fn w(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn w_full(mut self) -> Self {
        self.width = Some(gpui::relative(1.0).into());
        self
    }

    pub fn w_px(mut self, px_val: f32) -> Self {
        self.width = Some(px(px_val).into());
        self
    }

    pub fn max_w(mut self, max_width: impl Into<DefiniteLength>) -> Self {
        self.max_width = Some(max_width.into());
        self
    }

    pub fn max_w_px(mut self, px_val: f32) -> Self {
        self.max_width = Some(px(px_val).into());
        self
    }

    pub fn rounded(mut self, radius: Pixels) -> Self {
        self.rounded = Some(radius);
        self
    }

    pub fn rounded_full(mut self) -> Self {
        self.rounded = Some(px(9999.0));
        self
    }

    pub fn bg(mut self, bg: Hsla) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn border_color(mut self, color: Hsla) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn focused_border_color(mut self, color: Hsla) -> Self {
        self.focused_border_color = Some(color);
        self
    }

    pub fn text_size(mut self, size: Pixels) -> Self {
        self.text_size = Some(size);
        self
    }

    pub fn max_visible_chars(mut self, max: usize) -> Self {
        self.max_visible_chars = max;
        self
    }

    pub fn cursor_visible(mut self, visible: bool) -> Self {
        self.cursor_visible = Some(visible);
        self
    }

    pub fn flex_1(mut self, flex_1: bool) -> Self {
        self.flex_1 = flex_1;
        self
    }

    pub fn padding_x(mut self, px_val: impl Into<Pixels>) -> Self {
        self.padding_x = Some(px_val.into());
        self
    }

    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Box::new(listener));
        self
    }

    pub fn on_mouse_down<F>(mut self, listener: F) -> Self
    where
        F: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        self.on_mouse_down = Some(Box::new(listener));
        self
    }

    pub fn on_mouse_move<F>(mut self, listener: F) -> Self
    where
        F: Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
    {
        self.on_mouse_move = Some(Box::new(listener));
        self
    }

    pub fn on_mouse_up<F>(mut self, listener: F) -> Self
    where
        F: Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    {
        self.on_mouse_up = Some(Box::new(listener));
        self
    }

    pub fn on_clear<F>(mut self, listener: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_clear = Some(Box::new(listener));
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let chars: Vec<char> = if self.masked {
            vec!['•'; self.value.chars().count()]
        } else {
            self.value.chars().collect()
        };

        let is_empty = chars.is_empty();
        let show_cursor = self
            .cursor_visible
            .unwrap_or(self.focused && !self.disabled);

        let height = self.height.unwrap_or(px(44.0));
        let rounded = self.rounded.unwrap_or(px(9999.0));
        let bg_color = self.bg.unwrap_or_else(|| theme.surface_container_high());
        let font_size = self.text_size.unwrap_or(px(14.0));

        let current_border = if self.error {
            theme.red()
        } else if self.focused {
            self.focused_border_color.unwrap_or_else(|| theme.primary())
        } else {
            self.border_color
                .unwrap_or_else(|| theme.outline_variant())
        };

        let mut container = div()
            .id(self.id)
            .flex()
            .items_center()
            .min_w_0()
            .overflow_hidden()
            .h(height)
            .rounded(rounded)
            .bg(bg_color)
            .border_1()
            .border_color(current_border)
            .px(self.padding_x.unwrap_or(px(14.0)))
            .gap(self.gap.unwrap_or(px(10.0)))
            .cursor_text()
            .transitions(|t| {
                t.border_color(millis(160).with_easing(ease_in_out))
                    .bg(millis(160).with_easing(ease_in_out))
            });

        if let Some(w) = self.width {
            container = container.w(w);
        } else if !self.flex_1 {
            container = container.w_full();
        }

        if self.flex_1 {
            container = container.flex_1();
        }

        if let Some(max_w) = self.max_width {
            container = container.max_w(max_w);
        }

        if self.disabled {
            container = container.opacity(0.5);
        }

        if let Some(on_click) = self.on_click {
            container = container.on_click(on_click);
        }

        if let Some(down) = self.on_mouse_down {
            container = container.on_mouse_down(MouseButton::Left, move |evt, win, app| {
                down(evt, win, app);
            });
        }

        if let Some(move_fn) = self.on_mouse_move {
            container = container.on_mouse_move(move |evt, win, app| {
                move_fn(evt, win, app);
            });
        }

        if let Some(up) = self.on_mouse_up {
            let up_rc = std::rc::Rc::new(up);
            let up_clone = up_rc.clone();
            container = container
                .on_mouse_up(MouseButton::Left, move |evt, win, app| {
                    up_rc(evt, win, app);
                })
                .on_mouse_up_out(MouseButton::Left, move |evt, win, app| {
                    up_clone(evt, win, app);
                });
        }

        if let Some(prefix) = self.prefix {
            container = container.child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(prefix),
            );
        }

        let mut text_container = div()
            .relative()
            .flex_1()
            .min_w_0()
            .h_full()
            .overflow_hidden()
            .flex()
            .items_center();

        if let Some(ref tracker) = self.tracker {
            let tracker_canvas = canvas(
                {
                    let tracker = tracker.clone();
                    move |bounds, _, _| {
                        tracker.track_bounds(bounds);
                        bounds
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0();

            text_container = text_container.child(tracker_canvas);
        }

        if self.centered {
            text_container = text_container.justify_center();
        }

        let total_chars = chars.len();
        let display_str: String = chars.iter().collect();

        // Calculate text metrics and horizontal scroll offset if text overflows container
        let mut scroll_offset_px = 0.0;
        if let (Some(tracker), Some(scroll_cell)) = (&self.tracker, &self.scroll_offset) {
            let available_width = tracker.right() - tracker.left();
            if available_width > 20.0 && !self.centered && !display_str.is_empty() {
                let cursor_pos = self.cursor.unwrap_or(total_chars).min(total_chars);
                let cursor_str: String = chars[..cursor_pos].iter().collect();

                let run = TextRun {
                    len: display_str.len(),
                    font: Font {
                        family: theme.font_family().into(),
                        ..Default::default()
                    },
                    color: theme.foreground(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                    letter_spacing: None,
                };

                let cursor_run = TextRun {
                    len: cursor_str.len(),
                    font: Font {
                        family: theme.font_family().into(),
                        ..Default::default()
                    },
                    color: theme.foreground(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                    letter_spacing: None,
                };

                let total_text_width: f32 = window
                    .text_system()
                    .shape_text(display_str.clone(), font_size, &[run], None, None)
                    .map(|s| f32::from(s.width))
                    .unwrap_or(0.0);

                let cursor_x: f32 = if cursor_pos == 0 {
                    0.0
                } else if cursor_pos == total_chars {
                    total_text_width
                } else {
                    window
                        .text_system()
                        .shape_text(cursor_str, font_size, &[cursor_run], None, None)
                        .map(|s| f32::from(s.width))
                        .unwrap_or(0.0)
                };

                let mut current_offset = scroll_cell.get();
                if cursor_x > current_offset + available_width - 14.0 {
                    current_offset = cursor_x - available_width + 14.0;
                }
                if cursor_x < current_offset + 10.0 {
                    current_offset = (cursor_x - 10.0).max(0.0);
                }
                if total_text_width <= available_width {
                    current_offset = 0.0;
                } else {
                    current_offset = current_offset
                        .min(total_text_width - available_width + 20.0)
                        .max(0.0);
                }
                scroll_cell.set(current_offset);
                scroll_offset_px = current_offset;
            }
        }

        let mut text_line = div()
            .flex()
            .items_center()
            .min_w_0()
            .whitespace_nowrap()
            .text_size(font_size);

        if scroll_offset_px > 0.0 {
            text_line = text_line.ml(-px(scroll_offset_px));
        }

        if self.centered {
            text_line = text_line.justify_center();
        }

        if is_empty {
            if show_cursor && !self.centered {
                text_line = text_line.child(
                    div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .bg(theme.accent())
                        .rounded_full(),
                );
            }
            text_line = text_line.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_ellipsis()
                    .whitespace_nowrap()
                    .text_size(font_size)
                    .text_color(theme.foreground_muted().opacity(if self.focused {
                        0.60
                    } else {
                        0.85
                    }))
                    .when(self.centered, |s| s.text_center())
                    .child(self.placeholder),
            );
        } else if let Some(ref sel) = self.selection {
            let start = sel.start.min(total_chars);
            let end = sel.end.min(total_chars);

            let before_str: String = chars[..start].iter().collect();
            let selected_str: String = chars[start..end].iter().collect();
            let after_str: String = chars[end..].iter().collect();

            if !before_str.is_empty() {
                text_line = text_line.child(
                    div()
                        .whitespace_nowrap()
                        .text_color(theme.foreground())
                        .child(before_str),
                );
            }

            if !selected_str.is_empty() {
                text_line = text_line.child(
                    div()
                        .whitespace_nowrap()
                        .bg(theme.accent().opacity(0.35))
                        .rounded(px(2.0))
                        .px(px(1.0))
                        .text_color(theme.foreground())
                        .child(selected_str),
                );
            }

            if !after_str.is_empty() {
                text_line = text_line.child(
                    div()
                        .whitespace_nowrap()
                        .text_color(theme.foreground())
                        .child(after_str),
                );
            }
        } else if self.focused {
            let cursor_pos = self.cursor.unwrap_or(total_chars).min(total_chars);
            let before_cursor: String = chars[..cursor_pos].iter().collect();
            let after_cursor: String = chars[cursor_pos..].iter().collect();

            if !before_cursor.is_empty() {
                text_line = text_line.child(
                    div()
                        .whitespace_nowrap()
                        .text_color(theme.foreground())
                        .child(before_cursor),
                );
            }

            if show_cursor {
                text_line = text_line.child(
                    div()
                        .w(px(2.0))
                        .h(px(18.0))
                        .bg(theme.accent())
                        .rounded_full(),
                );
            }

            if !after_cursor.is_empty() {
                text_line = text_line.child(
                    div()
                        .whitespace_nowrap()
                        .text_color(theme.foreground())
                        .child(after_cursor),
                );
            }
        } else {
            text_line = text_line.child(
                div()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .text_color(theme.foreground())
                    .when(self.centered, |s| s.text_center())
                    .child(display_str),
            );
        }

        text_container = text_container.child(text_line);
        container = container.child(text_container);

        if self.cleanable && !is_empty {
            let mut clear_btn = div()
                .id("input-clear-btn")
                .size(px(22.0))
                .flex_shrink_0()
                .rounded_full()
                .bg(theme.surface().opacity(0.8))
                .hover(|s| s.bg(theme.surface()))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .child(
                    svg()
                        .path("close.svg")
                        .size(px(10.0))
                        .text_color(theme.foreground_muted()),
                );

            if let Some(on_clear) = self.on_clear {
                clear_btn = clear_btn.on_click(on_clear);
            }

            container = container.child(clear_btn);
        }

        if let Some(suffix) = self.suffix {
            container = container.child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(suffix),
            );
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_basic() {
        let mut state = InputState::new("hello");
        assert_eq!(state.text(), "hello");
        assert_eq!(state.len(), 5);
        assert_eq!(state.cursor(), 5);

        state.insert(" world");
        assert_eq!(state.text(), "hello world");
        assert_eq!(state.cursor(), 11);

        state.backspace();
        assert_eq!(state.text(), "hello worl");
        assert_eq!(state.cursor(), 10);

        state.move_to_start(false);
        assert_eq!(state.cursor(), 0);
        state.delete();
        assert_eq!(state.text(), "ello worl");
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_select_all() {
        let mut state = InputState::new("Capsule Desktop");
        state.select_all();
        assert_eq!(state.selection, Some(0..15));
        assert_eq!(state.selected_text(), Some("Capsule Desktop".to_string()));

        // Backspace on select_all deletes everything
        state.backspace();
        assert_eq!(state.text(), "");
        assert_eq!(state.selection, None);
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_typing_over_selection() {
        let mut state = InputState::new("Hello World");
        state.select_all();
        state.insert("Replaced");
        assert_eq!(state.text(), "Replaced");
        assert_eq!(state.selection, None);
        assert_eq!(state.cursor(), 8);
    }

    #[test]
    fn test_arrow_collapses_selection() {
        let mut state = InputState::new("Hello World");
        state.set_cursor(5);
        state.move_right(true); // selects ' '
        assert_eq!(state.selection, Some(5..6));

        // Pressing Left arrow without shift collapses cursor to range.start
        state.move_left(false);
        assert_eq!(state.selection, None);
        assert_eq!(state.cursor(), 5);

        state.select_all();
        // Pressing Right arrow without shift collapses cursor to range.end
        state.move_right(false);
        assert_eq!(state.selection, None);
        assert_eq!(state.cursor(), 11);
    }

    #[test]
    fn test_shift_selection() {
        let mut state = InputState::new("abcdef");
        state.set_cursor(3); // between c and d
        state.move_left(true);
        assert_eq!(state.selection, Some(2..3));
        assert_eq!(state.selected_text(), Some("c".to_string()));

        state.move_left(true);
        assert_eq!(state.selection, Some(1..3));
        assert_eq!(state.selected_text(), Some("bc".to_string()));

        // Now move right with shift
        state.move_right(true);
        assert_eq!(state.selection, Some(2..3));

        state.move_right(true);
        assert_eq!(state.selection, None); // cursor back at anchor 3

        state.move_right(true);
        assert_eq!(state.selection, Some(3..4));
        assert_eq!(state.selected_text(), Some("d".to_string()));
    }

    #[test]
    fn test_select_word_at() {
        let mut state = InputState::new("foo bar baz");
        state.select_word_at(5); // inside "bar"
        assert_eq!(state.selection, Some(4..7));
        assert_eq!(state.selected_text(), Some("bar".to_string()));
        assert_eq!(state.cursor, 7);

        state.select_word_at(0); // at 'f' of "foo"
        assert_eq!(state.selection, Some(0..3));
        assert_eq!(state.selected_text(), Some("foo".to_string()));

        state.select_word_at(10); // at 'z' of "baz"
        assert_eq!(state.selection, Some(8..11));
        assert_eq!(state.selected_text(), Some("baz".to_string()));
    }

    #[test]
    fn test_word_boundaries_and_word_deletion() {
        let mut state = InputState::new("foo bar baz");
        state.set_cursor(11); // end of baz

        assert_eq!(state.prev_word_boundary(), 8); // start of baz
        state.delete_word_backward();
        assert_eq!(state.text(), "foo bar ");

        state.delete_word_backward();
        assert_eq!(state.text(), "foo ");

        state.move_to_start(false);
        assert_eq!(state.next_word_boundary(), 4);
        state.delete_word_forward();
        assert_eq!(state.text(), "");
    }

    #[test]
    fn test_max_len_and_control_chars() {
        let mut state = InputState::new("").with_max_len(5);
        state.insert("hello world\n\t");
        assert_eq!(state.text(), "hello");
        assert_eq!(state.len(), 5);

        // Cannot insert more
        state.insert("x");
        assert_eq!(state.text(), "hello");
    }

    #[test]
    fn test_masked_input() {
        let state = InputState::new("secret123").with_masked(true);
        assert!(state.masked);
        assert_eq!(state.text(), "secret123");
    }
}
