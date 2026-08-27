use crossterm::event::KeyCode;

use super::Settings;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Level {
    Root,
    Plane,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuAction {
    None,
    Close,
    Back,
    ToggleAxis,
    ToggleGrid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Menu {
    level: Level,
    pub cursor: usize,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            level: Level::Root,
            cursor: 0,
        }
    }
    pub fn title(&self) -> &'static str {
        if self.level == Level::Root {
            "settings"
        } else {
            "plane"
        }
    }
    pub fn action(&mut self, key: KeyCode, settings: &mut Settings) -> MenuAction {
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(if self.level == Level::Root { 0 } else { 1 })
            }
            KeyCode::Enter | KeyCode::Char(' ') if self.level == Level::Root => {
                self.level = Level::Plane;
                self.cursor = 0;
            }
            KeyCode::Enter | KeyCode::Char(' ') if self.cursor == 0 => {
                settings.plane.axis_lines = !settings.plane.axis_lines;
                return MenuAction::ToggleAxis;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                settings.plane.grid_lines = !settings.plane.grid_lines;
                return MenuAction::ToggleGrid;
            }
            KeyCode::Esc | KeyCode::Char('b') if self.level == Level::Plane => {
                self.level = Level::Root;
                self.cursor = 0;
                return MenuAction::Back;
            }
            KeyCode::Esc | KeyCode::Char('b') => return MenuAction::Close,
            _ => {}
        }
        MenuAction::None
    }
    pub fn content(&self, settings: &Settings) -> Vec<String> {
        if self.level == Level::Root {
            return vec!["> plane  →".into()];
        }
        vec![
            format!(
                "{} axis-lines: {}",
                mark(self.cursor == 0),
                yes(settings.plane.axis_lines)
            ),
            format!(
                "{} grid-lines: {}",
                mark(self.cursor == 1),
                yes(settings.plane.grid_lines)
            ),
        ]
    }
}
fn mark(selected: bool) -> &'static str {
    if selected { ">" } else { " " }
}
fn yes(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
