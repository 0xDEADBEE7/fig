use fig::figure::graph::layout::force_directed;
use fig::models::{Edge, Graph, Node};
use serde_json::Value;
use std::ops::Range;

fn graph(nodes: usize, edges: &[(usize, usize)]) -> Graph {
    Graph {
        nodes: (0..nodes)
            .map(|index| Node {
                id: index.to_string(),
                label: None,
                metadata: Value::Null,
            })
            .collect(),
        edges: edges
            .iter()
            .map(|&(from, to)| Edge {
                from: from.to_string(),
                to: to.to_string(),
            })
            .collect(),
    }
}

#[test]
fn connected_nodes_are_closer_than_unrelated_nodes() {
    let points = force_directed(&graph(4, &[(0, 1), (1, 2), (2, 3)]));
    let edge = distance(points[0], points[1]);
    let unrelated = distance(points[0], points[3]);
    assert!(edge < unrelated);
}

#[test]
fn disconnected_nodes_stay_within_a_reasonable_range() {
    let points = force_directed(&graph(6, &[(0, 1), (1, 2)]));
    let linked = [(0, 1), (1, 2)]
        .iter()
        .map(|&(from, to)| distance(points[from], points[to]))
        .sum::<f64>()
        / 2.0;
    let unrelated = [(0, 3), (1, 4), (2, 5)]
        .iter()
        .map(|&(from, to)| distance(points[from], points[to]))
        .sum::<f64>()
        / 3.0;
    assert!(unrelated < linked * 1.8);
}

#[test]
fn layout_is_deterministic_and_separates_nodes() {
    let graph = graph(8, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    let first = force_directed(&graph);
    assert_eq!(first, force_directed(&graph));
    assert!(minimum_distance(&first) > 0.01);
}


#[test]
fn dense_neighborhoods_do_not_collapse_into_a_ball() {
    let mut edges = Vec::new();
    for start in [0, 10, 20] {
        for node in start + 1..start + 10 {
            edges.push((start, node));
            edges.push((node, start + 1 + (node - start) % 9));
        }
    }
    edges.extend([(0, 10), (10, 20)]);

    let points = force_directed(&graph(30, &edges));
    assert!(minimum_distance(&points) > 0.025);

    let linked = edges
        .iter()
        .map(|&(from, to)| distance(points[from], points[to]))
        .sum::<f64>()
        / edges.len() as f64;
    assert!(linked < average_distance(&points));
}

#[test]
fn primary_branches_occupy_distinct_regions() {
    let (graph, branches) = organization(3, 40);
    let points = force_directed(&graph);
    assert_distinct_regions(&points, &branches);
}

fn organization(branch_count: usize, branch_size: usize) -> (Graph, Vec<Range<usize>>) {
    let branches = (0..branch_count)
        .map(|branch| 1 + branch * branch_size..1 + (branch + 1) * branch_size)
        .collect::<Vec<_>>();
    let mut edges = Vec::new();
    for branch in &branches {
        edges.push((0, branch.start));
        for node in branch.start + 1..branch.end {
            let parent = branch.start + (node - branch.start - 1) / 3;
            edges.push((parent, node));
        }
    }
    (graph(1 + branch_count * branch_size, &edges), branches)
}

fn assert_distinct_regions(points: &[(f64, f64)], branches: &[Range<usize>]) {
    let centers = branches
        .iter()
        .map(|branch| centroid(&points[branch.clone()]))
        .collect::<Vec<_>>();

    for (index, branch) in branches.iter().enumerate() {
        let correctly_grouped = branch
            .clone()
            .filter(|&node| {
                distance(points[node], centers[index])
                    < centers
                        .iter()
                        .enumerate()
                        .filter(|(other, _)| *other != index)
                        .map(|(_, &center)| distance(points[node], center))
                        .fold(f64::INFINITY, f64::min)
            })
            .count();
        assert!(correctly_grouped >= branch.len() * 4 / 5);
    }
}

fn distance(left: (f64, f64), right: (f64, f64)) -> f64 {
    (left.0 - right.0).hypot(left.1 - right.1)
}

fn centroid(points: &[(f64, f64)]) -> (f64, f64) {
    let total = points
        .iter()
        .fold((0.0, 0.0), |sum, point| (sum.0 + point.0, sum.1 + point.1));
    (total.0 / points.len() as f64, total.1 / points.len() as f64)
}

fn minimum_distance(points: &[(f64, f64)]) -> f64 {
    pairs(points)
        .map(|(left, right)| distance(left, right))
        .fold(f64::INFINITY, f64::min)
}

fn average_distance(points: &[(f64, f64)]) -> f64 {
    let count = points.len() * (points.len() - 1) / 2;
    pairs(points)
        .map(|(left, right)| distance(left, right))
        .sum::<f64>()
        / count as f64
}

fn pairs(points: &[(f64, f64)]) -> impl Iterator<Item = ((f64, f64), (f64, f64))> + '_ {
    points
        .iter()
        .enumerate()
        .flat_map(|(left, &point)| points[left + 1..].iter().map(move |&right| (point, right)))
}
