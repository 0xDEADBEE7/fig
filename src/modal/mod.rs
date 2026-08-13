use std::cmp::min;

mod search;

pub use search::{SearchAction, SearchModal};

use crossterm::event::KeyCode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModalAction {
    None,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum SearchInputAction {
    None,
    Submitted,
    Close,
}

pub(super) fn handle_search_input(query: &mut String, key: KeyCode) -> SearchInputAction {
    match key {
        KeyCode::Esc => SearchInputAction::Close,
        KeyCode::Enter => SearchInputAction::Submitted,
        KeyCode::Backspace => {
            query.pop();
            SearchInputAction::None
        }
        KeyCode::Char(character) => {
            query.push(character);
            SearchInputAction::None
        }
        _ => SearchInputAction::None,
    }
}

pub struct Item<T> {
    pub label: &'static str,
    pub value: fn(&T) -> String,
    pub activate: fn(&mut T),
}

pub struct Modal<T> {
    title: &'static str,
    items: Vec<Item<T>>,
    query: String,
    selected: usize,
    width_ratio: (usize, usize),
    height_ratio: (usize, usize),
    top: usize,
}

impl<T> Modal<T> {
    pub fn new(title: &'static str, items: Vec<Item<T>>) -> Self {
        Self {
            title,
            items,
            query: String::new(),
            selected: 0,
            width_ratio: (1, 2),
            height_ratio: (1, 3),
            top: 1,
        }
    }

    pub fn reset(&mut self) {
        self.query.clear();
        self.selected = 0;
    }

    pub fn handle(&mut self, key: KeyCode, context: &mut T) -> ModalAction {
        let result_count = self.matches().len();
        if self.selected == 0 {
            return match handle_search_input(&mut self.query, key) {
                SearchInputAction::Close => ModalAction::Close,
                SearchInputAction::Submitted if result_count > 0 => {
                    self.selected = 1;
                    ModalAction::None
                }
                _ => ModalAction::None,
            };
        }

        match key {
            KeyCode::Esc | KeyCode::Char('b' | 'q' | 'x') => return ModalAction::Close,
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(result_count);
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(index) = self.matches().get(self.selected - 1) {
                    (self.items[*index].activate)(context);
                }
            }
            _ => {}
        }
        ModalAction::None
    }

    fn matches(&self) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| fuzzy_match(item.label, &self.query))
            .map(|(index, _)| index)
            .collect()
    }

    pub fn content(&self, context: &T) -> Vec<String> {
        let matches = self.matches();
        let mut rows = vec![format!(
            "{} search: {}",
            mark(self.selected == 0),
            self.query
        )];
        rows.extend(matches.iter().enumerate().map(|(offset, index)| {
            let item = &self.items[*index];
            format!(
                "{} {}: {}",
                mark(self.selected == offset + 1),
                item.label,
                (item.value)(context)
            )
        }));
        rows
    }

    pub fn draw(&self, lines: &mut [String], context: &T, width: usize, height: usize) {
        let modal_width = min(
            width,
            (width * self.width_ratio.0 / self.width_ratio.1).max(24),
        );
        let modal_height = min(
            height,
            (height * self.height_ratio.0 / self.height_ratio.1).max(5),
        );
        let inner_width = modal_width.saturating_sub(2);
        let content_width = inner_width.saturating_sub(1);
        let left = (width.saturating_sub(modal_width)) / 2;
        let content = self.content(context);
        let mut rows = vec![format!("┌{:─^inner_width$}┐", format!(" {} ", self.title))];
        rows.push(format!(
            "│ {}│",
            render_search_row(&self.query, self.selected == 0, content_width)
        ));
        rows.push(format!("│ {:<content_width$}│", "─".repeat(content_width)));
        rows.extend(
            content
                .iter()
                .skip(1)
                .take(modal_height.saturating_sub(4))
                .enumerate()
                .map(|(offset, line)| {
                    format!(
                        "│ {}│",
                        render_row(
                            line.trim_start_matches('>').trim_start(),
                            self.selected == offset + 1,
                            content_width
                        )
                    )
                }),
        );
        while rows.len() < modal_height.saturating_sub(1) {
            rows.push(format!("│ {:<content_width$}│", ""));
        }
        rows.push(format!("└{}┘", "─".repeat(inner_width)));
        for (offset, row) in rows.into_iter().enumerate() {
            let line_index = self.top + offset;
            if line_index >= lines.len() {
                break;
            }
            lines[line_index] = overlay_line(&lines[line_index], &row, left, width);
        }
    }
}

pub(super) fn render_row(content: &str, selected: bool, width: usize) -> String {
    let row = format!("{content:<width$}");
    if selected {
        format!("\x1b[48;2;37;37;37m{row}\x1b[0m")
    } else {
        row
    }
}
pub(super) fn render_search_row(query: &str, active: bool, width: usize) -> String {
    render_row(query, active, width)
}
pub(super) fn overlay_line(base: &str, overlay: &str, left: usize, width: usize) -> String {
    let overlay = styled_columns(overlay);
    let mut output = String::with_capacity(base.len() + overlay.len());
    let mut style = String::new();
    let mut column = 0;
    let mut chars = base.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\x1b' {
            let mut sequence = String::from('\x1b');
            if chars.next_if_eq(&'[').is_some() {
                sequence.push('[');
                while let Some(character) = chars.next() {
                    sequence.push(character);
                    if ('@'..='~').contains(&character) {
                        break;
                    }
                }
                if sequence.ends_with('m') {
                    if sequence == "\x1b[0m" || sequence == "\x1b[m" {
                        style.clear();
                    } else {
                        style.push_str(&sequence);
                    }
                }
            }
            output.push_str(&sequence);
            continue;
        }
        if left <= column && column < left + overlay.len() && column < width {
            output.push_str("\x1b[0m");
            output.push_str(&overlay[column - left]);
            output.push_str(&style);
        } else {
            output.push(character);
        }
        column += 1;
    }
    while column < width {
        if left <= column && column < left + overlay.len() {
            output.push_str("\x1b[0m");
            output.push_str(&overlay[column - left]);
        } else {
            output.push(' ');
        }
        column += 1;
    }
    output
}

fn styled_columns(text: &str) -> Vec<String> {
    let mut columns = Vec::new();
    let mut style = String::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\x1b' {
            let mut sequence = String::from('\x1b');
            if chars.next_if_eq(&'[').is_some() {
                sequence.push('[');
                while let Some(character) = chars.next() {
                    sequence.push(character);
                    if ('@'..='~').contains(&character) {
                        break;
                    }
                }
            }
            if sequence.ends_with('m') {
                if sequence == "\x1b[0m" || sequence == "\x1b[m" {
                    style.clear();
                } else {
                    style.push_str(&sequence);
                }
            }
        } else {
            columns.push(format!("{style}{character}"));
        }
    }
    columns
}
#[cfg(test)]
mod tests {
    use super::{overlay_line, render_row};

    #[test]
    fn selected_row_style_covers_every_cell() {
        let row = render_row("abc", true, 5);
        let result = overlay_line("     ", &row, 0, 5);

        assert_eq!(result.matches("\x1b[48;2;37;37;37m").count(), 5);
    }
    #[test]
    fn overlay_counts_visible_columns_without_splitting_ansi_sequences() {
        let base = "a\x1b[38;5;196mb\x1b[0mc";
        let result = overlay_line(base, "XYZ", 1, 4);

        assert!(result.contains("a\x1b[38;5;196m"));
        assert!(result.contains("\x1b[0mX"));
        assert!(result.contains("Y"));
        assert!(result.contains("Z"));
        assert_eq!(visible_width(&result), 4);
    }

    fn visible_width(text: &str) -> usize {
        let mut width = 0;
        let mut escape = false;
        let mut csi = false;
        for character in text.chars() {
            if escape {
                if !csi {
                    csi = character == '[';
                } else if ('@'..='~').contains(&character) {
                    escape = false;
                    csi = false;
                }
            } else if character == '\x1b' {
                escape = true;
            } else {
                width += 1;
            }
        }
        width
    }
}

fn mark(selected: bool) -> &'static str {
    if selected { "" } else { "" }
}

fn fuzzy_match(label: &str, query: &str) -> bool {
    let mut chars = label.chars().flat_map(char::to_lowercase);
    query
        .chars()
        .flat_map(char::to_lowercase)
        .all(|needle| chars.any(|candidate| candidate == needle))
}
