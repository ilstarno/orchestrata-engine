// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use orchestrata_solver::routing::paths::{
    AStarPotential, PathError, Road, TravelGraph, dijkstra_shortest_path,
};

const fn road(from: usize, to: usize, travel_time: i64) -> Road {
    Road {
        from,
        to,
        travel_time,
    }
}

#[test]
fn optimal_directed_route_uses_permitted_edges_only() {
    let roads = [
        road(0, 1, 2),
        road(1, 2, 2),
        road(0, 2, 9),
        road(2, 3, 1),
        road(0, 3, 12),
    ];
    let graph = TravelGraph::new(4, &roads).unwrap();
    let baseline = dijkstra_shortest_path(&graph, 0, 3, 4).unwrap().unwrap();
    let guided = AStarPotential::new(&graph, 3, vec![4, 2, 1, 0])
        .unwrap()
        .shortest_path(0, 4)
        .unwrap()
        .unwrap();

    assert_eq!(baseline.travel_time, 5);
    assert_eq!(guided.travel_time, baseline.travel_time);
    assert_eq!(baseline.nodes, [0, 1, 2, 3]);
    assert_eq!(guided.nodes, baseline.nodes);
    assert_eq!(dijkstra_shortest_path(&graph, 3, 0, 4), Ok(None));
}

#[test]
fn a_star_expands_fewer_nodes_on_a_metric_grid_with_the_same_cost() {
    const WIDTH: usize = 40;
    let mut roads = Vec::new();
    for row in 0..WIDTH {
        for col in 0..WIDTH {
            let from = row * WIDTH + col;
            if col + 1 < WIDTH {
                roads.extend([road(from, from + 1, 1), road(from + 1, from, 1)]);
            }
            if row + 1 < WIDTH {
                roads.extend([road(from, from + WIDTH, 1), road(from + WIDTH, from, 1)]);
            }
        }
    }
    let node_count = WIDTH * WIDTH;
    let goal = node_count - 1;
    let estimates = (0..node_count)
        .map(|node| (2 * (WIDTH - 1) - (node / WIDTH + node % WIDTH)) as u64)
        .collect();
    let graph = TravelGraph::new(node_count, &roads).unwrap();
    let baseline = dijkstra_shortest_path(&graph, 0, goal, node_count)
        .unwrap()
        .unwrap();
    let guided = AStarPotential::new(&graph, goal, estimates)
        .unwrap()
        .shortest_path(0, node_count)
        .unwrap()
        .unwrap();

    assert_eq!(baseline.travel_time, 78);
    assert_eq!(guided.travel_time, baseline.travel_time);
    assert_eq!(baseline.nodes_expanded, 1600);
    assert_eq!(guided.nodes_expanded, 79);
    for nodes in [&baseline.nodes, &guided.nodes] {
        assert_eq!(nodes.first(), Some(&0));
        assert_eq!(nodes.last(), Some(&goal));
        assert_eq!(nodes.len() as u64 - 1, baseline.travel_time);
        assert!(nodes.windows(2).all(|link| {
            roads
                .iter()
                .any(|road| road.from == link[0] && road.to == link[1])
        }));
    }
}

#[test]
fn budget_is_not_an_unreachable_answer() {
    let graph = TravelGraph::new(3, &[road(0, 1, 1)]).unwrap();
    assert_eq!(
        dijkstra_shortest_path(&graph, 0, 2, 1),
        Err(PathError::ExpansionLimitReached { expanded: 1 })
    );
    assert_eq!(dijkstra_shortest_path(&graph, 0, 2, 2), Ok(None));
    let potential = AStarPotential::new(&graph, 2, vec![0; 3]).unwrap();
    assert_eq!(potential.shortest_path(0, 2), Ok(None));
    assert_eq!(
        potential.shortest_path(2, 0),
        Err(PathError::ExpansionLimitReached { expanded: 0 })
    );
    assert_eq!(potential.shortest_path(2, 1).unwrap().unwrap().nodes, [2]);
}

#[test]
fn rejects_invalid_endpoints_weights_and_heuristics() {
    assert_eq!(
        TravelGraph::new(2, &[road(0, 2, 3)]).unwrap_err(),
        PathError::UnknownNode(2)
    );
    assert_eq!(
        TravelGraph::new(2, &[road(0, 1, -1)]).unwrap_err(),
        PathError::NegativeTravelTime { road_index: 0 }
    );
    let graph = TravelGraph::new(2, &[road(0, 1, 3)]).unwrap();
    assert_eq!(
        AStarPotential::new(&graph, 1, vec![4, 0]).unwrap_err(),
        PathError::InconsistentEstimate { from: 0, to: 1 }
    );
    assert_eq!(
        AStarPotential::new(&graph, 1, vec![1, 1]).unwrap_err(),
        PathError::NonzeroGoalEstimate
    );
    assert_eq!(
        AStarPotential::new(&graph, 1, vec![]).unwrap_err(),
        PathError::WrongEstimateCount {
            expected: 2,
            actual: 0
        }
    );
    assert_eq!(
        dijkstra_shortest_path(&graph, 2, 1, 2),
        Err(PathError::UnknownNode(2))
    );
}

#[test]
fn zero_cost_cycles_and_stale_entries_do_not_change_results() {
    let graph = TravelGraph::new(
        4,
        &[
            road(0, 1, 8),
            road(0, 2, 1),
            road(2, 1, 0),
            road(1, 2, 0),
            road(1, 3, 10),
        ],
    )
    .unwrap();
    let answer = dijkstra_shortest_path(&graph, 0, 3, 4).unwrap().unwrap();
    assert_eq!(answer.nodes, [0, 2, 1, 3]);
    assert_eq!(answer.travel_time, 11);
    assert_eq!(answer.nodes_expanded, 4);
    assert_eq!(
        AStarPotential::new(&graph, 3, vec![0; 4])
            .unwrap()
            .shortest_path(0, 4),
        Ok(Some(answer))
    );
}

#[test]
fn ignores_overflow_in_losing_detours_but_reports_unrepresentable_optimum() {
    let mut roads = vec![
        road(0, 1, i64::MAX),
        road(1, 2, i64::MAX),
        road(2, 3, i64::MAX),
    ];
    let graph = TravelGraph::new(4, &roads).unwrap();
    assert_eq!(
        dijkstra_shortest_path(&graph, 0, 3, 4),
        Err(PathError::TravelTimeOverflow)
    );
    assert_eq!(
        AStarPotential::new(&graph, 3, vec![0; 4])
            .unwrap()
            .shortest_path(0, 4),
        Err(PathError::TravelTimeOverflow)
    );
    roads.push(road(0, 3, 1));
    let graph = TravelGraph::new(4, &roads).unwrap();
    assert_eq!(
        dijkstra_shortest_path(&graph, 0, 3, 4)
            .unwrap()
            .unwrap()
            .travel_time,
        1
    );
}

#[test]
fn matches_an_all_pairs_oracle_across_small_deterministic_directed_graphs() {
    const N: usize = 5;
    for seed in 0..64 {
        let mut roads = Vec::new();
        let mut oracle = [[None::<u64>; N]; N];
        for (from, row) in oracle.iter_mut().enumerate() {
            row[from] = Some(0);
            for (to, cost) in row.iter_mut().enumerate() {
                if from != to && (from * 13 + to * 19 + seed * 11 + from * to * seed) % 7 < 4 {
                    let time = ((from * 17 + to * 5 + seed) % 9) as i64;
                    *cost = Some(time as u64);
                    roads.push(road(from, to, time));
                }
            }
        }
        for via in 0..N {
            for from in 0..N {
                for to in 0..N {
                    if let (Some(first), Some(second)) = (oracle[from][via], oracle[via][to])
                        && oracle[from][to].is_none_or(|old| first + second < old)
                    {
                        oracle[from][to] = Some(first + second);
                    }
                }
            }
        }
        let graph = TravelGraph::new(N, &roads).unwrap();
        for goal in 0..N {
            // Infinity is not a finite estimate. Use the max finite distance in
            // unreachable regions, preserving consistency on edges entering them.
            let furthest = oracle.iter().filter_map(|row| row[goal]).max().unwrap();
            let estimates = oracle
                .iter()
                .map(|row| row[goal].unwrap_or(furthest))
                .collect();
            let potential = AStarPotential::new(&graph, goal, estimates).unwrap();
            for (start, row) in oracle.iter().enumerate() {
                for result in [
                    dijkstra_shortest_path(&graph, start, goal, N).unwrap(),
                    potential.shortest_path(start, N).unwrap(),
                ] {
                    assert_eq!(result.as_ref().map(|path| path.travel_time), row[goal]);
                    if let Some(path) = result {
                        assert_eq!(path.nodes.first(), Some(&start));
                        assert_eq!(path.nodes.last(), Some(&goal));
                        let path_cost: u64 = path
                            .nodes
                            .windows(2)
                            .map(|link| {
                                roads
                                    .iter()
                                    .find(|edge| edge.from == link[0] && edge.to == link[1])
                                    .unwrap()
                                    .travel_time as u64
                            })
                            .sum();
                        assert_eq!(path_cost, path.travel_time);
                    }
                }
            }
        }
    }
}
