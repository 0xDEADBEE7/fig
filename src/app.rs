use std::io::{self, Write};

use anyhow::Context;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    controls::{self, Action},
    figure::{self, Plane},
    modal::{Modal, ModalAction, SearchAction, SearchModal},
    models::Figure,
    settings::{self, Settings},
};

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Visualisation,
    Information,
    Settings,
}

struct State {
    screen: Screen,
    plane: Plane,
    focus: Option<usize>,
    search: Option<SearchModal>,
    help: bool,
    labels: bool,
    settings: Settings,
    modal: Modal<Settings>,
}

pub fn run(figure: Figure, max_width: u16, max_height: u16) -> anyhow::Result<()> {
    figure.validate()?;
    let visualization = figure::visualizer(&figure);
    let default_plane = visualization.default_plane();
    let mut state = State {
        screen: Screen::Visualisation,
        plane: default_plane,
        focus: None,
        search: None,
        help: false,
        labels: true,
        settings: Settings {
            plane: settings::PlaneSettings::from_plane(default_plane),
            render_options: true,
        },
        modal: Modal::new("settings", settings::items()),
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
        if !handle_key(key, &mut state, &*visualization) {
            break;
        }
    }
    Ok(())
}

fn handle_key(
    key: crossterm::event::KeyEvent,
    state: &mut State,
    visualization: &dyn figure::Visualization,
) -> bool {
    if state.screen == Screen::Settings {
        return handle_settings(key.code, state);
    }
    if state.search.is_some() {
        handle_search(key.code, state, visualization);
        return true;
    }
    if state.help {
        state.help = false;
        return true;
    }
    controls::action(key).is_none_or(|action| dispatch(action, state, visualization))
}

fn handle_settings(key: crossterm::event::KeyCode, state: &mut State) -> bool {
    let closes = matches!(
        state.modal.handle(key, &mut state.settings),
        ModalAction::Close
    );
    state.settings.plane.apply(&mut state.plane);
    if closes {
        state.screen = Screen::Visualisation;
    }
    true
}

fn handle_search(
    key: crossterm::event::KeyCode,
    state: &mut State,
    visualization: &dyn figure::Visualization,
) {
    let search = state.search.as_mut().expect("search checked by caller");
    let action = search.handle(key);
    search.update(visualization.labels().into_iter().enumerate());
    match action {
        SearchAction::Close => state.search = None,
        SearchAction::Select(index) => {
            state.focus = Some(index);
            center_on(&mut state.plane, state.focus, visualization);
            state.search = None;
        }
        SearchAction::None => {}
    }
}

fn dispatch(action: Action, state: &mut State, visualization: &dyn figure::Visualization) -> bool {
    match action {
        Action::Pan(x, y) => state.plane.pan(x, y),
        Action::Zoom(factor) => state.plane.zoom(factor),
        Action::Previous | Action::Next => move_focus(action, state, visualization),
        Action::OpenInformation => open_information(state),
        Action::Back => return go_back(state),
        Action::Search => start_search(state, visualization),
        Action::Help => state.help = true,
        Action::ToggleLabels => state.labels = !state.labels,
        Action::Settings => open_settings(state),
        Action::Reset => reset(state, visualization),
    }
    true
}

fn move_focus(action: Action, state: &mut State, visualization: &dyn figure::Visualization) {
    let index = match action {
        Action::Previous => previous(state.focus, visualization.len()),
        Action::Next => next(state.focus, visualization.len()),
        _ => unreachable!(),
    };
    state.focus = Some(index);
    center_on(&mut state.plane, state.focus, visualization);
}

fn open_information(state: &mut State) {
    if state.focus.is_some() {
        state.screen = Screen::Information;
    }
}

fn go_back(state: &mut State) -> bool {
    match state.screen {
        Screen::Settings | Screen::Information => state.screen = Screen::Visualisation,
        Screen::Visualisation if state.focus.is_some() => state.focus = None,
        Screen::Visualisation => return false,
    }
    true
}

fn start_search(state: &mut State, visualization: &dyn figure::Visualization) {
    let mut search = SearchModal::new();
    search.reset();
    search.update(visualization.labels().into_iter().enumerate());
    state.search = Some(search);
}

fn open_settings(state: &mut State) {
    state.modal.reset();
    state.screen = Screen::Settings;
}

fn reset(state: &mut State, visualization: &dyn figure::Visualization) {
    state.plane = visualization.default_plane();
    state.focus = None;
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
        Screen::Visualisation | Screen::Settings => visualization.draw(
            width,
            height - 1,
            state.focus,
            state.plane,
            state.labels,
            state.settings.render_options,
        ),
        Screen::Information => visualization.information(state.focus, width, height - 1),
    };
    if state.screen == Screen::Settings {
        state
            .modal
            .draw(&mut lines, &state.settings, width, height - 1);
    }
    if state.search.is_some() {
        if let Some(search) = &state.search {
            search.draw(&mut lines, width, height);
        }
    }
    lines.push(status(state));
    let mut stdout = io::stdout();
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    write!(stdout, "{}", lines.join("\r\n"))?;
    stdout.flush()?;
    Ok(())
}

fn status(state: &State) -> String {
    if state.help {
        return controls::HELP.to_owned();
    }
    if state.search.is_some() {
        return "search  type to filter  j/k navigate  Enter focus  Esc close".to_owned();
    }
    match state.screen {
        Screen::Visualisation => format!(
            "visualisation  labels: {}  Enter info  ? help",
            if state.labels { "on" } else { "off" }
        ),
        Screen::Settings => {
            "settings  type to search  j/k navigate  Enter/space toggle  b back".to_owned()
        }
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
