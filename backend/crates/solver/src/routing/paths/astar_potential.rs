// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{PathError, ShortestPath, TravelGraph, search::search};

/// A consistent travel-time estimate for one target in one immutable graph.
#[derive(Debug)]
pub struct AStarPotential<'a> {
    graph: &'a TravelGraph,
    goal: usize,
    estimates: Vec<u64>,
}

impl<'a> AStarPotential<'a> {
    /// Checks the goal estimate and every directed edge once before searches.
    pub fn new(
        graph: &'a TravelGraph,
        goal: usize,
        estimates: Vec<u64>,
    ) -> Result<Self, PathError> {
        graph.require_node(goal)?;
        if graph.node_count() != estimates.len() {
            return Err(PathError::WrongEstimateCount {
                expected: graph.node_count(),
                actual: estimates.len(),
            });
        }
        if estimates[goal] != 0 {
            return Err(PathError::NonzeroGoalEstimate);
        }
        for from in 0..graph.node_count() {
            for &(to, time) in graph.outgoing(from) {
                if u128::from(estimates[from]) > u128::from(time) + u128::from(estimates[to]) {
                    return Err(PathError::InconsistentEstimate { from, to });
                }
            }
        }
        Ok(Self {
            graph,
            goal,
            estimates,
        })
    }

    /// Returns the exact shortest path, or `None` only after exhaustion proves no route.
    pub fn shortest_path(
        &self,
        start: usize,
        max_expansions: usize,
    ) -> Result<Option<ShortestPath>, PathError> {
        search(self.graph, start, self.goal, max_expansions, |node| {
            self.estimates[node]
        })
    }
}
