use crossterm::event::{KeyCode, KeyEvent};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Pan(i8, i8),
    Zoom(f64),
    Previous,
    Next,
    OpenInformation,
    Back,
    Search,
    Help,
    ToggleLabels,
    Reset,
}

pub fn action(key: KeyEvent) -> Option<Action> {
    pan(key.code)
        .or_else(|| zoom(key.code))
        .or_else(|| navigation(key.code))
        .or_else(|| utility(key.code))
}

fn pan(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('h') => Some(Action::Pan(1, 0)),
        KeyCode::Char('j') => Some(Action::Pan(0, -1)),
        KeyCode::Char('k') => Some(Action::Pan(0, 1)),
        KeyCode::Char('l') => Some(Action::Pan(-1, 0)),
        _ => None,
    }
}

fn zoom(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('J') => Some(Action::Zoom(1.25)),
        KeyCode::Char('K') => Some(Action::Zoom(0.8)),
        _ => None,
    }
}

fn navigation(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::Char('H') => Some(Action::Previous),
        KeyCode::Char('L') => Some(Action::Next),
        KeyCode::Enter => Some(Action::OpenInformation),
        _ => None,
    }
}

fn utility(key: KeyCode) -> Option<Action> {
    match key {
        KeyCode::BackTab => Some(Action::ToggleLabels),
        KeyCode::Char('/') => Some(Action::Search),
        KeyCode::Char('?') => Some(Action::Help),
        KeyCode::Char('r') => Some(Action::Reset),
        KeyCode::Char('b' | 'q' | 'x') => Some(Action::Back),
        _ => None,
    }
}

pub const HELP: &str = "h/j/k/l pan   J/K zoom   H/L previous/next node   Enter information   Shift-Tab labels   / search   ? help   r reset   b/q/x back or quit";
