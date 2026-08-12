use crate::figure::{Plane, plane};
use crate::models::{Edge, Graph};

const NODE_DOTS: u8 = 0b0001_1011;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tone {
    Normal,
    Dim,
    Red,
    Green,
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
    graph: &Graph,
    points: &[(f64, f64)],
    width: usize,
    height: usize,
    focus: Option<usize>,
    plane: Plane,
    labels: bool,
) -> Vec<String> {
    let inner_width = width.saturating_sub(2);
    let inner_height = height.saturating_sub(2);
    let mut canvas = plane::background(plane, inner_width, inner_height)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|character| Cell {
                    text: (character != ' ').then_some(character),
                    ..Cell::default()
                })
                .collect()
        })
        .collect::<Vec<Vec<Cell>>>();
    let projected = project(points, plane, inner_width, inner_height);
    for edge in &graph.edges {
        draw_edge(graph, edge, &projected, &mut canvas, focus);
    }
    for (index, point) in projected.iter().enumerate() {
        draw_node(graph, index, *point, &mut canvas, focus, labels);
    }
    frame(canvas, width)
}

fn project(
    points: &[(f64, f64)],
    plane: Plane,
    width: usize,
    height: usize,
) -> Vec<(isize, isize)> {
    points
        .iter()
        .map(|&(x, y)| plane.project_unclipped(x, y, width * 2, height * 4))
        .collect()
}

fn draw_edge(
    graph: &Graph,
    edge: &Edge,
    points: &[(isize, isize)],
    canvas: &mut [Vec<Cell>],
    focus: Option<usize>,
) {
    let from_index = index_of(graph, &edge.from);
    let to_index = index_of(graph, &edge.to);
    let (from, to) = (points[from_index], points[to_index]);
    let tone = if focus.is_some_and(|selected| selected != from_index && selected != to_index) {
        Tone::Dim
    } else {
        Tone::Normal
    };
    raster_line(canvas, from, to, tone);
}

fn draw_node(
    graph: &Graph,
    index: usize,
    point: (isize, isize),
    canvas: &mut [Vec<Cell>],
    focus: Option<usize>,
    labels: bool,
) {
    let (x, y) = point;
    let visible = focus.is_none_or(|selected| graph.connected(selected, index));
    let tone = if focus == Some(index) {
        Tone::Green
    } else if visible {
        Tone::Red
    } else {
        Tone::Dim
    };
    put_node(canvas, x, y, tone);
    if visible && labels {
        let offset = if tone == Tone::Green { 3 } else { 1 };
        let label_tone = if tone == Tone::Green {
            Tone::Green
        } else {
            Tone::Normal
        };
        write_label(
            canvas,
            x / 2 + offset,
            y / 4,
            graph.node_label(index),
            label_tone,
        );
    }
}

fn raster_line(canvas: &mut [Vec<Cell>], from: (isize, isize), to: (isize, isize), tone: Tone) {
    let Some((from, to)) = clip(from, to, canvas[0].len() * 2 - 1, canvas.len() * 4 - 1) else {
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
    max_x: usize,
    max_y: usize,
) -> Option<((isize, isize), (isize, isize))> {
    let (x, y) = (from.0 as f64, from.1 as f64);
    let (dx, dy) = (to.0 as f64 - x, to.1 as f64 - y);
    let (mut start, mut end) = (0.0_f64, 1.0_f64);
    for (p, q) in [
        (-dx, x),
        (dx, max_x as f64 - x),
        (-dy, y),
        (dy, max_y as f64 - y),
    ] {
        if p == 0.0 && q < 0.0 {
            return None;
        }
        if p < 0.0 {
            start = start.max(q / p);
        }
        if p > 0.0 {
            end = end.min(q / p);
        }
    }
    (start <= end)
        .then(|| {
            (
                (x + dx * start).round() as isize,
                (y + dy * start).round() as isize,
                (x + dx * end).round() as isize,
                (y + dy * end).round() as isize,
            )
        })
        .map(|(x0, y0, x1, y1)| ((x0, y0), (x1, y1)))
}

fn put_node(canvas: &mut [Vec<Cell>], x: isize, y: isize, tone: Tone) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize / 2, y as usize / 4);
    put_node_cell(canvas, x, y, tone);
    if tone == Tone::Green {
        put_node_cell(canvas, x + 1, y, tone);
    }
}

fn put_node_cell(canvas: &mut [Vec<Cell>], x: usize, y: usize, tone: Tone) {
    let Some(row) = canvas.get_mut(y) else { return };
    let Some(cell) = row.get_mut(x) else { return };
    let existing_dots = cell.dots;
    cell.dots |= if tone == Tone::Green {
        u8::MAX
    } else {
        NODE_DOTS
    };
    if tone != Tone::Dim || existing_dots == 0 {
        cell.tone = tone;
    }
}

fn put_dot(canvas: &mut [Vec<Cell>], x: isize, y: isize, tone: Tone) {
    if x < 0 || y < 0 || y as usize >= canvas.len() * 4 || x as usize >= canvas[0].len() * 2 {
        return;
    }
    let cell = &mut canvas[y as usize / 4][x as usize / 2];
    if tone == Tone::Dim && cell.dots == 0 {
        cell.tone = Tone::Dim;
    }
    cell.dots |= dot(x as usize % 2, y as usize % 4);
    if tone != Tone::Dim {
        cell.tone = tone;
    }
}

fn dot(x: usize, y: usize) -> u8 {
    [[1, 2, 4, 64], [8, 16, 32, 128]][x][y]
}

fn write_label(canvas: &mut [Vec<Cell>], x: isize, y: isize, label: &str, tone: Tone) {
    if x < 0 || y < 0 || y as usize >= canvas.len() {
        return;
    }
    for (offset, character) in label.chars().enumerate() {
        let Some(cell) = canvas[y as usize].get_mut(x as usize + offset) else {
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
        push_colored(&mut line, character, cell.tone);
        line
    })
}

fn braille(dots: u8) -> char {
    char::from_u32(0x2800 + u32::from(dots)).unwrap()
}

fn push_colored(line: &mut String, character: char, tone: Tone) {
    match tone {
        Tone::Normal => line.push(character),
        Tone::Dim => line.push_str(&format!("\x1b[38;5;240m{character}\x1b[0m")),
        Tone::Red => line.push_str(&format!("\x1b[38;5;196m{character}\x1b[0m")),
        Tone::Green => line.push_str(&format!("\x1b[38;5;40m{character}\x1b[0m")),
    }
}

fn index_of(graph: &Graph, id: &str) -> usize {
    graph.nodes.iter().position(|node| node.id == id).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimmed_node_cannot_override_focused_node_colour() {
        let mut canvas = vec![vec![Cell::default()]];
        put_node_cell(&mut canvas, 0, 0, Tone::Green);
        put_node_cell(&mut canvas, 0, 0, Tone::Dim);

        assert_eq!(canvas[0][0].tone, Tone::Green);
    }
}
