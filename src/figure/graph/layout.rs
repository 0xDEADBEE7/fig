use crate::models::Graph;

mod topology;

use topology::Distances;

const ITERATIONS: usize = 300;
const COOLING: f64 = 0.0228;
const VELOCITY_RETENTION: f64 = 0.6;
const HOP_DISTANCE: f64 = 1.0;
const STRESS_STRENGTH: f64 = 0.45;
const COMPONENT_CHARGE: f64 = 0.8;
const COLLISION_DISTANCE: f64 = 0.35;
const COLLISION_STRENGTH: f64 = 0.7;
const CENTER_STRENGTH: f64 = 0.01;
const MAX_SPEED: f64 = 0.5;
const GOLDEN_ANGLE: f64 = 2.399_963_229_728_653;

/// Computes a deterministic force-directed layout.
///
/// Shortest-path stress gives the simulation a view of the whole topology, so
/// distant branches do not collapse into a locally balanced ball. The resting
/// position is calculated once before rendering to keep terminal redraws cheap.
pub fn force_directed(graph: &Graph) -> Vec<(f64, f64)> {
    let count = graph.nodes.len();
    if count == 0 {
        return Vec::new();
    }
    if count == 1 {
        return vec![(0.0, 0.0)];
    }

    let distances = Distances::new(graph);
    let mut points = initial_points(count);
    let mut velocity = vec![(0.0, 0.0); count];
    let mut alpha = 1.0;

    for _ in 0..ITERATIONS {
        let mut force = vec![(0.0, 0.0); count];
        stress(&points, &distances, &mut force, alpha);
        center(&points, &mut force, alpha);
        apply(&mut points, &mut velocity, force, alpha);
        alpha *= 1.0 - COOLING;
    }
    normalize(&mut points);
    points
}

fn initial_points(count: usize) -> Vec<(f64, f64)> {
    (0..count)
        .map(|index| {
            let radius = 0.4 * (index as f64 + 0.5).sqrt();
            let angle = index as f64 * GOLDEN_ANGLE;
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

fn stress(points: &[(f64, f64)], distances: &Distances, force: &mut [(f64, f64)], alpha: f64) {
    for left in 0..points.len() {
        for right in left + 1..points.len() {
            let (dx, dy) = difference(points[left], points[right], left, right);
            let distance_squared = (dx * dx + dy * dy).max(0.01);
            let distance = distance_squared.sqrt();
            if let Some(hops) = distances.between(left, right) {
                let hops = f64::from(hops);
                let ideal = hops * HOP_DISTANCE;
                let weight = 1.0 / (hops * hops);
                let strength = (ideal - distance) / distance * STRESS_STRENGTH * weight * alpha;
                displace(force, left, right, dx * strength, dy * strength);
            } else {
                let strength = COMPONENT_CHARGE * alpha / distance_squared;
                displace(force, left, right, dx * strength, dy * strength);
            }

            if distance < COLLISION_DISTANCE {
                let strength =
                    (COLLISION_DISTANCE - distance) / distance * COLLISION_STRENGTH * alpha;
                displace(force, left, right, dx * strength, dy * strength);
            }
        }
    }
}

fn displace(force: &mut [(f64, f64)], left: usize, right: usize, dx: f64, dy: f64) {
    force[left].0 += dx;
    force[left].1 += dy;
    force[right].0 -= dx;
    force[right].1 -= dy;
}

fn center(points: &[(f64, f64)], force: &mut [(f64, f64)], alpha: f64) {
    for (point, force) in points.iter().zip(force) {
        force.0 -= point.0 * CENTER_STRENGTH * alpha;
        force.1 -= point.1 * CENTER_STRENGTH * alpha;
    }
}

fn apply(
    points: &mut [(f64, f64)],
    velocity: &mut [(f64, f64)],
    force: Vec<(f64, f64)>,
    alpha: f64,
) {
    let speed_limit = MAX_SPEED * alpha.sqrt().max(0.1);
    for ((point, velocity), force) in points.iter_mut().zip(velocity).zip(force) {
        velocity.0 = (velocity.0 + force.0) * VELOCITY_RETENTION;
        velocity.1 = (velocity.1 + force.1) * VELOCITY_RETENTION;
        let speed = velocity.0.hypot(velocity.1);
        if speed > speed_limit {
            velocity.0 *= speed_limit / speed;
            velocity.1 *= speed_limit / speed;
        }
        point.0 += velocity.0;
        point.1 += velocity.1;
    }
}

fn difference(
    left: (f64, f64),
    right: (f64, f64),
    left_index: usize,
    right_index: usize,
) -> (f64, f64) {
    let dx = left.0 - right.0;
    let dy = left.1 - right.1;
    if dx != 0.0 || dy != 0.0 {
        return (dx, dy);
    }
    // Preserve determinism if two nodes happen to occupy exactly the same point.
    let angle = (left_index * 31 + right_index * 17) as f64;
    (angle.cos() * 0.01, angle.sin() * 0.01)
}

fn normalize(points: &mut [(f64, f64)]) {
    let center = points
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.0, y + point.1));
    let center = (
        center.0 / points.len() as f64,
        center.1 / points.len() as f64,
    );
    let extent = points.iter().fold(0.01_f64, |extent, (x, y)| {
        extent.max((x - center.0).abs()).max((y - center.1).abs())
    });
    for point in points {
        point.0 = (point.0 - center.0) / extent;
        point.1 = (point.1 - center.1) / extent;
    }
}
