use std::collections::HashMap;

use crate::Graph;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    fn length(self) -> f64 {
        self.x.hypot(self.y)
    }
}

pub(crate) fn force_directed(graph: &Graph, iterations: usize) -> Vec<Point> {
    let count = graph.nodes.len();
    if count == 1 {
        return vec![Point::default()];
    }

    let index: HashMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id.as_str(), i))
        .collect();
    let mut positions: Vec<Point> = (0..count)
        .map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / count as f64;
            let radius = 1.0 + (i % 3) as f64 * 0.08;
            Point {
                x: angle.cos() * radius,
                y: angle.sin() * radius,
            }
        })
        .collect();

    let area = (count as f64).max(4.0);
    let ideal = (area / count as f64).sqrt();
    for step in 0..iterations {
        let mut movement = vec![Point::default(); count];

        for a in 0..count {
            for b in (a + 1)..count {
                let dx = positions[a].x - positions[b].x;
                let dy = positions[a].y - positions[b].y;
                let distance = dx.hypot(dy).max(0.01);
                let force = ideal * ideal / distance;
                let fx = dx / distance * force;
                let fy = dy / distance * force;
                movement[a].x += fx;
                movement[a].y += fy;
                movement[b].x -= fx;
                movement[b].y -= fy;
            }
        }

        for edge in &graph.edges {
            let a = index[edge.from.as_str()];
            let b = index[edge.to.as_str()];
            if a == b {
                continue;
            }
            let dx = positions[a].x - positions[b].x;
            let dy = positions[a].y - positions[b].y;
            let distance = dx.hypot(dy).max(0.01);
            let force = distance * distance / ideal;
            let fx = dx / distance * force;
            let fy = dy / distance * force;
            movement[a].x -= fx;
            movement[a].y -= fy;
            movement[b].x += fx;
            movement[b].y += fy;
        }

        let temperature = 0.15 * (1.0 - step as f64 / iterations.max(1) as f64);
        for i in 0..count {
            // A weak center force keeps disconnected components in view.
            movement[i].x -= positions[i].x * 0.04;
            movement[i].y -= positions[i].y * 0.04;
            let length = movement[i].length().max(0.001);
            let distance = length.min(temperature);
            positions[i].x += movement[i].x / length * distance;
            positions[i].y += movement[i].y / length * distance;
        }
    }
    positions
}
