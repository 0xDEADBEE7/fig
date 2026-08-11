use crate::models::Graph;

pub fn force_directed(graph: &Graph) -> Vec<(f64, f64)> {
    let count = graph.nodes.len();
    let mut points = (0..count)
        .map(|i| {
            let angle = std::f64::consts::TAU * i as f64 / count as f64;
            (angle.cos(), angle.sin())
        })
        .collect::<Vec<_>>();
    for step in 0..80 {
        let mut movement = vec![(0.0, 0.0); count];
        repel(&points, &mut movement);
        attract(graph, &points, &mut movement);
        apply(&mut points, movement, step);
    }
    normalize(&mut points);
    points
}

fn repel(points: &[(f64, f64)], movement: &mut [(f64, f64)]) {
    for left in 0..points.len() {
        for right in left + 1..points.len() {
            let (dx, dy) = (
                points[left].0 - points[right].0,
                points[left].1 - points[right].1,
            );
            let distance = dx.hypot(dy).max(0.05);
            let force = (0.045 / distance.powi(2)).min(0.2);
            movement[left].0 += dx / distance * force;
            movement[left].1 += dy / distance * force;
            movement[right].0 -= dx / distance * force;
            movement[right].1 -= dy / distance * force;
        }
    }
}

fn attract(graph: &Graph, points: &[(f64, f64)], movement: &mut [(f64, f64)]) {
    for edge in &graph.edges {
        let from = graph
            .nodes
            .iter()
            .position(|node| node.id == edge.from)
            .unwrap();
        let to = graph
            .nodes
            .iter()
            .position(|node| node.id == edge.to)
            .unwrap();
        let (dx, dy) = (points[to].0 - points[from].0, points[to].1 - points[from].1);
        let distance = dx.hypot(dy).max(0.05);
        let force = ((distance - 0.8) * 0.12).clamp(-0.2, 0.2);
        movement[from].0 += dx / distance * force;
        movement[from].1 += dy / distance * force;
        movement[to].0 -= dx / distance * force;
        movement[to].1 -= dy / distance * force;
    }
}

fn apply(points: &mut [(f64, f64)], movement: Vec<(f64, f64)>, step: usize) {
    let cooling = 0.12 * (1.0 - step as f64 / 80.0);
    for ((x, y), (dx, dy)) in points.iter_mut().zip(movement) {
        let length = dx.hypot(dy);
        if length > 0.0 {
            *x += dx / length * length.min(cooling);
            *y += dy / length * length.min(cooling);
        }
        *x *= 0.95;
        *y *= 0.95;
    }
}

fn normalize(points: &mut [(f64, f64)]) {
    let extent = points
        .iter()
        .fold(0.01_f64, |extent, (x, y)| extent.max(x.abs()).max(y.abs()));
    for (x, y) in points {
        *x /= extent;
        *y /= extent;
    }
}
