mod info;
mod render;

use crate::figure::{Plane, Visualization};
use crate::models::{Line, Point};

pub struct LineView<'a> {
    line: &'a Line,
    bounds: (f64, f64, f64, f64),
}

impl<'a> LineView<'a> {
    pub fn new(line: &'a Line) -> Self {
        Self {
            line,
            bounds: line.bounds(),
        }
    }

    fn project(&self, point: Point, plane: Plane, width: usize, height: usize) -> (isize, isize) {
        let (min_x, max_x, min_y, max_y) = self.bounds;
        let x_span = (max_x - min_x).max(f64::EPSILON);
        let y_span = (max_y - min_y).max(f64::EPSILON);
        let x = (point.x - min_x) / x_span * 2.0 - 1.0;
        let y = (point.y - min_y) / y_span * 2.0 - 1.0;
        plane.project_unclipped(x, y, width * 2, height * 4)
    }
}

impl Visualization for LineView<'_> {
    fn default_plane(&self) -> Plane {
        Plane::new(true, true)
    }

    fn draw(
        &self,
        width: usize,
        height: usize,
        focus: Option<usize>,
        plane: Plane,
        labels: bool,
    ) -> Vec<String> {
        let points = self
            .line
            .series
            .iter()
            .map(|series| {
                series
                    .points
                    .iter()
                    .map(|&point| {
                        self.project(
                            point,
                            plane,
                            width.saturating_sub(2),
                            height.saturating_sub(2),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        render::draw(
            &points,
            &self.line.series,
            width,
            height,
            focus,
            plane,
            labels,
        )
    }

    fn information(&self, focus: Option<usize>, width: usize, height: usize) -> Vec<String> {
        info::information(self.line, focus, width, height)
    }

    fn find(&self, query: &str) -> Option<usize> {
        let query = query.to_lowercase();
        self.line
            .series
            .iter()
            .position(|series| series.label.to_lowercase().contains(&query))
    }

    fn suggestion(&self, query: &str) -> Option<String> {
        self.find(query)
            .map(|index| self.line.series[index].label.clone())
    }

    fn position(&self, _index: usize) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn len(&self) -> usize {
        self.line.series.len()
    }
}
