use crate::layout::Point;
use crate::plane::{Bounds, Plot, PlotEdge, PlotNode, Viewport};
use crate::{DataPoint, LineFigure, RenderOptions};

const COLORS: [u8; 8] = [39, 208, 46, 201, 226, 51, 196, 129];

#[derive(Clone, Copy, Default)]
struct Cell {
    ch: char,
    color: Option<usize>,
}

pub(crate) fn render_line(figure: &LineFigure, options: RenderOptions) -> anyhow::Result<String> {
    anyhow::ensure!(options.width >= 30, "line figure width must be at least 30");
    anyhow::ensure!(
        options.height >= 10,
        "line figure height must be at least 10"
    );
    anyhow::ensure!(
        options.x_min.is_none_or(f64::is_finite) && options.x_max.is_none_or(f64::is_finite),
        "x viewport bounds must be finite"
    );
    anyhow::ensure!(
        options.y_min.is_none_or(f64::is_finite) && options.y_max.is_none_or(f64::is_finite),
        "y viewport bounds must be finite"
    );
    if let (Some(min), Some(max)) = (options.y_min, options.y_max) {
        anyhow::ensure!(min < max, "y-min must be less than y-max");
    }
    if let (Some(min), Some(max)) = (options.x_min, options.x_max) {
        anyhow::ensure!(min < max, "x-min must be less than x-max");
    }

    let legend_rows = figure.series.len().div_ceil(3);
    let left = 10_usize;
    let right = options.width - 2;
    let top = 1_usize;
    let bottom = options.height - 3 - legend_rows;
    anyhow::ensure!(bottom > top + 2, "line figure is too short for its legend");

    let (data_min_x, data_max_x, _, _) = bounds(figure);
    let min_x = options.x_min.unwrap_or(data_min_x);
    let max_x = options.x_max.unwrap_or(data_max_x);
    let visible: Vec<Vec<DataPoint>> = figure
        .series
        .iter()
        .map(|series| points_in_view(&series.points, min_x, max_x))
        .collect();
    let (_, _, data_min_y, data_max_y) = bounds(figure);
    let min_y = options.y_min.unwrap_or(data_min_y);
    let max_y = options.y_max.unwrap_or(data_max_y);
    anyhow::ensure!(min_y < max_y, "the y viewport contains no range");
    let sample_count = right - left + 1;
    let sampled_visible: Vec<Vec<DataPoint>> = visible
        .iter()
        .map(|points| largest_triangle_three_buckets(points, sample_count))
        .collect();
    let mut dots = vec![vec![0_u8; options.width]; options.height];
    let mut owners = vec![vec![None; options.width]; options.height];

    let mut canvas = vec![
        vec![
            Cell {
                ch: ' ',
                color: None
            };
            options.width
        ];
        options.height
    ];
    draw_grid(&mut canvas, left, right, top, bottom);

    let mut plot_nodes = Vec::new();
    let mut plot_edges = Vec::new();
    for (series_index, series) in sampled_visible.iter().enumerate() {
        let start = plot_nodes.len();
        plot_nodes.extend(series.iter().map(|point| PlotNode {
            position: Point {
                x: point.x,
                y: point.y,
            },
            label: None,
            owner: Some(series_index),
        }));
        plot_edges.extend(
            (start..start + series.len().saturating_sub(1)).map(|from| PlotEdge {
                from,
                to: from + 1,
                owner: Some(series_index),
            }),
        );
    }
    let plot = Plot {
        nodes: plot_nodes,
        edges: plot_edges,
    };
    let view_bounds = Bounds {
        min_x,
        max_x,
        min_y,
        max_y,
    };
    let (plot_dots, plot_owners, _) = plot.pixels(
        Viewport::plot(view_bounds),
        right - left + 1,
        bottom - top + 1,
    );
    for (plot_y, row) in plot_dots.into_iter().enumerate() {
        for (plot_x, dots_value) in row.into_iter().enumerate() {
            dots[top + plot_y][left + plot_x] = dots_value;
            owners[top + plot_y][left + plot_x] = plot_owners[plot_y][plot_x];
        }
    }

    for y in top..=bottom {
        for x in left..=right {
            if dots[y][x] != 0 {
                canvas[y][x] = Cell {
                    ch: braille_char(dots[y][x]),
                    color: owners[y][x],
                };
            }
        }
    }

    for row in canvas.iter_mut().take(bottom + 1).skip(top) {
        row[left - 1].ch = '│';
    }
    for cell in canvas[bottom + 1].iter_mut().take(right + 1).skip(left - 1) {
        cell.ch = '─';
    }
    canvas[bottom + 1][left - 1].ch = '└';
    write_text(&mut canvas, 0, top, &format_number(max_y), None);
    write_text(&mut canvas, 0, bottom, &format_number(min_y), None);
    write_text(
        &mut canvas,
        left - 1,
        bottom + 2,
        &format_number(min_x),
        None,
    );
    let max_x_text = format_number(max_x);
    write_text(
        &mut canvas,
        right + 1 - max_x_text.chars().count(),
        bottom + 2,
        &max_x_text,
        None,
    );

    if let Some(label) = &figure.y_label {
        write_text(&mut canvas, 0, 0, label, None);
    }
    if let Some(label) = &figure.x_label {
        let start = left + (right - left).saturating_sub(label.chars().count()) / 2;
        write_text(&mut canvas, start, bottom + 2, label, None);
    }

    for (index, series) in figure.series.iter().enumerate() {
        let row = options.height - legend_rows + index / 3;
        let col = (index % 3) * (options.width / 3);
        write_text(
            &mut canvas,
            col,
            row,
            &format!("● {}", series.label),
            Some(index),
        );
    }

    Ok(canvas
        .into_iter()
        .map(|row| render_row(row, options.color))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_owned())
}

fn draw_grid(canvas: &mut [Vec<Cell>], left: usize, right: usize, top: usize, bottom: usize) {
    const TICKS: usize = 5;
    for tick in 0..TICKS {
        let x = left + tick * (right - left) / (TICKS - 1);
        let y = top + tick * (bottom - top) / (TICKS - 1);

        for row in canvas.iter_mut().take(bottom + 1).skip(top) {
            if row[x].ch == ' ' {
                row[x].ch = '┆';
            }
        }
        for cell in canvas[y].iter_mut().take(right + 1).skip(left) {
            cell.ch = if cell.ch == '┆' { '┼' } else { '┄' };
        }
    }
}

fn bounds(figure: &LineFigure) -> (f64, f64, f64, f64) {
    figure.series.iter().flat_map(|series| &series.points).fold(
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

fn points_in_view(points: &[DataPoint], min_x: f64, max_x: f64) -> Vec<DataPoint> {
    points
        .iter()
        .copied()
        .filter(|point| point.x >= min_x && point.x <= max_x)
        .collect()
}

/// Reduce a dense series to one point per drawable terminal column while
/// retaining endpoints and visually significant peaks.
fn largest_triangle_three_buckets(points: &[DataPoint], threshold: usize) -> Vec<DataPoint> {
    if threshold >= points.len() || threshold < 3 {
        return points.to_vec();
    }

    let mut sampled = Vec::with_capacity(threshold);
    sampled.push(points[0]);
    let bucket_width = (points.len() - 2) as f64 / (threshold - 2) as f64;
    let mut selected = 0;

    for bucket in 0..threshold - 2 {
        let avg_start = ((bucket + 1) as f64 * bucket_width).floor() as usize + 1;
        let avg_end = (((bucket + 2) as f64 * bucket_width).floor() as usize + 1).min(points.len());
        let average_slice =
            &points[avg_start.min(points.len() - 1)..avg_end.max(avg_start + 1).min(points.len())];
        let (avg_x, avg_y) = average_slice
            .iter()
            .fold((0.0, 0.0), |sum, point| (sum.0 + point.x, sum.1 + point.y));
        let average_len = average_slice.len() as f64;
        let (avg_x, avg_y) = (avg_x / average_len, avg_y / average_len);

        let range_start = (bucket as f64 * bucket_width).floor() as usize + 1;
        let range_end = (((bucket + 1) as f64 * bucket_width).floor() as usize + 1)
            .min(points.len() - 1)
            .max(range_start + 1);
        let anchor = points[selected];
        let mut max_area = -1.0;
        let mut next = range_start;
        for (index, point) in points[range_start..range_end].iter().enumerate() {
            let area = ((anchor.x - avg_x) * (point.y - anchor.y)
                - (anchor.x - point.x) * (avg_y - anchor.y))
                .abs();
            if area > max_area {
                max_area = area;
                next = range_start + index;
            }
        }
        sampled.push(points[next]);
        selected = next;
    }
    sampled.push(*points.last().expect("non-empty input"));
    sampled
}

fn braille_char(dots: u8) -> char {
    char::from_u32(0x2800 + u32::from(dots)).unwrap()
}

fn write_text(canvas: &mut [Vec<Cell>], x: usize, y: usize, text: &str, color: Option<usize>) {
    if let Some(row) = canvas.get_mut(y) {
        for (offset, ch) in text.chars().enumerate() {
            if let Some(cell) = row.get_mut(x + offset) {
                *cell = Cell { ch, color };
            }
        }
    }
}

fn render_row(row: Vec<Cell>, use_color: bool) -> String {
    let last = row.iter().rposition(|cell| cell.ch != ' ').unwrap_or(0);
    let mut output = String::new();
    let mut active = None;
    for cell in row.into_iter().take(last + 1) {
        if use_color && cell.color != active {
            if let Some(color) = cell.color {
                output.push_str(&format!("\x1b[38;5;{}m", COLORS[color % COLORS.len()]));
            } else if active.is_some() {
                output.push_str("\x1b[0m");
            }
            active = cell.color;
        }
        output.push(cell.ch);
    }
    if use_color && active.is_some() {
        output.push_str("\x1b[0m");
    }
    output
}

fn format_number(value: f64) -> String {
    if value == 0.0 {
        "0".into()
    } else if value.abs() >= 10_000.0 || value.abs() < 0.001 {
        format!("{value:.2e}")
    } else {
        format!("{value:.3}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LineSeries;

    #[test]
    fn renders_multiple_series_axes_and_legend() {
        let figure = LineFigure {
            series: vec![
                LineSeries {
                    label: "up".into(),
                    points: vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 2.0, y: 2.0 }],
                },
                LineSeries {
                    label: "down".into(),
                    points: vec![DataPoint { x: 0.0, y: 2.0 }, DataPoint { x: 2.0, y: 0.0 }],
                },
            ],
            x_label: Some("x".into()),
            y_label: Some("y".into()),
        };
        let output = render_line(
            &figure,
            RenderOptions {
                width: 50,
                height: 14,
                iterations: 0,
                color: false,
                x_min: None,
                x_max: None,
                y_min: None,
                y_max: None,
                trim_output: true,
            },
        )
        .unwrap();
        assert!(output.contains("● up"));
        assert!(output.contains("● down"));
        assert!(output.contains('└'));
        assert!(output.contains('┄'));
        assert!(output.contains('┆'));
        assert!(
            output
                .chars()
                .any(|ch| ('\u{2801}'..='\u{28ff}').contains(&ch))
        );
    }

    #[test]
    fn line_series_edges_do_not_cross_series_boundaries() {
        let first = vec![DataPoint { x: 0.0, y: 0.0 }, DataPoint { x: 1.0, y: 1.0 }];
        let second = vec![DataPoint { x: 0.0, y: 1.0 }, DataPoint { x: 1.0, y: 0.0 }];
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        for (owner, series) in [first, second].into_iter().enumerate() {
            let start = nodes.len();
            nodes.extend(series.into_iter().map(|point| PlotNode {
                position: Point {
                    x: point.x,
                    y: point.y,
                },
                label: None,
                owner: Some(owner),
            }));
            edges.extend((start..start + 1).map(|from| PlotEdge {
                from,
                to: from + 1,
                owner: Some(owner),
            }));
        }
        assert_eq!((edges[0].from, edges[0].to), (0, 1));
        assert_eq!((edges[1].from, edges[1].to), (2, 3));
    }

    #[test]
    fn dense_sampling_respects_the_plot_width() {
        let points: Vec<DataPoint> = (0..1_000)
            .map(|index| DataPoint {
                x: f64::from(index),
                y: (f64::from(index) / 20.0).sin(),
            })
            .collect();
        let sampled = largest_triangle_three_buckets(&points, 69);
        assert_eq!(sampled.len(), 69);
        assert_eq!(sampled.first().unwrap().x, 0.0);
        assert_eq!(sampled.last().unwrap().x, 999.0);
    }

    #[test]
    fn viewport_clips_points_before_sampling() {
        let points: Vec<DataPoint> = (0..10)
            .map(|index| DataPoint {
                x: f64::from(index),
                y: f64::from(index),
            })
            .collect();
        let visible = points_in_view(&points, 2.0, 4.0);
        assert_eq!(visible.len(), 3);
        assert_eq!(visible.first().unwrap().x, 2.0);
        assert_eq!(visible.last().unwrap().x, 4.0);
    }
}
