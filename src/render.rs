use std::collections::HashMap;

use crate::{
    Figure, Graph, HistogramFigure,
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
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    /// Optional y-axis viewport bounds.
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
    /// Selected histogram bucket, rendered in the data table and below the plot.
    pub selected_index: Option<usize>,
    /// Keep the full raster canvas instead of removing empty outer rows.
    pub trim_output: bool,
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
            selected_index: None,
            trim_output: true,
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
        Figure::Histogram(histogram) => histogram_bounds(histogram),
    }
}

fn histogram_bounds(figure: &HistogramFigure) -> (f64, f64, f64, f64) {
    let max_y = figure
        .buckets
        .iter()
        .map(|bucket| bucket.values.values().sum::<f64>())
        .fold(0.0, f64::max);
    (
        0.0,
        (figure.buckets.len().saturating_sub(1)) as f64,
        0.0,
        max_y,
    )
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
        Figure::Histogram(histogram) => crate::histogram::render_histogram(histogram, options),
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
                owner: Some(0),
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
    let (dots, mut owners, projected_nodes) = plot.pixels(
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
        draw_label(
            &mut canvas,
            &mut owners,
            position,
            label.unwrap_or_default(),
        );
    }
    let lines: Vec<String> = canvas
        .into_iter()
        .enumerate()
        .map(|(y, row)| {
            let mut line = String::new();
            let mut red = false;
            for (x, ch) in row.into_iter().enumerate() {
                let is_red = owners[y][x].is_some() && ch != ' ';
                if options.color && is_red != red {
                    line.push_str(if is_red { "\x1b[38;5;196m" } else { "\x1b[0m" });
                    red = is_red;
                }
                line.push(ch);
            }
            if options.color && red {
                line.push_str("\x1b[0m");
            }
            line.trim_end().to_owned()
        })
        .collect();
    if !options.trim_output {
        return Ok(lines.join("\n"));
    }
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

fn draw_label(
    canvas: &mut [Vec<char>],
    owners: &mut [Vec<Option<usize>>],
    position: Pixel,
    label: &str,
) {
    let clean: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let text = format!("[{clean}]");
    let width = canvas.first().map_or(0, Vec::len);
    let y = position.y.round() as isize + 1;
    if position.x < 0.0 || position.x >= width as f64 || y < 0 || y >= canvas.len() as isize {
        return;
    }
    let start = (position.x.round() as isize - text.chars().count() as isize / 2)
        .clamp(0, width.saturating_sub(text.chars().count()) as isize) as usize;
    let y = y as usize;
    for (offset, ch) in text.chars().take(width).enumerate() {
        canvas[y][start + offset] = ch;
        owners[y][start + offset] = None;
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
                selected_index: None,
                trim_output: true,
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
    fn preserves_empty_rows_for_interactive_viewport() {
        let graph = Graph {
            nodes: vec![Node {
                id: "center".into(),
                label: None,
            }],
            edges: vec![],
        };
        let output = render(
            &Figure::Graph(graph),
            RenderOptions {
                width: 30,
                height: 11,
                iterations: 0,
                color: false,
                x_min: None,
                x_max: None,
                y_min: Some(-10.0),
                y_max: Some(10.0),
                selected_index: None,
                trim_output: false,
            },
        )
        .unwrap();

        assert_eq!(output.split('\n').count(), 11);
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

        draw_label(
            &mut canvas,
            &mut vec![vec![None; 80]; 19],
            Pixel { x: 40.0, y: 17.6 },
            "Worker",
        );

        assert!(canvas.iter().flatten().all(|cell| *cell == ' '));
    }
}
