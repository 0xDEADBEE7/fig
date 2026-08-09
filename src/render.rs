use std::collections::HashMap;

use crate::{
    Figure, Graph,
    layout::{Point, force_directed},
    plane::{Bounds, Pixel, Plot, PlotEdge, PlotNode, Viewport},
};

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub width: usize,
    pub height: usize,
    pub iterations: usize,
    /// Emit ANSI colors for figure types that support them.
    pub color: bool,
    /// Optional x-axis viewport bounds.
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    /// Optional y-axis viewport bounds.
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

pub fn figure_bounds(figure: &Figure, iterations: usize) -> (f64, f64, f64, f64) {
    match figure {
        Figure::Graph(graph) => bounds(&force_directed(graph, iterations)),
        Figure::Line(line) => line.series.iter().flat_map(|series| &series.points).fold(
            (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ),
            |(min_x, max_x, min_y, max_y), point| {
                (
                    min_x.min(point.x),
                    max_x.max(point.x),
                    min_y.min(point.y),
                    max_y.max(point.y),
                )
            },
        ),
    }
}

fn bounds(points: &[Point]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), point| {
            (
                min_x.min(point.x),
                max_x.max(point.x),
                min_y.min(point.y),
                max_y.max(point.y),
            )
        },
    )
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
    validate_viewport(&options)?;

    let points = force_directed(graph, options.iterations);
    let indexes: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();
    let plot = Plot {
        nodes: graph
            .nodes
            .iter()
            .zip(points.iter())
            .map(|(node, position)| PlotNode {
                position: *position,
                label: Some(node.display_label().to_owned()),
                owner: None,
            })
            .collect(),
        edges: graph
            .edges
            .iter()
            .map(|edge| PlotEdge {
                from: indexes[edge.from.as_str()],
                to: indexes[edge.to.as_str()],
                owner: None,
            })
            .collect(),
    };
    let data_bounds = plot.bounds();
    let padded_bounds = data_bounds.padded();
    let view_bounds = Bounds {
        min_x: options.x_min.unwrap_or(padded_bounds.min_x),
        max_x: options.x_max.unwrap_or(padded_bounds.max_x),
        min_y: options.y_min.unwrap_or(padded_bounds.min_y),
        max_y: options.y_max.unwrap_or(padded_bounds.max_y),
    };
    anyhow::ensure!(
        view_bounds.min_x < view_bounds.max_x && view_bounds.min_y < view_bounds.max_y,
        "graph viewport contains no range"
    );
    let (dots, _owners, projected_nodes) = plot.pixels(
        Viewport::with_aspect(view_bounds),
        options.width,
        options.height,
    );
    let mut canvas: Vec<Vec<char>> = dots
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|dots| if dots == 0 { ' ' } else { braille_char(dots) })
                .collect()
        })
        .collect();
    for (position, label) in projected_nodes {
        draw_label(&mut canvas, position, label.unwrap_or_default());
    }
    let lines: Vec<String> = canvas
        .into_iter()
        .map(|row| row.into_iter().collect::<String>().trim_end().to_owned())
        .collect();
    let first = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
    let last = lines
        .iter()
        .rposition(|line| !line.is_empty())
        .unwrap_or(first);
    Ok(lines[first..=last].join("\n"))
}

fn validate_viewport(options: &RenderOptions) -> anyhow::Result<()> {
    anyhow::ensure!(
        options.x_min.is_none_or(f64::is_finite)
            && options.x_max.is_none_or(f64::is_finite)
            && options.y_min.is_none_or(f64::is_finite)
            && options.y_max.is_none_or(f64::is_finite),
        "viewport bounds must be finite"
    );
    for (min, max) in [
        (options.x_min, options.x_max),
        (options.y_min, options.y_max),
    ] {
        if let (Some(min), Some(max)) = (min, max) {
            anyhow::ensure!(min < max, "viewport minimum must be less than maximum");
        }
    }
    Ok(())
}

fn braille_char(dots: u8) -> char {
    char::from_u32(0x2800 + u32::from(dots)).expect("Braille code points are valid")
}

fn draw_label(canvas: &mut [Vec<char>], position: Pixel, label: &str) {
    let clean: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let text = format!("[{clean}]");
    let width = canvas.first().map_or(0, Vec::len);
    let y = position.y.round();
    if position.x < 0.0 || position.x >= width as f64 || y < 0.0 || y >= canvas.len() as f64 {
        return;
    }
    let start = (position.x.round() as isize - text.chars().count() as isize / 2)
        .clamp(0, width.saturating_sub(text.chars().count()) as isize) as usize;
    let y = y as usize;
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

    #[test]
    fn graph_label_is_skipped_when_rounded_row_is_outside_canvas() {
        let mut canvas = vec![vec![' '; 80]; 19];

        draw_label(&mut canvas, Pixel { x: 40.0, y: 18.6 }, "Worker");

        assert!(canvas.iter().flatten().all(|cell| *cell == ' '));
    }
}
