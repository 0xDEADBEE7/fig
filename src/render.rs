use std::collections::HashMap;

use crate::{
    Figure, Graph,
    layout::{Point, force_directed},
};

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub width: usize,
    pub height: usize,
    pub iterations: usize,
    /// Emit ANSI colors for figure types that support them.
    pub color: bool,
    /// Optional x-axis viewport bounds for line figures.
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    /// Optional y-axis viewport bounds for line figures.
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            iterations: 300,
            color: true,
            x_min: None,
            x_max: None,
            y_min: None,
            y_max: None,
        }
    }
}

pub fn render(figure: &Figure, options: RenderOptions) -> anyhow::Result<String> {
    figure.validate()?;
    match figure {
        Figure::Graph(graph) => render_graph(graph, options),
        Figure::Line(line) => crate::line::render_line(line, options),
    }
}

fn render_graph(graph: &Graph, options: RenderOptions) -> anyhow::Result<String> {
    graph.validate()?;
    anyhow::ensure!(options.width >= 10, "width must be at least 10");
    anyhow::ensure!(options.height >= 5, "height must be at least 5");

    let positions = fit(
        force_directed(graph, options.iterations),
        options.width,
        options.height,
    );
    let indexes: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();
    // Like Clin's default Ratatui canvas, keep edge geometry as Braille dots.
    // Each terminal cell becomes a 2x4 pixel grid instead of one slash.
    let mut dots = vec![vec![0_u8; options.width]; options.height];

    for edge in &graph.edges {
        let from = positions[indexes[edge.from.as_str()]];
        let to = positions[indexes[edge.to.as_str()]];
        draw_braille_line(&mut dots, from, to);
    }
    let mut canvas: Vec<Vec<char>> = dots
        .into_iter()
        .map(|row| row.into_iter().map(braille_char).collect())
        .collect();
    for (node, position) in graph.nodes.iter().zip(&positions) {
        draw_label(&mut canvas, *position, node.display_label());
    }

    let lines: Vec<String> = canvas
        .into_iter()
        .map(|row| {
            row.into_iter()
                .collect::<String>()
                .trim_end_matches([' ', '\u{2800}'])
                .to_owned()
        })
        .collect();
    let first = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
    let last = lines
        .iter()
        .rposition(|line| !line.is_empty())
        .unwrap_or(first);
    Ok(lines[first..=last].join("\n"))
}

fn fit(points: Vec<Point>, width: usize, height: usize) -> Vec<Point> {
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let usable_w = width.saturating_sub(12).max(1) as f64;
    let usable_h = height.saturating_sub(3).max(1) as f64;
    points
        .into_iter()
        .map(|p| Point {
            x: 5.0 + (p.x - min_x) / (max_x - min_x).max(0.001) * usable_w,
            y: 1.0 + (p.y - min_y) / (max_y - min_y).max(0.001) * usable_h,
        })
        .collect()
}

fn draw_braille_line(canvas: &mut [Vec<u8>], from: Point, to: Point) {
    let (mut x, mut y) = (
        (from.x * 2.0).round() as isize,
        (from.y * 4.0).round() as isize,
    );
    let (x2, y2) = ((to.x * 2.0).round() as isize, (to.y * 4.0).round() as isize);
    let (dx, dy) = ((x2 - x).abs(), -(y2 - y).abs());
    let (sx, sy) = (if x < x2 { 1 } else { -1 }, if y < y2 { 1 } else { -1 });
    let mut error = dx + dy;
    loop {
        put_braille_dot(canvas, x, y);
        if x == x2 && y == y2 {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
}

fn put_braille_dot(canvas: &mut [Vec<u8>], x: isize, y: isize) {
    if x < 0 || y < 0 {
        return;
    }
    let cell_x = x as usize / 2;
    let cell_y = y as usize / 4;
    if let Some(cell) = canvas.get_mut(cell_y).and_then(|row| row.get_mut(cell_x)) {
        const DOTS: [[u8; 2]; 4] = [
            [0b0000_0001, 0b0000_1000],
            [0b0000_0010, 0b0001_0000],
            [0b0000_0100, 0b0010_0000],
            [0b0100_0000, 0b1000_0000],
        ];
        *cell |= DOTS[y as usize % 4][x as usize % 2];
    }
}

fn braille_char(dots: u8) -> char {
    char::from_u32(0x2800 + u32::from(dots)).expect("Braille code points are valid")
}

fn draw_label(canvas: &mut [Vec<char>], position: Point, label: &str) {
    let clean: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let text = format!("[{clean}]");
    let width = canvas.first().map_or(0, Vec::len);
    let start = (position.x.round() as isize - text.chars().count() as isize / 2)
        .clamp(0, width.saturating_sub(text.chars().count()) as isize) as usize;
    let y = (position.y.round() as usize).min(canvas.len() - 1);
    for (offset, ch) in text.chars().take(width).enumerate() {
        canvas[y][start + offset] = ch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node};

    #[test]
    fn renders_nodes_and_an_edge() {
        let graph = Graph {
            nodes: vec![
                Node {
                    id: "a".into(),
                    label: Some("Alpha".into()),
                },
                Node {
                    id: "b".into(),
                    label: None,
                },
            ],
            edges: vec![Edge {
                from: "a".into(),
                to: "b".into(),
            }],
        };
        let output = render(
            &Figure::Graph(graph),
            RenderOptions {
                width: 40,
                height: 10,
                iterations: 20,
                color: false,
                x_min: None,
                x_max: None,
                y_min: None,
                y_max: None,
            },
        )
        .unwrap();
        assert!(output.contains("[Alpha]"));
        assert!(output.contains("[b]"));
        assert!(
            output
                .chars()
                .any(|character| ('\u{2801}'..='\u{28ff}').contains(&character))
        );
    }

    #[test]
    fn braille_dots_map_to_unicode_code_points() {
        assert_eq!(braille_char(0), '\u{2800}');
        assert_eq!(braille_char(1), '⠁');
        assert_eq!(braille_char(0xff), '⣿');
    }
}
