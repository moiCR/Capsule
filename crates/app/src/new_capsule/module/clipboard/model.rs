use services::{ClipboardItem, Snippet};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ClipboardTab {
    #[default]
    History,
    Snippets,
}

#[derive(Default)]
pub(crate) struct ClipboardModel {
    pub items: Vec<ClipboardItem>,
    pub snippets: Vec<Snippet>,
    pub query: String,
    pub tab: ClipboardTab,
    pub selected: usize,
    pub filtered: Vec<usize>,
}

impl ClipboardModel {
    fn selected_id(&self) -> Option<&str> {
        let index = *self.filtered.get(self.selected)?;
        match self.tab {
            ClipboardTab::History => self.items.get(index).map(|item| item.id.as_str()),
            ClipboardTab::Snippets => self.snippets.get(index).map(|item| item.id.as_str()),
        }
    }
    pub fn update(&mut self, items: Vec<ClipboardItem>, snippets: Vec<Snippet>) {
        let selected = self.selected_id().map(str::to_owned);
        self.items = items;
        self.snippets = snippets;
        self.filter();
        if let Some(selected) = selected {
            self.selected = self
                .filtered
                .iter()
                .position(|index| match self.tab {
                    ClipboardTab::History => self.items[*index].id == selected,
                    ClipboardTab::Snippets => self.snippets[*index].id == selected,
                })
                .unwrap_or(self.selected);
        }
        self.selected = self.selected.min(self.filtered.len().saturating_sub(1));
    }
    pub fn search(&mut self, query: String) {
        self.query = query;
        self.selected = 0;
        self.filter();
    }
    pub fn switch(&mut self, tab: ClipboardTab) {
        self.tab = tab;
        self.selected = 0;
        self.filter();
    }
    fn filter(&mut self) {
        let query = self.query.trim().to_lowercase();
        self.filtered = match self.tab {
            ClipboardTab::History => self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    query.is_empty()
                        || item.preview.to_lowercase().contains(&query)
                        || (item.is_image
                            && ("image".contains(&query) || "imagen".contains(&query)))
                })
                .map(|(index, _)| index)
                .collect(),
            ClipboardTab::Snippets => self
                .snippets
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    query.is_empty()
                        || item.title.to_lowercase().contains(&query)
                        || item.content.to_lowercase().contains(&query)
                })
                .map(|(index, _)| index)
                .collect(),
        };
    }
    pub fn navigate(&mut self, delta: i32) {
        self.selected = (self.selected as i64 + delta as i64)
            .clamp(0, self.filtered.len().saturating_sub(1) as i64)
            as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: &str, preview: &str, is_image: bool) -> ClipboardItem {
        ClipboardItem {
            id: id.into(),
            preview: preview.into(),
            is_image,
            image_path: None,
        }
    }
    #[test]
    fn gallery_navigation_handles_a_partial_last_row() {
        let mut model = ClipboardModel::default();
        model.update(
            (0..5)
                .map(|index| item(&index.to_string(), "text", false))
                .collect(),
            Vec::new(),
        );
        model.navigate(2);
        assert_eq!(model.selected, 2);
        model.navigate(1);
        assert_eq!(model.selected, 3);
        model.navigate(2);
        assert_eq!(model.selected, 4);
        model.navigate(-2);
        assert_eq!(model.selected, 2);
        model.navigate(-10);
        assert_eq!(model.selected, 0);
    }

    #[test]
    fn search_handles_images_unicode_and_full_snippet_content() {
        let mut model = ClipboardModel::default();
        model.update(
            vec![item("1", "ÁRBOL", false), item("2", "PNG • 100×80", true)],
            vec![Snippet {
                id: "p".into(),
                title: "Saludo".into(),
                content: "hola mundo".into(),
            }],
        );
        model.search("árb".into());
        assert_eq!(model.filtered, vec![0]);
        model.search("imagen".into());
        assert_eq!(model.filtered, vec![1]);
        model.switch(ClipboardTab::Snippets);
        model.search("MUNDO".into());
        assert_eq!(model.filtered, vec![0]);
    }
    #[test]
    fn refresh_retains_selection_and_removing_last_item_is_safe() {
        let mut model = ClipboardModel::default();
        let a = item("a", "first", false);
        let b = item("b", "second", false);
        model.update(vec![a.clone(), b.clone()], Vec::new());
        model.navigate(1);
        model.update(vec![item("c", "new", false), a, b], Vec::new());
        assert_eq!(model.selected, 2);
        model.navigate(50);
        assert_eq!(model.selected, 2);
        model.update(Vec::new(), Vec::new());
        model.navigate(-1);
        assert_eq!(model.selected, 0);
    }
}
