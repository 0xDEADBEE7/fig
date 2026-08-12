use std::collections::{HashMap, VecDeque};

use crate::models::Graph;

const UNREACHABLE: u32 = u32::MAX;

pub struct Distances {
    hops: Vec<Vec<u32>>,
}

impl Distances {
    pub fn new(graph: &Graph) -> Self {
        let indexes = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut neighbors = vec![Vec::new(); graph.nodes.len()];
        for edge in &graph.edges {
            let from = indexes[edge.from.as_str()];
            let to = indexes[edge.to.as_str()];
            if from != to {
                neighbors[from].push(to);
                neighbors[to].push(from);
            }
        }
        let hops = (0..graph.nodes.len())
            .map(|start| distances_from(start, &neighbors))
            .collect();
        Self { hops }
    }

    pub fn between(&self, left: usize, right: usize) -> Option<u32> {
        let hops = self.hops[left][right];
        (hops != UNREACHABLE).then_some(hops)
    }
}

fn distances_from(start: usize, neighbors: &[Vec<usize>]) -> Vec<u32> {
    let mut distance = vec![UNREACHABLE; neighbors.len()];
    distance[start] = 0;
    let mut pending = VecDeque::from([start]);
    while let Some(node) = pending.pop_front() {
        let next_distance = distance[node] + 1;
        for &next in &neighbors[node] {
            if distance[next] == UNREACHABLE {
                distance[next] = next_distance;
                pending.push_back(next);
            }
        }
    }
    distance
}
