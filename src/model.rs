use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Figure {
    Graph(Graph),
    Line(LineFigure),
    Histogram(HistogramFigure),
}

impl Figure {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Graph(graph) => graph.validate(),
            Self::Line(line) => line.validate(),
            Self::Histogram(histogram) => histogram.validate(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistogramFigure {
    pub series: Vec<HistogramSeries>,
    pub buckets: Vec<HistogramBucket>,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistogramSeries {
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistogramBucket {
    pub label: String,
    #[serde(default)]
    pub values: HashMap<String, f64>,
}

impl HistogramFigure {
    pub fn validate(&self) -> Result<()> {
        if self.series.is_empty() {
            bail!("a histogram must contain at least one series");
        }
        if self.buckets.is_empty() {
            bail!("a histogram must contain at least one bucket");
        }
        let mut labels = HashSet::new();
        for series in &self.series {
            if series.label.trim().is_empty() || !labels.insert(series.label.as_str()) {
                bail!("histogram series labels must be non-empty and unique");
            }
        }
        for bucket in &self.buckets {
            if bucket.label.trim().is_empty() {
                bail!("histogram bucket labels cannot be empty");
            }
            for (label, value) in &bucket.values {
                if !labels.contains(label.as_str()) {
                    bail!(
                        "histogram bucket {:?} references unknown series {:?}",
                        bucket.label,
                        label
                    );
                }
                if !value.is_finite() || *value < 0.0 {
                    bail!(
                        "histogram value {:?} in bucket {:?} must be finite and non-negative",
                        value,
                        bucket.label
                    );
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineFigure {
    pub series: Vec<LineSeries>,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineSeries {
    pub label: String,
    pub points: Vec<DataPoint>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl LineFigure {
    pub fn validate(&self) -> Result<()> {
        if self.series.is_empty() {
            bail!("a line figure must contain at least one series");
        }
        for series in &self.series {
            if series.label.trim().is_empty() {
                bail!("line series labels cannot be empty");
            }
            if series.points.is_empty() {
                bail!(
                    "line series {:?} must contain at least one point",
                    series.label
                );
            }
            if series
                .points
                .iter()
                .any(|point| !point.x.is_finite() || !point.y.is_finite())
            {
                bail!("line series {:?} contains a non-finite point", series.label);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
}

impl Node {
    pub fn display_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.id)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    #[serde(alias = "source")]
    pub from: String,
    #[serde(alias = "target")]
    pub to: String,
}

impl Graph {
    pub fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            bail!("the graph must contain at least one node");
        }

        let mut ids = HashSet::new();
        for node in &self.nodes {
            if node.id.trim().is_empty() {
                bail!("node IDs cannot be empty");
            }
            if !ids.insert(node.id.as_str()) {
                bail!("duplicate node ID {:?}", node.id);
            }
        }

        for edge in &self.edges {
            if !ids.contains(edge.from.as_str()) {
                bail!("edge references unknown node {:?}", edge.from);
            }
            if !ids.contains(edge.to.as_str()) {
                bail!("edge references unknown node {:?}", edge.to);
            }
        }
        Ok(())
    }
}
