use serde::Deserialize;

use super::{Graph, Line};

/// The extensibility boundary: future figure kinds belong here, not in the UI.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Figure {
    Graph(Graph),
    Line(Line),
}

impl Figure {
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::Graph(graph) => graph.validate(),
            Self::Line(line) => line.validate(),
        }
    }
}
