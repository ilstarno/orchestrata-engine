# Shortest travel-time paths

The Rust solver's `routing::paths` module finds an exact point-to-point path on a caller-provided, static directed graph. It does not fetch or license map data, choose the stop sequence for a fleet, or handle capacity, shifts, service times, pickups before deliveries, turn restrictions or live traffic. Use the existing route-construction module for checked multi-stop fleet planning; path costs can be supplied by a road-network adapter after validation.

## Choosing a search

- **Dijkstra** is the default when no trustworthy lower bound exists. It can also be adapted to a one-to-many search to avoid repeating work when building a travel-time matrix.
- **A\*** uses a target-specific estimate to prioritize the frontier. `AStarPotential` accepts estimates only when the goal's estimate is zero and every directed edge satisfies `h(from) <= travel_time + h(to)`. This ensures a consistent lower bound on the selected graph and measures estimates in the same travel-time units as edges. Zero is a valid estimate; it provides no pruning advantage. A validated potential can be reused across different starts for that graph and target. The validation costs a full graph scan, which should be amortized or included in end-to-end benchmarks.
- Both algorithms return the same optimal cost on the same valid static graph when the expansion limit is sufficient. An equal-cost path need not contain identical nodes. A strong estimate can reduce search work, but the result is still the same travel-time optimum.
- Graphs contain only permitted edges. Model each materially different vehicle-access profile separately. Reject routes beyond the graph's modeling capability; e.g. turn restrictions need graph state that captures the incoming edge.

A missing result means the graph was exhausted without reaching the target. An expansion-limit error is distinct and does not prove the target is unreachable. The limit counts settled nodes, including the goal and excluding stale queue entries; it is not a limit on memory, wall-clock time, or edges processed per node. Hosts accepting external graph data must also bound node count, edge count and degree before building a graph. The public total is a `u64`; search compares wider costs and reports overflow if the chosen optimum cannot be represented.

## Checked fixture

A 40×40 bidirectional synthetic grid of unit-cost edges, routing from one corner to the opposite, has an optimal cost of 78 in both modes. With a valid Manhattan estimate and deterministic ties, Dijkstra settles 1,600 nodes and A* 79. This counts algorithmic work on a favorable fixture only; it is not a latency benchmark or evidence for any city. The test also verifies every consecutive pair is a permitted edge. An independent all-pairs oracle covers 64 deterministic directed graphs and 1,600 source/target pairs, while targeted tests cover unreachable goals, negative edge rejection, invalid estimates, search limits, zero-cost cycles and arithmetic overflow.
