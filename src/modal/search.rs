use crossterm::event::KeyCode;

use super::{SearchInputAction, handle_search_input, overlay_line, render_row, render_search_row};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SearchAction {
    None,
    Close,
    Select(usize),
}

pub struct SearchModal {
    query: String,
    editing: bool,
    results: Vec<(usize, String)>,
    selected: usize,
    top: usize,
}

impl SearchModal {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            editing: true,
            results: Vec::new(),
            selected: 0,
            top: 1,
        }
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.results.clear();
        self.editing = true;
        self.selected = 0;
    }

    pub fn update(&mut self, labels: impl IntoIterator<Item = (usize, String)>) {
        let mut ranked = labels
            .into_iter()
            .filter_map(|(index, label)| {
                score(&label, &self.query).map(|rank| (index, label, rank))
            })
            .collect::<Vec<_>>();
        ranked.sort_by(|a, b| a.2.cmp(&b.2).then_with(|| a.1.cmp(&b.1)));
        self.results = ranked
            .into_iter()
            .map(|(index, label, _)| (index, label))
            .collect();
        self.selected = self.selected.min(self.results.len());
    }

    pub fn handle(&mut self, key: KeyCode) -> SearchAction {
        if self.editing {
            match handle_search_input(&mut self.query, key) {
                SearchInputAction::Close => return SearchAction::Close,
                SearchInputAction::Submitted => {
                    self.editing = false;
                    return SearchAction::None;
                }
                SearchInputAction::None => return SearchAction::None,
            }
        }
        match key {
            KeyCode::Esc => SearchAction::Close,
            KeyCode::Up if self.selected == 0 => {
                self.editing = true;
                SearchAction::None
            }
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                SearchAction::None
            }
            KeyCode::Down => {
                self.selected = (self.selected + 1).min(self.results.len());
                SearchAction::None
            }
            KeyCode::Char('k') if self.selected == 0 => {
                self.editing = true;
                SearchAction::None
            }
            KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
                SearchAction::None
            }
            KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(self.results.len());
                SearchAction::None
            }
            KeyCode::Backspace | KeyCode::Char(_) => SearchAction::None,
            KeyCode::Enter => self
                .results
                .get(self.selected)
                .map(|(index, _)| SearchAction::Select(*index))
                .unwrap_or(SearchAction::None),
            _ => SearchAction::None,
        }
    }

    pub fn draw(&self, lines: &mut [String], width: usize, height: usize) {
        let modal_width = width.min((width / 2).max(32));
        let modal_height = height.min((height / 3).max(7));
        let left = (width.saturating_sub(modal_width)) / 2;
        let inner = modal_width.saturating_sub(2);
        let content_width = inner.saturating_sub(1);
        let mut rows = vec![format!("┌{:─^inner$}┐", " search ")];
        rows.push(format!(
            "│ {}│",
            render_search_row(&self.query, self.editing, content_width)
        ));
        rows.push(format!("│ {:<content_width$}│", "─".repeat(content_width)));
        for (offset, (_, label)) in self
            .results
            .iter()
            .take(modal_height.saturating_sub(4))
            .enumerate()
        {
            rows.push(format!(
                "│ {}│",
                render_row(
                    label,
                    !self.editing && offset == self.selected,
                    content_width
                )
            ));
        }
        while rows.len() < modal_height.saturating_sub(1) {
            rows.push(format!("│ {:<content_width$}│", ""));
        }
        rows.push(format!("└{}┘", "─".repeat(inner)));
        for (offset, row) in rows.into_iter().enumerate() {
            let line = self.top + offset;
            if line < lines.len() {
                lines[line] = overlay_line(&lines[line], &row, left, width);
            }
        }
    }
}

fn score(label: &str, query: &str) -> Option<(usize, usize)> {
    if query.is_empty() {
        return Some((0, 0));
    }
    let query = query.to_lowercase();
    let label = label.to_lowercase();
    if label == query {
        return Some((0, 0));
    }
    if label.starts_with(&query) {
        return Some((1, label.len()));
    }
    if label.contains(&query) {
        return Some((2, label.find(&query).unwrap_or(usize::MAX)));
    }
    let mut position = 0;
    let mut gaps = 0;
    for wanted in query.chars() {
        let found = label[position..].find(wanted)?;
        gaps += found;
        position += found + wanted.len_utf8();
    }
    Some((3, gaps))
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyCode;

    use super::score;

    #[test]
    fn navigation_does_not_edit_query_and_up_returns_to_input() {
        let mut search = super::SearchModal::new();
        search.update([(0, "alpha".into()), (1, "beta".into())]);
        assert_eq!(search.handle(KeyCode::Char('a')), super::SearchAction::None);
        assert_eq!(search.query, "a");
        assert_eq!(search.handle(KeyCode::Enter), super::SearchAction::None);
        assert!(!search.editing);

        assert_eq!(search.handle(KeyCode::Char('b')), super::SearchAction::None);
        assert_eq!(search.query, "a");
        assert_eq!(search.handle(KeyCode::Char('j')), super::SearchAction::None);
        assert_eq!(search.selected, 1);
        assert_eq!(search.handle(KeyCode::Char('k')), super::SearchAction::None);
        assert_eq!(search.selected, 0);
        assert_eq!(search.handle(KeyCode::Char('k')), super::SearchAction::None);
        assert!(search.editing);
    }
    #[test]
    fn ranks_exact_and_prefix_before_fuzzy_matches() {
        assert!(score("API", "api").unwrap() < score("API server", "api").unwrap());
        assert!(score("API server", "api").unwrap() < score("Application", "api").unwrap());
    }
}
