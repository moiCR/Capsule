use crate::theme::Theme;
use crate::tracker::dimension_tracker::DimensionTracker;
use gpui::{
    App, ClickEvent, ColorExt, DefiniteLength, ElementId, Font, Hsla, InteractiveElement,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, TextRun, Window, canvas,
    div, point, px,
};
use std::ops::Range;

pub type TextAreaClickListener = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseDownListener = Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseMoveListener = Box<dyn Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static>;
pub type TextAreaMouseUpListener = Box<dyn Fn(&MouseUpEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineInfo {
    pub line_idx: usize,
    pub start_char_idx: usize,
    pub end_char_idx: usize,
    pub chars: Vec<char>,
}

#[derive(Clone, Debug)]
pub struct TextAreaState {
    pub chars: Vec<char>,
    pub cursor: usize,
    pub selection: Option<Range<usize>>,
    pub selection_anchor: Option<usize>,
    pub placeholder: String,
    pub focused: bool,
    pub max_len: usize,
    pub is_dragging: bool,
    pub tracker: DimensionTracker,
    pub padding_left: f32,
    pub padding_top: f32,
    pub font_size: Pixels,
    pub line_height: Pixels,
}

impl Default for TextAreaState {
    fn default() -> Self {
        Self {
            chars: Vec::new(),
            cursor: 0,
            selection: None,
            selection_anchor: None,
            placeholder: String::new(),
            focused: false,
            max_len: 16384,
            is_dragging: false,
            tracker: DimensionTracker::new(),
            padding_left: 14.0,
            padding_top: 14.0,
            font_size: px(14.0),
            line_height: px(24.0),
        }
    }
}

impl TextAreaState {
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
            focused: false,
            max_len: 16384,
            is_dragging: false,
            tracker: DimensionTracker::new(),
            padding_left: 14.0,
            padding_top: 14.0,
            font_size: px(14.0),
            line_height: px(24.0),
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
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

    pub fn with_line_height(mut self, line_height: Pixels) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn with_padding(mut self, padding_x: f32, padding_y: f32) -> Self {
        self.padding_left = padding_x;
        self.padding_top = padding_y;
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
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
        self.selection = None;
        self.selection_anchor = None;
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

    pub fn lines(&self) -> Vec<LineInfo> {
        let mut lines = Vec::new();
        let mut current_chars = Vec::new();
        let mut line_start = 0;
        let mut char_idx = 0;

        for &c in &self.chars {
            if c == '\n' {
                lines.push(LineInfo {
                    line_idx: lines.len(),
                    start_char_idx: line_start,
                    end_char_idx: char_idx,
                    chars: current_chars,
                });
                current_chars = Vec::new();
                line_start = char_idx + 1;
            } else {
                current_chars.push(c);
            }
            char_idx += 1;
        }

        lines.push(LineInfo {
            line_idx: lines.len(),
            start_char_idx: line_start,
            end_char_idx: char_idx,
            chars: current_chars,
        });

        lines
    }

    pub fn cursor_line_and_col(&self) -> (usize, usize) {
        let lines = self.lines();
        let cur = self.cursor.min(self.chars.len());
        for line in &lines {
            if cur >= line.start_char_idx && cur <= line.end_char_idx {
                return (line.line_idx, cur - line.start_char_idx);
            }
        }
        if let Some(last) = lines.last() {
            (last.line_idx, last.chars.len())
        } else {
            (0, 0)
        }
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

    pub fn update_cursor_selection(&mut self, new_cursor: usize, select: bool) {
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

    pub fn insert(&mut self, text: &str) {
        self.delete_selection();
        for ch in text.chars() {
            if self.chars.len() >= self.max_len {
                break;
            }
            if ch.is_control() && ch != '\n' && ch != '\t' {
                continue;
            }
            if self.cursor <= self.chars.len() {
                self.chars.insert(self.cursor, ch);
                self.cursor += 1;
            }
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
        self.update_cursor_selection(new_cursor, select);
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
        self.update_cursor_selection(new_cursor, select);
    }

    pub fn move_up(&mut self, select: bool) {
        let (line_idx, col) = self.cursor_line_and_col();
        if line_idx == 0 {
            self.move_to_start(select);
            return;
        }
        let lines = self.lines();
        let prev_line = &lines[line_idx - 1];
        let target_col = col.min(prev_line.chars.len());
        let new_cursor = prev_line.start_char_idx + target_col;
        self.update_cursor_selection(new_cursor, select);
    }

    pub fn move_down(&mut self, select: bool) {
        let (line_idx, col) = self.cursor_line_and_col();
        let lines = self.lines();
        if line_idx + 1 >= lines.len() {
            self.move_to_end(select);
            return;
        }
        let next_line = &lines[line_idx + 1];
        let target_col = col.min(next_line.chars.len());
        let new_cursor = next_line.start_char_idx + target_col;
        self.update_cursor_selection(new_cursor, select);
    }

    pub fn move_to_start(&mut self, select: bool) {
        self.update_cursor_selection(0, select);
    }

    pub fn move_to_end(&mut self, select: bool) {
        let len = self.chars.len();
        self.update_cursor_selection(len, select);
    }

    pub fn move_to_line_start(&mut self, select: bool) {
        let (line_idx, _) = self.cursor_line_and_col();
        let lines = self.lines();
        if let Some(line) = lines.get(line_idx) {
            self.update_cursor_selection(line.start_char_idx, select);
        }
    }

    pub fn move_to_line_end(&mut self, select: bool) {
        let (line_idx, _) = self.cursor_line_and_col();
        let lines = self.lines();
        if let Some(line) = lines.get(line_idx) {
            self.update_cursor_selection(line.end_char_idx, select);
        }
    }

    pub fn prev_word_boundary(&self, from: usize) -> usize {
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

    pub fn next_word_boundary(&self, from: usize) -> usize {
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
        let prev = self.prev_word_boundary(self.cursor);
        self.update_cursor_selection(prev, select);
    }

    pub fn move_word_forward(&mut self, select: bool) {
        let next = self.next_word_boundary(self.cursor);
        self.update_cursor_selection(next, select);
    }

    pub fn delete_word_backward(&mut self) {
        if !self.delete_selection() {
            let prev = self.prev_word_boundary(self.cursor);
            if prev < self.cursor {
                self.chars.drain(prev..self.cursor);
                self.cursor = prev;
            }
        }
    }

    pub fn delete_word_forward(&mut self) {
        if !self.delete_selection() {
            let next = self.next_word_boundary(self.cursor);
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

    pub fn select_line_at(&mut self, index: usize) {
        let lines = self.lines();
        for line in lines {
            if index >= line.start_char_idx && index <= line.end_char_idx {
                let start = line.start_char_idx;
                let end = line.end_char_idx;
                self.selection = Some(start..end);
                self.selection_anchor = Some(start);
                self.cursor = end;
                return;
            }
        }
    }

    pub fn char_index_from_local_point(
        &self,
        local_x: Pixels,
        local_y: Pixels,
        window: &Window,
        theme: &Theme,
    ) -> usize {
        let lines = self.lines();
        if lines.is_empty() {
            return 0;
        }

        let line_height_val = f32::from(self.line_height).max(1.0);
        let line_idx = (f32::from(local_y) / line_height_val).floor() as usize;

        if line_idx >= lines.len() {
            return self.chars.len();
        }

        let line = &lines[line_idx];
        let line_str: String = line.chars.iter().collect();
        if line_str.is_empty() {
            return line.start_char_idx;
        }

        let run = TextRun {
            len: line_str.len(),
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

        let byte_idx = if let Ok(shaped) = window.text_system().shape_text(
            line_str.clone(),
            self.font_size,
            &[run],
            None,
            None,
        ) {
            match shaped.closest_byte_index_for_pixel_point(
                point(local_x.max(px(0.0)), px(0.0)),
                self.line_height,
            ) {
                Ok(idx) => idx,
                Err(idx) => idx,
            }
        } else {
            0
        };

        let char_col = line_str[..byte_idx.min(line_str.len())].chars().count();
        (line.start_char_idx + char_col).min(self.chars.len())
    }

    pub fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &Window,
        theme: &Theme,
    ) {
        let left = px(self.tracker.left());
        let top = px(self.tracker.top());
        let local_x = (event.position.x - left - px(self.padding_left)).max(px(0.0));
        let local_y = (event.position.y - top - px(self.padding_top)).max(px(0.0));

        let global_cursor = self.char_index_from_local_point(local_x, local_y, window, theme);

        if event.click_count >= 3 {
            self.select_line_at(global_cursor);
            self.is_dragging = false;
        } else if event.click_count == 2 {
            self.select_word_at(global_cursor);
            self.is_dragging = false;
        } else if event.modifiers.shift {
            let anchor = self.selection_anchor.unwrap_or(self.cursor);
            self.cursor = global_cursor;
            self.selection_anchor = Some(anchor);
            if anchor != global_cursor {
                self.selection = Some(anchor.min(global_cursor)..anchor.max(global_cursor));
            } else {
                self.selection = None;
            }
            self.is_dragging = true;
        } else {
            self.cursor = global_cursor;
            self.selection_anchor = Some(global_cursor);
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
        let top = px(self.tracker.top());
        let local_x = (event.position.x - left - px(self.padding_left)).max(px(0.0));
        let local_y = (event.position.y - top - px(self.padding_top)).max(px(0.0));

        let global_cursor = self.char_index_from_local_point(local_x, local_y, window, theme);
        self.cursor = global_cursor;
        let anchor = self.selection_anchor.unwrap_or(global_cursor);
        if anchor != global_cursor {
            self.selection = Some(anchor.min(global_cursor)..anchor.max(global_cursor));
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
        let text = self.selected_text().unwrap_or_else(|| self.text());
        if !text.is_empty() {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        }
    }

    pub fn cut(&mut self, cx: &mut App) {
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
        let shift = event.keystroke.modifiers.shift;
        let alt = event.keystroke.modifiers.alt;

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
                "backspace" | "w" => {
                    self.delete_word_backward();
                    return true;
                }
                "delete" => {
                    self.delete_word_forward();
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
            "up" => {
                self.move_up(shift);
                true
            }
            "down" => {
                self.move_down(shift);
                true
            }
            "home" => {
                self.move_to_line_start(shift);
                true
            }
            "end" => {
                self.move_to_line_end(shift);
                true
            }
            "enter" | "return" => {
                self.insert("\n");
                true
            }
            "space" => {
                self.insert(" ");
                true
            }
            "tab" => {
                self.insert("    ");
                true
            }
            "escape" => {
                if self.selection.is_some() {
                    self.selection = None;
                    self.selection_anchor = None;
                    true
                } else {
                    false
                }
            }
            _ => {
                if let Some(ref ch) = event.keystroke.key_char {
                    let clean: String = ch
                        .chars()
                        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                        .collect();
                    if !clean.is_empty() {
                        self.insert(&clean);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

#[derive(IntoElement)]
pub struct TextArea {
    id: ElementId,
    state: TextAreaState,
    height: Option<DefiniteLength>,
    width: Option<DefiniteLength>,
    flex_1: bool,
    bordered: bool,
    bg: Option<Hsla>,
    font_size: Pixels,
    line_height: Pixels,
    padding: Pixels,
    on_click: Option<TextAreaClickListener>,
    on_mouse_down: Option<TextAreaMouseDownListener>,
    on_mouse_move: Option<TextAreaMouseMoveListener>,
    on_mouse_up: Option<TextAreaMouseUpListener>,
}

impl TextArea {
    pub fn new(id: impl Into<ElementId>, state: &TextAreaState) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            height: None,
            width: None,
            flex_1: true,
            bordered: true,
            bg: None,
            font_size: state.font_size,
            line_height: state.line_height,
            padding: px(state.padding_left),
            on_click: None,
            on_mouse_down: None,
            on_mouse_move: None,
            on_mouse_up: None,
        }
    }

    pub fn height(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn flex_1(mut self, flex_1: bool) -> Self {
        self.flex_1 = flex_1;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn bg(mut self, bg: Hsla) -> Self {
        self.bg = Some(bg);
        self
    }

    pub fn font_size(mut self, font_size: Pixels) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn line_height(mut self, line_height: Pixels) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn padding(mut self, padding: Pixels) -> Self {
        self.padding = padding;
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
}

impl RenderOnce for TextArea {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();

        let tracker_canvas = canvas(
            {
                let tracker = self.state.tracker.clone();
                move |bounds, _, _| {
                    tracker.track_bounds(bounds);
                    bounds
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();

        let mut container = div()
            .id(self.id)
            .relative()
            .min_h_0()
            .cursor_text()
            .overflow_y_scroll();

        if self.bordered {
            let border_color = if self.state.focused {
                theme.primary()
            } else {
                theme.outline_variant()
            };
            container = container
                .rounded(px(14.0))
                .border_1()
                .border_color(border_color);
        }

        if let Some(bg) = self.bg {
            container = container.bg(bg);
        } else if self.bordered {
            container = container.bg(theme.surface_container_high());
        }

        container = container.p(self.padding);

        if let Some(w) = self.width {
            container = container.w(w);
        } else if !self.flex_1 {
            container = container.w_full();
        }

        if self.flex_1 {
            container = container.flex_1();
        }

        if let Some(h) = self.height {
            container = container.h(h);
        } else {
            container = container.h_full();
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

        let lines = self.state.lines();
        let total_lines = lines.len();
        let total_chars = self.state.chars.len();

        let mut lines_col = div().flex().flex_col().w_full().min_h_full();

        if self.state.is_empty() {
            let mut empty_line = div()
                .flex()
                .items_center()
                .gap(px(2.0))
                .min_h(self.line_height)
                .line_height(self.line_height);

            if self.state.focused {
                empty_line = empty_line.child(
                    div()
                        .w(px(2.0))
                        .h(px(20.0))
                        .bg(theme.accent())
                        .rounded_full(),
                );
            }

            empty_line = empty_line.child(
                div()
                    .text_size(self.font_size)
                    .text_color(theme.foreground_muted().opacity(0.60))
                    .child(self.state.placeholder.clone()),
            );

            lines_col = lines_col.child(empty_line);
        } else {
            for (line_idx, line) in lines.iter().enumerate() {
                let line_start = line.start_char_idx;
                let line_end = line.end_char_idx;
                let line_chars = &line.chars;

                let is_cursor_line = self.state.focused
                    && self.state.cursor >= line_start
                    && (self.state.cursor < line_end
                        || (self.state.cursor == line_end
                            && (line_idx + 1 == total_lines || self.state.cursor == total_chars)));
                let cursor_col = if is_cursor_line {
                    self.state.cursor.saturating_sub(line_start)
                } else {
                    usize::MAX
                };

                let mut line_el = div()
                    .id(("textarea-line", line_idx))
                    .w_full()
                    .min_h(self.line_height)
                    .line_height(self.line_height)
                    .text_size(self.font_size)
                    .flex()
                    .items_center()
                    .whitespace_normal();

                let sel_range = self.state.selection.as_ref();
                let (has_sel, sel_start_col, sel_end_col) = if let Some(range) = sel_range {
                    let s = range.start.min(total_chars);
                    let e = range.end.min(total_chars);
                    if s < line_end && e > line_start {
                        let start_col = s.saturating_sub(line_start).min(line_chars.len());
                        let end_col = e.saturating_sub(line_start).min(line_chars.len());
                        (true, start_col, end_col)
                    } else {
                        (false, 0, 0)
                    }
                } else {
                    (false, 0, 0)
                };

                if line_chars.is_empty() {
                    if is_cursor_line {
                        line_el = line_el.child(
                            div()
                                .w(px(2.0))
                                .h(px(20.0))
                                .bg(theme.accent())
                                .rounded_full(),
                        );
                    } else {
                        line_el = line_el.child(div().child("\u{00A0}"));
                    }
                } else if has_sel {
                    let before: String = line_chars[..sel_start_col].iter().collect();
                    let selected: String = line_chars[sel_start_col..sel_end_col].iter().collect();
                    let after: String = line_chars[sel_end_col..].iter().collect();

                    if !before.is_empty() {
                        line_el = line_el.child(div().text_color(theme.foreground()).child(before));
                    }

                    if !selected.is_empty() {
                        line_el = line_el.child(
                            div()
                                .bg(theme.accent().opacity(0.30))
                                .rounded(px(2.0))
                                .text_color(theme.foreground())
                                .child(selected),
                        );
                    }

                    if !after.is_empty() {
                        line_el = line_el.child(div().text_color(theme.foreground()).child(after));
                    }

                    if is_cursor_line {
                        line_el = line_el.child(
                            div()
                                .w(px(2.0))
                                .h(px(20.0))
                                .bg(theme.accent())
                                .rounded_full(),
                        );
                    }
                } else if is_cursor_line {
                    let col = cursor_col.min(line_chars.len());
                    let before: String = line_chars[..col].iter().collect();
                    let after: String = line_chars[col..].iter().collect();

                    if !before.is_empty() {
                        line_el = line_el.child(div().text_color(theme.foreground()).child(before));
                    }
                    line_el = line_el.child(
                        div()
                            .w(px(2.0))
                            .h(px(20.0))
                            .bg(theme.accent())
                            .rounded_full(),
                    );
                    if !after.is_empty() {
                        line_el = line_el.child(div().text_color(theme.foreground()).child(after));
                    }
                } else {
                    let content: String = line_chars.iter().collect();
                    line_el = line_el.child(div().text_color(theme.foreground()).child(content));
                }

                lines_col = lines_col.child(line_el);
            }
        }

        container.child(tracker_canvas).child(lines_col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_textarea_multiline() {
        let mut state = TextAreaState::new("Hello");
        assert_eq!(state.text(), "Hello");

        state.insert("\nWorld");
        assert_eq!(state.text(), "Hello\nWorld");

        state.move_up(false);
        state.insert("!");
        assert_eq!(state.text(), "Hello!\nWorld");

        state.move_down(false);
        state.insert("?");
        assert_eq!(state.text(), "Hello!\nWorld?");
    }

    #[test]
    fn test_selection_and_deletion() {
        let mut state = TextAreaState::new("First Line\nSecond Line\nThird Line");
        assert_eq!(state.lines().len(), 3);

        state.select_all();
        assert_eq!(state.selected_text(), Some("First Line\nSecond Line\nThird Line".to_string()));

        state.delete_selection();
        assert_eq!(state.text(), "");
        assert_eq!(state.cursor(), 0);
        assert_eq!(state.selection, None);
    }

    #[test]
    fn test_word_selection() {
        let mut state = TextAreaState::new("Quick brown fox");
        state.select_word_at(7); // "brown"
        assert_eq!(state.selected_text(), Some("brown".to_string()));
    }

    #[test]
    fn test_line_selection() {
        let mut state = TextAreaState::new("Line One\nLine Two\nLine Three");
        state.select_line_at(12); // inside "Line Two"
        assert_eq!(state.selected_text(), Some("Line Two".to_string()));
    }

    #[test]
    fn test_shift_navigation() {
        let mut state = TextAreaState::new("Hello World");
        state.set_cursor(5);
        state.move_right(true);
        assert_eq!(state.selected_text(), Some(" ".to_string()));
        state.move_right(true);
        assert_eq!(state.selected_text(), Some(" W".to_string()));
    }
}
