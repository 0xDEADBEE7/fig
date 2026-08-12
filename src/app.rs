use std::io::{self, Write};

use anyhow::Context;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    controls::{self, Action},
    figure::{self, Plane},
    models::Figure,
};

#[derive(Clone, Copy)]
enum Screen {
    Visualisation,
    Information,
}

struct State {
    screen: Screen,
    plane: Plane,
    focus: Option<usize>,
    search: Option<String>,
    help: bool,
    labels: bool,
}

pub fn run(figure: Figure, max_width: u16, max_height: u16) -> anyhow::Result<()> {
    figure.validate()?;
    let visualization = figure::visualizer(&figure);
    let mut state = State {
        screen: Screen::Visualisation,
        plane: visualization.default_plane(),
        focus: None,
        search: None,
        help: false,
        labels: true,
    };
    let _terminal = Terminal::enter()?;
    loop {
        draw(&*visualization, &state, max_width, max_height)?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            continue;
        }
        if handle_search(&mut state, key.code, &*visualization) {
            continue;
        }
        if state.help {
            state.help = false;
            continue;
        }
        let Some(action) = controls::action(key) else {
            continue;
        };
        if !dispatch(action, &mut state, &*visualization) {
            break;
        }
    }
    Ok(())
}

fn handle_search(
    state: &mut State,
    key: KeyCode,
    visualization: &dyn figure::Visualization,
) -> bool {
    let Some(query) = &mut state.search else {
        return false;
    };
    match key {
        KeyCode::Enter => {
            state.focus = visualization.find(query);
            center_on(&mut state.plane, state.focus, visualization);
            state.search = None;
        }
        KeyCode::Esc => state.search = None,
        KeyCode::Backspace => {
            query.pop();
        }
        KeyCode::Char(character) => query.push(character),
        _ => {}
    }
    true
}

fn dispatch(action: Action, state: &mut State, visualization: &dyn figure::Visualization) -> bool {
    match action {
        Action::Pan(x, y) => {
            state.plane.pan(x, y);
        }
        Action::Zoom(factor) => state.plane.zoom(factor),
        Action::Previous => {
            state.focus = Some(previous(state.focus, visualization.len()));
            center_on(&mut state.plane, state.focus, visualization);
        }
        Action::Next => {
            state.focus = Some(next(state.focus, visualization.len()));
            center_on(&mut state.plane, state.focus, visualization);
        }
        Action::OpenInformation => {
            if state.focus.is_some() {
                state.screen = Screen::Information;
            }
        }
        Action::Back => match state.screen {
            Screen::Information => state.screen = Screen::Visualisation,
            Screen::Visualisation if state.focus.is_some() => state.focus = None,
            Screen::Visualisation => return false,
        },
        Action::Search => state.search = Some(String::new()),
        Action::Help => state.help = true,
        Action::ToggleLabels => state.labels = !state.labels,
        Action::Reset => {
            state.plane = visualization.default_plane();
            state.focus = None;
        }
    }
    true
}

fn previous(focus: Option<usize>, count: usize) -> usize {
    focus.unwrap_or(count).saturating_sub(1)
}

fn next(focus: Option<usize>, count: usize) -> usize {
    focus.map_or(0, |index| (index + 1).min(count - 1))
}

fn center_on(plane: &mut Plane, focus: Option<usize>, visualization: &dyn figure::Visualization) {
    if let Some(index) = focus {
        (plane.center_x, plane.center_y) = visualization.position(index);
    }
}

fn draw(
    visualization: &dyn figure::Visualization,
    state: &State,
    max_width: u16,
    max_height: u16,
) -> anyhow::Result<()> {
    let (terminal_width, terminal_height) =
        terminal::size().context("could not read terminal size")?;
    let width = usize::from(terminal_width.min(max_width));
    let height = usize::from(terminal_height.min(max_height));
    anyhow::ensure!(width >= 30 && height >= 8, "terminal must be at least 30x8");
    let mut lines = match state.screen {
        Screen::Visualisation => {
            visualization.draw(width, height - 1, state.focus, state.plane, state.labels)
        }
        Screen::Information => visualization.information(state.focus, width, height - 1),
    };
    lines.push(status(state, visualization));
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    write!(stdout, "{}", lines.join("\r\n"))?;
    stdout.flush()?;
    Ok(())
}

fn status(state: &State, visualization: &dyn figure::Visualization) -> String {
    if state.help {
        return controls::HELP.to_owned();
    }
    if let Some(query) = &state.search {
        let match_label = visualization
            .suggestion(query)
            .map(|label| format!("  → {label}"))
            .unwrap_or_default();
        return format!("search: {query}{match_label}");
    }
    match state.screen {
        Screen::Visualisation => format!(
            "visualisation  labels: {}  Enter info  ? help",
            if state.labels { "on" } else { "off" }
        ),
        Screen::Information => "information  b/q/x back".to_owned(),
    }
}

struct Terminal;

impl Terminal {
    fn enter() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
