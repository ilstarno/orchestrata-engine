# Opt-in route coverage certificate

The Rust API can bound the maximum number of fully served shipments for the
compiled static directed routing problem. Existing XML, CLI commands and solve
result bytes are unchanged. Certification is separate from construction and
incurs no work unless called explicitly. It does not improve the incumbent.

For a validated route package, compile the same selection, records and insertion
budget again after solving, then call:

```rust,ignore
let solved = package.compile_route(selection, records, insertion_budget)?.solve()?;
let compiled = package.compile_route(selection, records, insertion_budget)?;
let checked = compiled.certify_coverage(&solved.solution, 5_000_000)?;
```

`RoutePackageCoverage` carries the compiled content, execution, input and request
identities alongside a Rust `CoverageOutcome`. These identify the compiled
revision and exact input bytes, not authorization or a solver binary version.
The verifier checks the supplied plan against that compiled problem; it does
not trust the plan's solver label or insertion counters. Lower-level callers
can use `route_dispatch::certify_route_coverage` or
`orchestrata_solver::routing::coverage::certify_coverage`.

A complete result contains `incumbent_covered`, `upper_bound`, and
`maximum_coverage`. The maximum feasible coverage lies in that closed interval.
Equality proves **maximum coverage only**. It does not prove minimum courier
minutes, minimum cost, uniqueness, or an objective from another model. A larger
bound means the certificate is inconclusive about whether the incumbent can
be improved. The existing solve result remains labeled heuristic.

A budget-exhausted result is `Incomplete { work }`: it contains no bound or
optimality claim. Invalid plans and inputs return errors. Limits are 16 couriers,
32 shipments and at most 5,000,000 work units. One initial unit is charged; subsequent units count one closure update,
travel sample, relaxed visit/return, service-count step, or flow edge/owner/update
operation. Input validation, bounded replay, location indexing, sorting and
witness checks are outside the counter. It is a deterministic algorithmic
budget, not a wall-clock deadline. The native travel callback must be static,
repeatable and bounded; the package API uses its immutable in-memory travel map.

## Relaxations and witness

The incumbent is replayed independently with the **original directed arcs**,
including mode, shifts, windows, service times, capacity, pickup/delivery pairing
and depot return. Unknown arcs cannot validate an incumbent. Package result IDs,
service fields, elapsed total and unassigned partition are checked as well.

For exclusion tests only, the certificate computes directed shortest paths per
mode over all problem depots and stop locations. They include every location
that a legal route may visit. Missing direct arcs do not exclude a shipment if
an optimistic supplied path exists. This is safe for sparse, nonmetric travel:
removing other visits from a feasible route and replacing their legs with these
shortest paths cannot delay the remaining visits. No external travel is fetched.

A relaxed depot–pickup–delivery–depot schedule determines job/courier edges.
Two conservative capacitated bipartite graphs then bound coverage:

* Service graph: courier slots have capacity equal to the greatest number of
  eligible jobs whose smallest combined pickup/delivery service times fit the
  shift. Travel and waiting are omitted.
* Conflict graph: eligible jobs are partitioned per courier, in stable input
  order, into disjoint pairwise-incompatible groups. Every pair in a group fails
  all six pickup-before-delivery orders even with optimistic travel. Such a group
  contributes one slot. Removing other jobs only reduces load and elapsed time,
  so a feasible route cannot contain two members of a group.

Each feasible routing assignment embeds in both graphs. The smaller of their
maximum flows is therefore an upper bound. Each graph includes its eligible
slots, capacities, matching and source-side cut membership. Source/job edges
have capacity one, job/slot edges exceed the job count, and slot/sink capacities
are supplied explicitly. `verify_flow_witness` checks matching feasibility,
cut closure, and equality of matching size and cut capacity independently of
flow construction. This checker certifies the graph witness; routing edge and
group validity still depends on the documented relaxation and original input.
The groups and both graphs are exposed for inspection.

This bound may be loose: the greedy groups are not a maximum clique cover, the
two graphs are separate relaxations, and travel interactions among three or more
jobs are not modeled. The certificate fully recomputes; it is not incremental.

## Regression checks

`cargo test --locked -p orchestrata-solver --test coverage` includes an independent
exhaustive route traversal on 512 small synthetic directed, nonmetric, mixed-mode
instances, sparse-path exclusions, service-only bounds, integer extremes,
invalid replay, witness tampering and bounded completion at the input limit.
Runtime/package tests reuse the synthetic dispatch fixture: its two-job baseline
has an upper bound of three; its replayed three-job plan attains that coverage
bound. Neither test makes a travel-time optimality claim.

## Explicit cumulative-travel relaxation

`CompiledRoutePackage::certify_coverage_with_travel` returns a separate
`RoutePackageTravelCoverage`/`TravelCoverageOutcome`. The runtime and native
counterparts are `certify_route_coverage_with_travel` and
`coverage::certify_coverage_with_travel`. Existing certification methods, result
types, commands and solve bytes retain their behavior.

This method first performs the original certificate, including incumbent replay,
then computes an additional courier-capacity flow bound. Its one deterministic
budget includes the original work, a fresh directed shortest-path closure and
all subsequent grouping, selection, distance, count and flow steps. Selection
scans charge comparisons rather than using an uncounted sort. Bounded allocation,
map/index operations and witness validation remain outside the primitive-work
counter. It is not a wall-clock limit. An incomplete stronger check carries no
partial optimality claim, even if the original check finished. Missing required
closure distances or checked-arithmetic failure also return `Incomplete`.

For each eligible job/courier pair, select one required representative location:
delivery unless it is the depot, otherwise pickup. Jobs whose representative is
the depot require no non-depot representative. Group the others by location.
For each admitted count `k`, count depot-only jobs first and then the largest
location multiplicities to obtain a lower bound `d` on distinct required nodes.
Shared locations are counted once. All grouping uses only jobs eligible under
the original optimistic single-job relaxation.

For `d = 0`, the travel lower bound is zero. Otherwise it is the independently
minimized depot-outbound distance plus independently minimized depot-return
distance, plus `(d - 1)` times the minimum directed closure distance between
distinct eligible representatives. For `d = 1` the between-node term is omitted.
These minima need not belong to a compatible route: this deliberately makes the
bound optimistic. Any real route has first visits to at least `d` distinct
required nodes. In their original order, each intervening route segment costs
at least its directed shortest-path distance. Nonnegative static travel and
sequential service make the travel sum a valid lower bound.

Add the `k` smallest eligible jobs' combined pickup/delivery service durations,
independently of representative selection. A count whose optimistic travel plus
service exceeds the supplied shift cannot be served. The remaining count bound
becomes that courier's capacity in another inspectable matching/min-cut graph.
The returned coverage upper bound is the minimum of this flow and the original
certificate. The result exposes representative multiplicities, distance minima,
service prefixes and per-count travel/distinct-node bounds for inspection.

Only a separately replayed incumbent reaching the complete bound proves maximum
coverage. There is still no travel-time optimality or infeasibility explanation
for individual unassigned jobs. Unknown direct arcs never replace shortest-path
reasoning; no external travel is fetched. All computations fully recompute.
