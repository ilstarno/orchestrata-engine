use std::{cmp::Reverse, collections::BinaryHeap};

use super::edge::Edge;

pub(super) struct ResidualNetwork {
    edges: Vec<Vec<Edge>>,
    potentials: Vec<i64>,
}

impl ResidualNetwork {
    pub(super) fn new(nodes: usize) -> Self {
        Self {
            edges: (0..nodes).map(|_| Vec::new()).collect(),
            potentials: vec![0; nodes],
        }
    }

    pub(super) fn add(&mut self, from: usize, to: usize, capacity: usize, cost: u32) -> usize {
        let edge_index = self.edges[from].len();
        let back_index = self.edges[to].len();
        let cost = i64::from(cost);
        self.edges[from].push(Edge {
            to,
            reverse: back_index,
            available: capacity,
            cost,
        });
        self.edges[to].push(Edge {
            to: from,
            reverse: edge_index,
            available: 0,
            cost: -cost,
        });
        edge_index
    }

    pub(super) fn saturated(&self, node: usize, edge: usize) -> bool {
        self.edges[node][edge].available == 0
    }

    pub(super) fn augment_cheapest(&mut self, source: usize, sink: usize) -> bool {
        match self.augment_cheapest_controlled(source, sink, &mut || {
            Ok::<(), std::convert::Infallible>(())
        }) {
            Ok(result) => result,
            Err(never) => match never {},
        }
    }

    pub(super) fn augment_cheapest_controlled<E>(
        &mut self,
        source: usize,
        sink: usize,
        check: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        check()?;
        let mut distances = vec![i64::MAX; self.edges.len()];
        let mut predecessor = vec![None; self.edges.len()];
        let mut queue = BinaryHeap::new();
        distances[source] = 0;
        queue.push(Reverse((0, source)));

        while let Some(Reverse((distance, from))) = queue.pop() {
            check()?;
            if distances[from] != distance {
                continue;
            }
            for (edge_index, edge) in self.edges[from].iter().enumerate() {
                check()?;
                if edge.available == 0 {
                    continue;
                }
                // The potentials make every reachable residual edge nonnegative.
                // A simple path has fewer than 402 edges with at most u32::MAX
                // magnitude each. Original distances and reduced distances fit i64.
                let reduced = edge.cost + self.potentials[from] - self.potentials[edge.to];
                debug_assert!(reduced >= 0);
                let next = distance + reduced;
                if next < distances[edge.to] {
                    distances[edge.to] = next;
                    predecessor[edge.to] = Some((from, edge_index));
                    queue.push(Reverse((next, edge.to)));
                }
            }
        }

        if distances[sink] == i64::MAX {
            return Ok(false);
        }
        for (potential, distance) in self.potentials.iter_mut().zip(distances) {
            if distance != i64::MAX {
                *potential += distance;
            }
        }

        let mut node = sink;
        while let Some((from, index)) = predecessor[node] {
            let edge = &mut self.edges[from][index];
            edge.available -= 1;
            let reverse = edge.reverse;
            self.edges[node][reverse].available += 1;
            node = from;
        }
        debug_assert_eq!(node, source);
        Ok(true)
    }
}
