use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Line {
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
    pub series: Vec<Series>,
}

#[derive(Debug, Deserialize)]
pub struct Series {
    pub label: String,
    pub points: Vec<Point>,
    #[serde(default)]
    pub fig: Value,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Line {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.series.is_empty(),
            "a line chart needs at least one series"
        );
        for series in &self.series {
            anyhow::ensure!(
                series.points.len() >= 2,
                "line series need at least two points"
            );
            anyhow::ensure!(
                series
                    .points
                    .windows(2)
                    .all(|points| points[0].x <= points[1].x),
                "line series x values must be ordered"
            );
        }
        Ok(())
    }

    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let points = self.series.iter().flat_map(|series| &series.points);
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        );
        for point in points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }
        (min_x, max_x, min_y, max_y)
    }
}
