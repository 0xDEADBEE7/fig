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
            Point {
                x: angle.cos(),
                y: angle.sin(),
            }
        })
        .collect();

    // Use a bounded spring simulation so a dense or disconnected graph cannot
    // fling nodes to infinity and collapse the viewport around bad geometry.
    for step in 0..iterations {
        let mut movement = vec![Point::default(); count];
        for a in 0..count {
            for b in (a + 1)..count {
                let dx = positions[a].x - positions[b].x;
                let dy = positions[a].y - positions[b].y;
                let distance = dx.hypot(dy).max(0.05);
                let force = (0.04 / (distance * distance)).min(0.2);
                movement[a].x += dx / distance * force;
                movement[a].y += dy / distance * force;
                movement[b].x -= dx / distance * force;
                movement[b].y -= dy / distance * force;
            }
        }
        for edge in &graph.edges {
            let a = index[edge.from.as_str()];
            let b = index[edge.to.as_str()];
            if a == b {
                continue;
            }
            let dx = positions[b].x - positions[a].x;
            let dy = positions[b].y - positions[a].y;
            let distance = dx.hypot(dy).max(0.05);
            let force = ((distance - 0.8) * 0.12).clamp(-0.2, 0.2);
            movement[a].x += dx / distance * force;
            movement[a].y += dy / distance * force;
            movement[b].x -= dx / distance * force;
            movement[b].y -= dy / distance * force;
        }

        let cooling = 0.12 * (1.0 - step as f64 / iterations.max(1) as f64);
        for (position, mut movement) in positions.iter_mut().zip(movement) {
            movement.x -= position.x * 0.05;
            movement.y -= position.y * 0.05;
            let length = movement.length();
            if length > 0.0 {
                let step = length.min(cooling);
                position.x += movement.x / length * step;
                position.y += movement.y / length * step;
            }
            position.x = position.x.clamp(-2.5, 2.5);
            position.y = position.y.clamp(-2.5, 2.5);
        }
    }
    let (min_x, max_x, min_y, max_y) = positions.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), point| {
            (
                min_x.min(point.x),
                max_x.max(point.x),
                min_y.min(point.y),
                max_y.max(point.y),
            )
        },
    );
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let scale = (max_x - min_x).max(max_y - min_y).max(f64::EPSILON) / 2.0;
    for point in &mut positions {
        point.x = (point.x - center_x) / scale;
        point.y = (point.y - center_y) / scale;
    }
    positions
}
