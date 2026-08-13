mod info;
pub mod layout;
pub mod render;

use crate::models::Graph;

use super::{Plane, Visualization};

pub struct GraphView<'a> {
    graph: &'a Graph,
    points: Vec<(f64, f64)>,
}

impl<'a> GraphView<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
            points: layout::force_directed(graph),
        }
    }
}

impl Visualization for GraphView<'_> {
    fn default_plane(&self) -> Plane {
        Plane::new(false, false)
    }

    fn draw(
        &self,
        width: usize,
        height: usize,
        focus: Option<usize>,
        plane: Plane,
        labels: bool,
    ) -> Vec<String> {
        render::draw(
            self.graph,
            &self.points,
            width,
            height,
            focus,
            plane,
            labels,
        )
    }

    fn information(&self, focus: Option<usize>, width: usize, height: usize) -> Vec<String> {
        info::information(self.graph, focus, width, height)
    }

    fn find(&self, query: &str) -> Option<usize> {
        let needle = query.to_lowercase();
        self.graph
            .nodes
            .iter()
            .position(|node| fuzzy_match(self.label(node), &needle))
    }

    fn labels(&self) -> Vec<String> {
        self.graph
            .nodes
            .iter()
            .map(|node| self.label(node).to_owned())
            .collect()
    }

    fn position(&self, index: usize) -> (f64, f64) {
        self.points[index]
    }

    fn len(&self) -> usize {
        self.graph.nodes.len()
    }
}

impl GraphView<'_> {
    fn label<'b>(&self, node: &'b crate::models::Node) -> &'b str {
        node.label.as_deref().unwrap_or(&node.id)
    }
}

fn fuzzy_match(label: &str, query: &str) -> bool {
    let mut query = query.chars();
    let mut wanted = query.next();
    for character in label.to_lowercase().chars() {
        if Some(character) == wanted {
            wanted = query.next();
        }
    }
    wanted.is_none()
}
