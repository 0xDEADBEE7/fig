use crate::{
    figure::{Plane, plane},
    models::Series,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    Normal,
    Dim,
    Red,
    Green,
    Blue,
    Yellow,
    Bright,
}

#[derive(Clone, Copy)]
struct Cell {
    dots: u8,
    text: Option<char>,
    tone: Tone,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            dots: 0,
            text: None,
            tone: Tone::Normal,
        }
    }
}

pub fn draw(
    points: &[Vec<(isize, isize)>],
    series: &[Series],
    width: usize,
    height: usize,
    focus: Option<usize>,
    plane: Plane,
    labels: bool,
    render_options: bool,
) -> Vec<String> {
    let inner_width = width.saturating_sub(2);
    let inner_height = height.saturating_sub(2);
    let mut canvas = background(plane, inner_width, inner_height);

    match focus {
        Some(selected) => draw_focused(&mut canvas, points, series, selected, render_options),
        None => draw_all(&mut canvas, points, series, render_options),
    }

    draw_legend(&mut canvas, series, focus, labels, render_options);
    frame(canvas, width)
}

fn draw_all(
    canvas: &mut [Vec<Cell>],
    points: &[Vec<(isize, isize)>],
    series: &[Series],
    render_options: bool,
) {
    for (index, points) in points.iter().enumerate() {
        draw_series(
            canvas,
            points,
            tone_for(style(&series[index], render_options).color),
        );
    }
}

fn draw_focused(
    canvas: &mut [Vec<Cell>],
    points: &[Vec<(isize, isize)>],
    series: &[Series],
    selected: usize,
    render_options: bool,
) {
    for (index, points) in points.iter().enumerate() {
        let configured = style(&series[index], render_options);
        let tone = if configured.override_focus || configured.color != Color::Default {
            tone_for(configured.color)
        } else {
            Tone::Dim
        };
        draw_series(canvas, points, tone);
    }
    if let Some(points) = points.get(selected) {
        let configured = style(&series[selected], render_options);
        let tone = if configured.override_focus || configured.color != Color::Default {
            tone_for(configured.color)
        } else {
            Tone::Normal
        };
        redraw_focused_series(canvas, points, tone);
    }
}

fn style(series: &Series, render_options: bool) -> crate::figure::render_options::Style {
    crate::figure::render_options::style(&series.fig, render_options)
}

use crate::figure::render_options::Color;
fn redraw_focused_series(canvas: &mut [Vec<Cell>], points: &[(isize, isize)], tone: Tone) {
    let width = canvas.first().map_or(0, Vec::len);
    let mut focused = vec![vec![Cell::default(); width]; canvas.len()];
    draw_series(&mut focused, points, tone);
    for (canvas_row, focused_row) in canvas.iter_mut().zip(focused) {
        for (cell, focused_cell) in canvas_row.iter_mut().zip(focused_row) {
            if focused_cell.dots != 0 {
                cell.dots = focused_cell.dots;
                cell.text = None;
                cell.tone = Tone::Normal;
            }
        }
    }
}

fn background(plane: Plane, width: usize, height: usize) -> Vec<Vec<Cell>> {
    plane::background(plane, width, height)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|character| Cell {
                    text: (character != ' ').then_some(character),
                    ..Cell::default()
                })
                .collect()
        })
        .collect()
}

fn draw_series(canvas: &mut [Vec<Cell>], points: &[(isize, isize)], tone: Tone) {
    for segment in points.windows(2) {
        raster_line(canvas, segment[0], segment[1], tone);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_line_replaces_other_lines_at_an_overlap() {
        let mut canvas = background(Plane::new(false, false), 2, 1);
        draw_series(&mut canvas, &[(0, 0), (3, 3)], Tone::Dim);
        redraw_focused_series(&mut canvas, &[(0, 3), (3, 0)], Tone::Normal);

        assert_eq!(canvas[0][0].dots, 96);
        assert_eq!(canvas[0][0].tone, Tone::Normal);
    }
}
fn tone_for(color: crate::figure::render_options::Color) -> Tone {
    use crate::figure::render_options::Color;
    match color {
        Color::Dim => Tone::Dim,
        Color::Bright => Tone::Bright,
        Color::Red => Tone::Red,
        Color::Green => Tone::Green,
        Color::Blue => Tone::Blue,
        Color::Yellow => Tone::Yellow,
        _ => Tone::Normal,
    }
}

fn raster_line(canvas: &mut [Vec<Cell>], from: (isize, isize), to: (isize, isize), tone: Tone) {
    let Some((from, to)) = clip(from, to, canvas) else {
        return;
    };
    let (mut x, mut y) = from;
    let (end_x, end_y) = to;
    let (dx, dy) = ((end_x - x).abs(), -(end_y - y).abs());
    let (step_x, step_y) = (
        if x < end_x { 1 } else { -1 },
        if y < end_y { 1 } else { -1 },
    );
    let mut error = dx + dy;
    loop {
        put_dot(canvas, x, y, tone);
        if (x, y) == (end_x, end_y) {
            return;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x += step_x;
        }
        if doubled <= dx {
            error += dx;
            y += step_y;
        }
    }
}

fn clip(
    from: (isize, isize),
    to: (isize, isize),
    canvas: &[Vec<Cell>],
) -> Option<((isize, isize), (isize, isize))> {
    let (width, height) = (
        canvas.first()?.len().checked_mul(2)?,
        canvas.len().checked_mul(4)?,
    );
    let (x, y) = (from.0 as f64, from.1 as f64);
    let (dx, dy) = (to.0 as f64 - x, to.1 as f64 - y);
    let (mut start, mut end) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, x),
        (dx, width as f64 - 1.0 - x),
        (-dy, y),
        (dy, height as f64 - 1.0 - y),
    ] {
        if p == 0.0 && q < 0.0 {
            return None;
        }
        if p < 0.0 {
            start = start.max(q / p);
        } else if p > 0.0 {
            end = end.min(q / p);
        }
    }
    (start <= end).then(|| {
        (
            (
                (x + dx * start).round() as isize,
                (y + dy * start).round() as isize,
            ),
            (
                (x + dx * end).round() as isize,
                (y + dy * end).round() as isize,
            ),
        )
    })
}

fn put_dot(canvas: &mut [Vec<Cell>], x: isize, y: isize, tone: Tone) {
    if x < 0 || y < 0 {
        return;
    }
    let Some(row) = canvas.get_mut(y as usize / 4) else {
        return;
    };
    let Some(cell) = row.get_mut(x as usize / 2) else {
        return;
    };
    if tone == Tone::Normal || cell.dots == 0 {
        cell.tone = tone;
    }
    // The plane is a background layer. A plotted braille dot must replace its
    // grid or axis character rather than remaining hidden beneath it.
    cell.text = None;
    cell.dots |= dot(x as usize % 2, y as usize % 4);
}

fn dot(x: usize, y: usize) -> u8 {
    [[1, 2, 4, 64], [8, 16, 32, 128]][x][y]
}

fn draw_legend(
    canvas: &mut [Vec<Cell>],
    series: &[Series],
    focus: Option<usize>,
    labels: bool,
    render_options: bool,
) {
    for (index, series) in series.iter().enumerate() {
        let configured = crate::figure::render_options::style(&series.fig, render_options);
        if !configured.show_label.unwrap_or(labels) {
            continue;
        }
        let tone = if focus.is_some_and(|selected| selected != index) {
            Tone::Dim
        } else {
            Tone::Normal
        };
        write_text(canvas, 1, index, &series.label, tone);
    }
}

fn write_text(canvas: &mut [Vec<Cell>], x: usize, y: usize, text: &str, tone: Tone) {
    let Some(row) = canvas.get_mut(y) else { return };
    for (offset, character) in text.chars().enumerate() {
        let Some(cell) = row.get_mut(x + offset) else {
            break;
        };
        cell.text = Some(character);
        cell.tone = tone;
    }
}

fn frame(canvas: Vec<Vec<Cell>>, width: usize) -> Vec<String> {
    let border = "─".repeat(width.saturating_sub(2));
    let mut lines = vec![format!("┌{border}┐")];
    lines.extend(
        canvas
            .into_iter()
            .map(|row| format!("│{}│", render_row(row))),
    );
    lines.push(format!("└{border}┘"));
    lines
}

fn render_row(row: Vec<Cell>) -> String {
    row.into_iter().fold(String::new(), |mut line, cell| {
        let character = cell.text.unwrap_or_else(|| braille(cell.dots));
        push_toned(&mut line, character, cell.tone);
        line
    })
}

fn braille(dots: u8) -> char {
    char::from_u32(0x2800 + u32::from(dots)).unwrap()
}

fn push_toned(line: &mut String, character: char, tone: Tone) {
    match tone {
        Tone::Normal => line.push(character),
        // Keep the escape sequences as control bytes, as in graph::render.
        Tone::Dim => line.push_str(&format!("\x1b[38;5;240m{character}\x1b[0m")),
        Tone::Red => line.push_str(&format!("\x1b[38;5;196m{character}\x1b[0m")),
        Tone::Green => line.push_str(&format!("\x1b[38;5;40m{character}\x1b[0m")),
        Tone::Blue => line.push_str(&format!("\x1b[38;5;33m{character}\x1b[0m")),
        Tone::Yellow => line.push_str(&format!("\x1b[38;5;226m{character}\x1b[0m")),
        Tone::Bright => line.push_str(&format!("\x1b[97m{character}\x1b[0m")),
    }
}
