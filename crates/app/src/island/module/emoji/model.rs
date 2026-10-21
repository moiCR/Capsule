use services::EmojiItem;

pub(crate) const COLUMNS: usize = 7;
pub(crate) const ROWS: usize = 5;
pub(crate) const PAGE_SIZE: usize = COLUMNS * ROWS;

pub(crate) struct EmojiModel {
    items: &'static [EmojiItem],
    pub filtered: Vec<&'static EmojiItem>,
    pub query: String,
    pub category: Option<&'static str>,
    pub selected: usize,
}

impl EmojiModel {
    pub fn new(items: &'static [EmojiItem]) -> Self {
        Self {
            items,
            filtered: items.iter().collect(),
            query: String::new(),
            category: None,
            selected: 0,
        }
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.category = None;
        self.filter();
    }

    pub fn search(&mut self, query: String) {
        self.query = query;
        self.filter();
    }

    pub fn set_category(&mut self, category: Option<&'static str>) {
        self.category = category;
        self.filter();
    }

    fn filter(&mut self) {
        let query = self.query.trim().to_lowercase();
        self.filtered = self
            .items
            .iter()
            .filter(|item| {
                self.category
                    .is_none_or(|category| item.category.eq_ignore_ascii_case(category))
                    && (query.is_empty()
                        || item.emoji.contains(&query)
                        || item.name.to_lowercase().contains(&query)
                        || item.category.to_lowercase().contains(&query)
                        || item
                            .keywords
                            .iter()
                            .any(|keyword| keyword.to_lowercase().contains(&query)))
            })
            .collect();
        self.selected = 0;
    }

    pub fn page(&self) -> usize {
        self.selected / PAGE_SIZE
    }

    pub fn pages(&self) -> usize {
        self.filtered.len().div_ceil(PAGE_SIZE).max(1)
    }

    pub fn selected_item(&self) -> Option<&'static EmojiItem> {
        self.filtered.get(self.selected).copied()
    }

    pub fn navigate(&mut self, key: &str) {
        let count = self.filtered.len();
        if count == 0 {
            return;
        }
        self.selected = match key {
            "left" => (self.selected + count - 1) % count,
            "right" => (self.selected + 1) % count,
            "home" => 0,
            "end" => count - 1,
            "pageup" => ((self.page() + self.pages() - 1) % self.pages()) * PAGE_SIZE,
            "pagedown" => ((self.page() + 1) % self.pages()) * PAGE_SIZE,
            "up" | "down" => {
                let rows = count.div_ceil(COLUMNS);
                let column = self.selected % COLUMNS;
                let row = self.selected / COLUMNS;
                (1..=rows)
                    .find_map(|step| {
                        let next = if key == "down" {
                            (row + step) % rows
                        } else {
                            (row + rows - step) % rows
                        };
                        let index = next * COLUMNS + column;
                        (index < count).then_some(index)
                    })
                    .unwrap_or(self.selected)
            }
            _ => self.selected,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_matches_names_keywords_categories_and_emoji() {
        let mut model = EmojiModel::new(services::EmojiService::new().load_emojis());
        assert!(!model.filtered.is_empty());
        model.search("  HAPPY  ".into());
        assert!(model.filtered.iter().any(|item| item.emoji == "😀"));
        model.set_category(Some("nature"));
        assert!(model.filtered.iter().all(|item| item.category == "nature"));
        model.reset();
        model.set_category(Some("animals"));
        model.search("ANIMALS".into());
        assert!(!model.filtered.is_empty());
        assert!(model.filtered.iter().all(|item| item.category == "animals"));
        model.reset();
        model.search("😶‍🌫️".into());
        assert_eq!(
            model.selected_item().map(|item| item.emoji.as_str()),
            Some("😶‍🌫️")
        );
        model.reset();
        model.search("😀".into());
        assert_eq!(
            model.selected_item().map(|item| item.emoji.as_str()),
            Some("😀")
        );
        model.search("this-emoji-does-not-exist".into());
        assert!(model.selected_item().is_none());
        model.navigate("pagedown");
        assert_eq!(model.selected, 0);
        model.reset();
        assert!(model.query.is_empty());
        assert!(model.category.is_none());
        assert!(model.selected_item().is_some());
    }

    #[test]
    fn arrows_and_pages_wrap_without_selecting_missing_cells() {
        let items = services::EmojiService::new().load_emojis();
        let mut model = EmojiModel::new(&items[..PAGE_SIZE + 2]);
        model.navigate("left");
        assert_eq!(model.selected, PAGE_SIZE + 1);
        model.navigate("right");
        assert_eq!(model.selected, 0);
        model.selected = 6;
        model.navigate("up");
        assert_eq!(model.selected, 34);
        model.navigate("down");
        assert_eq!(model.selected, 6);
        model.navigate("pagedown");
        assert_eq!(model.selected, PAGE_SIZE);
        model.navigate("pagedown");
        assert_eq!(model.selected, 0);
        model.navigate("pageup");
        assert_eq!(model.selected, PAGE_SIZE);
        model.search("grinning".into());
        assert_eq!(model.page(), 0);
        assert_eq!(model.selected, 0);
    }
}
