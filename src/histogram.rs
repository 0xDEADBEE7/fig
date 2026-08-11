use crate::{HistogramFigure, RenderOptions};

const COLORS: [u8; 8] = [39, 208, 46, 201, 226, 51, 196, 129];
const LEFT: usize = 10;
const MIN_BAR_STEP: usize = 2;

#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    color: Option<usize>,
}

pub(crate) fn render_histogram(
    figure: &HistogramFigure,
    options: RenderOptions,
) -> anyhow::Result<String> {
    anyhow::ensure!(options.width >= 30, "histogram width must be at least 30");
    anyhow::ensure!(options.height >= 14, "histogram height must be at least 14");
    validate_bounds(options.x_min, options.x_max, "x")?;
    validate_bounds(options.y_min, options.y_max, "y")?;

    let right = options.width - 2;
    let plot_width = right - LEFT + 1;
    let selected = options
        .selected_index
        .unwrap_or(0)
        .min(figure.buckets.len() - 1);
    let legend_rows = legend_row_count(figure, options.width);
    let table_rows = table_row_count(figure, selected, options.width);
    anyhow::ensure!(
        options.height > table_rows + legend_rows + 7,
        "histogram is too short for its legend and data table"
    );
    let top = 1 + legend_rows;
    let table_top = options.height - table_rows;
    let label_row = table_top - 1;
    let indicator_row = label_row - 1;
    let axis_row = indicator_row - 1;
    let bottom = axis_row - 1;
    debug_assert!(bottom > top + 2);

    let max_value = figure
        .buckets
        .iter()
        .map(|bucket| bucket.values.values().sum::<f64>())
        .fold(0.0, f64::max);
    let min_y = options.y_min.unwrap_or(0.0);
    let max_y = options.y_max.unwrap_or(max_value.max(1.0));
    anyhow::ensure!(min_y < max_y, "the y viewport contains no range");

    let mut canvas = vec![
        vec![
            Cell {
                ch: ' ',
                color: None,
            };
            options.width
        ];
        options.height
    ];
    draw_grid(&mut canvas, LEFT, right, top, bottom);

    let positions = bar_positions(figure.buckets.len(), selected, LEFT, right);
    for (index, x) in positions.iter().copied() {
        let bucket = &figure.buckets[index];
        let mut total = 0.0;
        for (series_index, series) in figure.series.iter().enumerate() {
            let value = bucket.values.get(&series.label).copied().unwrap_or(0.0);
            let next = total + value;
            draw_segment(
                &mut canvas,
                x,
                total,
                next,
                min_y,
                max_y,
                top,
                bottom,
                series_index,
            );
            total = next;
        }
    }

    for row in canvas.iter_mut().take(bottom + 1).skip(top) {
        row[LEFT - 1].ch = '│';
    }
    for cell in canvas[axis_row].iter_mut().take(right + 1).skip(LEFT - 1) {
        cell.ch = '─';
    }
    canvas[axis_row][LEFT - 1].ch = '└';
    write_text(&mut canvas, 0, top, &format_number(max_y), None);
    write_text(&mut canvas, 0, bottom, &format_number(min_y), None);

    if let Some(label) = &figure.y_label {
        write_text(&mut canvas, 0, 0, label, None);
    }
    if let Some(label) = &figure.x_label {
        let start = LEFT + plot_width.saturating_sub(label.chars().count()) / 2;
        write_text(&mut canvas, start, label_row, label, None);
    }

    if let Some((_, selected_x)) = positions.iter().find(|(index, _)| *index == selected) {
        canvas[indicator_row][*selected_x].ch = '^';
    }
    draw_legend(&mut canvas, figure);
    draw_data_table(&mut canvas, figure, selected, table_top);

    let lines = canvas
        .into_iter()
        .map(|row| render_row(row, options.color))
        .collect::<Vec<_>>();
    let output = lines.join("\n");
    if options.trim_output {
        Ok(output.trim_end().to_owned())
    } else {
        Ok(output)
    }
}

/// Spread bars across the plot when they fit. Once they would touch, keep one
/// empty column between them and scroll the visible window to the selection.
fn bar_positions(
    bucket_count: usize,
    selected: usize,
    left: usize,
    right: usize,
) -> Vec<(usize, usize)> {
    let width = right - left + 1;
    if bucket_count * MIN_BAR_STEP <= width {
        let spacing = width as f64 / bucket_count as f64;
        return (0..bucket_count)
            .map(|index| {
                let x = left as f64 + (index as f64 + 0.5) * spacing;
                (index, x.floor().min(right as f64) as usize)
            })
            .collect();
    }

    let capacity = width.div_ceil(MIN_BAR_STEP).max(1);
    let max_start = bucket_count.saturating_sub(capacity);
    let start = selected
        .saturating_sub(capacity.saturating_sub(1))
        .min(max_start);
    (start..(start + capacity).min(bucket_count))
        .map(|index| (index, left + (index - start) * MIN_BAR_STEP))
        .filter(|(_, x)| *x <= right)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn draw_segment(
    canvas: &mut [Vec<Cell>],
    x: usize,
    start: f64,
    end: f64,
    min_y: f64,
    max_y: f64,
    top: usize,
    bottom: usize,
    series_index: usize,
) {
    if end <= min_y || start >= max_y || end <= start {
        return;
    }
    let visible_start = start.max(min_y);
    let visible_end = end.min(max_y);
    let start_row = project_y(visible_start, min_y, max_y, top, bottom);
    let end_row = project_y(visible_end, min_y, max_y, top, bottom);
    for row in canvas.iter_mut().take(start_row + 1).skip(end_row) {
        row[x] = Cell {
            ch: '█',
            color: Some(series_index),
        };
    }
}

fn legend_row_count(figure: &HistogramFigure, width: usize) -> usize {
    if figure.series.len() < 2 {
        return 0;
    }
    packed_rows(
        figure
            .series
            .iter()
            .map(|series| series.label.chars().count() + 2),
        width.saturating_sub(1),
    )
}

fn draw_legend(canvas: &mut [Vec<Cell>], figure: &HistogramFigure) {
    if figure.series.len() < 2 {
        return;
    }
    let width = canvas[0].len();
    let title = "Legend";
    write_text(canvas, width.saturating_sub(title.len()), 0, title, None);
    let mut row = 1;
    let mut col = 0;
    for (index, series) in figure.series.iter().enumerate() {
        let text = format!("■ {}", series.label);
        let text_width = text.chars().count();
        if col > 0 && col + text_width > width {
            row += 1;
            col = 0;
        }
        write_text(canvas, col, row, &text, Some(index));
        col += text_width + 2;
    }
}

fn table_row_count(figure: &HistogramFigure, selected: usize, width: usize) -> usize {
    let bucket = &figure.buckets[selected];
    let entry_rows = packed_rows(
        figure.series.iter().map(|series| {
            let value = bucket.values.get(&series.label).copied().unwrap_or(0.0);
            series.label.chars().count() + format_number(value).chars().count() + 4
        }),
        width.saturating_sub(4),
    );
    // Border, total row, series rows, border.
    entry_rows + 3
}

fn draw_data_table(
    canvas: &mut [Vec<Cell>],
    figure: &HistogramFigure,
    selected: usize,
    top: usize,
) {
    let bucket = &figure.buckets[selected];
    let width = canvas[0].len();
    let right = width - 1;
    let bottom = canvas.len() - 1;
    for (x, cell) in canvas[top].iter_mut().enumerate() {
        cell.ch = if x == 0 {
            '┌'
        } else if x == right {
            '┐'
        } else {
            '─'
        };
    }
    for (x, cell) in canvas[bottom].iter_mut().enumerate() {
        cell.ch = if x == 0 {
            '└'
        } else if x == right {
            '┘'
        } else {
            '─'
        };
    }
    for row in canvas.iter_mut().take(bottom).skip(top + 1) {
        row[0].ch = '│';
        row[right].ch = '│';
    }

    let title = truncate(&format!(" DATA TABLE — {} ", bucket.label), width - 4);
    write_text(canvas, 2, top, &title, None);
    let total = bucket.values.values().sum::<f64>();
    write_text(
        canvas,
        2,
        top + 1,
        &format!("Total: {}", format_number(total)),
        None,
    );

    let inner_width = width - 4;
    let mut row = top + 2;
    let mut col = 2;
    for (index, series) in figure.series.iter().enumerate() {
        let value = bucket.values.get(&series.label).copied().unwrap_or(0.0);
        let text = truncate(
            &format!("■ {}: {}", series.label, format_number(value)),
            inner_width,
        );
        let text_width = text.chars().count();
        if col > 2 && col - 2 + text_width > inner_width {
            row += 1;
            col = 2;
        }
        write_text(canvas, col, row, &text, Some(index));
        col += text_width + 2;
    }
}

fn packed_rows(widths: impl IntoIterator<Item = usize>, available_width: usize) -> usize {
    let mut rows = 1;
    let mut used = 0;
    for width in widths {
        let width = width.min(available_width);
        if used > 0 && used + width > available_width {
            rows += 1;
            used = 0;
        }
        used += width + 2;
    }
    rows
}

fn truncate(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn project_y(value: f64, min: f64, max: f64, top: usize, bottom: usize) -> usize {
    let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);
    bottom - (ratio * (bottom - top) as f64).round() as usize
}

fn validate_bounds(min: Option<f64>, max: Option<f64>, axis: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        min.is_none_or(f64::is_finite) && max.is_none_or(f64::is_finite),
        "{axis} viewport bounds must be finite"
    );
    if let (Some(min), Some(max)) = (min, max) {
        anyhow::ensure!(min < max, "{axis}-min must be less than {axis}-max");
    }
    Ok(())
}

fn draw_grid(canvas: &mut [Vec<Cell>], left: usize, right: usize, top: usize, bottom: usize) {
    for tick in 0..5 {
        let x = left + tick * (right - left) / 4;
        let y = top + tick * (bottom - top) / 4;
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
    use std::collections::HashMap;

    use super::*;
    use crate::{HistogramBucket, HistogramSeries};

    fn figure(bucket_count: usize) -> HistogramFigure {
        HistogramFigure {
            series: vec![
                HistogramSeries {
                    label: "success".into(),
                },
                HistogramSeries {
                    label: "failure".into(),
                },
            ],
            buckets: (0..bucket_count)
                .map(|index| HistogramBucket {
                    label: format!("bucket {index}"),
                    values: HashMap::from([
                        ("success".into(), (index + 1) as f64),
                        ("failure".into(), 2.0),
                    ]),
                })
                .collect(),
            x_label: Some("Latency".into()),
            y_label: Some("Count".into()),
        }
    }

    #[test]
    fn renders_legend_selection_indicator_and_complete_data_table() {
        let output = render_histogram(
            &figure(3),
            RenderOptions {
                width: 60,
                height: 20,
                color: false,
                selected_index: Some(1),
                ..RenderOptions::default()
            },
        )
        .unwrap();

        assert!(output.contains("Legend"));
        assert!(output.contains("■ success"));
        assert!(output.contains("■ failure"));
        assert!(output.contains('^'));
        assert!(output.contains("DATA TABLE — bucket 1"));
        assert!(output.contains("success: 2"));
        assert!(output.contains("failure: 2"));
        assert!(output.contains("Total: 4"));
    }

    #[test]
    fn overflowing_bars_keep_a_blank_column_and_scroll_to_selection() {
        let positions = bar_positions(40, 39, LEFT, 38);
        assert_eq!(positions.last().map(|(index, _)| *index), Some(39));
        assert!(
            positions
                .windows(2)
                .all(|pair| pair[1].1 - pair[0].1 >= MIN_BAR_STEP)
        );
    }

    #[test]
    fn evenly_spaced_bars_use_the_available_width() {
        let positions = bar_positions(3, 0, LEFT, 38);
        let gaps: Vec<usize> = positions
            .windows(2)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(gaps.iter().all(|gap| *gap >= MIN_BAR_STEP));
        assert!(gaps.iter().max().unwrap() - gaps.iter().min().unwrap() <= 1);
    }

    #[test]
    fn crowded_metadata_reports_a_size_error_instead_of_overflowing() {
        let mut figure = figure(1);
        figure.series = (0..20)
            .map(|index| HistogramSeries {
                label: format!("series {index}"),
            })
            .collect();
        let error = render_histogram(
            &figure,
            RenderOptions {
                width: 30,
                height: 14,
                color: false,
                ..RenderOptions::default()
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("too short"));
    }
}
