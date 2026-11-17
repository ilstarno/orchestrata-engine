// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{PathError, Road};

/// A static directed graph with dense node indices and nonnegative travel times.
#[derive(Clone, Debug)]
pub struct TravelGraph {
    outgoing: Vec<Vec<(usize, u64)>>,
}

impl TravelGraph {
    /// Builds an adjacency list. Omit disallowed roads or build a graph per vehicle profile.
    pub fn new(node_count: usize, roads: &[Road]) -> Result<Self, PathError> {
        let mut graph = Self {
            outgoing: vec![Vec::new(); node_count],
        };
        for (road_index, road) in roads.iter().enumerate() {
            graph.require_node(road.from)?;
            graph.require_node(road.to)?;
            let time = u64::try_from(road.travel_time)
                .map_err(|_| PathError::NegativeTravelTime { road_index })?;
            graph.outgoing[road.from].push((road.to, time));
        }
        Ok(graph)
    }

    pub fn node_count(&self) -> usize {
        self.outgoing.len()
    }

    pub(super) fn require_node(&self, node: usize) -> Result<(), PathError> {
        if node >= self.node_count() {
            return Err(PathError::UnknownNode(node));
        }
        Ok(())
    }

    pub(super) fn outgoing(&self, node: usize) -> &[(usize, u64)] {
        &self.outgoing[node]
    }
}
