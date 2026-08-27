use std::collections::HashSet;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Graph {
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub fig: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    #[serde(alias = "source")]
    pub from: String,
    #[serde(alias = "target")]
    pub to: String,
    #[serde(default)]
    pub fig: Value,
}

impl Graph {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.nodes.is_empty(), "a graph needs at least one node");
        let ids: HashSet<_> = self.nodes.iter().map(|node| node.id.as_str()).collect();
        anyhow::ensure!(ids.len() == self.nodes.len(), "node ids must be unique");
        for edge in &self.edges {
            anyhow::ensure!(
                ids.contains(edge.from.as_str()),
                "unknown edge source: {}",
                edge.from
            );
            anyhow::ensure!(
                ids.contains(edge.to.as_str()),
                "unknown edge target: {}",
                edge.to
            );
        }
        Ok(())
    }

    pub fn node_label(&self, index: usize) -> &str {
        self.nodes[index]
            .label
            .as_deref()
            .unwrap_or(&self.nodes[index].id)
    }

    pub fn connected(&self, selected: usize, candidate: usize) -> bool {
        let selected_id = &self.nodes[selected].id;
        let candidate_id = &self.nodes[candidate].id;
        selected == candidate
            || self.edges.iter().any(|edge| {
                (edge.from == *selected_id && edge.to == *candidate_id)
                    || (edge.to == *selected_id && edge.from == *candidate_id)
            })
    }
}
