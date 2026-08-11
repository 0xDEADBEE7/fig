use crate::models::Line;

pub fn information(line: &Line, focus: Option<usize>, width: usize, height: usize) -> Vec<String> {
    let text = focus
        .and_then(|index| line.series.get(index))
        .map(|series| series.label.as_str())
        .unwrap_or("No line selected. Press H or L to select one.");
    (0..height)
        .map(|row| {
            (row == height / 2)
                .then(|| format!("{text:^width$}"))
                .unwrap_or_default()
        })
        .collect()
}
