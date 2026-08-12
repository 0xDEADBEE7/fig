#[cfg(feature = "test-support")]
pub mod graph;
#[cfg(not(feature = "test-support"))]
mod graph;
#[cfg(feature = "test-support")]
pub mod line;
#[cfg(not(feature = "test-support"))]
mod line;
mod plane;

use crate::models::Figure;

pub use graph::GraphView;
pub use line::LineView;
pub use plane::Plane;

pub trait Visualization {
    fn default_plane(&self) -> Plane;
    fn draw(
        &self,
        width: usize,
        height: usize,
        focus: Option<usize>,
        plane: Plane,
        labels: bool,
    ) -> Vec<String>;
    fn information(&self, focus: Option<usize>, width: usize, height: usize) -> Vec<String>;
    fn find(&self, query: &str) -> Option<usize>;
    fn suggestion(&self, query: &str) -> Option<String>;
    fn position(&self, index: usize) -> (f64, f64);
    fn len(&self) -> usize;
}

pub fn visualizer(figure: &Figure) -> Box<dyn Visualization + '_> {
    match figure {
        Figure::Graph(graph) => Box::new(GraphView::new(graph)),
        Figure::Line(line) => Box::new(LineView::new(line)),
    }
}
