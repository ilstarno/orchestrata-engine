# Orchestrata engine

Orchestrata is a Rust library for assigning integer values under hard rules and ranked preferences. Use the Rust API directly or call the native C ABI through the C#, Java, Python, Node.js, PHP, or Go bindings.

## Try a solve

Build the native library with stable Rust. On Linux, run the following from the repository root with Python 3.10 or later:

    cargo build --release --locked -p orchestrata-ffi

    PYTHONPATH=sdks/python python3 - <<'PY'
    from orchestrata import Solver

    problem = {
        "variables": [{"name": "job", "domain": [0, 1]}],
        "constraints": [
            {"kind": "prefer_value", "variable": 0, "value": 1, "soft_weight": 1}
        ],
    }
    solver = Solver("target/release/liborchestrata_ffi.so")
    print(solver.solve(problem, 2)["values"])
    PY

This prints `[1]`. Exact search either returns a verified optimum, proves the submitted model infeasible, or rejects a search that exceeds the evaluation budget. Bounded local search can return a feasible plan but cannot prove optimality or infeasibility. The Rust API also compares verified plans and verifies or constructs small pickup-and-delivery routes using caller-supplied travel times.

See the [SDK guide](sdks/README.md) for the library name on your platform, native ABI details, and other languages. See [engine tests](backend_tests/README.md) for verification commands.

## Assign jobs within worker capacity

Use the [Rust capacity example](docs/capacity-matching.md) to assign as many
independent jobs as possible to eligible workers without exceeding their job
limits.

    cargo run --locked -p orchestrata-solver --example capacity_matching

## Count assignments to a value

Use `count_value` to say how many selected variables may take one value. For
example, if shift `1` must have at least one and at most two people from the
three listed workers:

```json
{
  "variables": [
    {"name": "worker_a", "domain": [0, 1]},
    {"name": "worker_b", "domain": [0, 1]},
    {"name": "worker_c", "domain": [0, 1]}
  ],
  "constraints": [
    {"kind": "count_value", "variables": [0, 1, 2], "value": 1, "min": 1, "max": 2, "weight": 5, "tier": "hard"}
  ]
}
```

The scope must contain distinct, valid zero-based variable indices and cannot
be empty. Bounds are inclusive and must satisfy `0 <= min <= max <= scope size`.
`weight` is a positive signed 64-bit integer. Each assignment outside the
range incurs the weight once per missing or excess match. Use `"tier": "hard"`
to require the range and `"tier": "soft"` to prefer it; use separate rules for
a mandatory floor and a preferred target. A valid hard rule can make a model
infeasible if the domains cannot meet the range. The evaluation budget still
applies. If a returned diagnostic score or the exact winner's score cannot
fit in 64 bits, Rust returns `ScoreOverflow`.

This is an added request kind in the existing native V1 JSON envelope, not
a new ABI version. Use a native library that includes this rule; older
libraries reject it. The public Rust enums have new variants, so downstream
exhaustive matches need updating when upgrading. Typed SDK builders require
separate updates unless they accept raw problem JSON.

## Reserve one exclusive resource over time

Use `no_overlap` for jobs sharing one piece of equipment or one room.
Give every job a distinct start variable and a fixed unsigned 64-bit
duration. Start domains use signed 64-bit values. Choose the same time unit
for all starts and durations. For example, a tool can be booked again at
minute 30 if its previous booking started at minute 0 and lasted 30 minutes:

```json
{
  "variables": [
    {"name": "first_booking", "domain": [0]},
    {"name": "next_booking", "domain": [20, 30]}
  ],
  "constraints": [
    {"kind": "no_overlap", "intervals": [
      {"start": 0, "duration": 30},
      {"start": 1, "duration": 15}
    ]}
  ]
}
```

This example has the solution `[0, 30]`. The `start` field is a zero-based
variable index, not a timestamp. Each rule needs at least two distinct
start references; zero-duration bookings occupy no time. A booking ending
exactly when another starts does not conflict. Internally, ends are computed
without truncation even beyond the signed 64-bit range. Diagnostic hard score
subtracts the shared time for each overlapping pair and reports an error if
the final score cannot fit in a signed 64-bit result. The exact solve still
requires a search budget and does not choose the room or equipment for you.

This is an additional raw native V1 JSON constraint. Use a library build
containing the rule; older builds do not recognize it. Typed SDK builders
are not extended by this change.


## Construct small pickup-and-delivery routes in Rust

`routing::construct_routes` attempts deterministic pickup/delivery insertion
for at most 16 couriers and 32 shipments. Set a budget of at most 20,000
candidate insertions. The caller supplies a repeatable, static travel-time
lookup by courier mode and ordered pair of locations; an unknown leg is not
used. This function does not fetch maps, calculate travel times, or use a
network service.

The result includes verified assigned routes, unassigned shipment indices, the
number of candidate insertions tried, and a stop reason. Reaching the budget
or finishing with unassigned shipments is not proof of infeasibility, and a
complete pass does not prove optimality. Oversized inputs and an oversized
budget return errors separately from invalid routing problems. This constructor
is a Rust API; the native V1 JSON solve endpoint and SDKs are unchanged by it.
