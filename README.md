# Orchestrata engine

Orchestrata is a Rust library for assigning integer values under hard rules and ranked preferences. Use the Rust API directly or call the native C ABI through the C#, Java, Python, Node.js, PHP, or Go bindings.

## Try a solve

Build the native library with stable Rust. On Linux, run the following from the repository root with Python 3.10 or later:

    cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi

    PYTHONPATH=sdks/python python3 - <<'PY'
    from orchestrata import Solver

    problem = {
        "variables": [{"name": "job", "domain": [0, 1]}],
        "constraints": [
            {"kind": "prefer_value", "variable": 0, "value": 1, "soft_weight": 1}
        ],
    }
    solver = Solver("backend/target/release/liborchestrata_ffi.so")
    print(solver.solve(problem, 2)["values"])
    PY

This prints `[1]`. Exact search either returns a verified optimum, proves the submitted model infeasible, or rejects a search that exceeds the evaluation budget. Bounded local search can return a feasible plan but cannot prove optimality or infeasibility. The Rust API also compares verified plans and checks supplied pickup-and-delivery routes; it does not calculate travel times or construct routes.

See the [SDK guide](sdks/README.md) for the library name on your platform, native ABI details, and other languages. See [backend tests](backend_tests/README.md) for verification commands.

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
