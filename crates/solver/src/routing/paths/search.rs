// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use std::{cmp::Reverse, collections::BinaryHeap};

use super::{PathError, ShortestPath, TravelGraph};

/// Finds an exact shortest path without a heuristic.
///
/// `None` means the goal was unreachable; hitting the expansion bound is an error.
pub fn dijkstra_shortest_path(
    graph: &TravelGraph,
    start: usize,
    goal: usize,
    max_expansions: usize,
) -> Result<Option<ShortestPath>, PathError> {
    search(graph, start, goal, max_expansions, |_| 0)
}

pub(super) fn search(
    graph: &TravelGraph,
    start: usize,
    goal: usize,
    max_expansions: usize,
    estimate: impl Fn(usize) -> u64,
) -> Result<Option<ShortestPath>, PathError> {
    graph.require_node(start)?;
    graph.require_node(goal)?;

    let mut distance = vec![None; graph.node_count()];
    let mut predecessor = vec![None; graph.node_count()];
    let mut frontier = BinaryHeap::new();
    distance[start] = Some(0);
    frontier.push(Reverse((
        u128::from(estimate(start)),
        Reverse(0_u128),
        start,
    )));
    let mut expanded = 0;

    while let Some(Reverse((_, Reverse(cost), node))) = frontier.pop() {
        if distance[node] != Some(cost) {
            continue;
        }
        if expanded == max_expansions {
            return Err(PathError::ExpansionLimitReached { expanded });
        }
        expanded += 1;
        if node == goal {
            return build_path(goal, cost, expanded, &predecessor).map(Some);
        }

        for &(next, time) in graph.outgoing(node) {
            let next_cost = cost
                .checked_add(u128::from(time))
                .ok_or(PathError::TravelTimeOverflow)?;
            if distance[next].is_none_or(|current| next_cost < current) {
                let priority = next_cost
                    .checked_add(u128::from(estimate(next)))
                    .ok_or(PathError::TravelTimeOverflow)?;
                distance[next] = Some(next_cost);
                predecessor[next] = Some(node);
                frontier.push(Reverse((priority, Reverse(next_cost), next)));
            }
        }
    }

    Ok(None)
}

fn build_path(
    goal: usize,
    cost: u128,
    expanded: usize,
    predecessor: &[Option<usize>],
) -> Result<ShortestPath, PathError> {
    let travel_time = u64::try_from(cost).map_err(|_| PathError::TravelTimeOverflow)?;
    let mut nodes = vec![goal];
    let mut current = goal;
    while let Some(previous) = predecessor[current] {
        nodes.push(previous);
        current = previous;
    }
    nodes.reverse();
    Ok(ShortestPath {
        nodes,
        travel_time,
        nodes_expanded: expanded,
    })
}
