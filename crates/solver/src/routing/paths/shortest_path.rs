// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// One optimal path and the number of settled nodes used to reach it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortestPath {
    /// Inclusive start and end node indices, in traversal order.
    pub nodes: Vec<usize>,
    pub travel_time: u64,
    /// Includes the goal and excludes stale frontier entries.
    pub nodes_expanded: usize,
}
