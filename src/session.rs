use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
use fig::{Figure, RenderOptions, figure_bounds, render};

const PAN_FRACTION: f64 = 0.1;
const ZOOM_IN: f64 = 0.8;
const ZOOM_OUT: f64 = 1.25;

#[derive(Debug, Clone, Copy)]
struct Axis {
    min: f64,
    max: f64,
}

impl Axis {
    fn new(
        data_min: f64,
        data_max: f64,
        requested_min: Option<f64>,
        requested_max: Option<f64>,
    ) -> Result<Self> {
        let min = requested_min.unwrap_or(data_min);
        let max = requested_max.unwrap_or(data_max);
        anyhow::ensure!(min < max, "viewport minimum must be less than maximum");
        Ok(Self { min, max })
    }

    fn pan(&mut self, direction: f64) {
        let distance = (self.max - self.min) * PAN_FRACTION * direction;
        let min = self.min + distance;
        let max = self.max + distance;
        if min.is_finite() && max.is_finite() && min < max {
            self.min = min;
            self.max = max;
        }
    }

    fn zoom(&mut self, factor: f64) {
        let center = (self.min + self.max) / 2.0;
        let width = (self.max - self.min) * factor;
        let min = center - width / 2.0;
        let max = center + width / 2.0;
        if min.is_finite() && max.is_finite() && min < max {
            self.min = min;
            self.max = max;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Viewport {
    x: Axis,
    y: Axis,
}

impl Viewport {
    fn new(
        data_min_x: f64,
        data_max_x: f64,
        data_min_y: f64,
        data_max_y: f64,
        requested_min_x: Option<f64>,
        requested_max_x: Option<f64>,
    ) -> Result<Self> {
        Ok(Self {
            x: Axis::new(data_min_x, data_max_x, requested_min_x, requested_max_x)?,
            y: Axis::new(data_min_y, data_max_y, None, None)?,
        })
    }

    fn reset(&mut self, data_min_x: f64, data_max_x: f64, data_min_y: f64, data_max_y: f64) {
        self.x = Axis {
            min: data_min_x,
            max: data_max_x,
        };
        self.y = Axis {
            min: data_min_y,
            max: data_max_y,
        };
    }
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode().context("failed to enable terminal raw mode")?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen, DisableLineWrap, Hide) {
            let _ = disable_raw_mode();
            return Err(error).context("failed to enter alternate screen");
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, EnableLineWrap, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub(crate) fn run(
    figure: &Figure,
    x_min: Option<f64>,
    x_max: Option<f64>,
    max_width: usize,
    max_height: usize,
    color: bool,
) -> Result<()> {
    anyhow::ensure!(
        io::stdin().is_terminal(),
        "interactive mode requires a terminal on standard input"
    );
    anyhow::ensure!(
        io::stdout().is_terminal(),
        "interactive mode requires a terminal on standard output"
    );
    let (data_min_x, data_max_x, data_min_y, data_max_y) = figure_bounds(figure, 0);
    anyhow::ensure!(
        data_min_x.is_finite()
            && data_max_x.is_finite()
            && data_min_y.is_finite()
            && data_max_y.is_finite(),
        "interactive mode requires finite figure bounds"
    );
    let (data_min_x, data_max_x) = expand_axis(data_min_x, data_max_x);
    let (data_min_y, data_max_y) = expand_axis(data_min_y, data_max_y);
    let mut viewport = Viewport::new(data_min_x, data_max_x, data_min_y, data_max_y, x_min, x_max)?;
    let mut selected = 0usize;
    let _terminal = TerminalGuard::enter()?;

    loop {
        draw(figure, viewport, max_width, max_height, color, selected)?;
        match event::read().context("failed to read terminal event")? {
            Event::Key(key) if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Left | KeyCode::Char('h') => {
                        if let Figure::Histogram(histogram) = figure {
                            selected = selected.saturating_sub(1).min(histogram.buckets.len() - 1);
                        } else {
                            viewport.x.pan(-1.0);
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if let Figure::Histogram(histogram) = figure {
                            selected = (selected + 1).min(histogram.buckets.len() - 1);
                        } else {
                            viewport.x.pan(1.0);
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => viewport.y.pan(-1.0),
                    KeyCode::Up | KeyCode::Char('k') => viewport.y.pan(1.0),
                    KeyCode::Char('K') => {
                        viewport.x.zoom(ZOOM_IN);
                        viewport.y.zoom(ZOOM_IN);
                    }
                    KeyCode::Char('J') => {
                        viewport.x.zoom(ZOOM_OUT);
                        viewport.y.zoom(ZOOM_OUT);
                    }
                    KeyCode::Char('r') => {
                        viewport.reset(data_min_x, data_max_x, data_min_y, data_max_y);
                        selected = 0;
                    }
                    _ => continue,
                }
            }
            Event::Resize(_, _) => continue,
            _ => continue,
        }
    }
    Ok(())
}

fn expand_axis(min: f64, max: f64) -> (f64, f64) {
    if min < max {
        return (min, max);
    }
    let padding = min.abs().max(1.0) * 0.05;
    (min - padding, max + padding)
}

fn draw(
    figure: &Figure,
    viewport: Viewport,
    max_width: usize,
    max_height: usize,
    color: bool,
    selected: usize,
) -> Result<()> {
    let (terminal_width, terminal_height) =
        terminal::size().context("failed to read terminal size")?;
    let width = usize::from(terminal_width).min(max_width);
    let session_height = usize::from(terminal_height).min(max_height);
    anyhow::ensure!(
        width >= 30 && session_height >= 11,
        "interactive canvas must be at least 30x11 (increase --width/--height or the terminal size)"
    );
    let figure_height = session_height - 1;
    let output = render(
        figure,
        RenderOptions {
            width,
            height: figure_height,
            iterations: 0,
            color,
            x_min: Some(viewport.x.min),
            x_max: Some(viewport.x.max),
            y_min: Some(viewport.y.min),
            y_max: Some(viewport.y.max),
            selected_index: Some(selected),
            trim_output: false,
        },
    )?;
    let status = match figure {
        Figure::Histogram(histogram) => format!(
            "h/l select  j/k pan y  J/K zoom out/in  r reset  q quit   bucket: {}/{} ({})  y: {:.4} .. {:.4}",
            selected + 1,
            histogram.buckets.len(),
            histogram.buckets[selected].label,
            viewport.y.min,
            viewport.y.max
        ),
        _ => format!(
            "h/l pan x  j/k pan y  J/K zoom out/in  r reset  q quit   x: {:.4} .. {:.4}  y: {:.4} .. {:.4}",
            viewport.x.min, viewport.x.max, viewport.y.min, viewport.y.max
        ),
    };
    let status = fit_text(&status, width);
    let frame = output.replace('\n', "\r\n");
    let mut stdout = io::stdout();
    queue!(
        stdout,
        MoveTo(0, 0),
        Clear(ClearType::All),
        Print(frame),
        MoveTo(0, (session_height - 1) as u16),
        Print(status)
    )?;
    stdout.flush().context("failed to redraw terminal")
}

fn fit_text(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_and_zoom_are_unbounded() {
        let mut viewport = Viewport::new(0.0, 100.0, 0.0, 100.0, None, None).unwrap();
        viewport.x.pan(-1.0);
        viewport.y.pan(-1.0);
        assert!(viewport.x.min < 0.0);
        assert!(viewport.y.min < 0.0);
        viewport.x.zoom(ZOOM_OUT);
        viewport.y.zoom(ZOOM_IN);
        assert!(viewport.x.max - viewport.x.min > 100.0);
        assert!(viewport.y.max - viewport.y.min < 100.0);
    }

    #[test]
    fn extreme_navigation_keeps_viewport_finite() {
        let mut viewport = Viewport::new(-1.0, 1.0, -1.0, 1.0, None, None).unwrap();
        for _ in 0..10_000 {
            viewport.x.zoom(ZOOM_OUT);
            viewport.x.pan(1.0);
        }
        assert!(viewport.x.min.is_finite());
        assert!(viewport.x.max.is_finite());
        assert!(viewport.x.min < viewport.x.max);
    }

    #[test]
    fn reset_restores_data_bounds() {
        let mut viewport = Viewport::new(0.0, 100.0, 0.0, 100.0, Some(20.0), Some(60.0)).unwrap();
        viewport.x.pan(1.0);
        viewport.y.pan(-1.0);
        viewport.reset(0.0, 100.0, 0.0, 100.0);
        assert_eq!((viewport.x.min, viewport.x.max), (0.0, 100.0));
        assert_eq!((viewport.y.min, viewport.y.max), (0.0, 100.0));
    }

    #[test]
    fn frame_lines_return_to_the_left_margin() {
        assert_eq!(frame_text("one\ntwo\nthree"), "one\r\ntwo\r\nthree");
    }

    fn frame_text(output: &str) -> String {
        output.replace('\n', "\r\n")
    }

    #[test]
    fn status_text_never_exceeds_the_canvas() {
        assert_eq!(fit_text("abcdefgh", 5), "abcde");
        assert_eq!(fit_text("abc", 5), "abc");
    }
}
